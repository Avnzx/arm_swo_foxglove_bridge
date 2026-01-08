use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;

use fixed::types::I16F16;
use heapless::Vec;
use thiserror::Error;

// TODO Technically up to 256 if we support page extension frames!
pub const NUM_ITM_PORTS: usize = 32;
// In a single ITM Packet how many messages can we possibly fit?
// Currently the smallest type we support is u8's and we can pack 4 per frame
pub const MAX_MSG_PER_PCKT: usize = 4;

pub struct ITMParser {
    byte_buffer: [Option<u8>; 6], // 6 So we can detect SYNC frames
    port_config: [Option<ITMPortConvType>; NUM_ITM_PORTS],
}

#[derive(Debug, Clone)]
pub struct ITMConvValue {
    // TODO timestamp
    pub port: usize,
    pub data: Vec<ITMPortConvType, { MAX_MSG_PER_PCKT }>,
}

#[derive(Error, Debug, Clone)]
pub enum ITMParseError {
    #[error("Not enough bytes in buf to decode packet on port {addr}")]
    UnderfullPacket { addr: usize }, // Not an error! just wait and call update() more times
    #[error("Dropping packet on unconfigured port {addr}")]
    UnconfiguredPort { addr: usize }, // Not an error! just an unconfig'd channel
    #[error("Invalid size field in port {addr} packet header")]
    InvalidTracePacketSize { addr: usize },
    #[error("Dropping, port {addr} packet <-> ITMConvValue size mismatch")]
    TracePacketSizeMismatch { addr: usize },
    #[error("ITM hardware buffer full")]
    ITMOverflow,
    #[error("Flushed full parse buffer, assuming ITM/TPIU was reset")]
    ParseBufFull,
    #[error("Unknown error")]
    UnknownError,
}

impl ITMParser {
    const MAX_PAYLOAD_SZ: usize = 4;

    const PCKT_SYNC: [Option<u8>; 6] = [Some(0), Some(0), Some(0), Some(0), Some(0), Some(0x80)];
    const PCKT_OVFW: Option<u8> = Some(0x70);
    //const PCKT_TLCL
    //const PCKT_EXTN
    //const PCKT_TGLB

    // Identity bit is 0 for software STIM source, 1 for hardware source
    const PCKT_HWSC: u8 = 0b00000100;

    pub fn new(port_conf: [Option<ITMPortConvType>; NUM_ITM_PORTS]) -> Self {
        Self {
            byte_buffer: Default::default(),
            port_config: port_conf,
        }
    }

    // Option<(Timestamp, Port, ITMPortConvType)>
    pub fn update(&mut self, byte: u8) -> Result<ITMConvValue, ITMParseError> {
        // Precheck before shifting in new data
        if self.byte_buffer.last().unwrap().is_some() {
            self.byte_buffer.fill(None);
            self.byte_buffer[0].replace(byte);
            Err(ITMParseError::ParseBufFull)?;
        }

        // Shift in new data
        for i in 0..self.byte_buffer.len() {
            if self.byte_buffer[i].is_none() {
                self.byte_buffer[i].replace(byte);
                break;
            }
        }

        // TODO Do we have a protocol (LocalTS / Paging / GlobalTS) packet?
        match self.byte_buffer {
            Self::PCKT_SYNC => todo!("SYNC Packet"),
            [Self::PCKT_OVFW, ..] => {
                self.pop_data(1, 0); // Pop overflow indicator
                Err(ITMParseError::ITMOverflow)
            }
            [Some(head), ..] if (head & Self::PCKT_HWSC) == Self::PCKT_HWSC => {
                todo!("HW Source Packet")
            }
            [Some(head), ..] if (head & Self::PCKT_HWSC) == 0 => {
                let addr = ((head >> 3) & 0b11111) as usize;
                let size = match head & 0b11 {
                    1 => 1,
                    2 => 2,
                    3 => 4,
                    _ => {
                        self.pop_data(1, 0);
                        return Err(ITMParseError::InvalidTracePacketSize { addr });
                    }
                };

                // Not enough bytes in buf to decode packet
                self.byte_buffer[size].ok_or(ITMParseError::UnderfullPacket { addr })?;

                // Remove header from buffer
                self.pop_data(1, 0);
                // Remove this data from the buffer
                let bytes = self.pop_data(size, 0);

                let parse_type = self.port_config[addr]
                    .ok_or(ITMParseError::UnconfiguredPort { addr })?;

                // XXX We only support one -> many (ITM packets -> target type)
                if size % parse_type.size() != 0 {
                    return Err(ITMParseError::TracePacketSizeMismatch { addr });
                }

                let data: Vec<ITMPortConvType, { MAX_MSG_PER_PCKT }> = bytes
                    .chunks_exact(parse_type.size())
                    .map(|raw| -> ITMPortConvType { parse_type.with_data(raw) })
                    .collect();

                Ok(ITMConvValue {
                    port: addr,
                    data,
                })
            }
            _ => Err(ITMParseError::UnknownError),
        }
    }

