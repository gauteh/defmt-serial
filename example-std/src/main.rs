#![feature(never_type)]
use core::convert::Infallible;
use embedded_io::{Write, ErrorType};
use static_cell::StaticCell;
use std::io::{self, Write as _};

struct StdoutSerial;

impl ErrorType for StdoutSerial {
    type Error = Infallible;
}

impl Write for StdoutSerial {
    fn write(&mut self, word: &[u8]) -> Result<usize, Infallible> {
        io::stdout().write(word).unwrap();
        Ok(word.len())
    }

    fn flush(&mut self) -> Result<(), Infallible> {
        io::stdout().flush().unwrap();
        Ok(())
    }
}

static SERIAL: StaticCell<StdoutSerial> = StaticCell::new();

fn main() {
    eprintln!("Hello, world!");

    let serial = StdoutSerial;
    let _ds = defmt_serial::defmt_serial(SERIAL.init(serial));

    eprintln!("Logging to info with defmt..");
    defmt::info!("Hello defmt-world!");

    for i in 0..50 {
        defmt::debug!("Now at: {}", i);
    }

    defmt::warn!("Done!");

    eprintln!("Good bye.");
}
