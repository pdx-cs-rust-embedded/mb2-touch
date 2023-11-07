#![no_main]
#![no_std]

use core::marker::PhantomData;

use microbit::hal::{prelude::*, pac, gpio, timer};

pub enum TouchpadIdle {}
pub enum TouchpadSetup {}
pub enum TouchpadSense {}

pub trait TouchpadState {}

impl TouchpadState for TouchpadIdle {}
impl TouchpadState for TouchpadSetup {}
impl TouchpadState for TouchpadSense {}

pub struct Touchpad<T: TouchpadState> {
    pin: gpio::p1::P1_04<gpio::Input<gpio::Floating>>,
    timer: timer::Timer<pac::TIMER0>,
    p: PhantomData<T>,
}

impl Touchpad<TouchpadIdle> {
    pub fn new(
        pin: gpio::p1::P1_04<gpio::Input<gpio::Floating>>,
        timer: timer::Timer<pac::TIMER0>,
    ) -> Self {
        Touchpad { pin, timer, p: PhantomData }
    }
}


impl Touchpad<TouchpadIdle> {
    pub fn setup(mut self) -> Touchpad<TouchpadSense> {
        let touch_pin = self.pin.into_push_pull_output(gpio::Level::Low);
        self.timer.delay_ms(10u16);
        let touch_pin = touch_pin.into_floating_input();
        let touchpad = Touchpad {
            pin: touch_pin,
            timer: self.timer,
            p: PhantomData,
        };
        touchpad
    }
}

impl Touchpad<TouchpadSense> {
    pub fn sense(mut self) -> (u32, Touchpad<TouchpadIdle>) {
        let mut count = 0u32;
        while self.pin.is_low().unwrap() {
            self.timer.delay_us(1u16);
            count += 1;
        }
        let touchpad = Touchpad {
            pin: self.pin,
            timer: self.timer,
            p: PhantomData,
        };
        (count, touchpad)
    }
}
