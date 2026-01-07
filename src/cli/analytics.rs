//! Analytics command handler - displays local usage statistics from stats-cache.json

use colored::Colorize;
use comfy_table::{presets::UTF8_FULL_CONDENSED, Cell, Color, Table};
use serde::Serialize;

use super::output::print_json;
use super::OutputFormat;
use crate::core::config::get_default_profile;
use crate::core::profile::{list_profiles, load_profile};
use crate::core::stats::{load_global_stats, load_profile_stats, StatsCache};
use crate::error::RafctlError;

struct ModelPricing {
    input_per_million: f64,
    output_per_million: f64,
}

const PRICING: &[(&str, ModelPricing)] = &[
    (
        "claude-sonnet-4-5",
        ModelPricing {
            input_per_million: 3.0,
            output_per_million: 15.0,
        },
    ),
    (
        "claude-opus-4-5",
        ModelPricing {
            input_per_million: 15.0,
            output_per_million: 75.0,
        },
    ),
    (
        "claude-haiku-4-5",
        ModelPricing {
            input_per_million: 0.80,
            output_per_million: 4.0,
        },
    ),
    (
        "claude-haiku-3-5",
        ModelPricing {
            input_per_million: 0.25,
            output_per_million: 1.25,
        },
    ),
];

const OUTPUT_TO_INPUT_RATIO: f64 = 3.0;

#[derive(Debug, Serialize)]
struct AnalyticsOutput {
    profile: Option<String>,
    days: usize,
    daily_activity: Vec<DailyActivityOutput>,
    totals: TotalsOutput,
    models: Vec<ModelOutput>,
}

#[derive(Debug, Serialize)]
struct DailyActivityOutput {
    date: String,
    messages: u64,
    sessions: u64,
    tools: u64,
    tokens: u64,
}

#[derive(Debug, Serialize)]
struct TotalsOutput {
    messages: u64,
    sessions: u64,
    tools: u64,
    tokens: u64,
}

#[derive(Debug, Serialize)]
struct ModelOutput {
    name: String,
    tokens: u64,
    percentage: f64,
}

#[derive(Debug, Serialize)]
struct AllProfilesOutput {
    profiles: Vec<ProfileSummary>,
    totals: TotalsOutput,
}

#[derive(Debug, Serialize, Clone)]
struct ProfileSummary {
    name: String,
    tool: String,
    messages_7d: u64,
    tokens_7d: u64,
    last_active: Option<String>,
}

#[derive(Debug, Serialize)]
struct CostOutput {
    profile: Option<String>,
    days: usize,
    models: Vec<ModelCostOutput>,
    total_estimated: f64,
}

#[derive(Debug, Serialize)]
struct ModelCostOutput {
    name: String,
    input_tokens: u64,
    input_cost: f64,
    output_cost_estimated: f64,
    total_cost_estimated: f64,
}

pub fn handle_analytics(
    profile_name: Option<&str>,
    days: usize,
    show_all: bool,
    show_cost: bool,
    format: OutputFormat,
) -> Result<(), RafctlError> {
    if show_cost {
        show_cost_estimate(profile_name, days, format)
    } else if show_all {
        show_all_profiles_analytics(days, format)
    } else {
        show_single_analytics(profile_name, days, format)
    }
}

fn show_single_analytics(
    profile_name: Option<&str>,
    days: usize,
    format: OutputFormat,
) -> Result<(), RafctlError> {
    // Determine which profile/stats to use
    let (stats, profile_display) = match profile_name {
        Some(name) => {
            let name_lower = name.to_lowercase();
            let profile = load_profile(&name_lower)?;
            let stats = load_profile_stats(&name_lower, profile.tool);
            (stats, Some(name_lower))
        }
        None => {
            // Try default profile, fall back to global
            if let Ok(Some(default)) = get_default_profile() {
                if let Ok(profile) = load_profile(&default) {
                    let stats = load_profile_stats(&default, profile.tool);
                    (stats, Some(default))
                } else {
                    (load_global_stats(), None)
                }
            } else {
                (load_global_stats(), None)
            }
        }
    };

    if stats.is_empty() {
        match format {
            OutputFormat::Json => {
                print_json(&AnalyticsOutput {
                    profile: profile_display,
                    days,
                    daily_activity: vec![],
                    totals: TotalsOutput {
                        messages: 0,
                        sessions: 0,
                        tools: 0,
                        tokens: 0,
                    },
                    models: vec![],
                });
            }
            _ => {
                println!(
                    "{} No usage data found. Run Claude Code to generate statistics.",
                    "ℹ".cyan()
                );
            }
        }
        return Ok(());
    }

    // Build output data
    let output = build_analytics_output(&stats, profile_display.clone(), days);

    match format {
        OutputFormat::Json => {
            print_json(&output);
        }
        OutputFormat::Plain => {
            print_plain_analytics(&output);
        }
        OutputFormat::Human => {
            print_human_analytics(&output, &stats);
        }
    }

    Ok(())
}

