#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    gpio::{AnyPin, Level, Output, OutputConfig},
    main,
};
use esp_println::println;

mod board_defs;

use board_defs::BoardPeripherals;

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    println!("Hello world!");

    let board_peripherals = BoardPeripherals::from_peripherals(peripherals);
    let mut led = Output::new(board_peripherals.led, Level::Low, OutputConfig::default());

    led.set_high();

    // Initialize the Delay peripheral, and use it to toggle the LED state in a
    // loop.
    let delay = Delay::new();

    loop {
        led.toggle();
        delay.delay_millis(500);
    }
}
