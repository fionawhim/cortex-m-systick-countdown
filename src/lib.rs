#![no_std]
#![warn(clippy::all)]

//! Wrappers around the Cortex-M SysTick peripheral for making
//! [`embedded_hal::timer::CountDown`](https://docs.rs/embedded-hal/0.2.3/embedded_hal/timer/trait.CountDown.html)
//! instances.
//!
//! The `CountDown` trait is by default non-blocking, but can be made blocking
//! with [`nb::block!`](https://docs.rs/nb/0.1.2/nb/macro.block.html).
//!
//! ## Usage
//!
//! Create an instance of [`PollingSysTick`](struct.PollingSysTick.html) after
//! you have configured your clocks. It consumes the `SYST` peripheral in order
//! to get exclusive control over it.
//!
//! You can use the [`embedded_hal::blocking::delay::DelayMs`
//! trait](https://docs.rs/embedded-hal/0.2.3/embedded_hal/blocking/delay/trait.DelayMs.html)
//! on `PollingSysTick` directly, or you can use `PollingSysTick` to make
//! `MillisCountDown` instances that are independent, non-blocking
//! counters.

use core::{num::Wrapping, time::Duration};

use cortex_m::peripheral::{syst::SystClkSource, SYST};

use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::timer::CountDown;

use nb;
use void::Void;

use core::cell::UnsafeCell;

/// Trait that abstracts a counter that increases as milliseconds go by.
///
/// Factored out to leave the door open for different SysTick counters, such as
/// counting via interrupts.
pub trait CountsMillis {
    /// Returns a value that must not increment faster than once per
    /// millisecond, and will wrap around.
    fn count(&self) -> Wrapping<u32>;
}

/// Configuration information for setting the SysTick reload value.
pub struct SysTickCalibration {
    /// The number of ticks of the SysTick’s clock source to get to 1ms.
    ///
    /// Note that reload values should typically be one fewer than the number of
    /// clock cycles, since an extra one is always needed to detect the rollover
    /// and reload the counter.
    pub ticks_per_ms: u32,
}

impl SysTickCalibration {
    /// Gets the calibration from the chip’s built-in "ticks per 10ms" value.
    ///
    /// Returns an `Option` because this value is is not present on all devices
    /// (SAMD51 for example seems to not have it, since its processor speed is
    /// configurable). In those cases, use
    /// [`from_clock_hz`](#method.from_clock_hz) instead.
    pub fn built_in() -> Option<SysTickCalibration> {
        let calibrated_tick_value = SYST::get_ticks_per_10ms();

        if calibrated_tick_value == 0 {
            None
        } else {
            Some(SysTickCalibration {
                // Leave one clock cycle for checking the overflow
                ticks_per_ms: (calibrated_tick_value + 1) / 10 - 1,
            })
        }
    }

    /// Creates a calibration from the underlying frequency of the clock that
    /// drives SysTick. This typically seems to be the same frequency that the
    /// processor is currently running at.
    ///
    /// For SAMD51 processors, if you don’t want to hard-code a known size, you
    /// can get this from the gclck0 frequency.
    pub fn from_clock_hz(hz: u32) -> SysTickCalibration {
        SysTickCalibration {
            ticks_per_ms: hz / 1_000 - 1,
        }
    }
}

/// Millisecond counter based on SysTick
///
/// Effectively a singleton because this struct will consume the only SYST value
/// in the program. (Use [`free`](#method.free) if you need to get it back.)
///
/// ## Usage
///
/// For simple blocking delays, use the
/// [`embedded_hal::blocking::delay::DelayMs`
/// trait](https://docs.rs/embedded-hal/0.2.3/embedded_hal/blocking/delay/trait.DelayMs.html)
/// to pause the program for a certain amount of time.
///
/// For timeouts or other non-blocking operations, create a
/// [`MillisCountDown`](struct.MillisCountDown.html) instance and use its
/// `start` and `wait` methods. You can have multiple `MillisCountDown`
/// instances active at the same time.
///
/// Because this uses polling for measuring SysTick, it will work even during
/// periods where interrupts are disabled.
///
/// ## Implementation
///
/// We configure SysTick’s reload value to a count that will take 1ms to
/// decrement to. When we detect that this count has wrapped over we increment
/// our internal count of the milliseconds that have ellapsed.
///
/// We use the polling pattern for querying the time, rather than relying on
/// interrupts, which means that our count is only guaranteed to be _no faster_
/// than SysTick. We only keep accurate count while the [`count`](#method.count)
/// method is being actively called, and may experience some jitter depending on
/// where SysTick is in its count when you start a timer.
///
/// This also means we need to use internal mutability so that we can access the
/// SYST.has_wrapped() method (which mutates on read) and update our counter.
pub struct PollingSysTick {
    syst: UnsafeCell<SYST>,
    counter: UnsafeCell<Wrapping<u32>>,
}

