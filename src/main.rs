use anyhow::{Context, Result};
use asdi::edb::Constant;
use clap::Parser;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;
use std::path::PathBuf;

use netlistquery::{Netlist, execute_query, kicad::parse_kicad_netlist};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the KiCad netlist file (.net)
    netlist: PathBuf,

    /// The datalog query to execute (e.g., 'find_mcu_pin(MCU_PinName) :- ...'). If omitted, starts a REPL.
    query: Option<String>,
}

fn run_query(netlist: &Netlist, query: &str) {
    match execute_query(netlist, query) {
        Ok(results) => {
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
        }
        Err(e) => {
            eprintln!("Error executing query:\n{:?}", e);
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    let repl = args.query.is_none();

    if repl {
        println!("Parsing KiCad netlist '{}'...", args.netlist.display());
    }

    let netlist_content = std::fs::read_to_string(&args.netlist)
        .with_context(|| format!("Failed to read netlist file: {}", args.netlist.display()))?;
    let netlist =
        parse_kicad_netlist(&netlist_content).context("Failed to parse netlist content")?;

    if repl {
        println!(
            "Loaded {} nets and {} physical pins.",
            netlist.nets.len(),
            netlist.pins.len()
        );
    }

    if let Some(query) = args.query {
        // Single-shot execution
        run_query(&netlist, &query);
    } else {
        // REPL mode
        println!("Starting interactive Datalog REPL. Type 'exit' or 'quit' to close.");
        println!("A query will be executed when the input ends with a dot ('.').");

        let mut rl = DefaultEditor::new()?;
        let mut buffer = String::new();

        loop {
            let prompt = if buffer.is_empty() {
                "netlistquery> "
            } else {
                "... "
            };
            let readline = rl.readline(prompt);

            match readline {
                Ok(line) => {
                    let trimmed = line.trim();

                    if buffer.is_empty() && (trimmed == "exit" || trimmed == "quit") {
                        break;
                    }

                    if buffer.is_empty() && trimmed.is_empty() {
                        continue;
                    }

                    buffer.push_str(&line);
                    buffer.push('\n');

                    if buffer.trim().ends_with('.') {
                        rl.add_history_entry(buffer.trim())?;
                        run_query(&netlist, &buffer);
                        buffer.clear();
                    }
                }
                Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                    break;
                }
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            }
        }
    }

    Ok(())
}

