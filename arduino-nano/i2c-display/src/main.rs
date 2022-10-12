#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]
use panic_halt as _;

use arduino_hal::prelude::*;
use core::fmt::Write;
use core::cell;
use ssd1306::{mode::TerminalMode, prelude::*, I2CDisplayInterface, Ssd1306};

use atmega_hal::port::{Pin, mode, self};
use atmega_hal::port::mode::{PullUp};
use core:: sync::atomic::{AtomicBool, Ordering};


static PIN_D8_CHANGED: AtomicBool = AtomicBool::new(true);

struct KeyInput<TPinMode, TPortPin> {
    key_input_pin: Pin<TPinMode, TPortPin>,
    key_input_state: AtomicBool
}

struct KeyInputs {
    key_input: Pin<mode::Input<PullUp>, port::PB0>
}

static mut COUNT: u16 = 0;

#[avr_device::interrupt(atmega328p)]
fn TIMER0_COMPA() {
    
}

#[arduino_hal::entry] 
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    dp.EXINT.pcicr.write(|w| w.pcie().bits(0b001));
    dp.EXINT.pcmsk0.write(|w| w.pcint().bits(0b001));

    let mut led = pins.a1.into_output();
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
    
    for c in "Count: ".chars() {
        let _ = display.print_char(c);
    }

    let timer0 = dp.TC0;
    timer0.tccr0A.write(|w| w.cs1().prescale_64());

    unsafe { avr_device::interrupt::enable() };
    loop {
        display.set_position(7, 0);

        avr_device::interrupt::free(|cs| {
            if PIN_D8_CHANGED.load(Ordering::SeqCst) {     
                led.set_high();            
            } else {
                led.set_low();
            }
        });

        unsafe {
            for c in "t".chars() {
                let _ = display.print_char(c);
            }

            if PIN_D8_CHANGED.load(Ordering::SeqCst) { 
                display.set_position(14, 0);    
                for c in "ON".chars() {
                    let _ = display.print_char(c);
                }        
            } else {
                display.set_position(14, 0);    
                for c in "OFF".chars() {
                    let _ = display.print_char(c);
                }        
            }
        }
    }
}