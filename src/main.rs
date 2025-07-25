use anyhow::Result;
use clap::Parser;
use colored::*;
use dialoguer::{theme::ColorfulTheme, Select, Input, Password};
use git_timetraveler::{create_time_traveled_repo, ProgressCallback, TimeTravelConfig};
use indicatif::{ProgressBar, ProgressStyle};
use std::process;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use ctrlc;
use std::panic;

/// Create GitHub repositories with backdated commits to show early years in your profile
#[derive(Parser)]
#[command(name = "git-timetraveler")]
#[command(about = "Travel back in time on your GitHub profile")]
#[command(version = "0.1.0")]
struct Args {
    /// Year to travel back to (e.g., 1990)
    #[arg(short, long, default_value = "1990")]
    year: u32,

    /// Range of years to travel back to (e.g., 1990-2001)
    #[arg(long)]
    years: Option<String>,

    /// GitHub username
    #[arg(short, long)]
    username: Option<String>,

    /// GitHub personal access token
    #[arg(short, long)]
    token: Option<String>,

    /// Month (1-12)
    #[arg(short, long, default_value = "1")]
    month: u32,

    /// Day (1-31)
    #[arg(short, long, default_value = "1")]
    day: u32,

    /// Hour (0-23)
    #[arg(long, default_value = "18")]
    hour: u32,

    /// Custom repository name (defaults to year)
    #[arg(long)]
    repo: Option<String>,

    /// Branch to push commits to (defaults to main)
    #[arg(long, default_value = "main")]
    branch: String,

    /// Skip confirmation prompts
    #[arg(short = 'y', long)]
    yes: bool,

    /// Force push (overwrite remote branch, use with caution)
    #[arg(long, default_value_t = false)]
    force: bool,
}

struct CliProgressBar {
    pb: ProgressBar,
}

impl CliProgressBar {
    fn new() -> Self {
        let pb = ProgressBar::new(6);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] {bar:40.cyan/blue} {pos:>2}/{len:2} {msg}")
                .unwrap()
                .progress_chars("█▉▊▋▌▍▎▏  "),
        );
        Self { pb }
    }
}

impl ProgressCallback for CliProgressBar {
    fn set_message(&self, message: &str) {
        self.pb.set_message(message.to_string());
    }

    fn increment(&self) {
        self.pb.inc(1);
    }

    fn finish(&self, message: &str) {
        self.pb.finish_with_message(message.to_string());
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Set a panic hook for user-friendly error reporting
    panic::set_hook(Box::new(|info| {
        eprintln!("\n❌ An unexpected error occurred: {}
If this is a bug, please report it at https://github.com/chama-x/Git-Timetraveler/issues", info);
        std::process::exit(1);
    }));

    // Handle Ctrl+C gracefully
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        println!("\nExiting. Goodbye!");
        r.store(false, Ordering::SeqCst);
        process::exit(0);
    }).expect("Error setting Ctrl-C handler");

