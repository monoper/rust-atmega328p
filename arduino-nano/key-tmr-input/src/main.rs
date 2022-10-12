//! A basic implementation of the `millis()` function from Arduino:
//!
//!     https://www.arduino.cc/reference/en/language/functions/time/millis/
//!
//! Uses timer 0 and one of its interrupts to update a global millisecond
//! counter.  A walkthough of this code is available here:
//!
//!     https://blog.rahix.de/005-avr-hal-millis/
#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use arduino_hal::port::{Pin, mode};
use core::cell;
use panic_halt as _;

// Possible Values:
//
// ╔═══════════╦══════════════╦═══════════════════╗
// ║ PRESCALER ║ TIMER_COUNTS ║ Overflow Interval ║
// ╠═══════════╬══════════════╬═══════════════════╣
// ║        64 ║          250 ║              1 ms ║
// ║       256 ║          125 ║              2 ms ║
// ║       256 ║          250 ║              4 ms ║
// ║      1024 ║          125 ║              8 ms ║
// ║      1024 ║          250 ║             16 ms ║
// ╚═══════════╩══════════════╩═══════════════════╝
const PRESCALER: u32 = 1024;
const TIMER_COUNTS: u32 = 125;
const DEBOUNCE_TIME: u32 = 50;

const MILLIS_INCREMENT: u32 = PRESCALER * TIMER_COUNTS / 16000;

static MILLIS_COUNTER: avr_device::interrupt::Mutex<cell::Cell<u32>> =
    avr_device::interrupt::Mutex::new(cell::Cell::new(0));

fn millis_init(tc0: arduino_hal::pac::TC0) {
    // Configure the timer for the above interval (in CTC mode)
    // and enable its interrupt.
    tc0.tccr0a.write(|w| w.wgm0().ctc());
    tc0.ocr0a.write(|w| unsafe { w.bits(TIMER_COUNTS as u8) });
    tc0.tccr0b.write(|w| match PRESCALER {
        8 => w.cs0().prescale_8(),
        64 => w.cs0().prescale_64(),
        256 => w.cs0().prescale_256(),
        1024 => w.cs0().prescale_1024(),
        _ => panic!(),
    });
    tc0.timsk0.write(|w| w.ocie0a().set_bit());

    // Reset the global millisecond counter
    avr_device::interrupt::free(|cs| {
        MILLIS_COUNTER.borrow(cs).set(0);
    });
}
struct KeyInput {
    pin: Pin<mode::Input<mode::PullUp>>,
    pin_output: Pin<mode::Output>,
    last_debounce_time: u32,
    state: bool,
    lockout_high: bool
}

#[avr_device::interrupt(atmega328p)]
fn TIMER0_COMPA() {
    avr_device::interrupt::free(|cs| {
        let counter_cell = MILLIS_COUNTER.borrow(cs);        
        let counter = counter_cell.get();
        counter_cell.set(counter + MILLIS_INCREMENT);
    })
}

fn millis() -> u32 {
    avr_device::interrupt::free(|cs| MILLIS_COUNTER.borrow(cs).get())
}

// ----------------------------------------------------------------------------

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    
    let mut led = pins.a1.into_output();
    
    millis_init(dp.TC0);

    let mut key_one_input = KeyInput {
        pin: pins.d8.into_pull_up_input().downgrade(),
        pin_output: pins.a2.into_output().downgrade(),
        state: false,
        last_debounce_time: millis(),
        lockout_high: false
    };

    led.set_high();
    // Enable interrupts globally
    unsafe { avr_device::interrupt::enable() };
    // Wait for a character and print current time once it is received
    loop {
        key_input_handler(&mut key_one_input);
    }
}

fn key_input_handler(key_input: &mut KeyInput) {
    let input_state = key_input.pin.is_low();

    if (millis() - key_input.last_debounce_time) > DEBOUNCE_TIME {            
        if input_state && !key_input.lockout_high {
            key_input.state = !key_input.state;

            if key_input.state {
                key_input.pin_output.set_high();
            } else {
                key_input.pin_output.set_low();
            }
            key_input.lockout_high = true;
        } else if !input_state {
            key_input.lockout_high = false;
        }
        key_input.last_debounce_time = millis();
    }
}
