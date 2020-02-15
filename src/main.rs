#![no_std]
#![no_main]

extern crate nrf51_hal;

mod display;

use core::panic::PanicInfo;
use cortex_m_rt::entry;
use display::Display;
use nrf51_hal::delay::Delay;
use nrf51_hal::gpio::gpio::Parts;
use nrf51_hal::nrf51::Peripherals;
use nrf51_hal::prelude::*;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();
    let gpio = peripherals.GPIO.split();

    let row1 = gpio.pin4.into_push_pull_output();
    let row2 = gpio.pin5.into_push_pull_output();
    let row3 = gpio.pin6.into_push_pull_output();
    let row4 = gpio.pin7.into_push_pull_output();
    let row5 = gpio.pin8.into_push_pull_output();
    let row6 = gpio.pin9.into_push_pull_output();
    let row7 = gpio.pin10.into_push_pull_output();
    let row8 = gpio.pin11.into_push_pull_output();
    let row9 = gpio.pin12.into_push_pull_output();

    let col1 = gpio.pin13.into_push_pull_output();
    let col2 = gpio.pin14.into_push_pull_output();
    let col3 = gpio.pin15.into_push_pull_output();

    let mut display = Display::new(
        row1, row2, row3, row4, row5, row6, row7, row8, row9, col1, col2, col3,
    );

    let mut delay = Delay::new(peripherals.TIMER0);

    loop {
        display.display(
            &mut delay,
            [
                [0, 1, 0, 1, 0],
                [1, 1, 1, 1, 1],
                [1, 1, 1, 1, 1],
                [0, 1, 1, 1, 0],
                [0, 0, 1, 0, 0],
            ],
        );
    }
}
