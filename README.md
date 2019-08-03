# cortex-m-systick-countdown

This crate provides `PollingSysTick`, a wrapper around the Cortex-M SysTick
peripheral that makes it easy to get values of the
`embedded_hal::timer::CountDown` trait.

Unlike the `atsamd-hal` `Delay` struct, `PollingSysTick` is non-blocking and
lets you have multiple separate `CountDown` values at once.

## Documentation

See the Rust documentation.

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