fn build_analytics_output(
    stats: &StatsCache,
    profile: Option<String>,
    days: usize,
) -> AnalyticsOutput {
    let recent_activity = stats.recent_activity(days);
    let _recent_tokens = stats.recent_tokens(days);

    // Build daily activity with tokens
    let daily_activity: Vec<DailyActivityOutput> = recent_activity
        .iter()
        .map(|a| {
            let tokens = stats.tokens_for_date(&a.date);
            DailyActivityOutput {
                date: a.date.clone(),
                messages: a.message_count,
                sessions: a.session_count,
                tools: a.tool_call_count,
                tokens,
            }
        })
        .collect();

    // Calculate totals
    let totals = TotalsOutput {
        messages: daily_activity.iter().map(|d| d.messages).sum(),
        sessions: daily_activity.iter().map(|d| d.sessions).sum(),
        tools: daily_activity.iter().map(|d| d.tools).sum(),
        tokens: daily_activity.iter().map(|d| d.tokens).sum(),
    };

    // Model breakdown
    let model_tokens = stats.aggregate_tokens_by_model(Some(days));
    let total_tokens: u64 = model_tokens.values().sum();

    let mut models: Vec<ModelOutput> = model_tokens
        .into_iter()
        .map(|(name, tokens)| {
            let percentage = if total_tokens > 0 {
                (tokens as f64 / total_tokens as f64) * 100.0
            } else {
                0.0
            };
            ModelOutput {
                name,
                tokens,
                percentage,
            }
        })
        .collect();

    // Sort by tokens descending
    models.sort_by(|a, b| b.tokens.cmp(&a.tokens));

    AnalyticsOutput {
        profile,
        days,
        daily_activity,
        totals,
        models,
    }
}

fn print_human_analytics(output: &AnalyticsOutput, _stats: &StatsCache) {
    // Header
    let profile_str = output
        .profile
        .as_ref()
        .map(|p| format!(" — Profile: {}", p))
        .unwrap_or_default();

    println!(
        "\n{} {} (last {} days)\n",
        "📊".cyan(),
        format!("Usage Analytics{}", profile_str).bold(),
        output.days
    );

    // Daily activity table
    if !output.daily_activity.is_empty() {
        let mut table = Table::new();
        table.load_preset(UTF8_FULL_CONDENSED);
        table.set_header(vec!["Date", "Messages", "Sessions", "Tools", "Tokens"]);

        for day in &output.daily_activity {
            table.add_row(vec![
                Cell::new(&day.date),
                Cell::new(day.messages),
                Cell::new(day.sessions),
                Cell::new(day.tools),
                Cell::new(format_tokens(day.tokens)),
            ]);
        }

        println!("{table}\n");
    }

    // Totals
    println!(
        "{}: {} messages · {} sessions · {} tool calls · {} tokens\n",
        "Totals".bold(),
        output.totals.messages.to_string().cyan(),
        output.totals.sessions.to_string().cyan(),
        output.totals.tools.to_string().cyan(),
        format_tokens(output.totals.tokens).cyan()
    );

    // Model breakdown with progress bars
    if !output.models.is_empty() {
        println!("{}", "Models Used:".bold());
        for model in &output.models {
            let bar = progress_bar(model.percentage, 10);
            let display_name = shorten_model_name(&model.name);
            println!(
                "  {} {:<20} {:>8} ({:.1}%)",
                bar,
                display_name,
                format_tokens(model.tokens),
                model.percentage
            );
        }
        println!();
    }
}

fn print_plain_analytics(output: &AnalyticsOutput) {
    println!(
        "PROFILE\t{}\tDAYS\t{}",
        output.profile.as_deref().unwrap_or("global"),
        output.days
    );
    println!("DATE\tMESSAGES\tSESSIONS\tTOOLS\tTOKENS");
    for day in &output.daily_activity {
        println!(
            "{}\t{}\t{}\t{}\t{}",
            day.date, day.messages, day.sessions, day.tools, day.tokens
        );
    }
    println!(
        "TOTAL\t{}\t{}\t{}\t{}",
        output.totals.messages, output.totals.sessions, output.totals.tools, output.totals.tokens
    );
}

