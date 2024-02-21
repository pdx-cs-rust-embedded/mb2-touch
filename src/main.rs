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
    let mut touch_pin = board.pins.p1_04.into_push_pull_output(gpio::Level::Low);
    let mut timer = timer::Timer::new(board.TIMER0);

    timer.delay_ms(500u16);
    loop {
        // Count the number of microseconds for the touchpad
        // to charge to the point the GPIO pin sees it as
        // high.
        let touch_pin_input = touch_pin.into_floating_input();
        let mut count = 0u32;
        while touch_pin_input.is_low().unwrap() {
            timer.delay_us(1u16);
            count += 1;
        }
        rprintln!("{}", count);

        // Pull the touchpad to ground to discharge any accumulated
        // voltage. Allow time to settle.
        touch_pin = touch_pin_input.into_push_pull_output(gpio::Level::Low);
        timer.delay_ms(500u16);
    }
}
