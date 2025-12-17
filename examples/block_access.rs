use slmp::*;

#[tokio::main]
async fn main() {

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

    let mut client = SLMPClient::new(connection_props);
    client.connect().await.unwrap();

    let data= [
        BlockedDeviceData { 
            access_type: AccessType::Word,
            start_device: Device{device_type: DeviceType::D, address: 10},
            data: &[ TypedData::U16(10), TypedData::U16(20) ]
        },
        BlockedDeviceData {
            access_type: AccessType::Word,
            start_device: Device{device_type: DeviceType::D, address: 20},
            data: &[ TypedData::U16(30), TypedData::U16(40) ]
        },
        BlockedDeviceData {
            access_type: AccessType::Bit,
            start_device: Device{device_type: DeviceType::M, address: 0},
            data: &[ TypedData::Bool(true), TypedData::Bool(false), TypedData::Bool(true) ]
        },
    ];
    client.block_write(&data).await.unwrap();

    let device_blocks = [
        DeviceBlock{ access_type: data[0].access_type, start_device: data[0].start_device, size: data[0].data.len()},
        DeviceBlock{ access_type: data[1].access_type, start_device: data[1].start_device, size: data[1].data.len()},
        DeviceBlock{ access_type: data[2].access_type, start_device: data[2].start_device, size: data[2].data.len()},
    ];

    let ret: Vec<DeviceData> = client.block_read(&device_blocks).await.unwrap();
    println!("\nDevice & Bit access:");
    for data in ret {
        println!("{:?}", data);
    }
    println!();

    client.close().await;
}


