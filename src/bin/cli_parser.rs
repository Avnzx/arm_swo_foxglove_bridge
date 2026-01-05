#![feature(read_array)]
#![feature(bool_to_result)]

use fixed::types::I16F16;
use std::{io::Read, net::TcpStream};

use iced_swviewer::itm_parser::{ITMParseError, ITMParser, ITMPortConvType, NUM_ITM_PORTS};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut listener = TcpStream::connect("127.0.0.1:3344")?;
    listener.set_nodelay(true)?;

    let mut conf = [None; NUM_ITM_PORTS];
    conf[0] = Some(ITMPortConvType::U32(0));
    conf[1] = Some(ITMPortConvType::I16F16(I16F16::ZERO));
    conf[2] = Some(ITMPortConvType::CHAR(0));

    let mut parser = ITMParser::new(conf);

    loop {
        let byte = listener.read_array::<1>()?[0];

        let parsed = parser.update(byte);

        if let Err(x) = &parsed {
            match x {
                ITMParseError::UnderfullPacket { .. } => {}
                _ => println!("{}", x),
            }

            continue;
        }
        let val = parsed.unwrap();

        print!("port {}: ", val.port);
        for value in val.data {
            print!("{}", value);
        }
        println!("");
    }
}
