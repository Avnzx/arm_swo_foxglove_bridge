use std::collections::VecDeque;

use thiserror::Error;

// TODO Technically up to 256 if we support page extension frames!
pub const NUM_ITM_PORTS: usize = 32;

pub struct ItmParser {
    byte_buffer: heapless::Deque<u8, 6>, // 6 So we can detect SYNC frames
    pub streams: [VecDeque<u8>; NUM_ITM_PORTS],
}

#[derive(Error, Debug, Clone)]
pub enum ItmParseError {
    #[error("Invalid size field in port {addr} packet header")]
    InvalidTracePacketSize { addr: usize },
    #[error("ITM hardware buffer full")]
    ItmOverflow,
    #[error("Flushed full parse buffer, assuming ITM/TPIU was reset")]
    ParseBufFull,
    #[error("Unknown error")]
    UnknownError,
}

impl ItmParser {
    const PCKT_SYNC: u8 = 0x80;
    const PCKT_OVFW: u8 = 0x70;

    // Identity bit is 0 for software STIM source, 1 for hardware source
    const PCKT_HWSC: u8 = 0b00000100;
    const PCKT_PROT: u8 = 0b00000011;

    pub fn new() -> Self {
        Self {
            byte_buffer: heapless::Deque::new(),
            streams: [const { VecDeque::new() }; NUM_ITM_PORTS],
        }
    }

    pub fn update(&mut self, byte: u8) -> Result<(), ItmParseError> {
        if self.byte_buffer.push_back(byte).is_err() {
            self.byte_buffer.clear();
            return Err(ItmParseError::ParseBufFull);
        }

        // TODO: Do we have a protocol (LocalTS / Paging / GlobalTS) packet?
        // Pray that the optimiser is smart enough to avoid allocations
        match Vec::from_iter(self.byte_buffer.clone().into_iter()).as_slice() {
            [0, 0, 0, 0, 0, Self::PCKT_SYNC] => todo!("SYNC Packet"),
            [Self::PCKT_OVFW, ..] => {
                self.byte_buffer.pop_front();
                return Err(ItmParseError::ItmOverflow);
            }
            [head, ..] if (head & Self::PCKT_PROT) == 0 => {
                // todo!("Protocol Packet")
                return Err(ItmParseError::UnknownError);
            }
            [head, tail @ ..] if (head & Self::PCKT_HWSC) == 0 => {
                let addr = ((head >> 3) & 0b11111) as usize;
                let size = match head & 0b11 {
                    1 => 1,
                    2 => 2,
                    3 => 4,
                    _ => {
                        self.byte_buffer.pop_front();
                        return Err(ItmParseError::InvalidTracePacketSize { addr });
                    }
                };

                if self.byte_buffer.len() <= size {
                    // Not enough bytes in buf to decode packet
                    return Ok(());
                }

                // This will always be executed when there is exactly the right amount of data
                self.byte_buffer.clear();
                self.streams[addr].extend(tail);

                return Ok(());
            }
            _ => return Err(ItmParseError::UnknownError),
        }
    }
}
