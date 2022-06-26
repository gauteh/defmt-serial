#![feature(never_type)]
use embedded_hal::serial::Write;
use std::io::{self, Write as _};

struct StdoutSerial;

impl Write<u8> for StdoutSerial {
    type Error = !;

    fn write(&mut self, word: u8) -> nb::Result<(), !> {
        io::stdout().write(&[word]).unwrap();
        Ok(())
    }

    fn flush(&mut self) -> nb::Result<(), !> {
        io::stdout().flush().unwrap();
        Ok(())
    }
}

fn main() {
    eprintln!("Hello, world!");

    let mut serial = StdoutSerial;
    defmt_serial::defmt_serial!(serial, StdoutSerial);

    eprintln!("Logging to info with defmt..");
    defmt::info!("Hello defmt-world!");

    for i in 0..50 {
        defmt::debug!("Now at: {}", i);
    }

    defmt::warn!("Done!");

    eprintln!("Good bye.");
}
