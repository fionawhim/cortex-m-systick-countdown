# cortex-m-systick-countdown

[![Crates.io](https://img.shields.io/crates/v/cortex-m-systick-countdown.svg)](https://crates.io/crates/cortex-m-systick-countdown)
[![rustdoc](https://docs.rs/cortex-m-systick-countdown/badge.svg)](https://docs.rs/cortex-m-systick-countdown/)
[![Travis Build Status](https://api.travis-ci.org/fionawhim/cortex-m-systick-countdown.svg?branch=develop)](https://travis-ci.org/fionawhim/cortex-m-systick-countdown/)

This crate provides `PollingSysTick`, a wrapper around the Cortex-M SysTick
peripheral that makes it easy to get values of the
`embedded_hal::timer::CountDown` trait.

Unlike the `atsamd-hal` `Delay` struct, `PollingSysTick` is non-blocking and
lets you have multiple separate `CountDown` values at once.

## Documentation

See the [rustdoc on Docs.rs](https://docs.rs/cortex-m-systick-countdown/).

## Development

There’s an example binary that’s set up for a lm3s6965evb with qemu. Run it with
`cargo run --example qemu`

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.