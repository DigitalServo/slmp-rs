# Seamless Message Protocol (SLMP) for Rust
This library provides SLMP client to access the PLCs of Mitsubishi Electronic.


## Get Started
First of all, **You should enable SLMP communication (binary mode) and open a port** using GX Works 2/3.

This library supports the connection to MELSEC-Q and MELSEC iQ-R PLCs, using a 4E frame.
You can pass a connection property with `new()` and try to connect with `connect()`.

```rust
use slmp::{SLMPClient, SLMP4EConnectionProps};

#[tokio::main]
async fn main() {

    const conn_props: SLMP4EConnectionProps = SLMP4EConnectionProps {...};

    let mut client = SLMPClient::new(conn_props);
    client.connect().await.unwrap();

    ...
    
    client.close().await;
}
```

## Access Method
This library supports 3 methods
- Bulk read/write
- Random read/write
- Block read/write.

and primitive number type
- u8
- i8
- u16
- i16
- u32
- i32
- f32

### Read/Write Examples
- Bulk access
    ```bash
    cargo r --example bulk_access 
    ```

- Random access
    ```bash
    cargo r --example random_access 
    ```

- Block access
    ```bash
    cargo r --example block_access 
    ```

## Multi-PLC Connection
`SLMPConnectionManager` allow you to connect a client to multi PLCs.

```rust
use slmp::{CPU, SLMP4EConnectionProps, SLMPConnectionManager};


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let manager = SLMPConnectionManager::new();

    const conn_props_1: SLMP4EConnectionProps = SLMP4EConnectionProps {...};
    const conn_props_2: SLMP4EConnectionProps = SLMP4EConnectionProps {...};

    let cyclic_task = async |data| {
        for x in data {
            println!("{:?}", x);
        }
        println!();
        Ok(())
    };

    manager.connect(conn_props_1, cyclic_task).await?;
    manager.connect(conn_props_2, cyclic_task).await?;
    
    ...

    manager.disconnect(conn_props_1).await?;
    manager.disconnect(conn_props_2).await?;

    Ok(())
}
```

### Cyclic Read Example
- Cyclic access
    ```bash
    cargo r --example cyclic_access
    ```