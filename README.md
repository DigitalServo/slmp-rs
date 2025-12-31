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
SLMP provides roughly 5 categories; 
- [x] Device access
- [ ] Label access
- [ ] Buffer-memory access
- [x] Unit control
- [ ] File control
  
This library supports **device access** and **unit control** methods.

### Device Control
This library enable you to use
- [x] Bulk read/write
- [x] Random read/write
- [x] Block read/write
- [x] Monitor register/read

and primitive types
- [x] bool
- [x] [bool; 16] (Word-size access)
- [x] u16
- [x] i16
- [x] u32
- [x] i32
- [x] f32
- [x] f64
- [x] String

The samples of those methods are prepared in `/examples`:
```bash
cargo r --example bulk_access 
cargo r --example random_access 
cargo r --example block_access 
cargo r --example monitor_read
```

### Unit Control
This library supports
- [x] Remote run
- [x]  Remote stop
- [x]  Remote pause
- [x]  Remote latch clear
- [x]  Remote reset *
- [x]  Get cpu type
- [x]  Remote unlock
- [x]  Remote lock
- [x]  Echo
- [ ]  Error Clear (for serial communication unit)

There are restrictions on use of remote reset.
Please check the document from Mitsubishi Electronics.

The sample is prepared in `/examples`:
```bash
cargo r --example unit_control
```

## Multi-PLC Connection
`SLMPConnectionManager` allow you to connect a client to multi PLCs.
You can give a cyclic task to each connection.

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

The sample of cyclic read is prepared in `/examples`:
```bash
cargo r --example cyclic_read
```

> [!CAUTION]
> The SLMP protocol features a concise presentation layer, and it allows device modifications, file operations, and changes to CPU operation settings without any authentication.
>
> As a result, these vulnerabilities have been registered with CISA.
> - CVE-2025-7405: Missing Authentication for Critical Function vulnerability
> - CVE-2025-7731: Cleartext Transmission of Sensitive Information vulnerability
> 
> In response to the above reports, Mitsubishi Electric has implemented the following countermeasures (as stated in advisory 2025-012):
> - Use a virtual private network (VPN) or similar technology to encrypt SLMP communications.
> - Restrict physical access to the LAN to which the affected products are connected.
> 
> (Note: No firmware fix is planned for this vulnerability.)
> 
> It should be noted that improper use of SLMP carries significant risks.
> Please use it with caution.
