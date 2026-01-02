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

    #[allow(unused)]
    let wait = async || {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await
    };

    let cpu_type = client.get_cpu_type().await.unwrap();
    println!("cpu type: {cpu_type}");

    client.echo().await.unwrap();
    println!("Echo succeess");

    // Lock & Unlock
    let password = "123456";

    client.lock_cpu(password).await.unwrap();
    println!("cpu locked");
    wait().await;

    client.unlock_cpu(password).await.unwrap();
    println!("cpu unlocked");
    wait().await;

    // Stop
    client.stop_cpu().await.unwrap();
    println!("cpu stopped");
    wait().await;

    // Latch clear
    client.clear_latch().await.unwrap();
    println!("latch cleared");
    wait().await;

    // Reset
    // client.reset_cpu().await.unwrap();
    // println!("cpu reset");
    // wait().await;

    // Run
    client.run_cpu().await.unwrap();
    println!("cpu started");
    wait().await;

    // Pause
    client.pause_cpu().await.unwrap();
    println!("cpu paused");
    wait().await;

    // Run
    client.run_cpu().await.unwrap();
    println!("cpu started");
    wait().await;

    client.close().await;
}
