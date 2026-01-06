#![feature(bool_to_result)]

use fixed::types::I16F16;
use iced::Length;
use iced::Theme;
use iced::alignment;
use iced::futures::SinkExt;
use iced::futures::channel::mpsc;
use iced::futures::executor::block_on;
use iced::padding;
use iced::widget::Grid;
use iced::widget::Space;
use iced::widget::button;
use iced::widget::column;
use iced::widget::grid::Sizing;
use iced::widget::pick_list;
use iced::widget::row;
use iced::widget::space;
use iced_plot::PlotUiMessage;
use iced_plot::PlotWidget;
use iced_plot::Series;
use iced_plot::{MarkerStyle, PlotWidgetBuilder};

use iced::widget::text;
use iced::{Color, Element};
use crate::{
    daq::{DAQEvent, DAQInput},
    itm_parser::{ITMPortConvType, NUM_ITM_PORTS},
};

mod daq;
mod itm_parser;

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

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .subscription(App::subscription)
        .theme(Theme::CatppuccinMacchiato)
        .run()
}

#[derive(Default)]
struct AppConfig {
    channels: [Option<ITMPortConvType>; NUM_ITM_PORTS],
}

#[derive(Default)]
struct AppState {
    global_run: bool,
    tcp_channel: Option<mpsc::Sender<DAQInput>>,
}

struct App {
    state: AppState,
    config: AppConfig,

    widget: PlotWidget,
    positions: Vec<[f64; 2]>,
    x: f64,
}

impl App {
    fn update(&mut self, message: Message) {
        match message {
            Message::ToggleGlobalRun => {
                //self.state.global_run = !self.state.global_run;

                if let Some(chan) = self.state.tcp_channel.as_mut() {
                    block_on(chan.send(if !self.state.global_run {
                        DAQInput::Connect(self.config.channels)
                    } else {
                        DAQInput::Disconnect
                    }))
                    .unwrap();
                }
            }
            Message::PlotMessage(plot_msg) => {
                self.widget.update(plot_msg);
            }
            Message::Tick => {
                // Add new point
                let y = (self.x * 0.5).sin();
                self.positions.push([self.x, y]);
                self.x += 0.1f64;

                // Keep only last 300 points for scrolling effect
                //if self.positions.len() > 300 {
                //    self.positions.remove(0);
                //}

                // Update the series
                self.widget.remove_series("scrolling");
                let series = Series::markers_only(self.positions.clone(), MarkerStyle::star(2.0))
                    .with_label("scrolling")
                    .with_color(Color::WHITE);
                self.widget.add_series(series).unwrap();

                // TODO self.widget.autoscale_on_updates(false);
            }
            Message::TraceTypeSelected(port, ty) => {
                self.config.channels[port] = ty.into();
            }
            Message::DAQEvent(evt) => {
                eprintln!("{:?}", evt);
                match evt {
                    DAQEvent::Ready(chan) => {
                        self.state.tcp_channel = Some(chan);
                    }
                    DAQEvent::Connected => {
                        self.state.global_run = true;
                    }
                    DAQEvent::Disconnected | DAQEvent::ConnectionError => {
                        self.state.global_run = false;
                    }
                    _ => {}
                }
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let mut channel_config: Grid<Message> = Grid::new()
            .columns(2)
            .spacing(5)
            .height(Sizing::EvenlyDistribute(Length::Shrink));
        for x in 0..NUM_ITM_PORTS {
            channel_config = channel_config.push(text(format!("CH{} ", x)));
            channel_config = channel_config.push(
                pick_list(
                    &TraceType::ALL[..],
                    Some::<TraceType>(self.config.channels[x].into()),
                    move |ty| -> Message { Message::TraceTypeSelected(x, ty) },
                )
                .width(Length::Fill),
            );
        }

        let btn_run = button(
            text!("{}", if self.state.global_run { "RUN" } else { "STOP" })
                .width(Length::Fill)
                .align_x(alignment::Horizontal::Center),
        )
        .on_press(Message::ToggleGlobalRun)
        .style(|theme: &Theme, status| {
            let off_state = button::danger(theme, status);
            let on_state = button::success(theme, status);

            match self.state.global_run {
                true => on_state,
                false => off_state,
            }
        })
        .width(Length::Fill);

        row![
            column![
                btn_run,
                space::vertical(),
                channel_config,
                space::vertical(),
                row![
                    button(text!("-").align_x(alignment::Horizontal::Center)).width(Length::Shrink),
                    space::horizontal(),
                    text!("CONSOLE").align_x(alignment::Horizontal::Center),
                    space::horizontal(),
                    button(text!("+").align_x(alignment::Horizontal::Center)).width(Length::Shrink)
                ]
                .align_y(alignment::Vertical::Center),
                row![
                    button(text!("-").align_x(alignment::Horizontal::Center)).width(Length::Shrink),
                    space::horizontal(),
                    text!("GRAPH").align_x(alignment::Horizontal::Center),
                    space::horizontal(),
                    button(text!("+").align_x(alignment::Horizontal::Center)).width(Length::Shrink)
                ]
                .align_y(alignment::Vertical::Center),
                space::vertical(),
                row![
                    button(
                        text!("Save")
                            .width(Length::Fill)
                            .align_x(alignment::Horizontal::Center)
                    )
                    .width(Length::Fill),
                    Space::new().width(Length::Fixed(10.0)),
                    button(
                        text!("Load")
                            .width(Length::Fill)
                            .align_x(alignment::Horizontal::Center)
                    )
                    .width(Length::Fill)
                ]
            ]
            .spacing(5)
            .padding(padding::right(5))
            .width(Length::Fixed(200.0)),
            column![
                // Pretend this first one is a textual console...
                row![
                    button("[Autoscale]"),
                    self.widget.view().map(Message::PlotMessage),
                ],
                row![
                    button("[Autoscale]"),
                    self.widget.view().map(Message::PlotMessage),
                ],
                row![
                    button("[Autoscale]"),
                    self.widget.view().map(Message::PlotMessage),
                ],
            ]
            .width(Length::Fill),
            column![
                // Pretend this first one is a textual console...
                row![
                    button("[Autoscale]"),
                    self.widget.view().map(Message::PlotMessage),
                ],
                row![
                    button("[Autoscale]"),
                    self.widget.view().map(Message::PlotMessage),
                ],
                row![
                    button("[Autoscale]"),
                    self.widget.view().map(Message::PlotMessage),
                ],
            ]
            .width(Length::Fill)
        ]
        .padding(10)
        .into()
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        //window::frames().map(|_| Message::Tick)
        daq::subscription().map(|x| x.into())
    }

    fn new() -> Self {
        Self {
            state: Default::default(),
            config: Default::default(),
            widget: PlotWidgetBuilder::new()
                .with_autoscale_on_updates(true)
                .build()
                .unwrap(),
            positions: Vec::new(),
            x: 0.0f64,
        }
    }
}
