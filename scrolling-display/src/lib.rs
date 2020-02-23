#![no_std]

extern crate nrf51_hal;

mod characters;

use bare_metal::Mutex;
use characters::*;
use core::cell::RefCell;
use core::ops::{Deref, DerefMut};
use cortex_m::interrupt::CriticalSection;
use heapless::consts::U32;
use heapless::Vec;
use microbit_simple_display::Display;
use nrf51_hal::delay::Delay;
use nrf51_hal::gpio::{gpio::*, Output, PushPull};
use nrf51_hal::lo_res_timer::{LoResTimer, FREQ_8HZ};
use nrf51_hal::nrf51::*;

type ImageBuffer = Vec<[u8; 5], U32>;

static ZERO_FRAME: [[u8; 5]; 5] = [[0; 5]; 5];

static TIMER: Mutex<RefCell<Option<LoResTimer<RTC1>>>> = Mutex::new(RefCell::new(None));

static CURRENT_INDEX: Mutex<RefCell<usize>> = Mutex::new(RefCell::new(usize::max_value()));
static CURRENT_FRAME: Mutex<RefCell<[[u8; 5]; 5]>> = Mutex::new(RefCell::new(ZERO_FRAME));

static IMAGE_BUFFER: Mutex<RefCell<ImageBuffer>> =
    Mutex::new(RefCell::new(Vec(heapless::i::Vec::new())));

pub struct ScrollingDisplay {
    display: Display,
}

impl ScrollingDisplay {
    pub fn new(
        rtc: RTC1,
        row1: PIN4<Output<PushPull>>,
        row2: PIN5<Output<PushPull>>,
        row3: PIN6<Output<PushPull>>,
        row4: PIN7<Output<PushPull>>,
        row5: PIN8<Output<PushPull>>,
        row6: PIN9<Output<PushPull>>,
        row7: PIN10<Output<PushPull>>,
        row8: PIN11<Output<PushPull>>,
        row9: PIN12<Output<PushPull>>,
        col1: PIN13<Output<PushPull>>,
        col2: PIN14<Output<PushPull>>,
        col3: PIN15<Output<PushPull>>,
    ) -> Self {
        let mut timer = LoResTimer::new(rtc);
        timer.set_frequency(FREQ_8HZ);
        timer.enable_tick_event();
        timer.enable_tick_interrupt();
        timer.start();

        cortex_m::interrupt::free(|cs| *TIMER.borrow(cs).borrow_mut() = Some(timer));

        unsafe { NVIC::unmask(Interrupt::RTC1) }

        Self {
            display: Display::new(
                row1, row2, row3, row4, row5, row6, row7, row8, row9, col1, col2, col3,
            ),
        }
    }

    pub fn start_display_loop(&mut self, delay: &mut Delay) -> ! {
        loop {
            let frame = cortex_m::interrupt::free(|cs| *CURRENT_FRAME.borrow(cs).borrow().deref());
            self.display.display(delay, frame);
        }
    }
}

pub fn display_text(text: &str) {
    cortex_m::interrupt::free(|cs| {
        let mut image_buffer = IMAGE_BUFFER.borrow(cs).borrow_mut();

        image_buffer.clear();
        add_content_to_image_buffer(text, image_buffer.deref_mut());

        image_buffer.extend_from_slice(&ZERO_FRAME).unwrap();
        *CURRENT_INDEX.borrow(cs).borrow_mut() = usize::max_value();
    });
}

pub fn display_number(number: u16) {
    cortex_m::interrupt::free(|cs| {
        let mut image_buffer = IMAGE_BUFFER.borrow(cs).borrow_mut();

        image_buffer.clear();
        add_number_to_image_buffer(number, image_buffer.deref_mut());

        image_buffer.extend_from_slice(&ZERO_FRAME).unwrap();
        *CURRENT_INDEX.borrow(cs).borrow_mut() = usize::max_value();
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

fn add_content_to_image_buffer(content: &str, image_buffer: &mut ImageBuffer) {
    content.chars().for_each(|character| {
        if let Some(image) = get_single_column_character(character) {
            image_buffer.push(image).unwrap();
        } else if let Some(image) = get_double_column_character(character) {
            image_buffer.extend_from_slice(&image).unwrap();
        } else if let Some(image) = get_triple_column_character(character) {
            image_buffer.extend_from_slice(&image).unwrap();
        }

        image_buffer.push([0; 5]).unwrap();
    });
}

#[interrupt]
fn RTC1() {
    cortex_m::interrupt::free(|cs| {
        scroll_text(cs);

        if let Some(rtc) = TIMER.borrow(cs).borrow_mut().deref_mut() {
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
