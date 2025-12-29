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
    let devices = [
        Device{device_type: DeviceType::D, address: 25},
        Device{device_type: DeviceType::D, address: 20},
        Device{device_type: DeviceType::D, address: 35},
        Device{device_type: DeviceType::D, address: 30},
        Device{device_type: DeviceType::M, address: 10},
        Device{device_type: DeviceType::M, address: 11},
        Device{device_type: DeviceType::M, address: 12},
        Device{device_type: DeviceType::M, address: 13},
    ];

    let data = [
        TypedData::U16(10),
        TypedData::U16(20),
        TypedData::U32(80000),
        TypedData::I16(-40),
        TypedData::Bool(true),
        TypedData::Bool(false),
        TypedData::Bool(true),
        TypedData::Bool(true),
    ];

    let wr_data = [
        DeviceData{device: devices[0], data: data[0]},
        DeviceData{device: devices[2], data: data[2]},
        DeviceData{device: devices[1], data: data[1]},
        DeviceData{device: devices[3], data: data[3]},
        DeviceData{device: devices[3], data: data[3]},
        DeviceData{device: devices[1], data: data[1]},
        DeviceData{device: devices[0], data: data[0]},
        DeviceData{device: devices[2], data: data[2]},
    ];
    client.random_write(&wr_data).await.unwrap();

    // Read
    let devices = [
        Device{device_type: DeviceType::D, address: 25},
        Device{device_type: DeviceType::D, address: 20},
        Device{device_type: DeviceType::D, address: 35},
        Device{device_type: DeviceType::D, address: 30},
        Device{device_type: DeviceType::M, address: 10},
        Device{device_type: DeviceType::M, address: 11},
        Device{device_type: DeviceType::M, address: 12},
        Device{device_type: DeviceType::M, address: 13},
    ];

    let devices = [
        TypedDevice{device: devices[0], data_type: DataType::U16},
        TypedDevice{device: devices[1], data_type: DataType::U16},
        TypedDevice{device: devices[2], data_type: DataType::U32},
        TypedDevice{device: devices[3], data_type: DataType::I16},
        TypedDevice{device: devices[7], data_type: DataType::Bool},
        TypedDevice{device: devices[6], data_type: DataType::Bool},
        TypedDevice{device: devices[5], data_type: DataType::Bool},
        TypedDevice{device: devices[4], data_type: DataType::Bool},
    ];

    let ret = client.random_read(&devices).await.unwrap();
    println!("\nDevice access:");
    for x in ret {
        println!("{:?}", x);
    }
    println!();


    client.close().await;
}
