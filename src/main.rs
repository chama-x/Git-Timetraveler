use anyhow::{Result, Context};
use clap::Parser;
use colored::*;
use dialoguer::{theme::ColorfulTheme, Select, Input, Password};
use git_timetraveler::{create_time_traveled_repo_with_options, ProgressCallback, TimeTravelConfig, InteractivePrompts, AuthorMode, format_error_for_user, display_and_confirm_dry_run};
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use std::process;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use ctrlc;
use std::panic;
use atty::Stream;

/// Create GitHub repositories with backdated commits to show early years in your profile
#[derive(Parser)]
#[command(name = "git-timetraveler")]
#[command(about = "Create backdated commits to enhance your GitHub contribution graph")]
#[command(version)]
#[command(long_about = "Git Time Traveler creates backdated commits and pushes them to GitHub repositories,
allowing you to populate your contribution graph with activity from earlier years.

EXAMPLES:
    # Interactive mode (recommended for first-time users)
    git-timetraveler

    # Non-interactive mode for single year
    git-timetraveler --no-menu --username myuser --token ghp_xxx --repo myrepo --year 1990

    # Non-interactive mode for year range
    git-timetraveler --no-menu --username myuser --token ghp_xxx --repo myrepo --years 1990-1995

    # Expert mode with all options
    git-timetraveler --no-menu --username myuser --token ghp_xxx --repo myrepo \\
        --years 1990,1992,1994 --hour 14 --author-name \"John Doe\" \\
        --author-email john@example.com --message \"Custom commit message\"

For more information, visit: https://github.com/chama-x/Git-Timetraveler")]
struct Args {
    /// Year to travel back to (e.g., 1990)
    #[arg(short, long, default_value = "1990")]
    year: u32,

    /// Range or list of years (e.g., 1990-1995 or 1990,1992,1994)
    #[arg(long, value_name = "YEARS")]
    years: Option<String>,

    /// GitHub username
    #[arg(short, long, value_name = "USERNAME")]
    username: Option<String>,

    /// GitHub personal access token
    #[arg(short, long, value_name = "TOKEN", hide = true)]
    token: Option<String>,

    /// Month (1-12)
    #[arg(short, long, default_value = "1", value_name = "MONTH")]
    month: u32,

    /// Day (1-31)
    #[arg(short, long, default_value = "1", value_name = "DAY")]
    day: u32,

    /// Hour for commits (0-23)
    #[arg(long, default_value = "18", value_name = "HOUR")]
    hour: u32,

    /// Repository name (defaults to year if not specified)
    #[arg(long, value_name = "REPO")]
    repo: Option<String>,

    /// Branch to push commits to
    #[arg(long, default_value = "main", value_name = "BRANCH")]
    branch: String,

    /// Author name for commits (overrides Git config)
    #[arg(long, value_name = "NAME")]
    author_name: Option<String>,

    /// Author email for commits (overrides Git config)
    #[arg(long, value_name = "EMAIL")]
    author_email: Option<String>,

    /// Custom commit message template (use {year} placeholder)
    #[arg(long, value_name = "MESSAGE")]
    message: Option<String>,

    /// Skip all confirmation prompts
    #[arg(long)]
    yes: bool,

    /// Force push (overwrite remote branch - use with caution)
    #[arg(long)]
    force: bool,

    /// Run in non-interactive mode (no menu, use only CLI args)
    #[arg(long)]
    no_menu: bool,

    /// Enable verbose output for debugging
    #[arg(short, long)]
    verbose: bool,

    /// Dry run - show what would be done without making changes
    #[arg(long)]
    dry_run: bool,

    /// Quiet mode - minimal output
    #[arg(short, long)]
    quiet: bool,

    /// Create repository if it doesn't exist
    #[arg(long)]
    create_repo: bool,

    /// Make repository private (only when creating new repo)
    #[arg(long)]
    private: bool,

    /// Repository description (only when creating new repo)
    #[arg(long, value_name = "DESCRIPTION")]
    description: Option<String>,
}

/// Enhanced progress bar with better visual feedback and status tracking
struct CliProgressBar {
    pb: ProgressBar,
    multi_progress: Option<Arc<MultiProgress>>,
    current_step: Arc<std::sync::atomic::AtomicUsize>,
    total_steps: usize,
}

