mod ir;
mod module;
mod cfg;
mod ssa;
mod codegen;

use std::{collections::HashMap, time::Instant};

// simple auxiliar function to format durations
fn format_time(secs: f32, total: f32) -> String {
    let abs_secs = secs.abs();
    let format_sec = if abs_secs >= 1.0 { (secs, "sec") }
                    else if abs_secs >= 1e-3 { (secs * 1_000.0, "ms") }
                    else if abs_secs >= 1e-6 { (secs * 1_000_000.0, "µs") }
                    else { (secs * 1_000_000_000.0, "ns") };
    format!("{:>5.1} {} ({:>5.1}%)", format_sec.0, format_sec.1, (secs / total) * 100.0)
    
}

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

    start = Instant::now();
    for func in &mut module.functions {
        // extern functions doesn't have a body
        if !func.is_extern {
            let param_types: HashMap<ssa::Vreg, module::instruction::Type> = func
                .params
                .iter()
                .map(|p| (ssa::Vreg(p.vreg), p.ty))
                .collect();

            ssa::builder::build_ssa(&mut func.cfg, &param_types);
        }
    }
    let ssa_con_time = start.elapsed().as_secs_f32();

    start = Instant::now();
    for func in &mut module.functions {
        func.cfg.split_critical_edges();
        let mut next_vreg = func.cfg.get_max_vreg();
        func.cfg.lower_phis_to_moves(&mut next_vreg);
    }
    let ssa_decon_time = start.elapsed().as_secs_f32();

    let mut codegen = codegen::c_backend::CBackend::new(output, &mut module);

    start = Instant::now();
    codegen.compile();
    let codegen_time = start.elapsed().as_secs_f32();

    let total_time = parse_time + cfg_build_time + ssa_con_time + ssa_decon_time + codegen_time;

    if time_report {
        println!(
            r#"
----- Optic Time Report -----
Parsing .............. {}
CFG build ............ {}
SSA construction ..... {}
SSA deconstruction ... {}
Codegen .............. {}
-----------------------------
Total Time:   {:.5}s
"#,
            format_time(parse_time, total_time),
            format_time(cfg_build_time, total_time),
            format_time(ssa_con_time, total_time),
            format_time(ssa_decon_time, total_time),
            format_time(codegen_time, total_time),
            total_time
        );
    }
}