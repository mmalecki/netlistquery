#!/usr/bin/env bash

dir=$(dirname "$0")
bin=$(realpath "$dir/../target/debug/netlistquery")

net="$dir/attiny85-imu-led/attiny85-imu-led.net"

# Find pin function of the MCU (U1) pin connected to U2's SDI pin.
"$bin" "$net" 'mcu_pin(Pin) :-
    pin(Id2, "U2", _),
    pin_feature(Id2, "SDI"),
    connected(Id2, NetId),
    connected(Id1, NetId),
    pin(Id1, "U1", _),
    pin_function(Id1, Pin).' | sort

# Find components connected to "SDA" net *and* +3.3V -
# should find pull ups and bus actors.
"$bin" "$net" 'bus_actors(Component) :-
    pin(Pin_A, Component, _),
    connected(Pin_A, "+3.3V"),
    pin(Pin_B, Component, _),
    connected(Pin_B, "/SDA").' | sort

# Find pin_function of U1 pin connected through resistor to DIN of D1
"$bin" "$net" 'mcu_pin(Pin) :-
    pin(Id1, "D1", _),
    pin_feature(Id1, "DIN"),
    connected(Id1, Net1),
    series_link(Net1, Net2),
    connected(Id2, Net2),
    pin(Id2, "U1", _),
    pin_function(Id2, Pin).' | sort

# Find 2 pin components that span between +3.3V and GND (e.g. bypass capacitors)
"$bin" "$net" 'power_components(Comp) :-
    pin_count(Comp, 2),
    pin(IdA, Comp, _), connected(IdA, "+3.3V"),
    pin(IdB, Comp, _), connected(IdB, "GND").' | sort

# Find pin count of ICs
"$bin" "$net" '
pin_u1(Count) :- pin_count("U1", Count).
pin_u2(Count) :- pin_count("U2", Count).
' | sort
