#![feature(never_type)]
#![feature(unboxed_closures)]
#![feature(fn_traits)]
use std::fs;
use core::sync::atomic::{AtomicU32, Ordering};
use embedded_hal::serial::Write;
use defmt_serial as _;

static COUNT: AtomicU32 = AtomicU32::new(0);
defmt::timestamp!("{}", COUNT.fetch_add(1, Ordering::Relaxed));

struct VecSerial {
    pub buf: Vec<u8>,
}

impl VecSerial {
    pub fn new() -> VecSerial {
        VecSerial { buf: Vec::new() }
    }
}

impl Write<u8> for VecSerial {
    type Error = !;

    fn write(&mut self, word: u8) -> nb::Result<(), !> {
        self.buf.push(word);
        Ok(())
    }

    fn flush(&mut self) -> nb::Result<(), !> {
        // nop
        Ok(())
    }
}

fn main() {
    println!("Hello, world!");

    let mut serial = VecSerial::new();
    defmt_serial::defmt_serial!(serial, VecSerial);

    println!("Logging to info with defmt..");
    defmt::info!("Hello defmt-world!");

    println!("Writing to defmt.out");
    fs::write("defmt.out", serial.buf).unwrap();
}
