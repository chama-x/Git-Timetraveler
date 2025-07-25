use anyhow::Result;
use clap::Parser;
use colored::*;
use dialoguer::{Input, Password};
use git_timetraveler::{create_time_traveled_repo, ProgressCallback, TimeTravelConfig};
use indicatif::{ProgressBar, ProgressStyle};

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
    let args = Args::parse();

    println!("{}", "🚀 Git Time Traveler".bright_blue().bold());
    println!("{}", "Travel back in time on your GitHub profile!".cyan());
    println!();

    // Get username and token (from args or prompt) - clone to avoid partial move
    let username = get_username(args.username.clone())?;
    let token = get_token(args.token.clone())?;

    // Parse years range if provided
    let years: Vec<u32> = if let Some(ref range) = args.years {
        let parts: Vec<&str> = range.split('-').collect();
        if parts.len() == 2 {
            let start: u32 = parts[0].parse().expect("Invalid start year");
            let end: u32 = parts[1].parse().expect("Invalid end year");
            (start..=end).collect()
        } else if parts.len() == 1 {
            vec![parts[0].parse().expect("Invalid year")] 
        } else {
            panic!("Invalid years format. Use e.g. 1990-2001");
        }
    } else {
        vec![args.year]
    };

    // Confirmation
    if !args.yes {
        let years_str = if years.len() > 1 {
            format!("{}-{}", years.first().unwrap(), years.last().unwrap())
        } else {
            years[0].to_string()
        };
        let confirm = dialoguer::Confirm::new()
            .with_prompt(&format!("Do you want to proceed with years: {}?", years_str))
            .default(true)
            .interact()?;
        if !confirm {
            println!("{}", "Operation cancelled.".yellow());
            return Ok(());
        }
    }

    for year in years {
        // Create and validate configuration
        let config = TimeTravelConfig::new(
            year,
            args.month,
            args.day,
            args.hour,
            username.clone(),
            token.clone(),
            args.repo.clone(),
            args.branch.clone(),
        )?;

        // Show summary
        println!("\n📅 Target date: {}", config.formatted_date());
        println!("👤 GitHub user: {}", username.bright_green());
        println!("📦 Repository: {}", config.repo_name().bright_yellow());
        println!();

        // Create progress bar
        let progress = CliProgressBar::new();

        // Execute the time travel
        create_time_traveled_repo(&config, Some(&progress)).await?;
    }

    println!();
    println!("{}", "🎉 Success!".bright_green().bold());
    println!("Check your profile: {}", 
             format!("https://github.com/{}", username).bright_blue().underline());

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
