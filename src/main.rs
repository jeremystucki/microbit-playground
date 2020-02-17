#![no_std]
#![no_main]

extern crate nrf51_hal;

mod display;

use bare_metal::{CriticalSection, Mutex};
use core::cell::RefCell;
use core::ops::{Deref, DerefMut};
use core::panic::PanicInfo;
use cortex_m_rt::entry;
use display::Display;
use heapless::consts::U10;
use heapless::Vec;
use nrf51_hal::delay::Delay;
use nrf51_hal::lo_res_timer::{LoResTimer, FREQ_16HZ};
use nrf51_hal::nrf51::*;
use nrf51_hal::prelude::*;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

static ZERO_FRAME: [[u8; 5]; 5] = [
    [0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0],
];

static RTC: Mutex<RefCell<Option<LoResTimer<RTC0>>>> = Mutex::new(RefCell::new(None));
static COUNTER: Mutex<RefCell<u8>> = Mutex::new(RefCell::new(0));

static CONTENT_TO_DISPLAY: Mutex<RefCell<Vec<[u8; 5], U10>>> =
    Mutex::new(RefCell::new(Vec(heapless::i::Vec::new())));
static CURRENT_INDEX: Mutex<RefCell<usize>> = Mutex::new(RefCell::new(0));
static CURRENT_FRAME: Mutex<RefCell<[[u8; 5]; 5]>> = Mutex::new(RefCell::new(ZERO_FRAME));

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

    peripherals
        .CLOCK
        .tasks_lfclkstart
        .write(|w| unsafe { w.bits(1) });
    while peripherals.CLOCK.events_lfclkstarted.read().bits() == 0 {}
    peripherals.CLOCK.events_lfclkstarted.reset();

    let mut rtc0 = LoResTimer::new(peripherals.RTC0);
    rtc0.set_frequency(FREQ_16HZ);
    rtc0.enable_tick_event();
    rtc0.enable_tick_interrupt();
    rtc0.start();

    cortex_m::interrupt::free(|cs| {
        *RTC.borrow(cs).borrow_mut() = Some(rtc0);

        let mut content = CONTENT_TO_DISPLAY.borrow(cs).borrow_mut();
        content.push([1, 1, 1, 1, 1]).unwrap();
        content.push([0, 0, 0, 0, 0]).unwrap();
    });

    unsafe { NVIC::unmask(Interrupt::RTC0) }

    loop {
        let frame = cortex_m::interrupt::free(|cs| *CURRENT_FRAME.borrow(cs).borrow().deref());

        display.display(&mut delay, frame);
    }
}

#[interrupt]
fn RTC0() {
    cortex_m::interrupt::free(|cs| {
        let mut counter = COUNTER.borrow(cs).borrow_mut();

        increase_counter(counter.deref_mut());

        if *counter == 0 {
            scroll_text(cs);
        }

        // TODO: Use unwrap
        if let Some(rtc) = RTC.borrow(cs).borrow_mut().deref_mut() {
            rtc.clear_tick_event();
        }
    });
}

fn increase_counter(counter: &mut u8) {
    if *counter > 10 {
        *counter = 0;
    } else {
        *counter += 1;
    }
}

fn scroll_text(cs: &CriticalSection) {
    let content = CONTENT_TO_DISPLAY.borrow(cs).borrow();

    if content.is_empty() {
        return;
    }

    let mut current_index = CURRENT_INDEX.borrow(cs).borrow_mut();

    if *current_index == content.len() - 1 {
        *current_index = 0;
    } else {
        *current_index += 1;
    }

    let mut current_frame = CURRENT_FRAME.borrow(cs).borrow_mut();

    *current_frame = [
        current_frame[1],
        current_frame[2],
        current_frame[3],
        current_frame[4],
        content[*current_index],
    ]
}