    // Welcome screen
    println!("{}", r#"
   ____ _ _   _                 _                     _           _           
  / ___(_) |_| |_ ___ _ __ __ _| |_ ___  ___ ___   __| | ___  ___| |_ ___ _ __ 
 | |  _| | __| __/ _ \ '__/ _` | __/ _ \/ __/ __| / _` |/ _ \/ __| __/ _ \ '__|
 | |_| | | |_| ||  __/ | | (_| | ||  __/\__ \__ \| (_| |  __/ (__| ||  __/ |   
  \____|_|\__|\__\___|_|  \__,_|\__\___||___/___(_)__,_|\___|\___|\__\___|_|   
    "#.bright_blue().bold());
    println!("{}", "Welcome to Git Time Traveler!\n".cyan().bold());
    println!("{}", "Travel back in time on your GitHub profile with ease!\n".cyan());

    let menu_items = vec![
        "Create backdated commit(s)",
        "View usage examples",
        "Learn about GitHub tokens",
        "Exit",
    ];
    assert!(!menu_items.is_empty(), "Menu items list must not be empty");
    assert!(0 < menu_items.len(), "Default index for menu must be valid");
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("What would you like to do?")
        .items(&menu_items)
        .default(0)
        .interact()?;

    match selection {
        0 => {
            // Guided workflow for creating backdated commits (robust)
            println!("\nLet's create a backdated commit! (Enter 'q' to quit at any prompt)\n");
            let username: String = loop {
                let input: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Enter your GitHub username")
                    .interact_text()?;
                if input.trim().eq_ignore_ascii_case("q") || input.trim().eq_ignore_ascii_case("quit") {
                    println!("Exiting. Goodbye!");
                    process::exit(0);
                }
                if input.trim().is_empty() {
                    println!("{}", "Username cannot be empty.".red());
                    continue;
                }
                break input;
            };
            let token: String = loop {
                let input: String = Password::with_theme(&ColorfulTheme::default())
                    .with_prompt("Enter your GitHub token (input hidden)")
                    .interact()?;
                if input.trim().eq_ignore_ascii_case("q") || input.trim().eq_ignore_ascii_case("quit") {
                    println!("Exiting. Goodbye!");
                    process::exit(0);
                }
                if input.trim().is_empty() {
                    println!("{}", "Token cannot be empty.".red());
                    continue;
                }
                break input;
            };
            let repo: String = loop {
                let input: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Enter the repository name")
                    .interact_text()?;
                if input.trim().eq_ignore_ascii_case("q") || input.trim().eq_ignore_ascii_case("quit") {
                    println!("Exiting. Goodbye!");
                    process::exit(0);
                }
                if input.trim().is_empty() {
                    println!("{}", "Repository name cannot be empty.".red());
                    continue;
                }
                break input;
            };
            let branch: String = loop {
                let input: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Enter the branch name (default: main)")
                    .default("main".to_string())
                    .interact_text()?;
                if input.trim().eq_ignore_ascii_case("q") || input.trim().eq_ignore_ascii_case("quit") {
                    println!("Exiting. Goodbye!");
                    process::exit(0);
                }
                if input.trim().is_empty() {
                    println!("{}", "Branch name cannot be empty.".red());
                    continue;
                }
                break input;
            };

            // Year or range
            let year_mode_items = vec!["Single year", "Range of years"];
            assert!(!year_mode_items.is_empty(), "Year mode items list must not be empty");
            assert!(0 < year_mode_items.len(), "Default index for year mode must be valid");
            let year_mode = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Would you like to enter a single year or a range?")
                .items(&year_mode_items)
                .default(0)
                .interact()?;
            let (years, years_str) = if year_mode == 0 {
                // Single year
                let year: u32 = loop {
                    let input: String = Input::with_theme(&ColorfulTheme::default())
                        .with_prompt("Enter the year (e.g., 2000)")
                        .interact_text()?;
                    if input.trim().eq_ignore_ascii_case("q") || input.trim().eq_ignore_ascii_case("quit") {
                        println!("Exiting. Goodbye!");
                        process::exit(0);
                    }
                    match input.trim().parse::<u32>() {
                        Ok(y) if y >= 1970 && y <= 2100 => break y,
                        _ => println!("{}", "Please enter a valid year between 1970 and 2100.".red()),
                    }
                };
                (vec![year], year.to_string())
            } else {
                // Range
                let (start, end) = loop {
                    let input_start: String = Input::with_theme(&ColorfulTheme::default())
                        .with_prompt("Enter the start year (e.g., 1990)")
                        .interact_text()?;
                    if input_start.trim().eq_ignore_ascii_case("q") || input_start.trim().eq_ignore_ascii_case("quit") {
                        println!("Exiting. Goodbye!");
                        process::exit(0);
                    }
                    if input_start.trim().is_empty() {
                        println!("{}", "Start year cannot be empty or whitespace.".red());
                        continue;
                    }
                    let input_end: String = Input::with_theme(&ColorfulTheme::default())
                        .with_prompt("Enter the end year (e.g., 1995)")
                        .interact_text()?;
                    if input_end.trim().eq_ignore_ascii_case("q") || input_end.trim().eq_ignore_ascii_case("quit") {
                        println!("Exiting. Goodbye!");
                        process::exit(0);
                    }
                    if input_end.trim().is_empty() {
                        println!("{}", "End year cannot be empty or whitespace.".red());
                        continue;
                    }
                    match (input_start.trim().parse::<u32>(), input_end.trim().parse::<u32>()) {
                        (Ok(s), Ok(e)) if s >= 1970 && s <= 2100 && e >= s && e <= 2100 => {
                            if e < s {
                                println!("[DEBUG] About to construct invalid range: start={}, end={}", s, e);
                                println!("Error: End year must be greater than or equal to start year ({}).", s);
                                process::exit(1);
                            }
                            break (s, e)
                        },
                        (Ok(s), Ok(e)) if e < s => println!("{}", format!("End year must be greater than or equal to start year ({}).", s).red()),
                        _ => println!("{}", "Please enter valid years between 1970 and 2100, and ensure end year >= start year.".red()),
                    }
                };
                let num_years = end - start + 1;
                if num_years > 10 {
                    let bulk_items = ["Yes, proceed", "No, cancel"];
                    assert!(!bulk_items.is_empty(), "Bulk confirm items list must not be empty");
                    assert!(1 < bulk_items.len(), "Default index for bulk confirm must be valid");
                    let confirm_bulk = Select::with_theme(&ColorfulTheme::default())
                        .with_prompt(&format!("You are about to create commits for {} years. Are you sure you want to proceed?", num_years))
                        .items(&bulk_items)
                        .default(1)
                        .interact()? == 0;
                    if !confirm_bulk {
                        println!("Operation cancelled by user.");
                        process::exit(0);
                    }
                }
                ((start..=end).collect(), format!("{}-{}", start, end))
            };

            let month: u32 = loop {
                let input: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Enter the month (1-12)")
                    .default("1".to_string())
                    .interact_text()?;
                if input.trim().eq_ignore_ascii_case("q") || input.trim().eq_ignore_ascii_case("quit") {
                    println!("Exiting. Goodbye!");
                    process::exit(0);
                }
                match input.trim().parse::<u32>() {
                    Ok(m) if m >= 1 && m <= 12 => break m,
                    _ => println!("{}", "Please enter a valid month (1-12).".red()),
                }
            };
            let day: u32 = loop {
                let input: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Enter the day (1-31)")
                    .default("1".to_string())
                    .interact_text()?;
                if input.trim().eq_ignore_ascii_case("q") || input.trim().eq_ignore_ascii_case("quit") {
                    println!("Exiting. Goodbye!");
                    process::exit(0);
                }
                match input.trim().parse::<u32>() {
                    Ok(d) if d >= 1 && d <= 31 => break d,
                    _ => println!("{}", "Please enter a valid day (1-31).".red()),
                }
            };
            let hour: u32 = loop {
                let input: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Enter the hour (0-23)")
                    .default("18".to_string())
                    .interact_text()?;
                if input.trim().eq_ignore_ascii_case("q") || input.trim().eq_ignore_ascii_case("quit") {
                    println!("Exiting. Goodbye!");
                    process::exit(0);
                }
                match input.trim().parse::<u32>() {
                    Ok(h) if h <= 23 => break h,
                    _ => println!("{}", "Please enter a valid hour (0-23).".red()),
                }
            };

            // Force push
            let force_items = vec!["No (safe, recommended)", "Yes (force push, overwrite history)"];
            assert!(!force_items.is_empty(), "Force items list must not be empty");
            assert!(0 < force_items.len(), "Default index for force must be valid");
            let force = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Do you want to force push? (use with caution)")
                .items(&force_items)
                .default(0)
                .interact()? == 1;

            // Summary and confirmation
            println!("\nSummary:");
            println!("- Username: {}", username.bright_green());
            println!("- Repo: {}", repo.bright_yellow());
            println!("- Branch: {}", branch.bright_cyan());
            println!("- Years: {}", years_str.bright_magenta());
            println!("- Date: {:02}-{:02} at {:02}:00", month, day, hour);
            println!("- Force push: {}", if force { "Yes".red() } else { "No".green() });

            let confirm_items = vec!["Proceed", "Cancel"];
            assert!(!confirm_items.is_empty(), "Confirm items list must not be empty");
            assert!(0 < confirm_items.len(), "Default index for confirm must be valid");
            let confirm = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Do you want to proceed?")
                .items(&confirm_items)
                .default(0)
                .interact()? == 0;
            if !confirm {
                println!("{}", "Operation cancelled by user.".yellow());
                return Ok(());
            }

            // (Stub) Call the actual commit logic here
            println!("\n(Stub) This is where the time travel commit logic would run.\n");
        }
        1 => {
            println!("\nUsage examples will be shown here. (Stub)\n");
        }
        2 => {
            println!("\nLearn about GitHub tokens will be shown here. (Stub)\n");
        }
        3 => {
            println!("\nGoodbye!\n");
            return Ok(());
        }
        _ => unreachable!(),
    }

    Ok(())
}

fn get_username(username: Option<String>) -> Result<String> {
    match username {
        Some(u) => Ok(u),
        None => {
            let username: String = Input::new()
                .with_prompt("GitHub Username")
                .interact_text()?;
            Ok(username)
        }
    }
}

fn get_token(token: Option<String>) -> Result<String> {
    match token {
        Some(t) => Ok(t),
        None => {
            let token: String = Password::new()
                .with_prompt("GitHub Personal Access Token")
                .interact()?;
            Ok(token)
        }
    }
}
