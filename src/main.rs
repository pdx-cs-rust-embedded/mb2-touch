#![no_main]
#![no_std]

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use cortex_m_rt::entry;
use microbit::{
    Peripherals,
    board::Board,
    hal::{prelude::*, comp, gpio, timer},
};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = Board::take().unwrap();
    let peripherals = unsafe { Peripherals::steal() };
    let mut touch_pin = board.pins.p1_04.into_floating_input();
    //let mut touch_pin = board.pins.p0_02.into_floating_input();
    let mut comp = peripherals.COMP;
    let mut timer = timer::Timer::new(board.TIMER0);

    loop {
        let touch_pin_i = touch_pin.into_push_pull_output(gpio::Level::Low);
        timer.delay_us(1000u16);
        touch_pin = touch_pin_i.into_floating_input();
        let comp_p = comp::Comp::new(comp, &touch_pin);
        comp_p
            .power_mode(comp::PowerMode::HighSpeed)
            .vref(comp::VRef::Int1V2)
            .hysteresis(true)
            .enable();
        let mut count = 0u32;
        while !comp_p.is_cross() {
            timer.delay_us(1000u16);
            count += 1;
        }
        comp = comp_p.free();
        rprintln!("{}", count);
        timer.delay_ms(500u16);
    }
}
