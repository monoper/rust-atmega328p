#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use arduino_hal::prelude::*;
use core::fmt::Write;
use core::cell;
use ssd1306::{mode::TerminalMode, prelude::*, I2CDisplayInterface, Ssd1306};

use atmega_hal::port::{Pin, mode, self};
use atmega_hal::port::mode::{PullUp};
use core::sync::atomic::{AtomicBool, Ordering};

use panic_halt as _;

fn duty(value: u8) -> u16 {
    20 * value as u16
}

#[arduino_hal::entry] 
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut led = pins.d5.into_output();

    pins.d9.into_output();

    let tc1 = dp.TC1;
    tc1.icr1.write(|w| unsafe { w.bits(4999) });
    tc1.tccr1a
        .write(|w| w.wgm1().bits(0b10).com1a().match_clear());
    tc1.tccr1b
        .write(|w| w.wgm1().bits(0b11).cs1().prescale_64());
        
    led.set_high();

    let mut duty = 100;

    loop {
        tc1.ocr1a.write(|w| unsafe { w.bits(0x3ff) });        
    }
}