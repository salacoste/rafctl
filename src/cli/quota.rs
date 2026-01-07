use std::time::Duration;

use colored::Colorize;
use serde::{Deserialize, Serialize};

use super::output::print_json;
use super::OutputFormat;
use crate::core::profile::{list_profiles, load_profile, profile_exists, AuthMode, ToolType};
use crate::error::RafctlError;

#[cfg(target_os = "macos")]
use crate::tools::keychain;

const ANTHROPIC_USAGE_API: &str = "https://api.anthropic.com/api/oauth/usage";
const API_TIMEOUT_SECS: u64 = 30;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageWindow {
    pub utilization: f64,
    pub resets_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageLimits {
    pub five_hour: Option<UsageWindow>,
    pub seven_day: Option<UsageWindow>,
}

#[derive(Debug, Serialize)]
struct QuotaOutput {
    profile: String,
    tool: String,
    auth_mode: String,
    usage: Option<UsageLimits>,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct AllQuotaOutput {
    profiles: Vec<QuotaOutput>,
}

pub fn handle_quota(profile_name: Option<&str>, format: OutputFormat) -> Result<(), RafctlError> {
    match profile_name {
        Some(name) => show_single_quota(name, format),
        None => show_all_quota(format),
    }
}

fn show_single_quota(profile_name: &str, format: OutputFormat) -> Result<(), RafctlError> {
    let name_lower = profile_name.to_lowercase();

    if !profile_exists(&name_lower)? {
        return Err(RafctlError::ProfileNotFound(name_lower));
    }

    let profile = load_profile(&name_lower)?;

    if profile.tool != ToolType::Claude {
        match format {
            OutputFormat::Json => {
                print_json(&QuotaOutput {
                    profile: name_lower.clone(),
                    tool: profile.tool.to_string(),
                    auth_mode: profile.auth_mode.to_string(),
                    usage: None,
                    error: Some("Quota monitoring only available for Claude profiles".to_string()),
                });
            }
            _ => {
                eprintln!(
                    "{} Quota monitoring only available for Claude profiles",
                    "ℹ".cyan()
                );
            }
        }
        return Ok(());
    }

    if profile.auth_mode != AuthMode::OAuth {
        match format {
            OutputFormat::Json => {
                print_json(&QuotaOutput {
                    profile: name_lower.clone(),
                    tool: profile.tool.to_string(),
                    auth_mode: profile.auth_mode.to_string(),
                    usage: None,
                    error: Some("Quota monitoring only available for OAuth mode".to_string()),
                });
            }
            _ => {
                eprintln!(
                    "{} Quota monitoring only available for OAuth mode (API key mode has no quota limits)",
                    "ℹ".cyan()
                );
            }
        }
        return Ok(());
    }

    let usage = fetch_usage_for_profile(&name_lower);

    match format {
        OutputFormat::Json => {
            let (usage_data, error_msg) = match &usage {
                Ok(u) => (Some(u.clone()), None),
                Err(e) => (None, Some(e.to_string())),
            };
            let output = QuotaOutput {
                profile: name_lower.clone(),
                tool: profile.tool.to_string(),
                auth_mode: profile.auth_mode.to_string(),
                usage: usage_data,
                error: error_msg,
            };
            print_json(&output);
        }
        OutputFormat::Plain => {
            print_usage_plain(&name_lower, &usage);
        }
        OutputFormat::Human => {
            print_usage_human(&name_lower, &usage);
        }
    }

    Ok(())
}

fn show_all_quota(format: OutputFormat) -> Result<(), RafctlError> {
    let profiles = list_profiles()?;

    if profiles.is_empty() {
        match format {
            OutputFormat::Json => {
                print_json(&AllQuotaOutput { profiles: vec![] });
            }
            _ => {
                println!("No profiles found.");
            }
        }
        return Ok(());
    }

    let mut outputs: Vec<QuotaOutput> = Vec::new();

    for name in &profiles {
        if let Ok(profile) = load_profile(name) {
            if profile.tool == ToolType::Claude && profile.auth_mode == AuthMode::OAuth {
                let usage = fetch_usage_for_profile(name);
                let (usage_data, error_msg) = match usage {
                    Ok(u) => (Some(u), None),
                    Err(e) => (None, Some(e.to_string())),
                };
                outputs.push(QuotaOutput {
                    profile: name.clone(),
                    tool: profile.tool.to_string(),
                    auth_mode: profile.auth_mode.to_string(),
                    usage: usage_data,
                    error: error_msg,
                });
            }
        }
    }

    if outputs.is_empty() {
        match format {
            OutputFormat::Json => {
                print_json(&AllQuotaOutput { profiles: vec![] });
            }
            _ => {
                println!(
                    "{} No Claude OAuth profiles found for quota monitoring",
                    "ℹ".cyan()
                );
            }
        }
        return Ok(());
    }

    match format {
        OutputFormat::Json => {
            print_json(&AllQuotaOutput { profiles: outputs });
        }
        OutputFormat::Plain => {
            println!("PROFILE\t5H_USAGE\t5H_RESET\t7D_USAGE\t7D_RESET");
            for o in &outputs {
                if let Some(usage) = &o.usage {
                    let five_h = usage
                        .five_hour
                        .as_ref()
                        .map(|w| format!("{:.1}%", w.utilization))
                        .unwrap_or_else(|| "-".to_string());
                    let five_h_reset = usage
                        .five_hour
                        .as_ref()
                        .and_then(|w| w.resets_at.clone())
                        .unwrap_or_else(|| "-".to_string());
                    let seven_d = usage
                        .seven_day
                        .as_ref()
                        .map(|w| format!("{:.1}%", w.utilization))
                        .unwrap_or_else(|| "-".to_string());
                    let seven_d_reset = usage
                        .seven_day
                        .as_ref()
                        .and_then(|w| w.resets_at.clone())
                        .unwrap_or_else(|| "-".to_string());
                    println!(
                        "{}\t{}\t{}\t{}\t{}",
                        o.profile, five_h, five_h_reset, seven_d, seven_d_reset
                    );
                } else {
                    println!(
                        "{}\t{}\t-\t-\t-",
                        o.profile,
                        o.error.as_deref().unwrap_or("error")
                    );
                }
            }
        }
        OutputFormat::Human => {
            println!("{}", "Quota Usage:".bold());
            for o in &outputs {
                match &o.usage {
                    Some(usage) => print_usage_human_data(&o.profile, usage),
                    None => {
                        println!("  {} {}", "•".cyan(), o.profile.white().bold());
                        println!(
                            "    {} {}",
                            "✗".red(),
                            o.error.as_deref().unwrap_or("Unknown error").dimmed()
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn fetch_usage_for_profile(profile_name: &str) -> Result<UsageLimits, RafctlError> {
    let token = keychain::read_oauth_token(profile_name)?
        .ok_or_else(|| RafctlError::NotAuthenticated(profile_name.to_string()))?;

    fetch_usage_from_api(&token)
}

#[cfg(not(target_os = "macos"))]
fn fetch_usage_for_profile(_profile_name: &str) -> Result<UsageLimits, RafctlError> {
    Err(RafctlError::KeychainError(
        "Quota monitoring requires macOS for keychain access".to_string(),
    ))
}

fn fetch_usage_from_api(token: &str) -> Result<UsageLimits, RafctlError> {
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(API_TIMEOUT_SECS))
        .build();

    let response = agent
        .get(ANTHROPIC_USAGE_API)
        .set("Accept", "application/json")
        .set("Content-Type", "application/json")
        .set(
            "User-Agent",
            &format!("rafctl/{}", env!("CARGO_PKG_VERSION")),
        )
        .set("Authorization", &format!("Bearer {}", token))
        .set("anthropic-beta", "oauth-2025-04-20")
        .call()
        .map_err(|e| RafctlError::KeychainError(format!("API request failed: {}", e)))?;

    let usage: UsageLimits = response
        .into_json()
        .map_err(|e| RafctlError::KeychainError(format!("Failed to parse response: {}", e)))?;

    Ok(usage)
}

fn print_usage_human(profile_name: &str, usage: &Result<UsageLimits, RafctlError>) {
    println!("  {} {}", "•".cyan(), profile_name.white().bold());

    match usage {
        Ok(u) => print_usage_data(u),
        Err(e) => {
            println!("    {} {}", "✗".red(), e.to_string().dimmed());
        }
    }
}

fn print_usage_human_data(profile_name: &str, usage: &UsageLimits) {
    println!("  {} {}", "•".cyan(), profile_name.white().bold());
    print_usage_data(usage);
}

fn print_usage_data(u: &UsageLimits) {
    if let Some(five_h) = &u.five_hour {
        let bar = usage_bar(five_h.utilization);
        let reset = five_h
            .resets_at
            .as_ref()
            .map(|r| format_reset_time(r))
            .unwrap_or_default();
        println!(
            "    5-hour:  {} {:.1}% {}",
            bar,
            five_h.utilization,
            reset.dimmed()
        );
    }
    if let Some(seven_d) = &u.seven_day {
        let bar = usage_bar(seven_d.utilization);
        let reset = seven_d
            .resets_at
            .as_ref()
            .map(|r| format_reset_time(r))
            .unwrap_or_default();
        println!(
            "    7-day:   {} {:.1}% {}",
            bar,
            seven_d.utilization,
            reset.dimmed()
        );
    }
}

fn print_usage_plain(profile_name: &str, usage: &Result<UsageLimits, RafctlError>) {
    match usage {
        Ok(u) => {
            let five_h = u
                .five_hour
                .as_ref()
                .map(|w| format!("{:.1}", w.utilization))
                .unwrap_or_else(|| "-".to_string());
            let seven_d = u
                .seven_day
                .as_ref()
                .map(|w| format!("{:.1}", w.utilization))
                .unwrap_or_else(|| "-".to_string());
            println!("{}: 5h={}% 7d={}%", profile_name, five_h, seven_d);
        }
        Err(e) => {
            println!("{}: error={}", profile_name, e);
        }
    }
}

fn usage_bar(percentage: f64) -> String {
    let filled = (percentage / 10.0).round() as usize;
    let empty = 10 - filled.min(10);

    let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));

    if percentage >= 80.0 {
        bar.red().to_string()
    } else if percentage >= 50.0 {
        bar.yellow().to_string()
    } else {
        bar.green().to_string()
    }
}

fn format_reset_time(iso_time: &str) -> String {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(iso_time) {
        let now = chrono::Utc::now();
        let duration = dt.signed_duration_since(now);

        if duration.num_hours() > 0 {
            format!("(resets in {}h)", duration.num_hours())
        } else if duration.num_minutes() > 0 {
            format!("(resets in {}m)", duration.num_minutes())
        } else {
            "(resets soon)".to_string()
        }
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usage_bar_low() {
        let bar = usage_bar(25.0);
        assert!(bar.contains("██"));
    }

    #[test]
    fn test_usage_bar_high() {
        let bar = usage_bar(85.0);
        assert!(bar.contains("████████"));
    }

    #[test]
    fn test_format_reset_time_invalid() {
        let result = format_reset_time("invalid");
        assert!(result.is_empty());
    }
}
