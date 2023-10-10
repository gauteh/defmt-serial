#![no_std]
//! A defmt target for logging messages over a serial interface. The serial interface must
//! implement e.g. [`embedded_hal::serial::Write`].
//!
//! The received defmt-frames can be read using e.g. `socat` and `defmt-print`, so that you can set
//! it up as `cargo run`ner. See the [example-artemis](https://github.com/gauteh/defmt-serial/tree/main/example-artemis) for how to do that.
//!
//! You can also use it to have defmt work on std/hosted OSes, see [example-std](https://github.com/gauteh/defmt-serial/tree/main/example-std).
//!
//! ```no_run
//! #![no_std]
//! #![no_main]
//!
//!
//! use panic_probe as _;
//! use cortex_m::asm;
//! use cortex_m_rt::entry;
//! use ambiq_hal::{self as hal, prelude::*};
//! use defmt;
//! use defmt_serial as _;
//!
//! #[entry]
//! fn main() -> ! {
//!     let mut dp = hal::pac::Peripherals::take().unwrap();
//!     let pins = hal::gpio::Pins::new(dp.GPIO);
//!
//!     // set up serial
//!     let mut serial = hal::uart::Uart0::new(dp.UART0, pins.tx0, pins.rx0);
//!     defmt_serial::defmt_serial(serial);
//!
//!     defmt::info!("Hello from defmt!");
//!
//!     loop {
//!         asm::wfi();
//!     }
//! }
//! ```

use core::sync::atomic::{AtomicBool, Ordering};
use defmt::global_logger;

static mut ENCODER: defmt::Encoder = defmt::Encoder::new();
static TAKEN: AtomicBool = AtomicBool::new(false);
static mut CS_RESTORE: critical_section::RestoreState = critical_section::RestoreState::invalid();

/// All of this nonsense is to try and erase the Error type of the `embedded_hal::serial::nb::Write` implementor.
type WriteCB = unsafe fn(SFn);
static mut WRITEFN: Option<WriteCB> = None;


enum SFn<'a> {
    Buf(&'a [u8]),
    Flush,
}

unsafe fn trampoline<F>(buf: SFn)
where
    F: FnMut(SFn),
{
    if let Some(wfn) = WRITEFN {
        let wfn = &mut *(wfn as *mut F);
        wfn(buf);
    }
}

fn get_trampoline<F>(_closure: &F) -> WriteCB
where
    F: FnMut(SFn),
{
    trampoline::<F>
}

/// Assign a serial peripheral to received defmt-messages using a blocking Write implementation.
/// The blocking Write trait implementation is the default and should always be present. When
/// the non blocking Write trait is implemented the blocking one is automatically provided.
pub fn defmt_serial(serial: impl embedded_hal::blocking::serial::Write<u8> + 'static) {
    let mut serial = core::mem::ManuallyDrop::new(serial);

    let wfn = move |a: SFn| {
        match a {
            SFn::Buf(buf) => {
                for b in buf {
                    serial.bwrite_all(&b.to_ne_bytes()).ok();
                }
            },
            SFn::Flush => { serial.bflush().ok(); },
        };
    };

    let trampoline = get_trampoline(&wfn);

    unsafe {
        let token = critical_section::acquire();
        WRITEFN = Some(trampoline);
        critical_section::release(token);
    }
}

#[global_logger]
struct GlobalSerialLogger;

unsafe impl defmt::Logger for GlobalSerialLogger {
    fn acquire() {
        let restore = unsafe { critical_section::acquire() };

        if TAKEN.load(Ordering::Relaxed) {
            panic!("defmt logger taken reentrantly");
        }

        TAKEN.store(true, Ordering::Relaxed);

        unsafe {
            CS_RESTORE = restore;
        }

        unsafe { ENCODER.start_frame(write_serial) }
    }

    unsafe fn release() {
        ENCODER.end_frame(write_serial);
        TAKEN.store(false, Ordering::Relaxed);

        let restore = CS_RESTORE;
        critical_section::release(restore);
    }

    unsafe fn write(bytes: &[u8]) {
        ENCODER.write(bytes, write_serial);
    }

    unsafe fn flush() {
        if let Some(wfn) = WRITEFN {
            wfn(SFn::Flush);
        }
    }
}

/// Write to serial using proxy function. We must ensure this function is not called
/// several times in parallel.
fn write_serial(remaining: &[u8]) {
    unsafe {
        if let Some(wfn) = WRITEFN {
            wfn(SFn::Buf(remaining));
        }
    }
}
