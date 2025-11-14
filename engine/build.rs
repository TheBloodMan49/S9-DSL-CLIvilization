use regex::Regex;
use std::env;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;

const AST_FILE_PATH: &str = "../lang/generated/ast.ts";
const NODE_REGEX: &str =
    r"export interface (?<NAME>[a-zA-Z_]+) extends langium\.AstNode \{\n(?<PROPS>[^}]+)\n}";
const PROPERTY_REGEX: &str = r"\s+(?<NAME>[a-zA-Z_]+)(?<OPTION>\?|): (?<TYPE>[a-zA-Z<>_]+);";
const ALIAS_REGEX: &str = r"export type (?<NAME>[a-zA-Z_]+) = (?<PROP>[a-zA-Z_]+);";
const ENUM_REGEX: &str =
    r#"export type (?<NAME>[a-zA-Z_]+) = (?<PROPS>[a-zA-Z_]+(\s*\|\s*[a-zA-Z_]+)+);"#;
const ENUM_VARIANT_REGEX: &str = r#"\s*(\|\s+|)(?<TYPE>[a-zA-Z_]+)"#;
const TAGGED_ENUM_REGEX: &str =
    r#"export type (?<NAME>[a-zA-Z_]+) = (?<PROPS>'[a-zA-Z_]+'(\s*\|\s*'[a-zA-Z_]+')+);"#;
const TAGGED_ENUM_VARIANT_REGEX: &str = r#"\s*(\|\s+|)(?<TYPE>'[a-zA-Z_]+')"#;

fn main() {
    // Tell cargo when to rerun
    println!("cargo:rerun-if-changed={}", AST_FILE_PATH);

    // Preparing output directory
    let out_dir_str = env::var("OUT_DIR").expect("missing OUT_DIR env var");
    let out_dir = Path::new(&out_dir_str);

    // Creating writer
    let mut source_file = BufWriter::new(
        File::create(out_dir.join("ast.rs")).expect("failed to create source source file"),
    );
    let content = std::fs::read_to_string(AST_FILE_PATH).expect("failed to read file");

    // Generate file
    writeln!(source_file, "use serde::{{Serialize, Deserialize}};\n")
        .expect("failed to write to source file");
    generate_aliases(&mut source_file, &content);
    generate_tagged_enums(&mut source_file, &content);
    generate_enums(&mut source_file, &content);
    generate_nodes(&mut source_file, &content);
}

fn process_type(type_name: &str) -> String {
    type_name
        .replace("string", "String")
        .replace("number", "u32")
        .replace("boolean", "bool")
        .replace("Array<", "Vec<")
}

fn generate_aliases(source_file: &mut BufWriter<File>, content: &str) {
    let alias_regex = Regex::new(ALIAS_REGEX).expect("failed to compile regex pattern");

    for capture in alias_regex.captures_iter(content) {
        writeln!(
            source_file,
            "pub type {} = {};\n",
            capture.name("NAME").expect("no capture group").as_str(),
            process_type(capture.name("PROP").expect("no capture group").as_str())
        )
        .expect("failed to write to source source file");
    }
}

fn generate_enums(source_file: &mut BufWriter<File>, content: &str) {
    let enum_regex = Regex::new(ENUM_REGEX).expect("failed to compile regex pattern");
    let enum_variant_regex =
        Regex::new(ENUM_VARIANT_REGEX).expect("failed to compile regex pattern");

    for capture in enum_regex.captures_iter(content) {
        let name = capture.name("NAME").expect("no capture group").as_str();

        if name.contains("TokenNames") || name.contains("KeywordNames") {
            continue;
        }

        writeln!(
            source_file,
            "#[derive(Serialize, Deserialize, Debug)]\n#[serde(untagged)]\npub enum {} {{",
            capture.name("NAME").expect("no capture group").as_str()
        )
        .expect("failed to write to source source file");

        for s_capture in enum_variant_regex
            .captures_iter(capture.name("PROPS").expect("no capture group").as_str())
        {
            let type_name = s_capture.name("TYPE").expect("no capture group").as_str();

            writeln!(source_file, "    {type_name}({type_name}),")
                .expect("failed to write to source source file");
        }

        writeln!(source_file, "}}\n").expect("failed to write to source source file");
    }
}

fn generate_tagged_enums(source_file: &mut BufWriter<File>, content: &str) {
    let tagged_enum_regex = Regex::new(TAGGED_ENUM_REGEX).expect("failed to compile regex pattern");
    let tagged_enum_variant_regex =
        Regex::new(TAGGED_ENUM_VARIANT_REGEX).expect("failed to compile regex pattern");

    for capture in tagged_enum_regex.captures_iter(content) {
        let name = capture.name("NAME").expect("no capture group").as_str();

        if name.contains("TokenNames") || name.contains("KeywordNames") {
            continue;
        }

        writeln!(
            source_file,
            "#[derive(Serialize, Deserialize, Debug)]\npub enum {} {{",
            capture.name("NAME").expect("no capture group").as_str()
        )
            .expect("failed to write to source source file");

        for s_capture in tagged_enum_variant_regex
            .captures_iter(capture.name("PROPS").expect("no capture group").as_str())
        {
            let type_name = s_capture.name("TYPE").expect("no capture group").as_str();
                writeln!(
                    source_file,
                    "    {},",
                    type_name.replace("\"", "").replace("'", "")
                )
                    .expect("failed to write to source source file");
        }

        writeln!(source_file, "}}\n").expect("failed to write to source source file");
    }
}

fn generate_nodes(source_file: &mut BufWriter<File>, content: &str) {
    let node_regex = Regex::new(NODE_REGEX).expect("failed to compile regex pattern");
    let property_regex = Regex::new(PROPERTY_REGEX).expect("failed to compile regex pattern");

    for capture in node_regex.captures_iter(content) {
        writeln!(
            source_file,
            "#[derive(Serialize, Deserialize, Debug)]\npub struct {} {{",
            capture.name("NAME").expect("no capture group").as_str(),
        )
        .expect("failed to write to source source file");

        for s_capture in
            property_regex.captures_iter(capture.name("PROPS").expect("no capture group").as_str())
        {
            if s_capture.name("OPTION").expect("no capture group").as_str().is_empty() {
                writeln!(
                    source_file,
                    "    pub {}: {},",
                    s_capture.name("NAME").expect("no capture group").as_str(),
                    process_type(s_capture.name("TYPE").expect("no capture group").as_str())
                )
                    .expect("failed to write to source source file");
            } else {
                writeln!(
                    source_file,
                    "    pub {}: Option<{}>,",
                    s_capture.name("NAME").expect("no capture group").as_str(),
                    process_type(s_capture.name("TYPE").expect("no capture group").as_str())
                )
                    .expect("failed to write to source source file");
            }
        }

        writeln!(source_file, "}}\n").expect("failed to write to source source file");
    }
}
