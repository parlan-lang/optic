mod ir;
mod module;
mod cfg;
mod ssa;
mod codegen;

use std::{collections::HashMap, time::Instant};

fn main() {
    let mut input = "";
    let mut output = "";
    let mut time_report = false;

    let args: Vec<String> = std::env::args().collect();

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        match arg.as_str() {
            "--help" => {
                println!(
                    r#"Usage: {} [OPTIONS] INPUT 

Options:
    --help        Display this message and exits
    -o <FILENAME> Write output to FILENAME
    --time-report Prints a simple time report
"#,
                    args[0]
                );
                return;
            }
            "--time-report" => time_report = true,
            "-o" => {
                i += 1;
                output = &args[i];
            }
            _ => input = arg
        }
        i += 1;
    }

    let source = std::fs::read_to_string(input).expect("error: could not open the source file");

    let mut parser = ir::ir_parser::IrParser::new(&source);

    let mut start = Instant::now();
    let mut module = parser.parse_module(input);
    let parse_time = start.elapsed().as_secs_f32();

    start = Instant::now();
    module.build_cfg();
    let cfg_build_time = start.elapsed().as_secs_f32();

    for func in &mut module.functions {
        ssa::builder::build_ssa(&mut func.cfg);
    }

    let mut vreg_aliases: HashMap<&String, ssa::VregAlias> = HashMap::new();
    for func in &module.functions {
        vreg_aliases.insert(&func.name, ssa::VregAlias::new());
        for blk in &func.cfg.blocks {
            for ins in &blk.instructions {
                match ins {
                    module::instruction::Instruction::Phi { vreg, srcs, .. } => {
                        srcs.iter().for_each(|v| vreg_aliases.get_mut(&func.name).unwrap().union(*vreg, v.as_vreg().unwrap()));
                    }
                    _ => continue
                }
            }
        }
    }

    let mut codegen = codegen::c_backend::CBackend::new(output, &module, vreg_aliases);

    start = Instant::now();
    codegen.compile();
    let codegen_time = start.elapsed().as_secs_f32();

    let total_time = parse_time + cfg_build_time + codegen_time;

    if time_report {
        println!(
            r#"--- Optic Time Report ---

Parsing ..... {:.5}s ({:.1}%)
CFG build ... {:.5}s ({:.1}%)
Codegen ..... {:.5}s ({:.1}%)
-----------------------------
Total Time:   {:.5}s
"#,
            parse_time, (parse_time / total_time) * 100.0,
            cfg_build_time, (cfg_build_time / total_time) * 100.0,
            codegen_time, (codegen_time / total_time) * 100.0,
            total_time
        );
    }
}