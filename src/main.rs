#![no_main]
#![no_std]

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use cortex_m_rt::entry;
use embedded_hal::{delay::DelayNs, digital::InputPin};
use microbit::{
    board::Board,
    hal::{gpio, timer},
};

/// Minimum charging time in microseconds to regard as
/// "touched".
const TOUCH_THRESHOLD: usize = 100;

/// Time in milliseconds to discharge the touchpad before
/// testing.
const DISCHARGE_TIME: u32 = 100;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = Board::take().unwrap();
    let mut touch_pin = board.pins.p1_04.into_push_pull_output(gpio::Level::Low);
    let mut timer = timer::Timer::new(board.TIMER0);
    // True for touched.
    let mut state = false;

    timer.delay_ms(500);
    loop {
        // Count the number of microseconds for the touchpad
        // to charge to the point the GPIO pin sees it as
        // high.
        let mut touch_pin_input = touch_pin.into_floating_input();
        let mut new_state = true;
        for _ in 0..TOUCH_THRESHOLD {
            if touch_pin_input.is_high().unwrap() {
                new_state = false;
                break;
            }
            timer.delay_us(1);
        }
        if new_state != state {
            match new_state {
                true => rprintln!("touched"),
                false => rprintln!("released"),
            }
        }
        state = new_state;

        // Pull the touchpad to ground to discharge any accumulated
        // voltage. Allow time to settle.
        touch_pin = touch_pin_input.into_push_pull_output(gpio::Level::Low);
        timer.delay_ms(DISCHARGE_TIME);
    }
}
