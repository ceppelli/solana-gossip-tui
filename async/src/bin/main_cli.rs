use std::{env, net::SocketAddr, str::FromStr};

use solana_gossip_async::{connection::Connection, handshake::handshake};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "141.98.219.218:8000".to_string());

    println!("[main] entrypoint addr:{addr}");

    let entrypoint_addr = SocketAddr::from_str(&addr).unwrap();
    let mut conn = Connection::connect(entrypoint_addr).await?;

    let info = handshake(&mut conn).await;
    match info {
        Ok(Some(info)) => {
            println!("[main] OK {info:?}");
        }
        Err(err) => {
            println!("{err:?}")
        }
        _ => {}
    }

    Ok(())
}
