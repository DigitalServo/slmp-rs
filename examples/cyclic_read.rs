use slmp::{CPU, DataType, Device, DeviceType, MonitorRequest, SLMP4EConnectionProps, SLMPConnectionManager, TypedDevice};


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let connection_props: SLMP4EConnectionProps = SLMP4EConnectionProps {
        ip: String::from("192.168.3.10"),
        port: 5007,
        cpu: CPU::R,
        serial_id: 0x0001,
        network_id: 0x00,
        pc_id: 0xff,
        io_id: 0x03ff,
        area_id: 0x00,
        cpu_timer: 0x0010,
    };

    let manager = SLMPConnectionManager::new();

    let cycle_ms: u64 = 100;
    let cyclic_task = async |data| {
        for x in data {
            println!("{:?}", x);
        }
        println!();
        Ok(())
    };

    manager.connect(&connection_props, cyclic_task, cycle_ms).await?;

    let target_devices = [
        MonitorRequest {
            connection_props: &connection_props,
            monitor_device: TypedDevice {
                device: Device { device_type: DeviceType::D, address: 4001 },
                data_type: DataType::U16
            }
        },
        MonitorRequest {
            connection_props: &connection_props,
            monitor_device: TypedDevice {
                device: Device { device_type: DeviceType::D, address: 4002 },
                data_type: DataType::U16
            }
        },
        MonitorRequest {
            connection_props: &connection_props,
            monitor_device: TypedDevice {
                device: Device { device_type: DeviceType::D, address: 4003 },
                data_type: DataType::U16
            }
        },
    ];
    manager.register_monitor_targets(&target_devices).await?;

    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    manager.disconnect(&connection_props).await?;

    Ok(())
}
