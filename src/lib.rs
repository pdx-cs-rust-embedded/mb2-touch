#![no_main]
#![no_std]

use panic_rtt_target as _;
use microbit::hal::{prelude::*, pac, gpio, timer};

pub enum TouchpadState {
    Idle,
    Setup,
    Sense,
}

pub struct TouchpadIdle;
pub struct TouchpadSetup;
pub struct TouchpadSense;

pub trait TouchpadState {}

impl TouchpadState for TouchpadIdle {}

pub struct Touchpad<T: TouchpadState> {
    pin: Option<gpio::p1::P1_04<gpio::Input<gpio::Floating>>>,
    timer: timer::Timer<pac::TIMER0>,
}

impl Touchpad {
    pub fn new(
        pin: gpio::p1::P1_04<gpio::Input<gpio::Floating>>,
        timer: timer::Timer<pac::TIMER0>,
    ) -> Self {
        Touchpad { pin: Some(pin), timer }
    }

    pub fn sense(&mut self) -> u32 {
        let pin = self.pin.take().unwrap();
        let touch_pin = pin.into_push_pull_output(gpio::Level::Low);
        self.timer.delay_ms(10u16);
        let touch_pin = touch_pin.into_floating_input();
        let mut count = 0u32;
        while touch_pin.is_low().unwrap() {
            self.timer.delay_us(1u16);
            count += 1;
        }
        self.pin = Some(touch_pin);
        count
    }
}