impl PollingSysTick {
    /// Configures SysTick based on the values provided in the calibration.
    pub fn new(mut syst: SYST, calibration: &SysTickCalibration) -> Self {
        syst.disable_interrupt();
        syst.set_clock_source(SystClkSource::Core);
        syst.set_reload(calibration.ticks_per_ms);
        syst.enable_counter();

        PollingSysTick {
            syst: UnsafeCell::new(syst),
            counter: UnsafeCell::default(),
        }
    }

    /// Turns this value back into the underlying SysTick.
    pub fn free(self) -> SYST {
        self.syst.into_inner()
    }
}

impl CountsMillis for PollingSysTick {
    /// Returns a number that goes up no faster than once per millisecond. This
    /// value will not increment unless polled (this is so it can operate
    /// during critical sections when interrupts are disabled).
    fn count(&self) -> Wrapping<u32> {
        // This is all unsafe because incrementing the internal count happens as
        // a side effect of reading it. We’re ok with that, because we know that
        // we have sole control over the SYST singleton, so we’re the only ones
        // who will see the wrapping.
        if unsafe { (*self.syst.get()).has_wrapped() } {
            // Disabled interrupts because += is non-atomic.
            cortex_m::interrupt::free(|_| unsafe {
                (*self.counter.get()) += Wrapping(1);
            });
        }

        unsafe { *self.counter.get() }
    }
}

impl DelayMs<u32> for PollingSysTick {
    fn delay_ms(&mut self, ms: u32) {
        let mut count_down = MillisCountDown::new(self);
        count_down.start_ms(ms);
        nb::block!(count_down.wait()).unwrap();
    }
}

/// `CountDown` that uses an underlying `CountsMillis` (probably
/// `PollingSysTick`).
pub struct MillisCountDown<'a, CM: CountsMillis> {
    counter: &'a CM,
    target_millis: Option<Wrapping<u32>>,
}

impl<'a, CM: CountsMillis> MillisCountDown<'a, CM> {
    /// Creates a `MillisCountDown` from a `CountsMillis` source.
    ///
    /// `CountsMillis` is probably going to be your instance of
    /// [`PollingSysTick`](struct.PollingSysTick.html).
    pub fn new(counter: &'a CM) -> Self {
        MillisCountDown {
            target_millis: None,
            counter,
        }
    }

    /// Underlying version of `CountDown`’s `start` that takes a `u32` of
    /// milliseconds rather than a `Duration`.
    ///
    /// Use this if you want to avoid the `u64`s in `Duration`.
    pub fn start_ms(&mut self, ms: u32) {
        self.target_millis = Some(self.counter.count() + Wrapping(ms));
    }

    /// Underlying implementation of `CountDown`’s `wait` that works directly on
    /// our underlying u32 ms values and can be used by any `CountDown` trait
    /// implementations.
    ///
    /// Calling this method before `start`, or after it has already returned
    /// `Ok` will panic.
    pub fn wait_ms(&mut self) -> Result<(), nb::Error<Void>> {
        // Rollover-safe duration check derived from:
        // https://playground.arduino.cc/Code/TimingRollover/
        if (self.counter.count() - self.target_millis.unwrap()).0 as i32 > 0 {
            self.target_millis.take();
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl<'a, CM: CountsMillis> CountDown for MillisCountDown<'a, CM> {
    type Time = Duration;

    /// Starts timing the given `Duration`.
    ///
    /// [`wait`](#method.wait) will return
    /// [`nb::Error::WouldBlock`](https://docs.rs/nb/0.1.2/nb/enum.Error.html#variant.WouldBlock)
    /// until this amount of time has passed.
    ///
    /// Calling this method before the time has fully ellapsed will reset the
    /// timer.
    fn start<T>(&mut self, count: T)
    where
        T: Into<Self::Time>,
    {
        let dur: Self::Time = count.into();
        let millis = (dur.as_secs() as u32) * 1000 + dur.subsec_millis() as u32;
        self.start_ms(millis);
    }

    /// Returns
    /// [`nb::Error::WillBlock`](https://docs.rs/nb/0.1.2/nb/enum.Error.html#variant.WouldBlock)
    /// while the timer runs, then will return `Result::Ok`.
    ///
    /// Calling this method before `start`, or after it has already returned
    /// `Ok` will panic.
    fn wait(&mut self) -> Result<(), nb::Error<Void>> {
        self.wait_ms()
    }
}
