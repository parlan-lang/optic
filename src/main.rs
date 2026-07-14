mod ir;
mod module;
mod cfg;
mod codegen;

use std::time::Instant;

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

    let mut codegen = codegen::c_backend::CBackend::new(output, &module);

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