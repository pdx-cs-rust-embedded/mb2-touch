#![no_main]
#![no_std]

use core::fmt;

use microbit::hal::{
    gpio::{self, p1::P1_04, Floating, Input, Output, Pin, PushPull},
    gpiote::Gpiote,
    pac,
    prelude::*,
    timer,
};
use panic_rtt_target as _;

pub enum TouchpadState {
    Disabled(Pin<Input<Floating>>),
    Idle(Pin<Input<Floating>>),
    Setup(Pin<Output<PushPull>>),
    Sense(Pin<Input<Floating>>, u32),
}

impl fmt::Debug for TouchpadState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Disabled(_) => f.debug_tuple("Disabled").finish(),
            Self::Idle(_) => f.debug_tuple("Idle").finish(),
            Self::Setup(_) => f.debug_tuple("Setup").finish(),
            Self::Sense(_, arg1) => f.debug_tuple("Sense").field(arg1).finish(),
        }
    }
}

pub struct Touchpad {
    timer: timer::Timer<pac::TIMER0>,
    state: Option<TouchpadState>,
    gpiote: Gpiote,
    interrupt: pac::Interrupt,
    threshold: u32,
}

impl Touchpad {
    pub fn new(
        pin: P1_04<Input<Floating>>,
        timer: timer::Timer<pac::TIMER0>,
        gpiote: Gpiote,
        interrupt: pac::Interrupt,
        threshold: u32,
    ) -> Self {
        let pin = pin.degrade();
        gpiote
            .channel0()
            .input_pin(&pin)
            .hi_to_lo()
            .disable_interrupt();

        let mut t = Touchpad {
            state: Some(TouchpadState::Idle(pin)),
            timer,
            gpiote,
            interrupt,
            threshold,
        };
        t.handle_timer_interrupt();
        t
    }

    pub fn handle_timer_interrupt(&mut self) {
        self.timer.event_compare_cc0().reset();
        let new_state = match self.state.take().unwrap() {
            TouchpadState::Idle(pin) => {
                let pin = pin.into_push_pull_output(gpio::Level::Low);
                self.timer.enable_interrupt();
                self.timer.start(10000u32); // TODO correct value
                TouchpadState::Setup(pin)
            }
            TouchpadState::Setup(pin) => {
                self.timer.start(1u32);
                self.timer.enable_interrupt();
                let pin = pin.into_floating_input();
                self.gpiote
                    .channel0()
                    .input_pin(&pin)
                    .hi_to_lo()
                    .enable_interrupt();
                TouchpadState::Sense(pin, 0)
            }
            TouchpadState::Sense(pin, count) => {
                self.timer.start(1u32);
                self.timer.enable_interrupt();
                TouchpadState::Sense(pin, count + 1)
            }
            s => s,
        };

        self.state.replace(new_state);
    }

    pub fn handle_gpio_interrupt(&mut self) {
        self.gpiote.channel0().reset_events();
        let new_state = match self.state.take().unwrap() {
            TouchpadState::Sense(pin, count) => {
                self.gpiote
                    .channel0()
                    .input_pin(&pin)
                    .hi_to_lo()
                    .disable_interrupt();
                if count > self.threshold {
                    cortex_m::peripheral::NVIC::pend(self.interrupt);
                }
                TouchpadState::Idle(pin)
            }
            s => s,
        };

        self.state.replace(new_state);
    }
}
