use solana_gossip_async::errors::{Result, Error};
use solana_gossip_async::{connection::Connection, handshake::handshake};
use solana_gossip_proto::utils::parse_addr;

use clap::{arg, Command};

fn parse_socket_addr(value: &str) -> ::std::result::Result<std::net::SocketAddr, std::io::Error> {
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
    let matches = Command::new("clap-test")
        .arg(
            arg!(--e <VALUE>)
                .default_value("141.98.219.218:8000")
                .value_parser(clap::builder::ValueParser::new(parse_socket_addr)),
        )
        .get_matches();

    let Some(entrypoint_addr) = matches.get_one::<std::net::SocketAddr>("e") else {
        return Err(Error::InputError);
    };

    let mut conn = Connection::connect(entrypoint_addr.to_owned()).await?;

    let info = handshake(&mut conn).await;
    match info {
        Ok(Some(info)) => {
            println!("[main] OK {info:?}");
        }
        Err(err) => {
            println!("[main] exists with error:{err:?}")
        }
        _ => {}
    }

    Ok(())
}
