# netlistquery
netlistquery is a Rust library and CLI for querying EDA netlists. For example,
it allows you to easily answer questions like "which pin of my MCU is connected
to the EN pin of my IMU", or "how many decoupling capacitors are in my schematic".

Queries are written in a Datalog format.

## Usage
First, export a netlist from your EDA. For now, only KiCad netlist exports are supported.

```sh
# Find pin function of the MCU (U1) pin connected to U2's SDI pin.
netlistquery project.net 'mcu_pin(Pin) :-
    pin(Id2, "U2", _),
    pin_feature(Id2, "SDI"),
    connected(Id2, NetId),
    connected(Id1, NetId),
    pin(Id1, "U1", _),
    pin_function(Id1, Pin).'
```

There are more example queries in [`examples/cli.sh`](examples/cli.sh).

### REPL
If you'd like to explore a netlist, or just play around with `netlistquery`, you can
start it as a REPL:

```sh
netlistquery project.net
```

### Build-time code generation
One of netlistquery use-cases is build-time generation of MCU pin bindings. This
allows for error-free board bring-up. You can find a [complete ESP32-based `blinky` example](examples/blinky),
including a Rust code generator.
