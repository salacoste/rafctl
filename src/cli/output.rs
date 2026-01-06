use serde::Serialize;

use super::OutputFormat;

pub fn print_success(message: &str, format: OutputFormat) {
    match format {
        OutputFormat::Human => println!("\u{2713} {}", message),
        OutputFormat::Plain => println!("OK: {}", message),
        OutputFormat::Json => print_json(&SuccessMessage { ok: true, message }),
    }
}

pub fn print_error(message: &str, format: OutputFormat) {
    match format {
        OutputFormat::Human => eprintln!("\u{2717} {}", message),
        OutputFormat::Plain => eprintln!("ERROR: {}", message),
        OutputFormat::Json => print_json(&ErrorMessage {
            ok: false,
            error: message,
        }),
    }
}

pub fn print_info(message: &str, format: OutputFormat) {
    match format {
        OutputFormat::Human => println!("\u{2139} {}", message),
        OutputFormat::Plain => println!("INFO: {}", message),
        OutputFormat::Json => {}
    }
}

pub fn print_json<T: Serialize>(data: &T) {
    if let Ok(json) = serde_json::to_string_pretty(data) {
        println!("{}", json);
    }
}

#[derive(Serialize)]
struct SuccessMessage<'a> {
    ok: bool,
    message: &'a str,
}

#[derive(Serialize)]
struct ErrorMessage<'a> {
    ok: bool,
    error: &'a str,
}
