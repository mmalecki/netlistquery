use esp_hal::gpio::AnyPin;

pub struct BoardPeripherals {
    pub led: AnyPin<'static>,
}

#[cfg(feature = "board-a")]
pub mod board_a;
