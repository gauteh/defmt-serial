[![Crates.io](https://img.shields.io/crates/v/defmt-serial.svg)](https://crates.io/crates/defmt-serial)
[![Documentation](https://docs.rs/defmt-serial/badge.svg)](https://docs.rs/defmt-serial/)
[![tests](https://github.com/gauteh/defmt-serial/actions/workflows/rust.yml/badge.svg)](https://github.com/gauteh/defmt-serial/actions/workflows/rust.yml)

# defmt-serial

A [defmt](https://github.com/knurling-rs/defmt) target for logging over a serial
port. Have a look at examples to see how to use library
[example-artemis](example-artemis) or [example-pi-pico](example-pi-pico). You
can also try it out in a hosted environment: [example-std](example-std). To
parse the logs have a look at [parsing logs](#Parsing-logs).

```rust
#[entry]
fn main() -> ! {
    let mut dp = hal::pac::Peripherals::take().unwrap();
    let pins = hal::gpio::Pins::new(dp.GPIO);

    // set up serial
    let mut serial = hal::uart::Uart0::new(dp.UART0, pins.tx0, pins.rx0);
    defmt_serial::defmt_serial(serial);

    defmt::info!("Hello from defmt!");

    loop {
        asm::wfi();
    }
}
```

Remember to set the `DEFMT_LOG` variable when testing, e.g.:

```
$ cd example-std/
$ DEFMT_LOG=debug cargo run
```

<img src="example-defmt-serial.png" width="80%"></img>

## Parsing logs

The easiest way to parse the logs is to use `socat` and `defmt-print` together. 
For example: 
```
socat ${PORT},rawer,b${BAUDRATE},crnl STDOUT | defmt-print -e ${ELF}
```
Just replace `${PORT}`, `${BAUDRATE}` and `${ELF}` with correct values.

To install the tools on Ubuntu 22.04 run these commands:
```
apt install socat
cargo install defmt-print
```