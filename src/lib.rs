use std::collections::HashMap;
use std::str::FromStr;

use anyhow::Result;
use asdi::edb::{Attribute, Constant, Predicate};
use asdi::idb::eval::NaiveEvaluator;
use asdi::parse::parse_str;
use asdi::{Collection, Labeled, ProgramCore};

pub mod kicad;

#[derive(Debug, Clone)]
pub struct Netlist {
    pub nets: Vec<Net>,
    pub pins: Vec<PinFact>,
}

#[derive(Debug, Clone)]
pub struct Net {
    pub code: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct PinFact {
    pub component_ref: String,
    pub pin_number: String,
    pub pin_type: String,
    pub pin_function: Option<String>,
    pub net_name: String,
}

fn is_series_passive(comp_ref: &str) -> bool {
    let prefixes = ["R", "L", "FB", "JP"];
    prefixes.iter().any(|&p| comp_ref.starts_with(p))
}

fn pin_id(pin: &PinFact) -> String {
    format!("{}_{}", pin.component_ref, pin.pin_number)
}

pub fn execute_query(
    netlist: &Netlist,
    query: &str,
) -> Result<HashMap<String, Vec<Vec<Constant>>>> {
    let user_query = query.replace(":-", "<-");

    let mut header = String::new();
    let mut user_rules = std::collections::HashSet::new();
    for line in user_query.lines() {
        if let Some(idx) = line.find("<-") {
            let head = line[..idx].trim();
            if let Some(paren_idx) = head.find('(') {
                let name = &head[..paren_idx];
                user_rules.insert(name.to_string());

                let args_part = &head[paren_idx + 1..head.len() - 1];
                let num_args = if args_part.trim().is_empty() {
                    0
                } else {
                    args_part.split(',').count()
                };
                if num_args > 0 {
                    let mut types = Vec::new();
                    for arg in args_part.split(',') {
                        if arg.trim().to_lowercase().contains("count") {
                            types.push("integer");
                        } else {
                            types.push("string");
                        }
                    }
                    let types_str = types.join(", ");
                    header.push_str(&format!(".infer {}({}).\n", name, types_str));
                } else {
                    header.push_str(&format!(".infer {}.\n", name));
                }
            }
        }
    }

    let datalog_source = format!(
        "{}\n{}\n",
        header,
        r#"

        % Standard library schema
        % pin(PinId, Component, PinNumber)
        .assert pin(string, string, string).
        % pin_function(PinId, PinNumber)
        .assert pin_function(string, string).
        % pin_feature(PinId, PinNumber)
        .assert pin_feature(string, string).
        % connected(Pin, Net)
        .assert connected(string, string).
        % series_link(Net, Net)
        .assert series_link(string, string).
        % pin_count(Component, Count)
        .assert pin_count(string, integer).

        % Standard library rules (recursive pathfinding)
        % path(Net, Net)
        .infer path(string, string).
        path(N, N) <- connected(_, N).
        path(A, B) <- series_link(A, I), path(I, B).
        "#
    );
    let datalog_source = format!("{}{}", datalog_source, user_query);

    let mut program = parse_str(&datalog_source)
        .map_err(|e| anyhow::anyhow!("Datalog parse error:\n{}", e))?
        .into_parsed();

    let p_pin = Predicate::from_str("pin").unwrap();
    let p_function = Predicate::from_str("pin_function").unwrap();
    let p_feature = Predicate::from_str("pin_feature").unwrap();
    let p_connected = Predicate::from_str("connected").unwrap();
    let p_series = Predicate::from_str("series_link").unwrap();
    let p_pin_count = Predicate::from_str("pin_count").unwrap();

    if program.extensional().get(&p_pin).is_none() {
        program
            .add_new_extensional_relation(
                p_pin.clone().into(),
                vec![
                    Attribute::string(),
                    Attribute::string(),
                    Attribute::string(),
                ],
            )
            .unwrap();
    }
    if program.extensional().get(&p_function).is_none() {
        program
            .add_new_extensional_relation(
                p_function.clone().into(),
                vec![Attribute::string(), Attribute::string()],
            )
            .unwrap();
    }
    if program.extensional().get(&p_feature).is_none() {
        program
            .add_new_extensional_relation(
                p_feature.clone().into(),
                vec![Attribute::string(), Attribute::string()],
            )
            .unwrap();
    }
    if program.extensional().get(&p_connected).is_none() {
        program
            .add_new_extensional_relation(
                p_connected.clone().into(),
                vec![Attribute::string(), Attribute::string()],
            )
            .unwrap();
    }
    if program.extensional().get(&p_series).is_none() {
        program
            .add_new_extensional_relation(
                p_series.clone().into(),
                vec![Attribute::string(), Attribute::string()],
            )
            .unwrap();
    }
    if program.extensional().get(&p_pin_count).is_none() {
        program
            .add_new_extensional_relation(
                p_pin_count.clone().into(),
                vec![Attribute::string(), Attribute::integer()],
            )
            .unwrap();
    }

    let mut comp_nets: HashMap<String, Vec<String>> = HashMap::new();
    let mut comp_pin_counts: HashMap<String, usize> = HashMap::new();

    for pin in &netlist.pins {
        comp_nets
            .entry(pin.component_ref.clone())
            .or_default()
            .push(pin.net_name.clone());
        *comp_pin_counts
            .entry(pin.component_ref.clone())
            .or_default() += 1;
    }

    {
        let ext = program.extensional_mut();

        if let Some(pin_rel) = ext.get_mut(&p_pin) {
            for pin in &netlist.pins {
                pin_rel
                    .add_as_fact(vec![
                        Constant::from(pin_id(&pin)),
                        Constant::from(pin.component_ref.clone()),
                        Constant::from(pin.pin_number.clone()),
                    ])
                    .ok();
            }
        }

        if let Some(function_rel) = ext.get_mut(&p_function) {
            for pin in &netlist.pins {
                if let Some(func) = &pin.pin_function {
                    function_rel
                        .add_as_fact(vec![
                            Constant::from(pin_id(&pin)),
                            Constant::from(func.clone()),
                        ])
                        .ok();
                }
            }
        }

        if let Some(feature_rel) = ext.get_mut(&p_feature) {
            for pin in &netlist.pins {
                let pin_id = pin_id(&pin);
                if let Some(func) = &pin.pin_function {
                    for part in func.split('/') {
                        feature_rel
                            .add_as_fact(vec![
                                Constant::from(pin_id.clone()),
                                Constant::from(part.to_string()),
                            ])
                            .ok();

                        if let Some(idx) = part.rfind('_') {
                            if part[idx + 1..].chars().all(|c| c.is_ascii_digit()) {
                                let base = &part[..idx];
                                feature_rel
                                    .add_as_fact(vec![
                                        Constant::from(pin_id.clone()),
                                        Constant::from(base.to_string()),
                                    ])
                                    .ok();
                            }
                        }
                    }
                }
            }
        }

        if let Some(conn_rel) = ext.get_mut(&p_connected) {
            for pin in &netlist.pins {
                conn_rel
                    .add_as_fact(vec![
                        Constant::from(pin_id(&pin)),
                        Constant::from(pin.net_name.clone()),
                    ])
                    .ok();
            }
        }

        if let Some(series_rel) = ext.get_mut(&p_series) {
            for (comp, nets) in comp_nets {
                if is_series_passive(&comp) && nets.len() == 2 {
                    let net_a = &nets[0];
                    let net_b = &nets[1];
                    if net_a != net_b && !net_a.is_empty() && !net_b.is_empty() {
                        series_rel
                            .add_as_fact(vec![
                                Constant::from(net_a.clone()),
                                Constant::from(net_b.clone()),
                            ])
                            .ok();
                        series_rel
                            .add_as_fact(vec![
                                Constant::from(net_b.clone()),
                                Constant::from(net_a.clone()),
                            ])
                            .ok();
                    }
                }
            }
        }

        if let Some(count_rel) = ext.get_mut(&p_pin_count) {
            for (comp, count) in comp_pin_counts {
                count_rel
                    .add_as_fact(vec![Constant::from(comp), Constant::from(count as i64)])
                    .ok();
            }
        }
    }

    program
        .run(NaiveEvaluator::default(), false)
        .map_err(|e| anyhow::anyhow!("Evaluation failed: {:?}", e))?;

    let intensional = program.intensional();
    let mut results: HashMap<String, Vec<Vec<Constant>>> = HashMap::new();

    for relation in intensional.iter() {
        let rel_name = relation.label().to_string();

        if !user_rules.contains(&rel_name) {
            continue;
        }

        let mut rows = Vec::new();
        for fact in relation.iter() {
            rows.push(fact.values().to_vec());
        }

        if !rows.is_empty() {
            results.insert(rel_name, rows);
        }
    }

    Ok(results)
}