impl CliProgressBar {
    /// Create a new progress bar for single operations
    fn new() -> Self {
        let pb = ProgressBar::new(6);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] {bar:40.cyan/blue} {pos:>2}/{len:2} {msg}")
                .unwrap()
                .progress_chars("█▉▊▋▌▍▎▏  "),
        );
        Self { 
            pb,
            multi_progress: None,
            current_step: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            total_steps: 6,
        }
    }

    /// Create a new progress bar for multi-year operations
    fn new_multi_year(total_years: usize) -> Self {
        let multi = Arc::new(MultiProgress::new());
        let pb = multi.add(ProgressBar::new((total_years * 6) as u64));
        
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] {bar:40.cyan/blue} {pos:>3}/{len:3} {msg}")
                .unwrap()
                .progress_chars("█▉▊▋▌▍▎▏  "),
        );

        Self {
            pb,
            multi_progress: Some(multi),
            current_step: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            total_steps: total_years * 6,
        }
    }

    /// Create a year-specific sub-progress bar
    fn create_year_progress(&self, year: u32) -> Option<ProgressBar> {
        if let Some(ref multi) = self.multi_progress {
            let year_pb = multi.add(ProgressBar::new(6));
            year_pb.set_style(
                ProgressStyle::default_bar()
                    .template(&format!("  {} {{spinner:.blue}} [{{elapsed_precise}}] {{bar:30.cyan/blue}} {{pos}}/{{len}} {{msg}}", 
                        format!("Year {}:", year).bright_yellow()))
                    .unwrap()
                    .progress_chars("█▉▊▋▌▍▎▏  "),
            );
            Some(year_pb)
        } else {
            None
        }
    }

    /// Set a status message with step information
    fn set_status(&self, message: &str, step: usize, total: usize) {
        let formatted_msg = format!("[{}/{}] {}", step, total, message);
        self.pb.set_message(formatted_msg);
    }

    /// Display operation summary
    fn display_summary(&self, years: &[u32], repository: &str, username: &str) {
        println!("\n{}", "📋 Operation Summary".bright_blue().bold());
        println!("  {} {}", "Repository:".cyan(), repository.bright_green());
        println!("  {} {}", "GitHub User:".cyan(), username.bright_green());
        
        let years_display = if years.len() == 1 {
            years[0].to_string()
        } else {
            format!("{} years ({}-{})", 
                years.len(),
                years.iter().min().unwrap(),
                years.iter().max().unwrap()
            )
        };
        println!("  {} {}", "Years:".cyan(), years_display.bright_magenta());
        println!();
    }

    /// Display completion summary with results
    fn display_completion(&self, years: &[u32], repository: &str, username: &str) {
        println!("\n{}", "🎉 Time Travel Complete!".bright_green().bold());
        let years_count = years.len();
        let years_text = if years_count == 1 { "1".to_string() } else { years_count.to_string() };
        println!("  {} commits created across {} years", 
            years_count.to_string().bright_yellow(),
            years_text.bright_magenta()
        );
        println!("  {} {}", "Repository:".cyan(), repository.bright_green());
        println!("  {} https://github.com/{}/{}", 
            "View at:".cyan(), 
            username.bright_blue(),
            repository.bright_blue().underline()
        );
        println!("\n{}", "Check your GitHub profile to see the backdated commits!".cyan());
    }
}

impl ProgressCallback for CliProgressBar {
    fn set_message(&self, message: &str) {
        let current = self.current_step.load(Ordering::Relaxed);
        self.set_status(message, current + 1, self.total_steps);
    }

    fn increment(&self) {
        let current = self.current_step.fetch_add(1, Ordering::Relaxed);
        self.pb.inc(1);
        
        // Update the status with current step
        if current < self.total_steps {
            let msg = self.pb.message();
            self.set_status(&msg, current + 1, self.total_steps);
        }
    }

    fn finish(&self, message: &str) {
        self.pb.finish_with_message(format!("✅ {}", message));
    }
}

