//! Debug/verbose output utilities.
//!
//! Provides macros and functions for conditional debug output
//! when --verbose flag is enabled.

use std::sync::atomic::{AtomicBool, Ordering};

use colored::Colorize;

/// Global flag for verbose mode
static VERBOSE: AtomicBool = AtomicBool::new(false);

/// Enable verbose mode globally
pub fn enable_verbose() {
    VERBOSE.store(true, Ordering::SeqCst);
}

/// Check if verbose mode is enabled
pub fn is_verbose() -> bool {
    VERBOSE.load(Ordering::SeqCst)
}

/// Print a debug message if verbose mode is enabled
pub fn debug(msg: &str) {
    if is_verbose() {
        eprintln!("{} {}", "[debug]".dimmed(), msg.dimmed());
    }
}

/// Print a debug message with a label if verbose mode is enabled
pub fn debug_labeled(label: &str, msg: &str) {
    if is_verbose() {
        eprintln!("{} {}: {}", "[debug]".dimmed(), label.cyan(), msg.dimmed());
    }
}

/// Print debug info about a path
pub fn debug_path(label: &str, path: &std::path::Path) {
    if is_verbose() {
        let exists = if path.exists() { "exists" } else { "missing" };
        eprintln!(
            "{} {}: {} ({})",
            "[debug]".dimmed(),
            label.cyan(),
            path.display().to_string().dimmed(),
            exists.dimmed()
        );
    }
}

/// Print debug info about an environment variable
pub fn debug_env(name: &str, value: &str) {
    if is_verbose() {
        eprintln!(
            "{} {}: {}={}",
            "[debug]".dimmed(),
            "env".cyan(),
            name.yellow(),
            value.dimmed()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verbose_default_off() {
        // Reset state
        VERBOSE.store(false, Ordering::SeqCst);
        assert!(!is_verbose());
    }

    #[test]
    fn test_enable_verbose() {
        VERBOSE.store(false, Ordering::SeqCst);
        enable_verbose();
        assert!(is_verbose());
        // Reset for other tests
        VERBOSE.store(false, Ordering::SeqCst);
    }
}
