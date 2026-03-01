use crate::NUM_ITM_PORTS;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NumericalFormat {
    U32,
    I32,
    F32,
    I16F16,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ItmChannelConfig {
    NUMERICAL(NumericalFormat),
    CHARSTREAM,
    HEXDUMP,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct AppConfig {
    pub port_conf: [Option<PortConfiguration>; NUM_ITM_PORTS],
}

#[derive(Debug, Clone, PartialEq)]
pub struct PortConfiguration {
    pub name: String,
    pub typ: ItmChannelConfig,
}
