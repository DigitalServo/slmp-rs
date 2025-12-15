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

#[tokio::test]
async fn test_bulk_access() {
    let mut client = SLMPClient::new(SLMP_PROPS);
    client.connect().await.unwrap();

    // Word data
    let start_device: Device = Device{device_type: DeviceType::D, address: 4000};
    let data = [
        TypedData::U32(10),
        TypedData::U32(20),
        TypedData::U32(30),
        TypedData::U32(40),
    ];

    client.bulk_write(start_device, &data).await.unwrap();

    let ret: Vec<DeviceData> = client.bulk_read(start_device, 1, DataType::U32).await.unwrap();
    let ret: Vec<TypedData> = ret.into_iter().map(|x| x.data).collect::<Vec<TypedData>>();

    assert_eq!(data.to_vec(), ret);

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
    let ret: Vec<TypedData> = ret.into_iter().map(|x| x.data).collect::<Vec<TypedData>>();

    assert_eq!(data, ret);

    client.close().await;
}


#[tokio::test]
async fn test_random_access() {
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
    let ret = ret.into_iter().map(|x| x.data).collect::<Vec<TypedData>>();

    assert_eq!(data.to_vec(), ret);

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
    let ret: Vec<TypedData> = ret.into_iter().map(|x| x.data).collect::<Vec<TypedData>>();

    assert_eq!(data.to_vec(), ret);

    client.close().await;
}


#[tokio::test]
async fn test_block_access() {
    let mut client = SLMPClient::new(SLMP_PROPS);
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
    for data in ret {
        println!("{:?}", data);
    }

    client.close().await;
}


