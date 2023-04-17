use clap::{arg, Command};
use log::{error, info, LevelFilter};
use simple_logger::SimpleLogger;

use solana_gossip_async::errors::{Error, Result};
use solana_gossip_async::{connection::Connection, handshake::handshake};
use solana_gossip_proto::utils::parse_addr;

fn parse_socket_addr(value: &str) -> ::std::result::Result<std::net::SocketAddr, std::io::Error> {
    println!("{value}");
    if let Some(addr) = parse_addr(value) {
        Ok(addr)
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Invalid value:{value}"),
        ))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .with_colors(true)
        .init()
        .unwrap();

    let matches = Command::new("solana gossip async")
        .arg(
            arg!(--entrypoint <VALUE> "a entrpoint address")
                .default_value("141.98.219.218:8000")
                .value_parser(clap::builder::ValueParser::new(parse_socket_addr)),
        )
        .get_matches();

    let Some(entrypoint_addr) = matches.get_one::<std::net::SocketAddr>("entrypoint") else {
        return Err(Error::InputError);
    };

    let mut conn = Connection::connect(entrypoint_addr.to_owned()).await?;

    let info = handshake(&mut conn).await;
    match info {
        Ok(Some(info)) => {
            info!("OK {info:?}");
        }
        Err(err) => {
            error!("exists with error:{err}")
        }
        _ => {}
    }

    Ok(())
}
