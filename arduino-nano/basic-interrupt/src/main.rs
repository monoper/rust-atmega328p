// main.rs

#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use panic_halt as _;
use arduino_uno::prelude::*;
use arduino_uno::hal::port;

static mut PIN: Option<port::portd::PD5<port::mode::Output>> = None;
static mut PIN_D8_Input: Option<port::portb::PB0<port::mode::Input<port::mode::PullUp>>> = None;

#[avr_device::interrupt(atmega328p)]
unsafe fn PCINT0() {
    avr_device::interrupt::free(|cs| {
        if PIN_D8_Input.as_mut().unwrap().is_low().void_unwrap() {
            PIN.as_mut().unwrap().toggle().void_unwrap();
        }
    });
}

#[arduino_uno::entry]
fn main() -> ! {
    let peripherals = arduino_uno::Peripherals::take().unwrap();
    peripherals.EXINT.pcicr.write(|w| w.pcie().bits(0x07));
    peripherals.EXINT.pcmsk0.write(|w| w.pcint().bits(0xFF));

    let mut pins = arduino_uno::Pins::new(
        peripherals.PORTB,
        peripherals.PORTC,
        peripherals.PORTD,
    );

    let mut input = pins.d8.into_pull_up_input(&mut pins.ddr);
    let mut led = pins.d5.into_output(&mut pins.ddr);

    unsafe {
        PIN = Some(led);
        PIN_D8_Input = Some(input);
    }

    // Enable interrupts
    unsafe {
        avr_device::interrupt::enable();
    }

    loop {
    }
}