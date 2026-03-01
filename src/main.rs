#![feature(read_array)]
#![feature(bool_to_result)]
#![feature(never_type)]

use std::{io::Read, net::TcpStream};

pub mod channel_adapter;
pub mod config;
pub mod itm_parser;
pub mod messages;

use crate::{
    channel_adapter::ChannelAdapter,
    config::{AppConfig, ItmChannelConfig, PortConfiguration},
    itm_parser::{ItmParser, NUM_ITM_PORTS},
};

const READ_BUF_SIZE: usize = 1024;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // FIXME Add save/load logic for the config
    let mut conf: AppConfig = AppConfig {
        port_conf: [const { None }; NUM_ITM_PORTS],
    };
    conf.port_conf[0] = Some(PortConfiguration {
        name: "CH0".into(),
        typ: ItmChannelConfig::CHARSTREAM,
    });

    let mut parser = ItmParser::new();

    let mut fox_chans: [Option<ChannelAdapter>; NUM_ITM_PORTS] =
        conf.port_conf.map(|x| x.map(|y| y.into()));
    foxglove::WebSocketServer::new()
        .start_blocking()
        .expect("Server failed to start");

    let mut listener = TcpStream::connect("127.0.0.1:3344")?;
    listener.set_nodelay(true)?;

    let mut buf = [0; READ_BUF_SIZE];
    loop {
        if let Ok(len) = listener.read(&mut buf) {
            for i in 0..len {
                if let Err(x) = parser.update(buf[i]) {
                    eprintln!("{}", x);
                }
            }
        }
        for (maybe_chan, stream) in fox_chans.iter_mut().zip(&mut parser.streams) {
            if let Some(chan) = maybe_chan {
                chan.update(stream);
            }
        }
    }
}
