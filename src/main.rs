#![no_std]
#![no_main]
#![allow(clippy::empty_loop)]
#![allow(dead_code)]

use cortex_m::peripheral::syst::SystClkSource;
//use board::*;
use button::*;
//use cortex_m::delay::{self, Delay};
use core::panic::PanicInfo;
use led::*;
use rtt_target::{rprintln, rtt_init_print};

use crate::board::{
    GREEN_LED_PIN, GREEN_LED_PORT, RED_LED_PIN, RED_LED_PORT, USER_BTN_PIN, USER_BTN_PORT,
};

// Removed static PERIPHERALS to avoid Sync error
//use crate::{button::{button_configure_interrupt, button_init}, led::{led_init, led_off}};

use itm_debug::*;

mod itm_debug;
mod board;
mod button;
mod gpio;
mod led;
mod mcu;
mod reg;
mod proc;
mod startup_stm32f429;
mod exti;

// Macro to initialize cortex-m peripherals
macro_rules! init_cortex_m_peripherals {
    () => {
        cortex_m::Peripherals::take().unwrap()
    };
}

// Macro to get ITM stimulus port 0
#[allow(unused_macros)]
macro_rules! get_itm_stim {
    ($cp:expr) => {
        &mut $cp.ITM.stim[0]
    };
}

// Macro to get ITM stimulus port directly (for use in interrupt handlers)
macro_rules! get_itm_stim_direct {
    ($port:expr) => {
        unsafe { &mut (*cortex_m::peripheral::ITM::PTR).stim[$port] }
    };
}

#[unsafe(no_mangle)]
fn main() {
    led_init(GREEN_LED_PORT, GREEN_LED_PIN);
    led_init(RED_LED_PORT, RED_LED_PIN);

    led_off(GREEN_LED_PORT, GREEN_LED_PIN);
    led_on(RED_LED_PORT, RED_LED_PIN);


    button::button_init(
        USER_BTN_PORT,
        USER_BTN_PIN,
        Mode::Interrupt(Trigger::FallingEdge),
    );

    let mut cp = init_cortex_m_peripherals!();

    // Initialize RTT for debug output
    rtt_init_print!();
    
    rprintln!("RTT Debug: Starting STM32F429I-DISCO program");

    itm_init(&mut cp);
    rprintln!("RTT Debug: ITM initialized");

    systick_init(&mut cp);
    rprintln!("RTT Debug: SysTick initialized");

    // Test ITM (won't work on F429I-DISCO due to missing SWO connection)
    unsafe {
        let itm = &mut *cortex_m::peripheral::ITM::PTR;
        
        if itm.stim[0].is_fifo_ready() {
            itm.stim[0].write_u8(b'H');
            itm.stim[0].write_u8(b'i');
            itm.stim[0].write_u8(b'\n');
        }
    }
    
    rprintln!("RTT Debug: ITM test completed - likely no output on F429I-DISCO");
    
    let mut counter = 0;
    loop {
        // Blink LED to show we're alive
        for _ in 0..2000000 {
            cortex_m::asm::nop();
        }
        led_toggle(GREEN_LED_PORT, GREEN_LED_PIN);
        
        counter += 1;
        rprintln!("RTT Debug: Loop iteration {}, LED toggled", counter);
    }
}

#[panic_handler]
fn panic_handler(_info: &PanicInfo) -> ! {
    loop {
    
    }
}

fn systick_init(cp: &mut cortex_m::Peripherals) {
    cp.SYST.set_clock_source(SystClkSource::Core);
    cp.SYST.set_reload(4_000_000); // 1/4 second at 16 MHz
    cp.SYST.enable_counter();
    cp.SYST.enable_interrupt();
}

//button interrupt handler
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
extern "C" fn EXTI0_Handler() {
    let stim0 = get_itm_stim_direct!(0);
    cortex_m::iprintln!(stim0, "Button pressed!\n");
    led_toggle(RED_LED_PORT, RED_LED_PIN);
    led_toggle(GREEN_LED_PORT, GREEN_LED_PIN);
    button::button_clear_interrupt(USER_BTN_PIN);
}
