#![feature(read_array)]
#![feature(bool_to_result)]
#![feature(never_type)]

use std::{env, fs::read_to_string, io::Read, net::TcpStream, path::Path, time::Duration};

pub mod channel_adapter;
pub mod config;
pub mod itm_parser;
pub mod messages;

use crate::{channel_adapter::ChannelAdapter, config::AppConfig, itm_parser::ItmParser};

const READ_BUF_SIZE: usize = 1024;
const READ_TIMEOUT: Duration = Duration::from_millis(1);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    assert!(args.len() == 1, "Invalid argument count");

    let conf: AppConfig =
        ron::from_str(&read_to_string(&Path::new(&args[0])).expect("Failed to open config file"))
            .expect("Failed to parse config file");
    let mut fox_chans: Vec<(usize, ChannelAdapter)> = Vec::from_iter(
        conf.port_conf
            .iter()
            .map(|(&chan, conf)| (chan, conf.clone().into())),
    );

    foxglove::WebSocketServer::new()
        .start_blocking()
        .expect("Server failed to start");

    let mut listener = TcpStream::connect("127.0.0.1:3344")?;
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
            adapter.update(&mut parser.streams[*chan]);
        }
    }
}