/// Run the interactive time travel workflow using the new smart defaults system
async fn run_interactive_time_travel() -> Result<()> {
    // Initialize the interactive prompts system
    let mut interactive = InteractivePrompts::new()
        .context("Failed to initialize interactive prompts system")?;

    // Get current working directory for context
    let current_path = std::env::current_dir().ok();
    let current_path_ref = current_path.as_deref();

    // Run the interactive workflow to collect user choices
    let choices = interactive.run_interactive_workflow(current_path_ref)
        .context("Failed to complete interactive workflow")?;

    // Convert UserChoices to the format needed for time travel
    let author_identity = match &choices.author_mode {
        AuthorMode::CurrentUser(identity) => Some(identity.clone()),
        AuthorMode::TimeTraveler => None, // Use default time traveler identity
        AuthorMode::Manual(identity) => Some(identity.clone()),
        AuthorMode::AskEachTime => {
            // For now, default to time traveler - in future could prompt each time
            None
        }
    };

    // Create appropriate progress bar based on number of years
    let progress_bar = if choices.years.len() > 1 {
        CliProgressBar::new_multi_year(choices.years.len())
    } else {
        CliProgressBar::new()
    };

    // Display operation summary
    progress_bar.display_summary(&choices.years, &choices.repository, &choices.github_username);

    // Create configurations for all years
    let mut configs = Vec::new();
    for year in &choices.years {
        let config = TimeTravelConfig::new(
            *year,
            1, // Default month
            1, // Default day  
            choices.hour,
            choices.github_username.clone(),
            choices.github_token.clone(),
            Some(choices.repository.clone()),
            choices.branch.clone(),
            author_identity.clone(),
        ).context("Failed to create time travel configuration")?;
        configs.push(config);
    }

    // Show dry run and get confirmation
    let confirmed = display_and_confirm_dry_run(&configs, true)
        .context("Failed to display dry run information")?;
    
    if !confirmed {
        println!("\n{} {}", "❌".red(), "Operation cancelled by user".red());
        return Ok(());
    }

    // Additional confirmation for potentially destructive operations
    if choices.force_push || choices.years.len() > 5 {
        println!("\n{}", "⚠️  Additional Confirmation Required".yellow().bold());
        
        if choices.force_push {
            println!("  {} Force push will overwrite remote history", "•".red());
        }
        if choices.years.len() > 5 {
            println!("  {} Processing {} years may take significant time", "•".yellow(), choices.years.len());
        }
        
        let final_confirm = dialoguer::Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Are you absolutely sure you want to proceed?")
            .default(false)
            .interact()
            .context("Failed to get final confirmation")?;
            
        if !final_confirm {
            println!("\n{} {}", "❌".red(), "Operation cancelled by user".red());
            return Ok(());
        }
    }

    println!("{}", "🚀 Starting time travel operation...".bright_green().bold());
    
    // Process each year with enhanced progress tracking
    for (index, year) in choices.years.iter().enumerate() {
        let year_progress = if choices.years.len() > 1 {
            progress_bar.create_year_progress(*year)
        } else {
            None
        };

        println!("\n{} {} ({}/{})", 
            "Processing year:".cyan(), 
            year.to_string().bright_yellow(),
            (index + 1).to_string().bright_white(),
            choices.years.len().to_string().bright_white()
        );
        
        let config = &configs[index];

        // Execute the time travel for this year with appropriate progress callback
        let progress_callback: &dyn ProgressCallback = if let Some(ref year_pb) = year_progress {
            &YearProgressWrapper { pb: year_pb.clone() }
        } else {
            &progress_bar
        };

        if let Err(e) = create_time_traveled_repo_with_options(config, Some(progress_callback), choices.force_push, false).await {
            eprintln!("\n{}", format_error_for_user(&e));
            std::process::exit(1);
        }
        
        if let Some(year_pb) = year_progress {
            year_pb.finish_with_message(format!("✅ Year {} complete", year));
        }
        
        println!("✅ {} {} {}", 
            "Commit for year".green(), 
            year.to_string().bright_yellow(), 
            "created successfully!".green()
        );
    }

    // Display completion summary
    progress_bar.display_completion(&choices.years, &choices.repository, &choices.github_username);
    
    Ok(())
}

/// Wrapper for year-specific progress bars
struct YearProgressWrapper {
    pb: ProgressBar,
}

