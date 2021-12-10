#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use core::fmt::Write;
use ssd1306::{mode::TerminalMode, prelude::*, I2CDisplayInterface, Ssd1306};
use panic_halt as _;
use atmega_hal::port::{Pin, mode, self};
use atmega_hal::port::mode::{PullUp};

static mut PIN: Option<Pin<mode::Output, port::PD5>> = None;
static mut PIN_D8_Input: Option<Pin<mode::Input<PullUp>, port::PB0>> = None;
static mut COUNT: u16 = 0;

#[avr_device::interrupt(atmega328p)]
unsafe fn PCINT0() {
    avr_device::interrupt::free(|cs| {
        if PIN_D8_Input.as_mut().unwrap().is_low() {
            COUNT += 1;
            PIN.as_mut().unwrap().toggle();
        }
    });
}

#[arduino_hal::entry] 
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut led = pins.d5.into_output();
    let mut input = pins.d8.into_pull_up_input();
    let mut led2 = pins.d13.into_output();

    let i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        400_000,
    );

    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(
        interface,
        DisplaySize128x32,
        DisplayRotation::Rotate0,
    ).into_terminal_mode();

    display.init().unwrap();
    display.clear().unwrap();

    led.set_high();
    led2.set_high();

    unsafe {
        PIN = Some(led);
        PIN_D8_Input = Some(input);
    }

    // Enable interrupts
    unsafe {
        avr_device::interrupt::enable();
    }
    let mut buf = [0 as u8; 20];
    let mut count = COUNT.clone();
    
    loop {
        for c in "Count: ".chars() {
            let _ = display.print_char(c);
        }

        base_10_bytes(COUNT, &mut buf);

        for c in &buf {
            let _ = display.write_str(unsafe { core::str::from_utf8_unchecked(&[*c]) });
        }
    }
}
fn base_10_bytes(mut n: u16, buf: &mut [u8]) -> &[u8] {
    if n == 0 {
        return b"0";
    }
    let mut i = 0;
    while n > 0 {
        buf[i] = (n % 10) as u8 + b'0';
        n /= 10;
        i += 1;
    }
    let slice = &mut buf[..i];
    slice.reverse();
    &*slice
}