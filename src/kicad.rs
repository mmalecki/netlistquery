use std::collections::HashMap;

use crate::{Net, Netlist, PinFact};
use anyhow::Result;

pub fn parse_kicad_netlist(content: &str) -> Result<Netlist> {
    // KiCad .net files are written as S-expressions (Lisp-like syntax).
    // The `lexpr` crate parses this into a nested tree of symbols, lists, and strings.
    let v = lexpr::from_str(content)?;

    let mut nets = HashMap::new();
    let mut pins = Vec::new();

    // The root of the file should be a list containing the entire netlist.
    if let Some(list) = v.list_iter() {
        for item in list {
            if let Some(sublist) = item.list_iter() {
                let mut iter = sublist.clone();

                // We are looking for the top-level `(nets ...)` section.
                if let Some(first) = iter.next() {
                    if first.as_symbol() == Some("nets") {
                        // Iterate through each `(net ...)` definition inside the `(nets ...)` block.
                        for net_item in iter {
                            if let Some(net_sublist) = net_item.list_iter() {
                                let mut code = String::new();
                                let mut name = String::new();
                                let mut nodes = Vec::new();

                                let mut net_iter = net_sublist.clone();
                                net_iter.next(); // Skip the "net" tag itself.

                                // Iterate through properties of the `(net ...)` block.
                                // Expected properties include: `(code "1")`, `(name "+3.3V")`, and multiple `(node ...)` entries.
                                for prop in net_iter {
                                    if let Some(prop_list) = prop.list_iter() {
                                        // Collect the property S-expression list into a vector for easier indexed access.
                                        // For example, `(code "1")` becomes a `prop_vec` where:
                                        //   prop_vec[0] = Symbol("code")
                                        //   prop_vec[1] = String("1")
                                        let prop_vec: Vec<_> = prop_list.collect();

                                        if prop_vec.len() >= 2 {
                                            if prop_vec[0].as_symbol() == Some("code") {
                                                code =
                                                    prop_vec[1].as_str().unwrap_or("").to_string();
                                            } else if prop_vec[0].as_symbol() == Some("name") {
                                                name =
                                                    prop_vec[1].as_str().unwrap_or("").to_string();
                                            } else if prop_vec[0].as_symbol() == Some("node") {
                                                // A `node` represents a physical pin connected to this net.
                                                // Store the entire `(node ...)` S-expression list to parse its attributes next.
                                                nodes.push(prop_vec);
                                            }
                                        }
                                    }
                                }

                                // Map the internal numeric net code to the human-readable net name.
                                nets.insert(code.clone(), name.clone());

                                // Process all the physical pins (nodes) attached to this specific net.
                                for node_prop in nodes {
                                    let mut comp_ref = String::new();
                                    let mut pin_num = String::new();
                                    let mut pin_type = String::new();
                                    let mut pin_function = None;

                                    // Skip the "node" tag at index 0, and parse its attributes.
                                    // Example node: `(node (ref "U1") (pin "5") (pinfunction "INT1_5") (pintype "input"))`
                                    for attr in node_prop.iter().skip(1) {
                                        if let Some(attr_list) = attr.list_iter() {
                                            let attr_vec: Vec<_> = attr_list.collect();
                                            if attr_vec.len() >= 2 {
                                                match attr_vec[0].as_symbol() {
                                                    Some("ref") => {
                                                        comp_ref = attr_vec[1]
                                                            .as_str()
                                                            .unwrap_or("")
                                                            .to_string()
                                                    }
                                                    Some("pin") => {
                                                        pin_num = attr_vec[1]
                                                            .as_str()
                                                            .unwrap_or("")
                                                            .to_string()
                                                    }
                                                    Some("pintype") => {
                                                        pin_type = attr_vec[1]
                                                            .as_str()
                                                            .unwrap_or("")
                                                            .to_string()
                                                    }
                                                    Some("pinfunction") => {
                                                        pin_function = Some(
                                                            attr_vec[1]
                                                                .as_str()
                                                                .unwrap_or("")
                                                                .to_string(),
                                                        )
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        }
                                    }

                                    // Commit the extracted physical pin data as a flat fact.
                                    pins.push(PinFact {
                                        component_ref: comp_ref,
                                        pin_number: pin_num,
                                        pin_type,
                                        pin_function,
                                        net_name: name.clone(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(Netlist {
        nets: nets
            .into_iter()
            .map(|(code, name)| Net { code, name })
            .collect(),
        pins,
    })
}
