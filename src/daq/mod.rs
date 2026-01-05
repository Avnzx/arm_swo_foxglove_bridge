use std::future::pending;

use iced::futures::future::Either;
use iced::{Subscription, futures::select};
use iced::futures::{FutureExt, Stream};
use iced::futures::channel::mpsc;
use iced::futures::sink::SinkExt;
use iced::stream;
use smol::io::AsyncReadExt;
use smol::stream::StreamExt;

use crate::itm_parser::{ITMConvValue, ITMParseError, ITMParser, ITMPortConvType};
use smol::{io::Bytes, net::TcpStream, prelude::*, Unblock};

#[derive(Debug, Clone)]
pub enum DAQEvent {
    Ready(mpsc::Sender<DAQInput>),
    Connected,
    Disconnected,
    ConnectionError,
    ITMValue(Result<ITMConvValue, ITMParseError>),
}

#[derive(Debug, Clone)]
pub enum DAQInput {
    Connect([Option<ITMPortConvType>; 32]),
    Disconnect,
}


enum InternalEvt<T> {
    ControlEvt(DAQInput),
    DataEvt(T)
}

impl<T> From<DAQInput> for InternalEvt<T> {
    fn from(val: DAQInput) -> Self { 
        Self::ControlEvt(val)
    }
}


pub fn some_worker() -> impl Stream<Item = DAQEvent> {
    stream::channel(100, async |mut output| {
        // Create channel
        let (sender, mut receiver) = mpsc::channel(100);
        output.send(DAQEvent::Ready(sender)).await.unwrap();

        let mut conn: Option<(ITMParser, Bytes<TcpStream>)> = None;
        loop {
            use iced_futures::futures::StreamExt;
            let mut control_fut = receiver.select_next_some();
            let mut data_fut = match conn.as_mut() {
                Some((_, stream)) => Either::Left(smol::stream::StreamExt::next(stream).fuse()),
                None => Either::Right(pending().fuse()),
            };

            // Wait for either data or app controls to come in..
            let comb_fut = select! {
                x = control_fut => x.into(),
                y = data_fut => InternalEvt::DataEvt(y),
            };
            
            match comb_fut {
                InternalEvt::ControlEvt(DAQInput::Connect(conf)) => {
                    let maybe_tcp_con = TcpStream::connect("127.0.0.1:3344").await;
                    if let Err(_) = maybe_tcp_con {
                        output.send(DAQEvent::ConnectionError).await.unwrap();
                        continue;
                    }
                    
                    let tcp_con = maybe_tcp_con.unwrap();
                    tcp_con.set_nodelay(true).expect("Unable to set_nodelay on TCP connection!");

                    conn = Some((ITMParser::new(conf), tcp_con.bytes()));
                    output.send(DAQEvent::Connected).await.unwrap();
                }
                InternalEvt::ControlEvt(DAQInput::Disconnect) | InternalEvt::DataEvt(None) => {
                    drop(conn);
                    conn = None;
                    output.send(DAQEvent::Disconnected).await.unwrap();
                }
                InternalEvt::DataEvt(Some(data)) => {
                    let byte = data.unwrap();
                    let msg = conn.as_mut().unwrap().0.update(byte);
                    output.send(DAQEvent::ITMValue(msg)).await.unwrap();
                }
            }
        }
    })
}

pub fn subscription() -> Subscription<DAQEvent> {
    Subscription::run(some_worker)
}
