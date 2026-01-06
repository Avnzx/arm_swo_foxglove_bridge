#![feature(bool_to_result)]

pub mod daq;
pub mod itm_parser;

use iced_plot::PlotUiMessage;

use crate::daq::DAQEvent;
use crate::itm_parser::ITMPortConvType;

use fixed::types::I16F16;

#[derive(Debug, Clone)]
pub enum Message {
    ToggleGlobalRun,
    PlotMessage(PlotUiMessage),
    Tick,
    TraceTypeSelected(usize, TraceType),
    DAQEvent(DAQEvent),
}

impl From<DAQEvent> for Message {
    fn from(evt: DAQEvent) -> Self {
        Message::DAQEvent(evt)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TraceType {
    #[default]
    NONE,
    CHAR,
    U32,
    I32,
    F32,
    I16F16,
}
impl TraceType {
    pub const ALL: [TraceType; 6] = [
        TraceType::NONE,
        TraceType::CHAR,
        TraceType::U32,
        TraceType::I32,
        TraceType::F32,
        TraceType::I16F16,
    ];
}

impl From<Option<ITMPortConvType>> for TraceType {
    fn from(val: Option<itm_parser::ITMPortConvType>) -> Self {
        match val {
            None => Self::NONE,
            Some(ITMPortConvType::CHAR(_)) => Self::CHAR,
            Some(ITMPortConvType::U32(_)) => Self::U32,
            Some(ITMPortConvType::I32(_)) => Self::I32,
            Some(ITMPortConvType::F32(_)) => Self::F32,
            Some(ITMPortConvType::I16F16(_)) => Self::I16F16,
        }
    }
}

impl From<TraceType> for Option<ITMPortConvType> {
    fn from(val: TraceType) -> Self {
        match val {
            TraceType::NONE => None,
            TraceType::CHAR => Some(ITMPortConvType::CHAR(0)),
            TraceType::U32 => Some(ITMPortConvType::U32(0)),
            TraceType::I32 => Some(ITMPortConvType::I32(0)),
            TraceType::F32 => Some(ITMPortConvType::F32(0.0)),
            TraceType::I16F16 => Some(ITMPortConvType::I16F16(I16F16::ZERO)),
        }
    }
}

impl std::fmt::Display for TraceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TraceType::NONE => "Off",
                TraceType::CHAR => "char",
                TraceType::U32 => "u32",
                TraceType::I32 => "i32",
                TraceType::F32 => "f32",
                TraceType::I16F16 => "I16F16",
            }
        )
    }
}
