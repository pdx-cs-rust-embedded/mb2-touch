#![no_main]
#![no_std]

use panic_rtt_target as _;
use embedded_hal::{delay::DelayNs, digital::InputPin};
use microbit::hal::gpio;

pub type TouchPin = gpio::Pin<gpio::Input<gpio::Floating>>;

pub struct Touchpad<T> {
    pin: Option<TouchPin>,
    timer: T,
}

impl<T: DelayNs> Touchpad<T> {
    pub fn new(
        pin: TouchPin,
        timer: T,
    ) -> Self {
        Touchpad { pin: Some(pin), timer }
    }

    pub fn sense(&mut self, max_ticks: u32) -> u32 {
        let pin = self.pin.take().unwrap();
        let touch_pin = pin.into_push_pull_output(gpio::Level::Low);
        self.timer.delay_ms(10);
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
