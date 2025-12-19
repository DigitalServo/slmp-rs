use std::collections::{HashSet, hash_map::Entry};
use std::sync::Arc;
use std::net::SocketAddr;
use std::time::SystemTime;
use std::collections::HashMap;

use tokio::sync::{Mutex, RwLock, mpsc::{unbounded_channel, UnboundedSender}};
use tokio_util::sync::CancellationToken;

use crate::*;
use tokio::task::JoinHandle;

type SharedResource<T> = Arc<Mutex<T>>;
type ConnectionMap = HashMap<SocketAddr, Arc<SLMPWorker>>;

impl<'a> TryFrom<&MonitorRequest<'a>> for MonitoredDevice {
    type Error = std::io::Error;
    fn try_from(value: &MonitorRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            socket_addr: SocketAddr::try_from(value.connection_props)?,
            monitor_device: value.monitor_device
        })
    }
}


pub struct SLMPWorker {
    client: SharedResource<SLMPClient>,
    connected_at: SystemTime,
    monitor_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    monitor_target: Arc<RwLock<MonitorList>>,
    sender_targets: Arc<Mutex<Option<UnboundedSender<Vec<TypedDevice>>>>>,
    cancel_token: CancellationToken,
}

impl SLMPWorker {
    pub fn new(client: SharedResource<SLMPClient>) -> Self{
        Self {
            client,
            connected_at: SystemTime::now(),
            monitor_handle: Arc::new(Mutex::new(None)),
            monitor_target: Arc::new(RwLock::new(MonitorList::new())),
            sender_targets: Arc::new(Mutex::new(None)),
            cancel_token: CancellationToken::new(),
        }
    }

    pub async fn close(&self) {
        // Stop a spawned thread and release resources
        self.cancel_token.cancel();
        if let Some(handle) = self.monitor_handle.lock().await.take() {
            let _ = handle.await;
        }

        // Close a connection
        let client = self.client.lock().await;
        client.close().await;
        drop(client);

        // Drop sender_targets
        let mut sender = self.sender_targets.lock().await;
        *sender = None;
    }
}

pub struct SLMPConnectionManager {
    pub connections: SharedResource<ConnectionMap>,
}

impl SLMPConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn connect<'a, T, F, Fut>(&self, connection_props: &'a SLMP4EConnectionProps, cyclic_task: F, cycle_ms: u64) -> std::io::Result<()>
        where 
            F: Fn(Vec<PLCData>) -> Fut + std::marker::Send + 'static,
            Fut: std::future::Future<Output = std::io::Result<T>> + std::marker::Send,
    {
        let socket_addr: SocketAddr = SocketAddr::try_from(connection_props)?;

        // Once close a channel if exist and then wait
        if self.disconnect(connection_props).await? == true {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        };

        let client = SLMPClient::new(connection_props.clone());
        client.connect().await?;

        let mut worker = SLMPWorker::new(Arc::new(tokio::sync::Mutex::new(client)));

        let (sender_targets, mut receiver_targets) = unbounded_channel::<Vec<TypedDevice>>();        

        let client = worker.client.clone();
        let monitor_target = worker.monitor_target.clone();
        let cancel_token = worker.cancel_token.clone();

        let monitor_handle = {

            tokio::spawn(async move {
                let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(cycle_ms));

                loop {
                    tokio::select! {
                        _ = cancel_token.cancelled() => {
                            break;
                        }

                        Some(targets) = receiver_targets.recv() => {
                            let monitor_list = {
                                let mut client = client.lock().await;
                                client.monitor_register(&targets).await
                            };

                            if let Ok(monitor_list) = monitor_list {
                                let mut monitor_target = monitor_target.write().await;
                                *monitor_target = monitor_list;
                            }
                        }

                        _ = interval.tick() => {

                            let target_devices = monitor_target.read().await;
                            

                            if target_devices.sorted_devices.len() != 0 {
                                let ret = {
                                    let mut client = client.lock().await;
                                    client.monitor_read(&target_devices).await
                                };
                                if let Ok(values) = ret {
                                    let data: Vec<PLCData> = values.clone().into_iter().map(|device_data| PLCData {socket_addr, device_data} ).collect();
                                    let _ = cyclic_task(data).await;
                                }
                            }
                        }
                    }
                }
            })
        };

        worker.monitor_handle = Arc::new(Mutex::new(Some(monitor_handle)));
        worker.sender_targets = Arc::new(Mutex::new(Some(sender_targets)));

        let mut map = self.connections.lock().await;
        map.insert(socket_addr, Arc::new(worker));

        Ok(())
    }

    pub async fn disconnect<'a>(&self, connection_props: &'a SLMP4EConnectionProps) -> std::io::Result<bool> {
        let socket_addr: SocketAddr = SocketAddr::try_from(connection_props)?;

        let mut map = self.connections.lock().await;

        if let Entry::Occupied(entry) = map.entry(socket_addr) {
            let worker = entry.get();
            worker.close().await;
            entry.remove();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn clear(&self) {
        let mut map = self.connections.lock().await;
        for (_, worker) in map.drain() {
            worker.close().await;
        }
    }

    pub async fn register_monitor_targets<'a>(&self, targets: &'a [MonitorRequest<'a>]) -> std::io::Result<Vec<MonitoredDevice>> {

        let mut socket_addrs: Vec<SocketAddr> = targets
            .iter()
            .map(|x| SocketAddr::try_from(x.connection_props))
            .collect::<Result<Vec<SocketAddr>, std::io::Error>>()?;

        let mut seen = HashSet::new();
        socket_addrs.retain(|item| seen.insert(*item));

        let map = self.connections.lock().await;

        for socket_addr in &socket_addrs {
            if !map.contains_key(socket_addr) {
                return Err(std::io::Error::new(std::io::ErrorKind::AddrNotAvailable, "Connection not found"))
            }
        }

        for socket_addr in &socket_addrs {
            let worker = map.get(&socket_addr)
                .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::AddrNotAvailable, "Connection not found"))?
                .clone();

            let targets: Vec<TypedDevice> = targets
                .iter()
                .filter(|&x| if let Ok(x) = SocketAddr::try_from(x.connection_props) { &x == socket_addr } else { false })
                .map(|x| x.monitor_device)
                .collect();

            let sender = worker.sender_targets.lock().await;

            if let Some(sender) = sender.clone() {
                let _ = sender.send(targets.to_vec());
            };
        }

        let monitored_devices: Vec<MonitoredDevice> = targets
            .iter()
            .map(|x| MonitoredDevice::try_from(x))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(monitored_devices)
    }   

    pub async fn get_connections_with_elapsed_time(&self) -> HashMap<SocketAddr, std::time::Duration> {
        let map = self.connections.lock().await;
        map.iter()
            .filter_map(|(&addr, worker)| {
                worker.connected_at.elapsed().ok().map(|d| (addr, d))
            })
            .collect()
    }

    pub async fn operate_worker<'a, T, F, Fut>(&self, connection_props: &'a SLMP4EConnectionProps, task: F) -> std::io::Result<T>
        where
            F: FnOnce(Arc<Mutex<SLMPClient>>) -> Fut,
            Fut: std::future::Future<Output = std::io::Result<T>>,
    {
        let socket_addr: SocketAddr = SocketAddr::try_from(connection_props)?;
        
        let worker = {
            let map = self.connections.lock().await;
            map.get(&socket_addr)
                .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::AddrNotAvailable, "Connection not found"))?
                .clone()
        };

        task(worker.client.clone()).await
    }
}