use slmp::{CPU, DataType, Device, DeviceType, MonitorDevice, PollingInterval, SLMP4EConnectionProps, SLMPConnectionManager, TypedDevice};


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

    let cyclic_task = async |data| {
        for x in data {
            println!("{:?}", x);
        }
        println!();
        Ok(())
    };

    manager.connect(&connection_props, cyclic_task).await?;

    let target_devices = [
        MonitorDevice {
            inverval: PollingInterval::Fast,
            device: TypedDevice {
                device: Device { device_type: DeviceType::D, address: 4001 },
                data_type: DataType::U16
            },
        },
        MonitorDevice {
            inverval: PollingInterval::Slow,
            device: TypedDevice {
                device: Device { device_type: DeviceType::D, address: 4005 },
                data_type: DataType::U16
            },
        },
        MonitorDevice {
            inverval: PollingInterval::Meduim,
            device: TypedDevice {
                device: Device { device_type: DeviceType::D, address: 4006 },
                data_type: DataType::U16
            },
        },
        MonitorDevice {
            inverval: PollingInterval::Meduim,
            device: TypedDevice {
                device: Device { device_type: DeviceType::D, address: 4007 },
                data_type: DataType::U16
            },
        },
    ];
    manager.register_monitor_targets(&connection_props, &target_devices).await?;

    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    manager.disconnect(&connection_props).await?;

    Ok(())
}
