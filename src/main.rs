//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

use bsp::{
    entry,
    hal::{
        gpio::{FunctionPio0, Pin},
        prelude::_rphal_pio_PIOExt,
    },
};
use defmt::*;
use defmt_rtt as _;

use panic_probe as _;

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use rp_pico as bsp;
// use sparkfun_pro_micro_rp2040 as bsp;

use bsp::hal::{clocks::init_clocks_and_plls, pac, sio::Sio, watchdog::Watchdog};

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let _clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // configure LED pin for Pio0.
    let _led: Pin<_, FunctionPio0> = pins.led.into_mode();
    // PIN id for use inside of PIO
    let led_pin_id = 25;

    // Define some simple PIO program.
    let program = pio_proc::pio_asm!(
        ".wrap_target",
        "set pins, 1 [31]",
        "set pins, 0 [31]",
        ".wrap"
    );

    // Initialize and start PIO
    let (mut pio, sm0, _, _, _) = pac.PIO0.split(&mut pac.RESETS);
    let installed = pio.install(&program.program).unwrap();
    let (int, frac) = (0, 0); // as slow as possible (0 is interpreted as 65536)
    let (mut sm, _, _) = bsp::hal::pio::PIOBuilder::from_program(installed)
        .set_pins(led_pin_id, 1)
        .clock_divisor_fixed_point(int, frac)
        .build(sm0);
    // The GPIO pin needs to be configured as an output.
    sm.set_pindirs([(led_pin_id, bsp::hal::pio::PinDir::Output)]);
    sm.start();

    // PIO runs in background, independently from CPU
    loop {
        cortex_m::asm::wfi();
    }
}

// End of file
