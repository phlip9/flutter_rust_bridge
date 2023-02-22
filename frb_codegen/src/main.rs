use std::process::exit;

use clap::Parser;
use lib_flutter_rust_bridge_codegen::{
    config_parse, frb_codegen, frb_codegen_all, get_symbols_if_no_duplicates, init_logger, RawOpts,
};
use log::{debug, error, info};

fn main() {
    let raw_opts = RawOpts::parse();

    init_logger("./logs/", raw_opts.verbose).unwrap();

    if let Err(_) = frb_codegen_all(&raw_opts) {
        exit(1);
    }
}
