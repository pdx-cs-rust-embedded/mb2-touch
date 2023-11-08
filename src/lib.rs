#![no_main]
#![no_std]
#![allow(clippy::identity_op)]

use core::fmt;

use microbit::hal::{
    gpio::{self, p1::P1_04, Floating, Input, Output, PushPull},
    pac,
    prelude::*,
    timer,
};
use panic_rtt_target as _;

type SensePin = P1_04<Input<Floating>>;
type DischargePin = P1_04<Output<PushPull>>;

pub enum TouchpadState {
    Disabled(SensePin),
    Idle(SensePin),
    Setup(DischargePin),
    Sense(SensePin, u32),
    SenseBackoff(SensePin),
}

impl fmt::Debug for TouchpadState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Disabled(_) => f.debug_tuple("Disabled").finish(),
            Self::Idle(_) => f.debug_tuple("Idle").finish(),
            Self::Setup(_) => f.debug_tuple("Setup").finish(),
            Self::Sense(_, arg1) => f.debug_tuple("Sense").field(arg1).finish(),
            Self::SenseBackoff(_) => f.debug_tuple("SenseBackoff").finish(),
        }
    }
}

pub struct Touchpad {
    timer: timer::Timer<pac::TIMER0>,
    state: Option<TouchpadState>,
    interrupt: pac::Interrupt,
    threshold: u32,
}

impl Touchpad {
    pub fn new(
        pin: P1_04<Input<Floating>>,
        timer: timer::Timer<pac::TIMER0>,
        interrupt: pac::Interrupt,
        threshold: u32,
    ) -> Self {
        let mut t = Touchpad {
            state: Some(TouchpadState::Idle(pin)),
            timer,
            interrupt,
            threshold,
        };
        t.handle_timer_interrupt();
        t
    }

    pub fn handle_timer_interrupt(&mut self) {
        self.timer.event_compare_cc0().reset();
        let Some(current_state) = self.state.take() else {
            return;
        };
        let new_state = match current_state {
            TouchpadState::Idle(pin) => {
                let pin = pin.into_push_pull_output(gpio::Level::Low);
                self.set_alarm(1000);
                TouchpadState::Setup(pin)
            }
            TouchpadState::Setup(pin) => {
                let pin = pin.into_floating_input();
                self.set_alarm(1);
                TouchpadState::Sense(pin, 0)
            }
            TouchpadState::Sense(pin, count) => {
                self.set_alarm(1);

                if count >= self.threshold {
                    cortex_m::peripheral::NVIC::pend(self.interrupt);
                    TouchpadState::SenseBackoff(pin)
                } else if pin.is_low().unwrap() {
                    TouchpadState::Sense(pin, count + 1)
                } else {
                    TouchpadState::SenseBackoff(pin)
                }
            }
            TouchpadState::SenseBackoff(pin) => {
                self.set_alarm(1);
                if pin.is_high().unwrap() {
                    TouchpadState::Idle(pin)
                } else {
                    TouchpadState::SenseBackoff(pin)
                }
            }
            s => s,
        };
        self.state.replace(new_state);
    }

    fn set_alarm(&mut self, micros: u32) {
        const N52_TIMER_FREQ_HZ: u32 = 64_000_000;
        const TIMER_TICKS_PER_US: u32 = N52_TIMER_FREQ_HZ / 1_000_000;

        self.timer.enable_interrupt();
        self.timer.start(micros * TIMER_TICKS_PER_US);
    }
}
