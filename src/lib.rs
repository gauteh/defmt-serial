#![no_std]

use core::{marker::PhantomData, any::Any};
use defmt::{global_logger, Logger};
use embedded_hal::serial::{Error, ErrorKind, ErrorType, nb::Write};


static mut LOGGER: Option<*mut WriteWrapper> = None;
static mut ENCODER: defmt::Encoder = defmt::Encoder::new();

struct WriteWrapper(&'static dyn Write<u8, Error = dyn Error>);

#[derive(Debug)]
struct WWErr;

impl ErrorType for WriteWrapper {
    type Error = WWErr;
}

impl Error for WWErr {
    fn kind(&self) -> ErrorKind {
        ErrorKind::Other
    }
}

impl Write<u8> for WriteWrapper {
    fn write(&mut self, b: u8) -> nb::Result<(), Self::Error> {
        self.0.write(b);
        Ok(())
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        Ok(())
    }
}

pub struct GlobalSerialLogger<W: Write<u8>>(PhantomData<W>);

// #[global_logger]
// pub struct UartGlobalLogger;

unsafe impl<W: Write<u8>> defmt::Logger for GlobalSerialLogger<W> {
    fn acquire() {
        // check atomic bool if taken
    }

    unsafe fn release() {}

    unsafe fn write(bytes: &[u8]) {
    }

    unsafe fn flush() {
    }
}

fn write_serial(mut remaining: &[u8]) {
    unsafe {
        // if let Some(writer) = LOGGER {
        //     for b in remaining {
                // Write::<u8>::write(&mut *writer, *b);
                // (*writer).write(*b);
            // }
        // }
    }
}
