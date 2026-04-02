use anyhow::{Context, Result};
use asdi::edb::Constant;
use clap::Parser;
use std::path::PathBuf;

use netlistquery::{execute_query, kicad::parse_kicad_netlist};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the KiCad netlist file (.net)
    netlist: PathBuf,
    /// The datalog query to execute (e.g., 'find_mcu_pin(MCU_PinName) :- ...')
    query: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // 1. Read and parse the EDA netlist file into memory
    let netlist_content = std::fs::read_to_string(&args.netlist)
        .with_context(|| format!("Failed to read netlist file: {}", args.netlist.display()))?;
    let netlist =
        parse_kicad_netlist(&netlist_content).context("Failed to parse netlist content")?;

    // 2. Execute the user query against the parsed netlist
    let results =
        execute_query(&netlist, &args.query).context("Failed to execute Datalog query")?;

    // 3. Print the results
    if results.is_empty() {
        println!("No results found.");
    } else {
        for (rel_name, rows) in results {
            for row in rows {
                let clean_vals: Vec<String> = row
                    .iter()
                    .filter_map(|c| match c {
                        Constant::String(s) => Some(s.clone()),
                        Constant::Number(n) => Some(n.to_string()),
                        Constant::Boolean(b) => Some(b.to_string()),
                    })
                    .collect();
                println!("Result: {}({})", rel_name, clean_vals.join(", "));
            }
        }
    }

    Ok(())
}
