#![no_main]
#![no_std]

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use cortex_m_rt::entry;
use microbit::{
    board::Board,
    hal::{prelude::*, gpio, gpiote},
};

use critical_section_lock_mut::LockMut;

/// Minimum charging time in microseconds to regard as
/// "touched".
const TOUCH_THRESHOLD: usize = 100;

/// Time in milliseconds to discharge the touchpad before
/// testing.
const DISCHARGE_TIME: u16 = 100;

/// Button press and release events.
enum TouchEvent {
    Press(usize),
    Release(usize),
}

/// The touch pin may be either "writing" or "reading".
enum TouchPin {
    /// "writing"
    Output(gpio::Pin<gpio::Output<gpio::PushPull>>),
    /// "reading"
    Input(gpio::Pin<gpio::Input<gpio::Floating>>),
}

/// Touchpad state.
struct Touchpad {
    /// True for pressed, false for released.
    state: bool,
    /// Most recent touch event since last change.
    event: Option<TouchEvent>,
    /// Touch pin.
    pin: TouchPin,
    gpiote: gpiote::Gpiote,
}

impl Touchpad {
    fn new(
        mut pin: gpio::Pin<gpio::Output<gpio::PushPull>>,
        gpiote: gpiote::Gpiote,
    ) -> Self {
        pin.set_high().unwrap();
        Self {
            state: false,
            event: None,
            pin: TouchPin::Output(pin),
            gpiote,
        }
    }
}

static TOUCHPAD: LockMut<Touchpad> = LockMut::new();

#[interrupt]
fn GPIOTE() {
    TOUCHPAD.with_locked(|touchpad|) {
        todo!()
    }
}

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = Board::take().unwrap();
    let touch_pin = board.pins.p1_04.into_push_pull_output(gpio::Level::Low);
    let gpiote = gpiote::Gpiote::new(board.GPIOTE);
    TOUCHPAD.init(Touchpad::new(touch_pin.into(), gpiote));

    loop {
        TOUCHPAD.with_locked(|touchpad| {
            if let Some(event) = touchpad.get_event() {
                rprintln!("{:?}", event);
                touchpad.clear_event();
            }
        }
    }
}
