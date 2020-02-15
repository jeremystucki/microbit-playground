use nrf51_hal::delay::Delay;
use nrf51_hal::gpio::{gpio::*, Output, PushPull};
use nrf51_hal::prelude::*;

type LED = PIN<Output<PushPull>>;

pub struct Display {
    rows: [LED; 9],
    columns: [LED; 3],
}

impl Display {
    pub fn new(
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
        Display {
            rows: [
                row1.into(),
                row2.into(),
                row3.into(),
                row4.into(),
                row5.into(),
                row6.into(),
                row7.into(),
                row8.into(),
                row9.into(),
            ],
            columns: [col1.into(), col2.into(), col3.into()],
        }
    }

    pub fn display(&mut self, delay: &mut Delay, frame: [[u8; 5]; 5]) {
        let frame = map_frame(frame);

        for (column_index, column_pin) in self.columns.iter_mut().enumerate() {
            column_pin.set_high().unwrap();

            for (row_index, row_pin) in self.rows.iter_mut().enumerate() {
                let value = frame[row_index][column_index];

                if value != 0 {
                    row_pin.set_low().unwrap();
                }
            }

            delay.delay_ms(5_u8);

            self.rows
                .iter_mut()
                .map(LED::set_high)
                .for_each(Result::unwrap);

            column_pin.set_low().unwrap();
        }
    }
}

fn map_frame(frame: [[u8; 5]; 5]) -> [[u8; 3]; 9] {
    let mut internal_frame = [[0_u8; 3]; 9];

    for row in 0..5 {
        for column in 0..5 {
            let position = map_led_position(row, column);
            internal_frame[position.0][position.1] = frame[row][column];
        }
    }

    internal_frame
}

fn map_led_position(row: usize, column: usize) -> (usize, usize) {
    match (row, column) {
        (0, 0) => (0, 0),
        (0, 1) => (3, 1),
        (0, 2) => (1, 0),
        (0, 3) => (4, 1),
        (0, 4) => (2, 0),
        (1, column) => (3 + column, 2),
        (2, 0) => (1, 1),
        (2, 1) => (8, 0),
        (2, 2) => (2, 1),
        (2, 3) => (8, 2),
        (2, 4) => (0, 1),
        (3, column) => (7 - column, 0),
        (4, 0) => (2, 2),
        (4, 1) => (6, 1),
        (4, 2) => (0, 2),
        (4, 3) => (5, 1),
        (4, 4) => (1, 2),
        _ => unreachable!(),
    }
}
