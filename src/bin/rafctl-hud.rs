//! rafctl-hud - Native Rust HUD for Claude Code statusline.

use std::process;

fn main() {
    if let Err(e) = rafctl::hud::run_hud() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
