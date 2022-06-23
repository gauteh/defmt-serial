#![no_std]

use core::ffi::c_void;
use core::ptr;
use core::{any::Any, marker::PhantomData};
use defmt::{global_logger, Logger};
use embedded_hal::serial::{nb::Write, Error, ErrorKind, ErrorType};

static mut ENCODER: defmt::Encoder = defmt::Encoder::new();
pub static mut WRITEFN: *const c_void = ptr::null_mut();

#[macro_export]
macro_rules! defmt_serial {
    ($serial:ident, $stype:ty) => {
        use core::ptr;
        static mut LOGGER: *mut $stype = ptr::null_mut();

        let wfn = |bytes: &[u8]| {
            unsafe {
                for b in bytes {
                    (*LOGGER).write(*b);
                }
            }
        };

        unsafe {
            LOGGER = &mut ($serial) as *mut _;

            defmt_serial::WRITEFN = &wfn as *const _ as *const c_void;
        }
    };
}

static mut LOGGER: Option<*mut WriteWrapper> = None;

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

static mut ERR_ARR: [u8; 124] = [0u8; 124];

impl Write<u8> for WriteWrapper {
    fn write(&mut self, b: u8) -> nb::Result<(), Self::Error> {
        unsafe {
            // ERR_ARR = core::mem::transmute_copy(&self.0.write(b));
            // ERR_ARR = self.0.write(b) as *const _;
        }
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

    unsafe fn write(bytes: &[u8]) {}

    unsafe fn flush() {}
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
