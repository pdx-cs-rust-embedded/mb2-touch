#![no_main]
#![no_std]

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use cortex_m_rt::entry;
use microbit::{
    board::Board,
    hal::{
        gpio, gpiote,
        pac::{self, interrupt},
        prelude::*,
        timer,
    },
};

use critical_section_lock_mut::LockMut;

/// Minimum charging time in microseconds to regard as
/// "touched".
const TOUCH_THRESHOLD: u32 = 100;

/// Time in milliseconds to discharge the touchpad before
/// testing.
const DISCHARGE_TIME: u32 = 100;

/// Button press and release events.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TouchEvent {
    Press,
    Release,
}

/// The touch pin may be either "writing" or "reading".
pub enum TouchPin {
    /// "writing"
    Output(gpio::Pin<gpio::Output<gpio::PushPull>>),
    /// "reading"
    Input(gpio::Pin<gpio::Input<gpio::Floating>>),
}

/// Touchpad state.
pub struct Touchpad<T> {
    /// Currently pressed or released.
    state: TouchEvent,
    /// Most recent touch event since last change.
    event: Option<TouchEvent>,
    /// Touch pin.
    pin: Option<TouchPin>,
    /// GPIOTE channel for interrupts.
    #[allow(unused)]
    channel: usize,
    /// Timer for measurements.
    timer: timer::Timer<T>,
}

impl<T: timer::Instance> Touchpad<T> {
    pub fn new(
        pin: gpio::Pin<gpio::Disconnected>,
        channel: usize,
        timer: timer::Timer<T>,
    ) -> Self {
        let pin = pin.into_push_pull_output(gpio::Level::Low);
        Self {
            state: TouchEvent::Release,
            event: None,
            pin: Some(TouchPin::Output(pin)),
            channel,
            timer,
        }
    }

    pub fn start_measurement(&mut self) {
        let pin = match self.pin.take().unwrap() {
            TouchPin::Output(pin) => pin,
            TouchPin::Input(pin) => pin.into_push_pull_output(gpio::Level::Low),
        };
        self.pin = Some(TouchPin::Output(pin));
        self.timer.enable_interrupt();
        self.timer.start(DISCHARGE_TIME * 1000);
    }

    pub fn get_event(&mut self) -> Option<TouchEvent> {
        let event = self.event;
        self.clear_event();
        event
    }

    pub fn clear_event(&mut self) {
        self.event = None;
    }

    pub fn timer_interrupt(&mut self) {
        rprintln!("timer interrupt");
        match self.pin.take() {
            Some(TouchPin::Output(pin)) => {
                // Done discharging: start measuring.
                let pin = pin.into_floating_input();
                enable_gpiote_interrupt(self.channel, &pin);
                self.pin = Some(TouchPin::Input(pin));
                self.timer.start(TOUCH_THRESHOLD);
            }
            Some(TouchPin::Input(pin)) => {
                // Done measuring: touchpad is pressed.
                if self.state != TouchEvent::Press {
                    self.event = Some(TouchEvent::Press);
                }
                self.state = TouchEvent::Press;
                self.timer.disable_interrupt();
                disable_gpiote_interrupt(self.channel, &pin);
                let pin = pin.into_push_pull_output(gpio::Level::Low);
                self.pin = Some(TouchPin::Output(pin));
            }
            None => panic!("missing pin in timer interrupt"),
        }
    }

    pub fn gpiote_interrupt(&mut self) {
        rprintln!("gpiote interrupt");
        match self.pin.take() {
            Some(TouchPin::Input(pin)) => {
                if self.state != TouchEvent::Release {
                    self.event = Some(TouchEvent::Release);
                }
                self.state = TouchEvent::Release;
                self.timer.disable_interrupt();
                disable_gpiote_interrupt(self.channel, &pin);
                let pin = pin.into_push_pull_output(gpio::Level::Low);
                self.pin = Some(TouchPin::Output(pin));
            }
            Some(_) => panic!("unexpected gpiote interrupt"),
            None => panic!("missing pin in gpiote interrupt"),
        }
    }
}

static TOUCHPAD: LockMut<Touchpad<pac::TIMER0>> = LockMut::new();
static GPIOTE: LockMut<gpiote::Gpiote> = LockMut::new();

#[interrupt]
fn GPIOTE() {
    TOUCHPAD.with_lock(|touchpad| touchpad.gpiote_interrupt());
}

fn enable_gpiote_interrupt(
    _channel: usize,
    pin: &gpio::Pin<gpio::Input<gpio::Floating>>,
) {
    GPIOTE.with_lock(|gpiote| {
        gpiote
            .channel0()
            .input_pin(pin)
            .lo_to_hi()
            .enable_interrupt();
    });
}

fn disable_gpiote_interrupt(
    _channel: usize,
    pin: &gpio::Pin<gpio::Input<gpio::Floating>>,
) {
    GPIOTE.with_lock(|gpiote| {
        gpiote
            .channel0()
            .input_pin(pin)
            .disable_interrupt();
    });
}

#[interrupt]
fn TIMER0() {
    TOUCHPAD.with_lock(|touchpad| touchpad.timer_interrupt());
}

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = Board::take().unwrap();
    let touch_pin = board.pins.p1_04.degrade();
    GPIOTE.init(gpiote::Gpiote::new(board.GPIOTE));
    let timer = timer::Timer::new(board.TIMER0);
    let channel = 0;
    TOUCHPAD.init(Touchpad::new(touch_pin.into(), channel, timer));
    unsafe {
        pac::NVIC::unmask(pac::Interrupt::GPIOTE);
        pac::NVIC::unmask(pac::Interrupt::TIMER0);
    }
    pac::NVIC::unpend(pac::Interrupt::GPIOTE);
    pac::NVIC::unpend(pac::Interrupt::TIMER0);

    let mut delay = timer::Timer::new(board.TIMER1);
    loop {
        rprintln!("starting");
        TOUCHPAD.with_lock(|touchpad| {
            touchpad.start_measurement();
        });
        rprintln!("delaying");
        delay.delay_ms(100u16);
        rprintln!("checking");
        TOUCHPAD.with_lock(|touchpad| {
            let event = touchpad.get_event();
            rprintln!("{:?}", event);
        });
    }
}
