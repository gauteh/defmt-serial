#![feature(never_type)]
use embedded_hal::blocking::serial::Write;
use static_cell::StaticCell;
use std::io::{self, Write as _};

struct StdoutSerial;

impl Write<u8> for StdoutSerial {
    type Error = !;

    fn bwrite_all(&mut self, word: &[u8]) -> Result<(), !> {
        io::stdout().write(word).unwrap();
        Ok(())
    }

    fn bflush(&mut self) -> Result<(), !> {
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