    // size = how many bytes to pop
    // start = where to start popping (0 = first)
    fn pop_data(&mut self, size: usize, start: usize) -> Vec<u8, { ITMParser::MAX_PAYLOAD_SZ }> {
        let mut bytes = Vec::<u8, { ITMParser::MAX_PAYLOAD_SZ }>::new();

        for i in start..(size + start) {
            bytes.push(self.byte_buffer[i].take().unwrap()).unwrap();
        }
        self.byte_buffer[start..].rotate_left(size);

        bytes
    }

    #[allow(dead_code)]
    fn print_buf(&self) {
        for i in 0..6 {
            match self.byte_buffer[i] {
                Some(byte) => eprint!("{:02x}", byte),
                None => eprint!("__"),
            };
        }
        eprintln!("");
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ITMPortConvType {
    CHAR(u8),
    U32(u32),
    I32(i32),
    F32(f32),
    I16F16(I16F16),
}

impl ITMPortConvType {
    pub fn size(&self) -> usize {
        match self {
            Self::CHAR(_) => 1,
            Self::U32(_) | Self::I32(_) | Self::F32(_) | Self::I16F16(_) => 4,
        }
    }

    // invalid to call this function w/o correct size of bytes
    pub fn with_data(&self, bytes: &[u8]) -> Self {
        match self {
            Self::CHAR(_) => Self::CHAR(bytes[0]),
            Self::U32(_) => Self::U32(u32::from_le_bytes(bytes.try_into().unwrap())),
            Self::I32(_) => Self::I32(i32::from_le_bytes(bytes.try_into().unwrap())),
            Self::F32(_) => Self::F32(f32::from_le_bytes(bytes.try_into().unwrap())),
            Self::I16F16(_) => Self::I16F16(I16F16::from_le_bytes(bytes.try_into().unwrap())),
        }
    }
}

impl Display for ITMPortConvType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::CHAR(x) => write!(f, "{}", *x as char),
            Self::U32(x) => write!(f, "{}", x),
            Self::I32(x) => write!(f, "{}", x),
            Self::F32(x) => write!(f, "{}", x),
            Self::I16F16(x) => write!(f, "{}", x),
        }
    }
}

impl From<ITMPortConvType> for f64 {
    fn from(val: ITMPortConvType) -> Self {
        match val {
            ITMPortConvType::CHAR(_) => panic!(),
            ITMPortConvType::U32(n) => n.into(),
            ITMPortConvType::I32(n) => n.into(),
            ITMPortConvType::F32(n) => n.into(),
            ITMPortConvType::I16F16(n) => n.into(),
        }
    }
}

impl From<ITMPortConvType> for u8 {
    fn from(val: ITMPortConvType) -> Self {
        if let ITMPortConvType::CHAR(c) = val {
            c
        } else {
            panic!()
        }
    }
}
