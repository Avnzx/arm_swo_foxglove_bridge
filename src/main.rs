#![feature(read_array)]
#![feature(bool_to_result)]
#![feature(never_type)]

use std::{env, fs::read_to_string, io::Read, net::TcpStream, path::Path, time::Duration};

pub mod channel_adapter;
pub mod config;
pub mod itm_parser;
pub mod messages;

use crate::{channel_adapter::ChannelAdapter, config::AppConfig, itm_parser::ItmParser};

const LISTEN_ADDRESS: &str = "127.0.0.1:3344";
const READ_BUF_SIZE: usize = 1024;
const READ_TIMEOUT: Duration = Duration::from_millis(10);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 || args.iter().any(|s| s == "--help" || s == "-h") {
        println!(
            "Usage: arm_swo_foxglove_bridge [/path/to/config/file]\n\nExample config file:\n{}",
            include_str!("example.ron")
        );
        return Ok(());
    }

    let conf: AppConfig =
        ron::from_str(&read_to_string(&Path::new(&args[1])).expect("Failed to open config file"))
            .expect("Failed to parse config file");
    let mut fox_chans: Vec<(usize, ChannelAdapter)> = Vec::from_iter(
        conf.port_conf
            .iter()
            .map(|(&chan, conf)| (chan, conf.clone().into())),
    );

    foxglove::WebSocketServer::new()
        .start_blocking()
        .expect("Server failed to start");

    let mut listener = TcpStream::connect(LISTEN_ADDRESS)?;
    listener.set_nodelay(true)?;
    listener.set_read_timeout(Some(READ_TIMEOUT))?;

    let mut parser = ItmParser::new();
    let mut buf = [0; READ_BUF_SIZE];
    loop {
        if let Ok(len) = listener.read(&mut buf) {
            for i in 0..len {
                if let Err(x) = parser.update(buf[i]) {
                    eprintln!("{}", x);
                }
            }
        }
        for (chan, adapter) in fox_chans.iter_mut() {
            // Keep updating until the stream is consumed
            while adapter.update(&mut parser.streams[*chan]).is_some() {}
        }
    }
}
