#![no_std]
#![no_main]

extern crate panic_halt;

use cortex_m_systick_countdown::{ MillisCountDown, PollingSysTick, SysTickCalibration};

use core::time::Duration;
use embedded_hal::{blocking::delay::DelayMs, timer::CountDown};

use cortex_m_rt::entry;
use cortex_m::peripheral::Peripherals;
use nb::block;

use cortex_m_semihosting::{debug, hprintln};

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();

    // The qemu lm3s6965evb has the clock calibration defined, so we can use it.
    // For some other devices (including the SAMD51) you will need to use
    // SysTickCalibration::from_clock_hz.
    let mut counter =
        PollingSysTick::new(peripherals.SYST, &SysTickCalibration::built_in().unwrap());

    hprintln!("Delaying 1s…").unwrap();
    counter.delay_ms(1_000);

    hprintln!("Delaying 2s…").unwrap();
    counter.delay_ms(2_000);

    hprintln!("Looping for 10s…").unwrap();

    let mut count_10s = MillisCountDown::new(&counter);
    let mut count_500ms = MillisCountDown::new(&counter);

    count_10s.start(Duration::from_secs(10));

    loop {
        if let Ok(_) = count_10s.wait() {
            break;
        }

        hprintln!("Not yet.").unwrap();
        count_500ms.start(Duration::from_millis(500));
        block!(count_500ms.wait()).unwrap();
    }

    hprintln!("All done!").unwrap();

    debug::exit(debug::EXIT_SUCCESS);

    loop {}
}
