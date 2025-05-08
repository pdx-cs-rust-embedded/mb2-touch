#![no_main]
#![no_std]

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use cortex_m_rt::entry;
use embedded_hal::delay::DelayNs;
use microbit::{board::Board, hal::timer};

use mb2_touch::*;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = Board::take().unwrap();
    let touch_pin = board.pins.p1_04.into_floating_input();
    let timer0 = timer::Timer::new(board.TIMER0);
    let mut timer1 = timer::Timer::new(board.TIMER1);
    let mut touchpad = Touchpad::new(touch_pin, timer0);

    loop {
        let count = touchpad.sense();
        rprintln!("{}", count);
        timer1.delay_ms(500);
    }
}
