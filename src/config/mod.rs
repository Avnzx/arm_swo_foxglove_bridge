use fixed::types::I16F16;

use crate::{ITMPortConvType, NUM_ITM_PORTS};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ITMChannelConfig {
    CHAR,
    U32,
    I32,
    F32,
    I16F16,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct AppConfig {
    pub port_conf: [Option<PortConfiguration>; NUM_ITM_PORTS],
}

#[derive(Debug, Clone, PartialEq)]
pub struct PortConfiguration {
    pub name: String,
    pub typ: ITMChannelConfig,
}

impl From<PortConfiguration> for ITMPortConvType {
    fn from(conf: PortConfiguration) -> Self {
        match conf.typ {
            ITMChannelConfig::CHAR => ITMPortConvType::CHAR(0),
            ITMChannelConfig::U32 => ITMPortConvType::U32(0),
            ITMChannelConfig::I32 => ITMPortConvType::I32(0),
            ITMChannelConfig::F32 => ITMPortConvType::F32(0.0),
            ITMChannelConfig::I16F16 => ITMPortConvType::I16F16(I16F16::ZERO),
        }
    }
}
