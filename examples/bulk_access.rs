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
    let start_device: Device = Device{device_type: DeviceType::D, address: 4000};
    let data = [
        TypedData::U16(10),
        TypedData::U16(20),
        TypedData::U16(30),
        TypedData::U16(40),
        TypedData::U16(50),
        TypedData::U16(60),
        TypedData::U16(70),
        TypedData::U16(80),
    ];

    client.bulk_write(start_device, &data).await.unwrap();

    let ret: Vec<DeviceData> = client.bulk_read(start_device, data.len(), DataType::U16).await.unwrap();
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

