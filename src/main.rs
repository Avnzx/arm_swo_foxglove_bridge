#![feature(bool_to_result)]

//! Example of a scrolling plot with new data points being added over time.
use std::net::TcpStream;

use iced::Length;
use iced::Theme;
use iced::alignment;
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
use iced::window;
use iced::{Color, Element};

pub mod daq;
pub mod itm_parser;

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .subscription(App::subscription)
        .theme(Theme::CatppuccinMacchiato)
        .run()
}

const ITM_CHANNELS: usize = 32;

#[derive(Debug, Clone)]
enum Message {
    ToggleGlobalRun,
    PlotMessage(PlotUiMessage),
    Tick,
    TraceTypeSelected(usize, TraceType),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum TraceType {
    #[default]
    NONE,
    CHAR,
    I16F16,
    F32,
}

impl TraceType {
    const ALL: [TraceType; 4] = [
        TraceType::NONE,
        TraceType::CHAR,
        TraceType::I16F16,
        TraceType::F32,
    ];
}

impl std::fmt::Display for TraceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TraceType::NONE => "OFF",
                TraceType::CHAR => "char",
                TraceType::I16F16 => "I16F16",
                TraceType::F32 => "f32",
            }
        )
    }
}

#[derive(Default)]
struct AppConfig {
    channels: [TraceType; ITM_CHANNELS],
}

#[derive(Default)]
struct AppState {
    global_run: bool,
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
                self.state.global_run = !self.state.global_run;
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
                self.config.channels[port] = ty;
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let mut channel_config: Grid<Message> = Grid::new()
            .columns(2)
            .spacing(5)
            .height(Sizing::EvenlyDistribute(Length::Shrink));
        for x in 0..ITM_CHANNELS {
            channel_config = channel_config.push(text(format!("CH{} ", x)));
            channel_config = channel_config.push(
                pick_list(
                    &TraceType::ALL[..],
                    Some(self.config.channels[x]),
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
        let a = TcpStream::connect("127.0.0.1:3344").unwrap();

        //a.read_array()
        window::frames().map(|_| Message::Tick)
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
