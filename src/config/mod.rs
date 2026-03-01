use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub enum NumericalFormat {
    U32,
    I32,
    F32,
    I16F16,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub enum ItmChannelConfig {
    NUMERICAL(NumericalFormat),
    TEXT,
    HEXDUMP,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PortConfiguration {
    pub name: String,
    pub typ: ItmChannelConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub port_conf: HashMap<usize, PortConfiguration>,
}
