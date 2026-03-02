use std::{
    collections::VecDeque,
    io::{BufRead, Read},
};

use fixed::types::I16F16;
use foxglove::{
    Channel,
    schemas::{Log, Timestamp},
};
use prost::bytes::Buf;

use crate::{
    config::{ItmChannelConfig, NumericalFormat, PortConfiguration},
    messages::NumericalMessage,
};

pub enum ChannelAdapter {
    Numerical {
        topic: Channel<NumericalMessage>,
        format: NumericalFormat,
    },
    Text {
        topic: Channel<Log>,
    },
    Hexdump {
        topic: Channel<Log>,
        len: Option<usize>,
    },
}

// Number of CHAR's after which we force a flush to a foxglove Log
// This should rarely happen as we flush on newlines from the ITM stream
const AUTOFLUSH_LIMIT: usize = 256;

impl ChannelAdapter {
    // Returns Some(()) if bytes were consumed from the stream so that update is called again
    pub fn update(&mut self, stream: &mut VecDeque<u8>) -> Option<()> {
        match self {
            Self::Text { topic } => {
                if !stream.contains(&b'\n') {
                    // Force a flush if there hasn't been a newline for a while
                    if stream.len() > AUTOFLUSH_LIMIT {
                        stream.push_back(b'\n');
                    } else {
                        return None;
                    }
                }

                let mut message = String::new();
                stream.read_line(&mut message).ok()?;

                topic.log(&Log {
                    message,
                    timestamp: Some(Timestamp::now()),
                    ..Default::default()
                });
            }
            Self::Numerical { topic, format } => {
                topic.log(&NumericalMessage {
                    timestamp: Some(Timestamp::now()),
                    number: match format {
                        NumericalFormat::U32 => stream.try_get_u32_le().ok()? as f64,
                        NumericalFormat::I32 => stream.try_get_i32_le().ok()? as f64,
                        NumericalFormat::F32 => stream.try_get_f32_le().ok()? as f64,
                        NumericalFormat::I16F16 => {
                            let mut buf = [0; 4];
                            stream.read_exact(&mut buf).ok()?;
                            I16F16::from_le_bytes(buf).into()
                        }
                    },
                });
            }
            Self::Hexdump { topic, len } => match *len {
                Some(l) if stream.len() >= l => {
                    // Two hex characters per byte, a space/newline per word
                    let mut message = String::with_capacity(l * 2 + l / 4);
                    for i in 0..l {
                        if i == 0 {
                            // Prevent initial space/newline
                        } else if i % 16 == 0 {
                            message.push('\n');
                        } else if i % 4 == 0 {
                            message.push(' ');
                        }

                        message.push_str(&format!("{:02X}", stream.pop_front().unwrap()));
                    }

                    topic.log(&Log {
                        message,
                        timestamp: Some(Timestamp::now()),
                        ..Default::default()
                    });
                }
                None => {
                    *len = Some(stream.try_get_u32_le().ok()? as usize);
                }
                _ => {
                    return None;
                }
            },
        }

        Some(())
    }
}

impl From<PortConfiguration> for ChannelAdapter {
    fn from(conf: PortConfiguration) -> Self {
        match conf.typ {
            ItmChannelConfig::NUMERICAL(format) => ChannelAdapter::Numerical {
                topic: Channel::<NumericalMessage>::new(conf.name),
                format,
            },
            ItmChannelConfig::TEXT => ChannelAdapter::Text {
                topic: Channel::<Log>::new(conf.name),
            },
            ItmChannelConfig::HEXDUMP => ChannelAdapter::Hexdump {
                topic: Channel::<Log>::new(conf.name),
                len: None,
            },
        }
    }
}
