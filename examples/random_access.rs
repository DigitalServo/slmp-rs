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

    // Write
    let wr_data = [
        DeviceData{device: Device{device_type: DeviceType::D, address: 25}, data: TypedData::U16(10)},
        DeviceData{device: Device{device_type: DeviceType::D, address: 35}, data: TypedData::U32(80000)},
        DeviceData{device: Device{device_type: DeviceType::D, address: 20}, data: TypedData::U16(20)},
        DeviceData{device: Device{device_type: DeviceType::D, address: 30}, data: TypedData::I16(-40)},
        DeviceData{device: Device{device_type: DeviceType::M, address: 10}, data: TypedData::Bool(true)},
        DeviceData{device: Device{device_type: DeviceType::M, address: 11}, data: TypedData::Bool(false)},
        DeviceData{device: Device{device_type: DeviceType::M, address: 12}, data: TypedData::Bool(true)},
        DeviceData{device: Device{device_type: DeviceType::M, address: 13}, data: TypedData::Bool(true)},
        DeviceData{device: Device{device_type: DeviceType::D, address: 40}, data: TypedData::from(("test", 4))},
        DeviceData{device: Device{device_type: DeviceType::D, address: 50}, data: TypedData::from(("TEST", 4))},
    ];
    client.random_write(&wr_data).await.unwrap();

    // Read
    let devices = [
        TypedDevice{device: Device{device_type: DeviceType::D, address: 40}, data_type: DataType::String(10)},
        TypedDevice{device: Device{device_type: DeviceType::D, address: 50}, data_type: DataType::String(10)},
        TypedDevice{device: Device{device_type: DeviceType::D, address: 25}, data_type: DataType::U16},
        TypedDevice{device: Device{device_type: DeviceType::D, address: 20}, data_type: DataType::U16},
        TypedDevice{device: Device{device_type: DeviceType::D, address: 35}, data_type: DataType::U32},
        TypedDevice{device: Device{device_type: DeviceType::D, address: 30}, data_type: DataType::I16},
        TypedDevice{device: Device{device_type: DeviceType::M, address: 13}, data_type: DataType::Bool},
        TypedDevice{device: Device{device_type: DeviceType::M, address: 12}, data_type: DataType::Bool},
        TypedDevice{device: Device{device_type: DeviceType::M, address: 11}, data_type: DataType::Bool},
        TypedDevice{device: Device{device_type: DeviceType::M, address: 10}, data_type: DataType::Bool},
        TypedDevice{device: Device{device_type: DeviceType::M, address: 10}, data_type: DataType::BitArray16},
    ];

    let ret = client.random_read(&devices).await.unwrap();
    println!("\nDevice access:");
    for x in ret {
        println!("{:?}", x);
    }
    println!();

    client.close().await;
}
