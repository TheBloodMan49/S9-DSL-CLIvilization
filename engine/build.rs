use regex::Regex;
use std::env;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;

const AST_FILE_PATH: &str = "../lang/generated/ast.ts";
const NODE_REGEX: &str =
    r"export interface (?<NAME>[^ ]+) extends langium\.AstNode \{\n(?<PROPS>[^}]+)\n}";
const PROPERTY_REGEX: &str = r" +(?<NAME>[a-zA-Z_]+): (?<TYPE>[a-zA-Z<>_]+);";

// TODO: Finish the parsing

fn main() {
    // Tell cargo when to rerun
    println!("cargo:rerun-if-changed={}", AST_FILE_PATH);

    // Preparing out dir
    let out_dir_str = env::var("OUT_DIR").expect("missing OUT_DIR env var");
    let out_dir = Path::new(&out_dir_str);

    // Creating writer
    let mut source_file = BufWriter::new(
        File::create(out_dir.join("ast.rs")).expect("failed to create source source file"),
    );

    let content = std::fs::read_to_string(AST_FILE_PATH).expect("failed to read file");

    let node_regex = Regex::new(NODE_REGEX).expect("failed to compile regex pattern");
    let property_regex = Regex::new(PROPERTY_REGEX).expect("failed to compile regex pattern");

    for capture in node_regex.captures_iter(&content) {
        writeln!(
            source_file,
            "-- {} --",
            capture.name("NAME").expect("no capture group").as_str()
        )
        .expect("failed to write to source source file");

        for s_capture in
            property_regex.captures_iter(capture.name("PROPS").expect("no capture group").as_str())
        {
            writeln!(
                source_file,
                "{}: {}",
                s_capture.name("NAME").expect("no capture group").as_str(),
                s_capture.name("TYPE").expect("no capture group").as_str()
            )
            .expect("failed to write to source source file");
        }
    }
}
