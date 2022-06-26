#![no_std]

use core::ffi::c_void;
use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use defmt::global_logger;

pub use critical_section;
pub use nb::block;

static mut ENCODER: defmt::Encoder = defmt::Encoder::new();
static TAKEN: AtomicBool = AtomicBool::new(false);
static INTERRUPTS: AtomicU8 = AtomicU8::new(0);

/// All of this nonsense is to try and erase the Error type of the `embedded_hal::serial::nb::Write` implementor.
pub type WriteCB = unsafe fn(&[u8]);
pub static mut WRITEFN: Option<WriteCB> = None;
pub static mut WRITECB: Option<*const c_void> = None;

unsafe fn trampoline<F>(buf: &[u8])
where
    F: FnMut(&[u8]),
{
    if let Some(wfn) = WRITEFN {
        let wfn = &mut *(wfn as *mut F);
        wfn(buf);
    }
}

pub fn get_trampoline<F>(_closure: &F) -> WriteCB
where
    F: FnMut(&[u8]),
{
    trampoline::<F>
}

#[macro_export]
macro_rules! defmt_serial {
    ($serial:ident, $stype:ty) => {{
        use core::{ptr, ffi::c_void};

        static mut LOGGER: *mut $stype = ptr::null_mut();

        let mut wfn = |buf: &[u8]| unsafe {
            for b in buf {
                defmt_serial::block!((*LOGGER).write(*b)).ok();
            }
        };
        let mut trampoline = defmt_serial::get_trampoline(&wfn);

        unsafe {
            let token = defmt_serial::critical_section::acquire();

            LOGGER = &mut ($serial) as *mut _;
            defmt_serial::WRITECB = Some(&mut wfn as *mut _ as *mut c_void);
            defmt_serial::WRITEFN = Some(trampoline);

            defmt_serial::critical_section::release(token);
        }
    }};
}

#[global_logger]
pub struct GlobalSerialLogger;

unsafe impl defmt::Logger for GlobalSerialLogger {
    fn acquire() {
        let token = unsafe { critical_section::acquire() };

        if TAKEN.load(Ordering::Relaxed) {
            panic!("defmt logger taken reentrantly");
        }

        TAKEN.store(true, Ordering::Relaxed);

        INTERRUPTS.store(token, Ordering::Relaxed);

        unsafe { ENCODER.start_frame(write_serial) }
    }

    unsafe fn release() {
        ENCODER.end_frame(write_serial);
        TAKEN.store(false, Ordering::Relaxed);
        critical_section::release(INTERRUPTS.load(Ordering::Relaxed));
    }

    unsafe fn write(bytes: &[u8]) {
        ENCODER.write(bytes, write_serial);
    }

    unsafe fn flush() {}
}

/// Write to serial using proxy function. Caller must ensure this function is not called
/// several times in parallel.
fn write_serial(remaining: &[u8]) {
    unsafe {
        if let Some(wfn) = WRITEFN {
            wfn(remaining);
        }
    }
}
