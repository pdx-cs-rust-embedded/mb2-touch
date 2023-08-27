#![no_main]
#![no_std]

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use cortex_m_rt::entry;
use microbit::{
    board::Board,
    hal::{prelude::*, gpio, timer},
};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = Board::take().unwrap();
    let mut touch_pin = board.pins.p1_04.into_floating_input();
    let mut timer = timer::Timer::new(board.TIMER0);

    loop {
        let touch_pin_i = touch_pin.into_push_pull_output(gpio::Level::Low);
        timer.delay_ms(10u16);
        touch_pin = touch_pin_i.into_floating_input();
        let mut count = 0u32;
        while touch_pin.is_low().unwrap() {
            timer.delay_us(1u16);
            count += 1;
        }
        rprintln!("{}", count);
        timer.delay_ms(500u16);
    }
}
