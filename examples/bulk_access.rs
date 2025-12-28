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

    // Word data
    let start_device: Device = Device{device_type: DeviceType::D, address: 0};

    for i in 0..200 {
        let data: Vec<TypedData> = [0u16; 120]
            .iter()
            .enumerate()
            .map(|(j, _)| TypedData::U16(i as u16 + j as u16))
            .collect();

        client.bulk_write(start_device, &data).await.unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    let ret: Vec<DeviceData> = client.bulk_read(start_device, 8, DataType::U16).await.unwrap();
    println!("\nDevice access:");
    for x in ret {
        println!("{:?}", x);
    }

    // Bit data
    let start_device: Device = Device{device_type: DeviceType::M, address: 0};
    let data = vec![
        TypedData::Bool(true),
        TypedData::Bool(false),
        TypedData::Bool(false),
        TypedData::Bool(true),
    ];

    client.bulk_write(start_device, &data).await.unwrap();

    let ret: Vec<DeviceData> = client.bulk_read(start_device, data.len(), DataType::Bool).await.unwrap();
    println!("\nBit access:");
    for x in ret {
        println!("{:?}", x);
    }
    println!();

    client.close().await;
}
