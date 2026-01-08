#![feature(read_array)]
#![feature(bool_to_result)]
#![feature(never_type)]

use foxglove::{
    Channel,
    schemas::{Log, Timestamp},
};
use std::{io::Read, net::TcpStream};

pub mod config;
pub mod itm_parser;
pub mod messages;

use crate::{
    config::{AppConfig, ITMChannelConfig, PortConfiguration},
    itm_parser::{ITMParseError, ITMParser, ITMPortConvType, MAX_MSG_PER_PCKT},
};
use crate::{itm_parser::NUM_ITM_PORTS, messages::NumericalMessage};

enum ChannelState {
    Numerical { topic: Channel<NumericalMessage> },
    CharStream { topic: Channel<Log>, buf: Vec<u8> },
}

// Number of CHAR's after which we force a flush to a foxglove Log
// This should rarely happen as we flush on newlines from the ITM stream
const AUTOFLUSH_LIMIT: usize = 100;

impl ChannelState {
    pub fn update(&mut self, values: heapless::Vec<ITMPortConvType, { MAX_MSG_PER_PCKT }>) {
        match self {
            ChannelState::CharStream { topic, buf } => {
                for val in values {
                    let chr: u8 = val.into();
                    if buf.len() >= AUTOFLUSH_LIMIT || b'\n' == chr {
                        topic.log(&Log {
                            message: String::from_utf8_lossy(buf).into(),
                            timestamp: Some(Timestamp::now()),
                            ..Default::default()
                        });
                        buf.clear();
                    }

                    if b'\n' != chr {
                        buf.push(chr);
                    }
                }
            }
            Self::Numerical { topic } => {
                for val in values {
                    topic.log(&NumericalMessage {
                        timestamp: Some(Timestamp::now()),
                        number: val.into(),
                    });
                }
            }
        }
    }
}

impl From<PortConfiguration> for ChannelState {
    fn from(conf: PortConfiguration) -> Self {
        match conf.typ {
            ITMChannelConfig::CHAR => ChannelState::CharStream {
                topic: Channel::<Log>::new(&conf.name),
                buf: Vec::new(),
            },
            _ => ChannelState::Numerical {
                topic: Channel::<NumericalMessage>::new(&conf.name),
            },
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // FIXME Add save/load logic for the config
    let mut conf: AppConfig = AppConfig {
        port_conf: [const { None }; NUM_ITM_PORTS],
    };
    conf.port_conf[0] = Some(PortConfiguration {
        name: "CH0".into(),
        typ: ITMChannelConfig::U32,
    });
    conf.port_conf[1] = Some(PortConfiguration {
        name: "CH1".into(),
        typ: ITMChannelConfig::I16F16,
    });
    conf.port_conf[2] = Some(PortConfiguration {
        name: "CH2".into(),
        typ: ITMChannelConfig::CHAR,
    });

    let mut parser = ITMParser::new(conf.port_conf.clone().map(|x| x.map(|y| y.into())));

    let mut fox_chans: [Option<ChannelState>; NUM_ITM_PORTS] =
        conf.port_conf.map(|x| x.map(|y| y.into()));
    foxglove::WebSocketServer::new()
        .start_blocking()
        .expect("Server failed to start");

    let mut listener = TcpStream::connect("127.0.0.1:3344")?;
    listener.set_nodelay(true)?;

    loop {
        let byte = listener.read_array::<1>()?[0];

        let parsed = parser.update(byte);

        if let Err(x) = &parsed {
            match x {
                ITMParseError::UnderfullPacket { .. } => {}
                _ => println!("{}", x),
            }

            continue;
        }
        let val = parsed.unwrap();

        print!("port {}: ", val.port);
        for value in &val.data {
            print!("{}", value);
        }
        println!("");

        fox_chans[val.port].as_mut().unwrap().update(val.data);
    }
}
