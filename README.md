[![Crates.io](https://img.shields.io/crates/v/defmt-serial.svg)](https://crates.io/crates/defmt-serial)
[![Documentation](https://docs.rs/defmt-serial/badge.svg)](https://docs.rs/defmt-serial/)
[![Rust nightly](https://img.shields.io/badge/rustc-nightly-orange)](https://rust-lang.github.io/rustup/installation/other.html)

# defmt-serial

A [defmt](https://github.com/knurling-rs/defmt) target for logging over a serial
port. Messages can e.g. be read using `socat` and passed through `defmt-print`,
see [example-artemis](example-artemis) for how to do that. You can also try it
out in a hosted environment: [example-std](example-std).
