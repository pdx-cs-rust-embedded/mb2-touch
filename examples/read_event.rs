#![no_main]
#![no_std]

use core::{cell::RefCell, sync::atomic::AtomicU32};

use cortex_m::interrupt::Mutex;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use cortex_m_rt::entry;
use microbit::{
    board::Board,
    hal::{pac, timer},
};
use pac::interrupt;

use mb2_touch::*;

static TOUCHPAD: Mutex<RefCell<Option<Touchpad>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = Board::take().unwrap();
    let touch_pin = board.pins.p1_04.into_floating_input();
    let timer0 = timer::Timer::new(board.TIMER0);

    let touchpad = Touchpad::new(touch_pin, timer0, interrupt::SWI0_EGU0, 100);
    cortex_m::interrupt::free(|cs| {
        TOUCHPAD.borrow(cs).borrow_mut().replace(touchpad);
    });

    // Safety: we're outside any critical sections, and are thus not messing with one
    unsafe {
        cortex_m::peripheral::NVIC::unmask(interrupt::SWI0_EGU0);
        cortex_m::peripheral::NVIC::unmask(interrupt::TIMER0);
    }

    loop {
        // rprintln!("Loop");
        cortex_m::asm::wfe();
    }
}

#[interrupt]
fn TIMER0() {
    cortex_m::interrupt::free(|cs| {
        if let Some(t) = TOUCHPAD.borrow(cs).borrow_mut().as_mut() {
            t.handle_timer_interrupt();
        }
    })
}

#[interrupt]
fn SWI0_EGU0() {
    static TOUCH_COUNT: AtomicU32 = AtomicU32::new(0);
    rprintln!(
        "Ouch! {}",
        TOUCH_COUNT.fetch_add(1, core::sync::atomic::Ordering::Relaxed)
    );
}