impl ProgressCallback for YearProgressWrapper {
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
        eprintln!("\n❌ An unexpected error occurred: {}\nIf this is a bug, please report it at https://github.com/chama-x/Git-Timetraveler/issues", info);
        std::process::exit(1);
    }));

    let args = Args::parse();
    if args.no_menu {
        return run_non_interactive_mode(args).await;
    }

    // TTY check for interactive menu only
    if !atty::is(Stream::Stdout) || !atty::is(Stream::Stdin) {
        eprintln!("\n❌ This CLI requires an interactive terminal (TTY) for interactive mode.\nIf you are using npx or a non-interactive shell, try running with the --no-menu flag and provide all required arguments.\nExample: npx git-timetraveler --no-menu --username <user> --token <token> --repo <repo> --year <year> ...");
        std::process::exit(1);
    }

    // Handle Ctrl+C gracefully
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        println!("\nExiting. Goodbye!");
        r.store(false, Ordering::SeqCst);
        process::exit(0);
    }).expect("Error setting Ctrl-C handler");

    // Enhanced welcome screen with better information
    println!("{}", r#"
   ____ _ _   _                 _                     _           _           
  / ___(_) |_| |_ ___ _ __ __ _| |_ ___  ___ ___   __| | ___  ___| |_ ___ _ __ 
 | |  _| | __| __/ _ \ '__/ _` | __/ _ \/ __/ __| / _` |/ _ \/ __| __/ _ \ '__|
 | |_| | | |_| ||  __/ | | (_| | ||  __/\__ \__ \| (_| |  __/ (__| ||  __/ |   
  \____|_|\__|\__\___|_|  \__,_|\__\___||___/___(_)__,_|\___|\___|\__\___|_|   
    "#.bright_blue().bold());
    
    println!("{}", "🕰️  Git Time Traveler".bright_blue().bold());
    println!("{}", "Create backdated commits to enhance your GitHub contribution graph\n".cyan());
    
    // Display helpful context information
    println!("{}", "What this tool does:".bright_yellow());
    println!("  • {} Create commits with custom timestamps", "✓".green());
    println!("  • {} Push to GitHub repositories safely", "✓".green());
    println!("  • {} Support single years or year ranges", "✓".green());
    println!("  • {} Remember your preferences for future use", "✓".green());
    println!();

    let menu_items = vec![
        "🚀 Create backdated commit(s)",
        "📖 View usage examples", 
        "🔑 Learn about GitHub tokens",
        "⚙️  Configuration options",
        "❌ Exit",
    ];
    
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("What would you like to do?")
        .items(&menu_items)
        .default(0)
        .interact()
        .context("Failed to get menu selection")?;

    match selection {
        0 => {
            // Use the new interactive prompts system with smart defaults
            if let Err(e) = run_interactive_time_travel().await {
                eprintln!("\n{}", format_error_for_user(&e));
                std::process::exit(1);
            }
        }
        1 => {
            display_usage_examples();
        }
        2 => {
            display_github_token_help();
        }
        3 => {
            display_configuration_options();
        }
        4 => {
            println!("\n{} {}", "👋".bright_blue(), "Thanks for using Git Time Traveler!".cyan());
            return Ok(());
        }
        _ => unreachable!(),
    }

    Ok(())
}

/// Display usage examples and common patterns
fn display_usage_examples() {
    println!("\n{}", "📖 Usage Examples".bright_blue().bold());
    println!("{}", "Here are common ways to use Git Time Traveler:\n".cyan());
    
    println!("{}", "Interactive Mode (Recommended):".bright_yellow());
    println!("  {}", "git-timetraveler".bright_green());
    println!("  {}", "npx git-timetraveler".bright_green());
    println!("  → Guided prompts with smart defaults\n");
    
    println!("{}", "Non-Interactive Mode:".bright_yellow());
    println!("  {}", "git-timetraveler --no-menu --username myuser --token ghp_xxx --repo myrepo --year 1990".bright_green());
    println!("  {}", "git-timetraveler --no-menu --username myuser --token ghp_xxx --repo myrepo --years 1990-1995".bright_green());
    println!("  → Perfect for scripts and automation\n");
    
    println!("{}", "Common Options:".bright_yellow());
    println!("  {} Create commits for a single year", "--year 1990".bright_cyan());
    println!("  {} Create commits across a range", "--years 1990-1995".bright_cyan());
    println!("  {} Specify custom repository name", "--repo my-project".bright_cyan());
    println!("  {} Set commit time (0-23)", "--hour 14".bright_cyan());
    println!("  {} Force push (use with caution)", "--force".bright_cyan());
    
    println!("\n{}", "💡 Pro Tips:".bright_yellow());
    println!("  • The tool remembers your preferences between sessions");
    println!("  • Use ranges like 1990-1995 for multiple years at once");
    println!("  • Commits are created at 6 PM by default for realistic timing");
    println!("  • Your GitHub token needs 'repo' permissions");
    
    println!("\n{}", "Press Enter to continue...".dimmed());
    let _ = std::io::stdin().read_line(&mut String::new());
}

/// Display GitHub token creation help
fn display_github_token_help() {
    println!("\n{}", "🔑 GitHub Personal Access Token Setup".bright_blue().bold());
    println!("{}", "You need a GitHub token to create and push to repositories.\n".cyan());
    
    println!("{}", "Step-by-step instructions:".bright_yellow());
    println!("  {} Go to: {}", "1.".bright_white(), "https://github.com/settings/tokens".bright_blue().underline());
    println!("  {} Click: {}", "2.".bright_white(), "Generate new token (classic)".bright_green());
    println!("  {} Set expiration: {}", "3.".bright_white(), "Choose your preferred duration".cyan());
    println!("  {} Select scopes: {}", "4.".bright_white(), "Check 'repo' (Full control of private repositories)".bright_green());
    println!("  {} Click: {}", "5.".bright_white(), "Generate token".bright_green());
    println!("  {} Copy the token: {}", "6.".bright_white(), "Save it securely - you won't see it again!".yellow());
    
    println!("\n{}", "Required Permissions:".bright_yellow());
    println!("  {} {} - Create, read, and write repositories", "✓".green(), "repo".bright_cyan());
    println!("  {} {} - Access public repositories (included in repo)", "✓".green(), "public_repo".bright_cyan());
    
    println!("\n{}", "Security Notes:".bright_yellow());
    println!("  • {} Never share your token with others", "⚠️".yellow());
    println!("  • {} Store it securely (password manager recommended)", "🔒".blue());
    println!("  • {} You can revoke it anytime from GitHub settings", "🔄".cyan());
    println!("  • {} This tool stores it locally and never transmits it elsewhere", "🛡️".green());
    
    println!("\n{}", "Press Enter to continue...".dimmed());
    let _ = std::io::stdin().read_line(&mut String::new());
}

/// Run the non-interactive mode with comprehensive argument validation and processing
async fn run_non_interactive_mode(args: Args) -> Result<()> {
    // Validate required arguments
    let validation_errors = validate_non_interactive_args(&args);
    if !validation_errors.is_empty() {
        eprintln!("{} {}", "❌ Validation errors:".red().bold(), "The following issues were found:");
        for error in &validation_errors {
            eprintln!("  • {}", error.red());
        }
        eprintln!("\n{}", "Example usage:".bright_yellow());
        eprintln!("  {}", "git-timetraveler --no-menu --username myuser --token ghp_xxx --repo myrepo --year 1990".bright_green());
        eprintln!("  {}", "git-timetraveler --no-menu --username myuser --token ghp_xxx --repo myrepo --years 1990-1995".bright_green());
        eprintln!("\n{}", "For more help, run: git-timetraveler --help".cyan());
        std::process::exit(1);
    }

    // Parse years from arguments
    let years = parse_years_from_args(&args)?;
    
    // Validate year range
    if years.is_empty() {
        eprintln!("❌ No valid years specified");
        std::process::exit(1);
    }
    
    if years.len() > 50 {
        eprintln!("❌ Too many years specified (maximum 50)");
        std::process::exit(1);
    }

    // Extract required values (already validated)
    let username = args.username.clone().unwrap();
    let token = args.token.clone().unwrap();
    let repo_name = args.repo.clone().unwrap_or_else(|| {
        if years.len() == 1 {
            years[0].to_string()
        } else {
            format!("timetravel-{}-{}", years.iter().min().unwrap(), years.iter().max().unwrap())
        }
    });

    // Create author identity if specified
    let author_identity = if args.author_name.is_some() || args.author_email.is_some() {
        Some(git_timetraveler::GitIdentity {
            name: args.author_name.clone().unwrap_or_else(|| "Git Time Traveler".to_string()),
            email: args.author_email.clone().unwrap_or_else(|| "timetraveler@example.com".to_string()),
        })
    } else {
        None // Use default time traveler identity
    };

    // Set up output verbosity
    let verbose = args.verbose;
    let quiet = args.quiet;
    
    if verbose && quiet {
        eprintln!("❌ Cannot use both --verbose and --quiet flags");
        std::process::exit(1);
    }

    // Create configurations for all years
    let mut configs = Vec::new();
    for year in &years {
        let config = TimeTravelConfig::new(
            *year,
            args.month,
            args.day,
            args.hour,
            username.clone(),
            token.clone(),
            Some(repo_name.clone()),
            args.branch.clone(),
            author_identity.clone(),
        ).context("Failed to create time travel configuration")?;
        configs.push(config);
    }

    // Handle dry run mode
    if args.dry_run {
        let confirmed = display_and_confirm_dry_run(&configs, false)
            .context("Failed to display dry run information")?;
        
        if !confirmed && !args.yes {
            println!("\n{} {}", "❌".red(), "Dry run completed - use --yes to proceed without confirmation".yellow());
        }
        return Ok(());
    }

    // Create appropriate progress bar (unless quiet mode)
    let progress_bar = if !quiet {
        Some(if years.len() > 1 {
            CliProgressBar::new_multi_year(years.len())
        } else {
            CliProgressBar::new()
        })
    } else {
        None
    };

    // Display operation summary (unless quiet)
    if let Some(ref pb) = progress_bar {
        pb.display_summary(&years, &repo_name, &username);
    }

    // Show confirmation for potentially destructive operations (unless --yes is used)
    if !args.yes && (args.force || years.len() > 5) {
        println!("\n{}", "⚠️  Confirmation Required".yellow().bold());
        
        if args.force {
            println!("  {} Force push will overwrite remote history", "•".red());
        }
        if years.len() > 5 {
            println!("  {} Processing {} years may take significant time", "•".yellow(), years.len());
        }
        
        println!("\n{}", "Use --yes flag to skip this confirmation or run interactively for more control".cyan());
        println!("{} {}", "❌".red(), "Operation cancelled - confirmation required".red());
        std::process::exit(1);
    }

    if !quiet {
        println!("{}", "🚀 Starting non-interactive time travel operation...".bright_green().bold());
    }
    
    // Process each year
    for (index, year) in years.iter().enumerate() {
        let year_progress = if let Some(ref pb) = progress_bar {
            if years.len() > 1 {
                pb.create_year_progress(*year)
            } else {
                None
            }
        } else {
            None
        };

        if !quiet {
            println!("\n{} {} ({}/{})", 
                "Processing year:".cyan(), 
                year.to_string().bright_yellow(),
                (index + 1).to_string().bright_white(),
                years.len().to_string().bright_white()
            );
        }

        let config = &configs[index];

        // Execute with appropriate progress callback
        let year_wrapper;
        let progress_callback: Option<&dyn ProgressCallback> = if let Some(ref year_pb) = year_progress {
            year_wrapper = YearProgressWrapper { pb: year_pb.clone() };
            Some(&year_wrapper)
        } else if let Some(ref pb) = progress_bar {
            Some(pb)
        } else {
            None
        };

        if let Err(e) = create_time_traveled_repo_with_options(config, progress_callback, args.force, false).await {
            if !quiet {
                eprintln!("\n{}", format_error_for_user(&e));
            }
            if verbose {
                eprintln!("\n{} {:?}", "Debug info:".dimmed(), e);
            }
            std::process::exit(1);
        }
        
        if let Some(year_pb) = year_progress {
            year_pb.finish_with_message(format!("✅ Year {} complete", year));
        }
        
        if !quiet {
            println!("✅ {} {} {}", 
                "Commit for year".green(), 
                year.to_string().bright_yellow(), 
                "created successfully!".green()
            );
        } else if verbose {
            println!("Year {} processed successfully", year);
        }
    }

    // Display completion summary (unless quiet)
    if let Some(ref pb) = progress_bar {
        pb.display_completion(&years, &repo_name, &username);
    } else if !quiet {
        println!("✅ All {} years processed successfully!", years.len());
    }
    
    Ok(())
}

/// Validate arguments for non-interactive mode
fn validate_non_interactive_args(args: &Args) -> Vec<String> {
    let mut errors = Vec::new();

    // Required arguments
    if args.username.is_none() {
        errors.push("Missing required argument: --username".to_string());
    }
    
    if args.token.is_none() {
        errors.push("Missing required argument: --token".to_string());
    }

    // Year validation
    let year_flag_present = std::env::args().any(|arg| arg == "--year" || arg == "-y");
    let years_flag_present = std::env::args().any(|arg| arg == "--years");
    
    if !year_flag_present && !years_flag_present {
        errors.push("Must specify either --year or --years".to_string());
    }

    // Validate year range
    if args.year < 1970 || args.year > 2030 {
        errors.push(format!("Year {} is out of valid range (1970-2030)", args.year));
    }

    // Validate month and day
    if args.month < 1 || args.month > 12 {
        errors.push(format!("Month {} is invalid (must be 1-12)", args.month));
    }
    
    if args.day < 1 || args.day > 31 {
        errors.push(format!("Day {} is invalid (must be 1-31)", args.day));
    }

    // Validate hour
    if args.hour > 23 {
        errors.push(format!("Hour {} is invalid (must be 0-23)", args.hour));
    }

    // Validate author email format if provided
    if let Some(ref email) = args.author_email {
        if !email.contains('@') || !email.contains('.') {
            errors.push("Author email format is invalid".to_string());
        }
    }

    // Validate conflicting flags
    if args.verbose && args.quiet {
        errors.push("Cannot use both --verbose and --quiet flags".to_string());
    }

    // Validate years format if provided
    if let Some(ref years_str) = args.years {
        if let Err(e) = parse_years_string(years_str) {
            errors.push(format!("Invalid years format: {}", e));
        }
    }

    errors
}

/// Parse years from command line arguments
fn parse_years_from_args(args: &Args) -> Result<Vec<u32>> {
    if let Some(ref years_str) = args.years {
        parse_years_string(years_str)
    } else {
        Ok(vec![args.year])
    }
}

/// Parse years string (handles ranges and lists)
fn parse_years_string(years_str: &str) -> Result<Vec<u32>> {
    let trimmed = years_str.trim();
    
    // Handle range (e.g., "1990-1995")
    if trimmed.contains('-') && !trimmed.contains(',') {
        let parts: Vec<&str> = trimmed.split('-').collect();
        if parts.len() == 2 {
            let start = parts[0].trim().parse::<u32>()
                .context("Invalid start year in range")?;
            let end = parts[1].trim().parse::<u32>()
                .context("Invalid end year in range")?;
            
            if start > end {
                anyhow::bail!("Start year must be less than or equal to end year");
            }
            if end - start > 50 {
                anyhow::bail!("Year range too large (max 50 years)");
            }
            if start < 1970 || end > 2030 {
                anyhow::bail!("Years must be between 1970 and 2030");
            }
            
            return Ok((start..=end).collect());
        }
    }
    
    // Handle comma-separated list (e.g., "1990,1992,1994")
    if trimmed.contains(',') {
        let years: Result<Vec<u32>> = trimmed
            .split(',')
            .map(|s| {
                let year = s.trim().parse::<u32>()
                    .context("Invalid year in list")?;
                if year < 1970 || year > 2030 {
                    anyhow::bail!("Year {} must be between 1970 and 2030", year);
                }
                Ok(year)
            })
            .collect();
        return years;
    }
    
    // Handle single year
    let year = trimmed.parse::<u32>()
        .context("Invalid year format")?;
    if year < 1970 || year > 2030 {
        anyhow::bail!("Year must be between 1970 and 2030");
    }
    
    Ok(vec![year])
}



/// Display configuration options and help
fn display_configuration_options() {
    println!("\n{}", "⚙️  Configuration Options".bright_blue().bold());
    println!("{}", "Git Time Traveler learns from your usage patterns.\n".cyan());
    
    println!("{}", "Stored Preferences:".bright_yellow());
    println!("  • {} Recent repository names", "📁".blue());
    println!("  • {} Preferred author modes", "👤".blue());
    println!("  • {} Common year patterns", "📅".blue());
    println!("  • {} GitHub username", "🐙".blue());
    println!("  • {} Default commit timing", "⏰".blue());
    
    println!("\n{}", "Configuration Location:".bright_yellow());
    if let Some(config_dir) = dirs::config_dir() {
        let config_path = config_dir.join("git-timetraveler");
        println!("  {}", config_path.display().to_string().bright_cyan());
    } else {
        println!("  {}", "~/.config/git-timetraveler/".bright_cyan());
    }
    
    println!("\n{}", "Reset Options:".bright_yellow());
    println!("  • Delete the configuration directory to start fresh");
    println!("  • Individual preferences reset automatically after 30 days of inactivity");
    println!("  • GitHub tokens are stored securely using your system keychain");
    
    println!("\n{}", "Command Line Override:".bright_yellow());
    println!("  Use {} to bypass all interactive prompts", "--no-menu".bright_cyan());
    println!("  All preferences can be overridden with command line arguments");
    
    println!("\n{}", "Press Enter to continue...".dimmed());
    let _ = std::io::stdin().read_line(&mut String::new());
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
