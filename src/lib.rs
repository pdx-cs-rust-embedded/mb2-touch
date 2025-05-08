#![no_main]
#![no_std]

/*!
This code demos capacitive touch sensing for the gold robot
"logo pad" and other designated "touch pads" on the MicroBit
v2. It works by driving the pad low for 5ms to drain any
on-board capacitance, then measuring the time for the pad to
charge through the 10MΩ resistor on the board until it reads
high. On my board, it takes less than 50µs to charge with no
finger, about 2.5ms to charge with a finger firmly planted.
*/

use embedded_hal::{delay::DelayNs, digital::InputPin};
use microbit::hal::gpio;
use panic_rtt_target as _;

/// Time in microseconds to wait for pad discharge.
pub const RESET_TIME: u32 = 10;

/// Pins must be degraded to this to be usable. Note that
/// there is no checking for valid pins.
pub type TouchPin = gpio::Pin<gpio::Input<gpio::Floating>>;

/// Touchpad instance.
pub struct Touchpad<T> {
    pin: Option<TouchPin>,
    timer: T,
}

impl<T: DelayNs> Touchpad<T> {
    /// Make a new touchpad driver from the given `pin` and `timer`.
    pub fn new(pin: TouchPin, timer: T) -> Self {
        Touchpad {
            pin: Some(pin),
            timer,
        }
    }

    /// Reset the touchpad, then count and return
    /// its charge time in microseconds. The count
    /// will proceed for no more than `max_ticks`
    /// microseconds.
    pub fn sense(&mut self, max_ticks: u32) -> u32 {
        let pin = self.pin.take().unwrap();
        let touch_pin = pin.into_push_pull_output(gpio::Level::Low);
        self.timer.delay_us(RESET_TIME);
        let mut touch_pin = touch_pin.into_floating_input();
        let mut count = 0u32;
        while count < max_ticks && touch_pin.is_low().unwrap() {
            self.timer.delay_us(1);
            count += 1;
        }
        self.pin = Some(touch_pin);
        count
    }
}
