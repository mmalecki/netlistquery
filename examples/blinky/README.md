# blinky with board definition file generationn
This is the blinky example from ESP32 `no_std` training, except the pins used come from a generated board definition.

The board JSON board definitions, generated using:

```sh
netlistquery project.net --json 'led(Pin) :-    
    pin(Id2, "D1", "2"),
    connected(Id2, NetId2),
    series_link(NetId2, NetId1),
    connected(Id1, NetId1),
    pin(Id1, "U1", _),
    pin_function(Id1, Pin).' > board-a.json
```

```json
{"led":[["GPIO10/TOUCH10/ADC1_CH9/FSPICS0/FSPIIO4/SUBSPICS0"]]}
```

Is turned into a Rust board definition by templating code in `bsp_gen`:

```rs
use crate::board_defs::BoardPeripherals;
use esp_hal::{gpio::Pin, peripherals::Peripherals};
impl BoardPeripherals {
    pub fn from_peripherals(p: Peripherals) -> Self {
        BoardPeripherals {
            led: p.GPIO10.degrade(),
        }
    }
}
```

Which is then conditionally imported by `src/board_defs.rs`:

```rs
#[cfg(feature = "board-a")]
pub mod board_a;
```

This allows for error-free board bring-up, especially in prototyping environments.