fn show_all_profiles_analytics(days: usize, format: OutputFormat) -> Result<(), RafctlError> {
    let profile_names = list_profiles()?;

    if profile_names.is_empty() {
        match format {
            OutputFormat::Json => {
                print_json(&AllProfilesOutput {
                    profiles: vec![],
                    totals: TotalsOutput {
                        messages: 0,
                        sessions: 0,
                        tools: 0,
                        tokens: 0,
                    },
                });
            }
            _ => {
                println!("{} No profiles found.", "ℹ".cyan());
            }
        }
        return Ok(());
    }

    let mut summaries: Vec<ProfileSummary> = Vec::new();
    let mut total_messages = 0u64;
    let mut total_tokens = 0u64;

    for name in &profile_names {
        if let Ok(profile) = load_profile(name) {
            let stats = load_profile_stats(name, profile.tool);

            let recent_activity = stats.recent_activity(days);
            let messages_7d: u64 = recent_activity.iter().map(|a| a.message_count).sum();
            let tokens_7d = stats.total_tokens(Some(days));

            let last_active = recent_activity.first().map(|a| a.date.clone());

            total_messages += messages_7d;
            total_tokens += tokens_7d;

            summaries.push(ProfileSummary {
                name: name.clone(),
                tool: profile.tool.to_string(),
                messages_7d,
                tokens_7d,
                last_active,
            });
        }
    }

    // Sort by tokens descending
    summaries.sort_by(|a, b| b.tokens_7d.cmp(&a.tokens_7d));

    let output = AllProfilesOutput {
        profiles: summaries.clone(),
        totals: TotalsOutput {
            messages: total_messages,
            sessions: 0, // Not aggregated for simplicity
            tools: 0,
            tokens: total_tokens,
        },
    };

    match format {
        OutputFormat::Json => {
            print_json(&output);
        }
        OutputFormat::Plain => {
            println!("PROFILE\tTOOL\tMESSAGES_7D\tTOKENS_7D\tLAST_ACTIVE");
            for s in &summaries {
                println!(
                    "{}\t{}\t{}\t{}\t{}",
                    s.name,
                    s.tool,
                    s.messages_7d,
                    s.tokens_7d,
                    s.last_active.as_deref().unwrap_or("-")
                );
            }
            println!("TOTAL\t-\t{}\t{}\t-", total_messages, total_tokens);
        }
        OutputFormat::Human => {
            println!(
                "\n{} {} (last {} days)\n",
                "📊".cyan(),
                "Cross-Profile Analytics".bold(),
                days
            );

            let mut table = Table::new();
            table.load_preset(UTF8_FULL_CONDENSED);
            table.set_header(vec!["Profile", "Tool", "Messages", "Tokens", "Last Active"]);

            for s in &summaries {
                table.add_row(vec![
                    Cell::new(&s.name).fg(Color::Cyan),
                    Cell::new(&s.tool),
                    Cell::new(s.messages_7d),
                    Cell::new(format_tokens(s.tokens_7d)),
                    Cell::new(s.last_active.as_deref().unwrap_or("—")),
                ]);
            }

            // Add totals row
            table.add_row(vec![
                Cell::new("Total").fg(Color::Yellow),
                Cell::new("—"),
                Cell::new(total_messages),
                Cell::new(format_tokens(total_tokens)),
                Cell::new("—"),
            ]);

            println!("{table}\n");
        }
    }

    Ok(())
}

/// Format token count for display (e.g., 1.5M, 320K, 1234)
fn format_tokens(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.0}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

/// Create a simple progress bar
fn progress_bar(percentage: f64, width: usize) -> String {
    let filled = ((percentage / 100.0) * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);

    let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));

    if percentage >= 50.0 {
        bar.green().to_string()
    } else if percentage >= 25.0 {
        bar.yellow().to_string()
    } else {
        bar.dimmed().to_string()
    }
}

/// Shorten model names for display
fn shorten_model_name(name: &str) -> String {
    name.replace("claude-", "")
        .replace("-20", " 20")
        .replace("-4-5", " 4.5")
        .replace("-3-5", " 3.5")
}

