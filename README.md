# netlistquery
netlistquery is a Rust library and CLI for querying EDA netlists. For example,
it allows you to easily answer questions like "which pin of my MCU is connected
to the EN pin of my IMU", or "how many decoupling capacitors are in my schematic".

Queries are written in a Datalog format.

## Installation
For now, due to some upstream depedencies needing to be installed from a git fork,
it is recommended to install `netlistquery` from the git repository:

```sh
cargo install --git https://github.com/mmalecki/netlistquery.git netlistquery
```

## Usage
First, export a netlist from your EDA. For now, only KiCad netlist exports are supported.
There is an example netlist exported in [`./examples/attiny85-imu-led/attiny85-imu-led.net`](./examples/attiny85-imu-led/attiny85-imu-led.net). All of the examples listed below are based on it. You can grab it separately to the repo like this:

```sh
wget -O project.net https://github.com/mmalecki/netlistquery/raw/refs/heads/latest/examples/attiny85-imu-led/attiny85-imu-led.net 
```

### List all the MCU pin functions
Let's start simple by listing all the MCU pins and their functions:

```sh
$ netlistquery project.net 'pins(N, P) :-
    pin(PinId, "U1", N), pin_function(PinId, P).'
Result: pins(1, ~{RESET}/PB5_1)
Result: pins(5, AREF/PB0_5)
Result: pins(2, XTAL1/PB3_2)
Result: pins(7, PB2_7)
Result: pins(6, PB1_6)
Result: pins(8, VCC_8)
Result: pins(4, GND_4)
Result: pins(3, XTAL2/PB4_3)
```

### Find the MCU pin connected to a named net
```sh
$ netlistquery project.net 'mcu_pin(Pin) :-
    pin(Id, "U1", _),
    connected(Id, "/SCL"),
    pin_function(Id, Pin).'
Result: mcu_pin(PB2_7)
```

### Find the MCU pin connected to a pin of other IC
If you prefer going the "target pin" route, you can also do that:

```sh
# Find pin function of the MCU (U1) pin connected to U2's SDI pin.
$ netlistquery project.net 'mcu_pin(Pin) :-
    pin(Id2, "U2", _),
    pin_feature(Id2, "SDI"),
    connected(Id2, NetId),
    connected(Id1, NetId),
    pin(Id1, "U1", _),
    pin_function(Id1, Pin).'
Result: mcu_pin(AREF/PB0_5)
```

There are more example queries in [`examples/cli.sh`](examples/cli.sh).
`netlistquery` also supports series connections between nets, e.g. via resistors.

### Standard library

#### `pin(PinId: string, Component: string, PinNumber: string)`

#### `pin_function(PinId: string, PinFunction: string)`

#### `pin_feature(PinId: string, PinFeature: string)`

#### `pin_count(Component: string, Count: number)`

#### `connected(PinId, Net: string)`

#### `series_link(NetA: string, NetB: string)`


### REPL
If you'd like to explore a netlist, or just play around with `netlistquery`, you can
start it as a REPL:

```sh
netlistquery project.net
```

### Bring-up-time code generation
One of netlistquery use-cases is bring-up-time generation of MCU pin bindings. This
allows for error-free board bring-up, as all the pin definitions are generated
from the PCB's netlist. You can find a [complete ESP32-based `blinky` example](examples/blinky),
including a Rust code generator.
