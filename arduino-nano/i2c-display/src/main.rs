#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use arduino_hal::prelude::*;
use core::fmt::Write;
use core::cell;
use ssd1306::{mode::TerminalMode, prelude::*, I2CDisplayInterface, Ssd1306};

use atmega_hal::port::{Pin, mode, self};
use atmega_hal::port::mode::{PullUp};

static mut PIN: Option<Pin<mode::Output, port::PD5>> = None;
static mut PIN_D8_Input: Option<Pin<mode::Input<PullUp>, port::PB0>> = None;
static mut COUNT: bool = false;

#[avr_device::interrupt(atmega328p)]
unsafe fn PCINT0() {
    avr_device::interrupt::free(|cs| {
        if PIN_D8_Input.as_mut().unwrap().is_low() {
            COUNT = true;
            PIN.as_mut().unwrap().toggle();
        }
    });
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    // disable interrupts - firmware has panicked so no ISRs should continue running
    avr_device::interrupt::disable();

    // get the peripherals so we can access serial and the LED.
    //
    // SAFETY: Because main() already has references to the peripherals this is an unsafe
    // operation - but because no other code can run after the panic handler was called,
    // we know it is okay.
    let dp = unsafe { arduino_hal::Peripherals::steal() };
    let pins = arduino_hal::pins!(dp);

    // Blink LED rapidly
    let mut led = pins.d13.into_output();
    led.set_high();
    loop {
    }
}

#[arduino_hal::entry] 
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    dp.EXINT.pcicr.write(|w| w.pcie().bits(0x07));
    dp.EXINT.pcmsk0.write(|w| w.pcint().bits(0xFF));

    let mut led = pins.d5.into_output();
    let mut input = pins.d8.into_pull_up_input();
    
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

    led.set_low();

    unsafe {
        PIN = Some(led);
        PIN_D8_Input = Some(input);
    }

    // Enable interrupts
    unsafe {
        avr_device::interrupt::enable();
    }

    let mut buf = [0 as u8; 20];
    unsafe {
        let mut count = COUNT.clone();
    }
    
    for c in "Count: ".chars() {
        let _ = display.print_char(c);
    }

    loop {
        display.set_position(7, 0);

        unsafe {
            if COUNT {
                for c in "t".chars() {
                    let _ = display.print_char(c);
                }
    //
            } else {
                for c in "f ".chars() {
                    let _ = display.print_char(c);
                }
    //
            }
        }

        //unsafe {
        //    base_10_bytes(COUNT, &mut buf);
        //}
        //
        //for c in &buf {
        //    let _ = display.write_str(unsafe { core::str::from_utf8_unchecked(&[*c]) });
        //}
    }
}
unsafe fn base_10_bytes(mut n: u16, buf: &mut [u8]) -> &[u8] {
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