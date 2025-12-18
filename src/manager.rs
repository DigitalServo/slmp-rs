use std::collections::hash_map::Entry;
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

const DATA_REQUEST_SIZE: usize = 256;

const MINUMUM_POLLING_PERIOD_MS: u64 = 100;
const MINUMUM_POLLING_PERIOD: tokio::time::Duration = tokio::time::Duration::from_millis(MINUMUM_POLLING_PERIOD_MS);
const LOOP_PERIOD_MS: usize = 5_000;

const LOOP_CNT_INIT: usize = 0;
const LOOP_CNT_MAX: usize = LOOP_PERIOD_MS / MINUMUM_POLLING_PERIOD_MS as usize - 1;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
#[cfg_attr(feature = "serde-camel", serde(rename_all = "camelCase"))]
pub struct PLCData {
    pub socket_addr: SocketAddr,
    pub device_data: DeviceData,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[cfg_attr(feature = "serde-camel", serde(rename_all = "camelCase"))]
pub enum PollingInterval {
    /// 100 ms
    Fast,
    /// 500 ms
    Meduim,
    /// 1s
    Slow,
    /// 5s
    Watch
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[cfg_attr(feature = "serde-camel", serde(rename_all = "camelCase"))]
pub struct MonitorDevice {
    pub inverval: PollingInterval,
    pub device: TypedDevice,
}

#[derive(Clone)]
struct MonitorDevices {
    fast: Vec<TypedDevice>,
    medium: Vec<TypedDevice>,
    slow: Vec<TypedDevice>,
    watch: Vec<TypedDevice>,
}

impl MonitorDevices {
    pub fn new() -> Self {
        Self {
            fast: Vec::with_capacity(DATA_REQUEST_SIZE),
            medium: Vec::with_capacity(DATA_REQUEST_SIZE),
            slow: Vec::with_capacity(DATA_REQUEST_SIZE),
            watch: Vec::with_capacity(DATA_REQUEST_SIZE),
        }
    }
}

pub struct SLMPWorker {
    client: SharedResource<SLMPClient>,
    connected_at: SystemTime,
    monitor_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    monitor_target: Arc<RwLock<MonitorDevices>>,
    sender_targets: Arc<Mutex<Option<UnboundedSender<Vec<MonitorDevice>>>>>,
    cancel_token: CancellationToken,
}

impl SLMPWorker {
    pub fn new(client: SharedResource<SLMPClient>) -> Self{
        Self {
            client,
            connected_at: SystemTime::now(),
            monitor_handle: Arc::new(Mutex::new(None)),
            monitor_target: Arc::new(RwLock::new(MonitorDevices::new())),
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

    pub async fn connect<'a, T, F, Fut>(&self, connection_props: &'a SLMP4EConnectionProps, cyclic_task: F) -> std::io::Result<()>
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

        let (sender_targets, mut receiver_targets) = unbounded_channel::<Vec<MonitorDevice>>();        

        let client = worker.client.clone();
        let monitor_target = worker.monitor_target.clone();
        let cancel_token = worker.cancel_token.clone();

        let monitor_handle = {

            tokio::spawn(async move {
                let mut interval = tokio::time::interval(MINUMUM_POLLING_PERIOD);
                let mut cnt = LOOP_CNT_INIT;

                loop {
                    tokio::select! {
                        _ = cancel_token.cancelled() => {
                            break;
                        }

                        Some(targets) = receiver_targets.recv() => {
                            let mut monitor_target = monitor_target.write().await;
                            monitor_target.fast = targets.iter().filter(|&x| x.inverval == PollingInterval::Fast).map(|x| x.device).collect();
                            monitor_target.medium = targets.iter().filter(|&x| x.inverval == PollingInterval::Meduim).map(|x| x.device).collect();
                            monitor_target.slow = targets.iter().filter(|&x| x.inverval == PollingInterval::Slow).map(|x| x.device).collect();
                            monitor_target.watch = targets.iter().filter(|&x| x.inverval == PollingInterval::Watch).map(|x| x.device).collect();
                        }

                        _ = interval.tick() => {

                            let target_devices = monitor_target.read().await;

                            let mut request_devices: Vec<TypedDevice> = Vec::with_capacity(DATA_REQUEST_SIZE * 4);

                            request_devices.extend_from_slice(&target_devices.fast);

                            if cnt % 5 == 0 {
                                request_devices.extend_from_slice(&target_devices.medium);
                            }

                            if cnt % 10 == 0 {
                                request_devices.extend_from_slice(&target_devices.slow);
                            }

                            if cnt == LOOP_CNT_MAX  {
                                request_devices.extend_from_slice(&target_devices.watch);
                            }

                            if request_devices.len() != 0 {
                                let ret = {
                                    let mut client = client.lock().await;
                                    client.random_read(&request_devices).await
                                };
                                if let Ok(values) = ret {
                                    let data: Vec<PLCData> = values.clone().into_iter().map(|device_data| PLCData {socket_addr, device_data} ).collect();
                                    let _ = cyclic_task(data).await;
                                }
                            }

                            cnt = if cnt == LOOP_CNT_MAX { 1 } else { cnt + 1 };
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

     pub async fn register_monitor_targets<'a>(&self, connection_props: &'a SLMP4EConnectionProps, targets: &'a [MonitorDevice]) -> std::io::Result<()> {
        let socket_addr: SocketAddr = SocketAddr::try_from(connection_props)?;

        let map = self.connections.lock().await;
        let worker = map.get(&socket_addr)
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::AddrNotAvailable, "Connection not found"))?
            .clone();

        let sender = worker.sender_targets.lock().await;

        if let Some(sender) = sender.clone() {
            let _ = sender.send(targets.to_vec());
        };

        Ok(())
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