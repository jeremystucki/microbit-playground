#![no_std]
#![no_main]

extern crate nrf51_hal;

use core::panic::PanicInfo;
use cortex_m_rt::entry;
use nrf51_hal::gpio::gpio::*;
use nrf51_hal::prelude::*;
use nrf51_hal::nrf51::Peripherals;
use nrf51_hal::gpio::{Output, Floating, PushPull};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop { }
}

#[entry]
fn main() -> ! {
    // CLion can't infer certain types, so I added type hints

    let peripherals = Peripherals::take().unwrap();
    let gpio: Parts = peripherals.GPIO.split();

    let mut pin4: PIN4<Output<PushPull>> = gpio.pin4.into_push_pull_output();
    let mut pin13: PIN13<Output<PushPull>> = gpio.pin13.into_push_pull_output();

    pin4.set_low();
    pin13.set_high();

    loop { }
}
