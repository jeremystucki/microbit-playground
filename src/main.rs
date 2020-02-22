#![no_std]
#![no_main]

extern crate nrf51_hal;

mod display;

use bare_metal::{CriticalSection, Mutex};
use core::cell::RefCell;
use core::ops::{Deref, DerefMut};
use core::panic::PanicInfo;
use cortex_m_rt::entry;
use display::characters::*;
use display::Display;
use heapless::consts::U32;
use heapless::Vec;
use nrf51_hal::delay::Delay;
use nrf51_hal::lo_res_timer::{LoResTimer, FREQ_8HZ};
use nrf51_hal::nrf51::*;
use nrf51_hal::prelude::*;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

type ImageBuffer = Vec<[u8; 5], U32>;

static ZERO_FRAME: [[u8; 5]; 5] = [
    [0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0],
];

static RTC_0: Mutex<RefCell<Option<LoResTimer<RTC0>>>> = Mutex::new(RefCell::new(None));
static RTC_1: Mutex<RefCell<Option<LoResTimer<RTC1>>>> = Mutex::new(RefCell::new(None));

static RTC_1_COUNTER: Mutex<RefCell<u16>> = Mutex::new(RefCell::new(0));
static COUNTDOWN_HOURS: Mutex<RefCell<u16>> = Mutex::new(RefCell::new(674));

static IMAGE_BUFFER: Mutex<RefCell<ImageBuffer>> =
    Mutex::new(RefCell::new(Vec(heapless::i::Vec::new())));
static CURRENT_INDEX: Mutex<RefCell<usize>> = Mutex::new(RefCell::new(127)); // TODO
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
    rtc0.set_frequency(FREQ_8HZ);
    rtc0.enable_tick_event();
    rtc0.enable_tick_interrupt();
    rtc0.start();

    let mut rtc1 = LoResTimer::new(peripherals.RTC1);
    rtc1.set_frequency(FREQ_8HZ);
    rtc1.enable_tick_event();
    rtc1.enable_tick_interrupt();
    rtc1.start();

    cortex_m::interrupt::free(|cs| {
        *RTC_0.borrow(cs).borrow_mut() = Some(rtc0);
        *RTC_1.borrow(cs).borrow_mut() = Some(rtc1);

        let countdown_hours = COUNTDOWN_HOURS.borrow(cs).borrow();
        let mut image_buffer = IMAGE_BUFFER.borrow(cs).borrow_mut();
        add_number_to_image_buffer((*countdown_hours) / 24, &mut image_buffer);
        image_buffer.extend_from_slice(&ZERO_FRAME).unwrap();
    });

    unsafe { NVIC::unmask(Interrupt::RTC0) }
    unsafe { NVIC::unmask(Interrupt::RTC1) }

    loop {
        let frame = cortex_m::interrupt::free(|cs| *CURRENT_FRAME.borrow(cs).borrow().deref());
        display.display(&mut delay, frame);
    }
}

fn add_content_to_image_buffer(content: &str, image_buffer: &mut ImageBuffer) {
    content.chars().for_each(|character| {
        if let Some(image) = get_single_column_character(character) {
            image_buffer.push(image).unwrap();
        } else if let Some(image) = get_double_column_character(character) {
            image_buffer.extend_from_slice(&image).unwrap();
        } else if let Some(image) = get_triple_column_character(character) {
            image_buffer.extend_from_slice(&image).unwrap();
        }

        image_buffer.push([0, 0, 0, 0, 0]).unwrap();
    });
}

fn add_number_to_image_buffer(number: u16, image_buffer: &mut ImageBuffer) {
    let remainder = number % 10;

    let content = match remainder {
        0 => "0",
        1 => "1",
        2 => "2",
        3 => "3",
        4 => "4",
        5 => "5",
        6 => "6",
        7 => "7",
        8 => "8",
        9 => "9",
        _ => unreachable!(),
    };

    let remaining_number = number - remainder;
    if remaining_number != 0 {
        add_number_to_image_buffer(remaining_number / 10, image_buffer);
    }

    add_content_to_image_buffer(content, image_buffer);
}

#[interrupt]
fn RTC0() {
    cortex_m::interrupt::free(|cs| {
        scroll_text(cs);

        // TODO: Use unwrap
        if let Some(rtc) = RTC_0.borrow(cs).borrow_mut().deref_mut() {
            rtc.clear_tick_event();
        }
    });
}

#[interrupt]
fn RTC1() {
    cortex_m::interrupt::free(|cs| {
        if let Some(rtc) = RTC_1.borrow(cs).borrow_mut().deref_mut() {
            let mut counter = RTC_1_COUNTER.borrow(cs).borrow_mut();

            *counter += 1;

            if *counter == 28_800 {
                *counter = 0;

                let mut countdown_hours = COUNTDOWN_HOURS.borrow(cs).borrow_mut();
                *countdown_hours -= 1;

                if *countdown_hours == 0 {
                    rtc.stop();
                    rtc.disable_tick_interrupt();
                    rtc.disable_tick_event();
                }

                let mut image_buffer = IMAGE_BUFFER.borrow(cs).borrow_mut();

                image_buffer.clear();
                add_number_to_image_buffer((*countdown_hours) / 24, image_buffer.deref_mut());
                image_buffer.extend_from_slice(&ZERO_FRAME).unwrap();

                let mut current_index = CURRENT_INDEX.borrow(cs).borrow_mut();
                *current_index = 0;
            }

            // TODO: Use unwrap
            rtc.clear_tick_event();
        }
    });
}

fn scroll_text(cs: &CriticalSection) {
    let image_buffer = IMAGE_BUFFER.borrow(cs).borrow();

    if image_buffer.is_empty() {
        return;
    }

    let mut current_index = CURRENT_INDEX.borrow(cs).borrow_mut();

    if *current_index >= image_buffer.len() - 1 {
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
        image_buffer[*current_index],
    ]
}
