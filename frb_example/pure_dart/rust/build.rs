use lib_flutter_rust_bridge_codegen::{frb_codegen_all, RawOpts};

/// Path of input Rust code
const RUST_INPUT: &str = "src/api.rs";
/// Path of output generated Dart code
const DART_OUTPUT: &str = "../dart/lib/bridge_generated.dart";

fn main() {
    // Tell Cargo that if the input Rust code changes, to rerun this build script.
    println!("cargo:rerun-if-changed={RUST_INPUT}");
    // Options for frb_codegen
    let raw_opts = RawOpts {
        // Path of input Rust code
        rust_input: vec![RUST_INPUT.to_string()],
        // Path of output generated Dart code
        dart_output: vec![DART_OUTPUT.to_string()],
        wasm: true,
        dart_decl_output: Some("../dart/lib/bridge_definitions.dart".into()),
        dart_format_line_length: 120,
        // TODO: try removing
        inline_rust: true,
        // for other options use defaults
        ..Default::default()
    };

    // Generate the Rust FFI binding code
    frb_codegen_all(raw_opts).expect("flutter_rust_bridge codegen failed");
}
