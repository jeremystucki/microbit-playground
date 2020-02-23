#![no_std]
#![no_main]

extern crate nrf51_hal;

use bare_metal::Mutex;
use core::cell::RefCell;
use core::ops::DerefMut;
use core::panic::PanicInfo;
use cortex_m_rt::entry;
use microbit_scrolling_display::ScrollingDisplay;
use nrf51_hal::delay::Delay;
use nrf51_hal::lo_res_timer::{LoResTimer, FREQ_8HZ};
use nrf51_hal::nrf51::*;
use nrf51_hal::prelude::*;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

static RTC_0: Mutex<RefCell<Option<LoResTimer<RTC0>>>> = Mutex::new(RefCell::new(None));
static RTC_0_COUNTER: Mutex<RefCell<u16>> = Mutex::new(RefCell::new(0));

static COUNTDOWN_HOURS: Mutex<RefCell<u16>> = Mutex::new(RefCell::new(651));

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

    peripherals
        .CLOCK
        .tasks_lfclkstart
        .write(|w| unsafe { w.bits(1) });
    while peripherals.CLOCK.events_lfclkstarted.read().bits() == 0 {}
    peripherals.CLOCK.events_lfclkstarted.reset();

    let mut rtc0 = LoResTimer::new(peripherals.RTC0);
    rtc0.set_frequency(FREQ_8HZ);
    rtc0.enable_tick_event();
    rtc0.enable_tick_interrupt();
    rtc0.start();

    let mut scrolling_display = ScrollingDisplay::new(
        peripherals.RTC1,
        row1,
        row2,
        row3,
        row4,
        row5,
        row6,
        row7,
        row8,
        row9,
        col1,
        col2,
        col3,
    );

    cortex_m::interrupt::free(|cs| {
        *RTC_0.borrow(cs).borrow_mut() = Some(rtc0);

        let countdown_hours = COUNTDOWN_HOURS.borrow(cs).borrow();
        microbit_scrolling_display::display_number((*countdown_hours) / 24);
    });

    unsafe { NVIC::unmask(Interrupt::RTC0) }

    let mut delay = Delay::new(peripherals.TIMER0);
    scrolling_display.start_display_loop(&mut delay);
}

#[interrupt]
fn RTC0() {
    cortex_m::interrupt::free(|cs| {
        if let Some(rtc) = RTC_0.borrow(cs).borrow_mut().deref_mut() {
            let mut counter = RTC_0_COUNTER.borrow(cs).borrow_mut();

            *counter += 1;

            if *counter == 28_800 {
                *counter = 0;

                let mut countdown_hours = COUNTDOWN_HOURS.borrow(cs).borrow_mut();
                *countdown_hours -= 1;

                if *countdown_hours == 0 {
                    rtc.stop();
                }

                microbit_scrolling_display::display_number((*countdown_hours) / 24);
            }

            rtc.clear_tick_event();
        }
    });
}