fn show_cost_estimate(
    profile_name: Option<&str>,
    days: usize,
    format: OutputFormat,
) -> Result<(), RafctlError> {
    let (stats, profile_display) = match profile_name {
        Some(name) => {
            let name_lower = name.to_lowercase();
            let profile = load_profile(&name_lower)?;
            let stats = load_profile_stats(&name_lower, profile.tool);
            (stats, Some(name_lower))
        }
        None => {
            if let Ok(Some(default)) = get_default_profile() {
                if let Ok(profile) = load_profile(&default) {
                    let stats = load_profile_stats(&default, profile.tool);
                    (stats, Some(default))
                } else {
                    (load_global_stats(), None)
                }
            } else {
                (load_global_stats(), None)
            }
        }
    };

    if stats.is_empty() {
        match format {
            OutputFormat::Json => {
                print_json(&CostOutput {
                    profile: profile_display,
                    days,
                    models: vec![],
                    total_estimated: 0.0,
                });
            }
            _ => {
                println!(
                    "{} No usage data found. Run Claude Code to generate statistics.",
                    "ℹ".cyan()
                );
            }
        }
        return Ok(());
    }

    let model_tokens = stats.aggregate_tokens_by_model(Some(days));
    let mut model_costs: Vec<ModelCostOutput> = model_tokens
        .into_iter()
        .map(|(name, input_tokens)| {
            let pricing = get_model_pricing(&name);
            let input_cost = (input_tokens as f64 / 1_000_000.0) * pricing.input_per_million;
            let estimated_output_tokens = (input_tokens as f64 * OUTPUT_TO_INPUT_RATIO) as u64;
            let output_cost =
                (estimated_output_tokens as f64 / 1_000_000.0) * pricing.output_per_million;
            let total = input_cost + output_cost;

            ModelCostOutput {
                name,
                input_tokens,
                input_cost,
                output_cost_estimated: output_cost,
                total_cost_estimated: total,
            }
        })
        .collect();

    model_costs.sort_by(|a, b| {
        b.total_cost_estimated
            .partial_cmp(&a.total_cost_estimated)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let total_estimated: f64 = model_costs.iter().map(|m| m.total_cost_estimated).sum();

    let output = CostOutput {
        profile: profile_display.clone(),
        days,
        models: model_costs,
        total_estimated,
    };

    match format {
        OutputFormat::Json => {
            print_json(&output);
        }
        OutputFormat::Plain => {
            print_plain_cost(&output);
        }
        OutputFormat::Human => {
            print_human_cost(&output);
        }
    }

    Ok(())
}

fn get_model_pricing(model_name: &str) -> ModelPricing {
    for (pattern, pricing) in PRICING {
        if model_name.contains(pattern) {
            return ModelPricing {
                input_per_million: pricing.input_per_million,
                output_per_million: pricing.output_per_million,
            };
        }
    }
    ModelPricing {
        input_per_million: 3.0,
        output_per_million: 15.0,
    }
}

fn print_human_cost(output: &CostOutput) {
    let profile_str = output
        .profile
        .as_ref()
        .map(|p| format!(" — Profile: {}", p))
        .unwrap_or_default();

    println!(
        "\n{} {} (last {} days)\n",
        "💰".cyan(),
        format!("Estimated Costs{}", profile_str).bold(),
        output.days
    );

    let mut table = Table::new();
    table.load_preset(UTF8_FULL_CONDENSED);
    table.set_header(vec![
        "Model",
        "Input Tokens",
        "Input Cost",
        "Output Cost*",
        "Total Est.",
    ]);

    for model in &output.models {
        table.add_row(vec![
            Cell::new(shorten_model_name(&model.name)),
            Cell::new(format_tokens(model.input_tokens)),
            Cell::new(format!("${:.2}", model.input_cost)),
            Cell::new(format!("~${:.2}", model.output_cost_estimated)),
            Cell::new(format!("~${:.2}", model.total_cost_estimated)).fg(Color::Cyan),
        ]);
    }

    table.add_row(vec![
        Cell::new(""),
        Cell::new(""),
        Cell::new(""),
        Cell::new("Total:").fg(Color::Yellow),
        Cell::new(format!("~${:.2}", output.total_estimated)).fg(Color::Yellow),
    ]);

    println!("{table}\n");

    println!(
        "{}",
        "* Output tokens estimated at 3:1 ratio (not tracked locally)".dimmed()
    );
    println!();
}

fn print_plain_cost(output: &CostOutput) {
    println!(
        "PROFILE\t{}\tDAYS\t{}",
        output.profile.as_deref().unwrap_or("global"),
        output.days
    );
    println!("MODEL\tINPUT_TOKENS\tINPUT_COST\tOUTPUT_COST_EST\tTOTAL_EST");
    for model in &output.models {
        println!(
            "{}\t{}\t{:.2}\t{:.2}\t{:.2}",
            model.name,
            model.input_tokens,
            model.input_cost,
            model.output_cost_estimated,
            model.total_cost_estimated
        );
    }
    println!("TOTAL\t\t\t\t{:.2}", output.total_estimated);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_tokens() {
        assert_eq!(format_tokens(500), "500");
        assert_eq!(format_tokens(1500), "2K");
        assert_eq!(format_tokens(150000), "150K");
        assert_eq!(format_tokens(1500000), "1.5M");
        assert_eq!(format_tokens(2500000), "2.5M");
    }

    #[test]
    fn test_shorten_model_name() {
        assert_eq!(shorten_model_name("claude-sonnet-4-5"), "sonnet 4.5");
        assert_eq!(shorten_model_name("claude-opus-4-5"), "opus 4.5");
        assert_eq!(shorten_model_name("claude-haiku-3-5"), "haiku 3.5");
    }

    #[test]
    fn test_progress_bar() {
        let bar = progress_bar(50.0, 10);
        assert!(bar.contains("█████"));
    }
}
