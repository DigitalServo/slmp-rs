use slmp::*;

const SLMP_PROPS: SLMP4EConnectionProps = SLMP4EConnectionProps {
    ip: "192.168.3.10",
    port: 5007,
    cpu: CPU::R,
    serial_id: 0x0001,
    network_id: 0x00,
    pc_id: 0xff,
    io_id: 0x03ff,
    area_id: 0x00,
    cpu_timer: 0x0010,
};

#[tokio::main]
async fn main() {
    let mut client = SLMPClient::new(SLMP_PROPS);
    client.connect().await.unwrap();

    // Word data
    let devices = [
        Device{device_type: DeviceType::D, address: 20},
        Device{device_type: DeviceType::D, address: 25},
        Device{device_type: DeviceType::D, address: 30},
        Device{device_type: DeviceType::D, address: 35},
    ];

    let data = [
        TypedData::U16(10),
        TypedData::U16(20),
        TypedData::I16(-40),
        TypedData::U32(80000),
    ];

    let wr_data = [
        DeviceData{device: devices[0], data: data[0]},
        DeviceData{device: devices[1], data: data[1]},
        DeviceData{device: devices[2], data: data[2]},
        DeviceData{device: devices[3], data: data[3]},
    ];
    client.random_write(&wr_data).await.unwrap();

    let devices = [
        TypedDevice{device: devices[0], data_type: DataType::U16},
        TypedDevice{device: devices[1], data_type: DataType::U16},
        TypedDevice{device: devices[2], data_type: DataType::I16},
        TypedDevice{device: devices[3], data_type: DataType::U32},
    ];

    let ret = client.random_read(&devices).await.unwrap();
    println!("\nDevice access:");
    for x in ret {
        println!("{:?}", x);
    }

    // Bit data
    let devices = [
        Device{device_type: DeviceType::M, address: 0},
        Device{device_type: DeviceType::M, address: 1},
        Device{device_type: DeviceType::M, address: 2},
        Device{device_type: DeviceType::M, address: 3},
    ];

    let data = [
        TypedData::Bool(true),
        TypedData::Bool(false),
        TypedData::Bool(true),
        TypedData::Bool(false),
    ];

    let wr_data = [
        DeviceData{device: devices[0], data: data[0]},
    ];
    client.random_write(&wr_data).await.unwrap();

    let ret: Vec<DeviceData> = client.bulk_read(devices[0], data.len(), DataType::Bool).await.unwrap();
    println!("\nBit access:");
    for x in ret {
        println!("{:?}", x);
    }
    println!();

    client.close().await;
}

