use anyhow::{Context, Result};
use colored::*;
use dialoguer::{theme::ColorfulTheme, Select, Input, Password, Confirm};
use std::path::Path;

use crate::defaults::{DefaultsEngine, IntelligentDefaults, AuthorMode};

use crate::session::SessionManager;

/// Interactive prompts with smart defaults integration
pub struct InteractivePrompts {
    defaults_engine: DefaultsEngine,
    session_manager: SessionManager,
    theme: ColorfulTheme,
}

/// User choices collected through interactive prompts
#[derive(Debug, Clone)]
pub struct UserChoices {
    pub repository: String,
    pub branch: String,
    pub author_mode: AuthorMode,
    pub years: Vec<u32>,
    pub hour: u32,
    pub github_username: String,
    pub github_token: String,
    pub force_push: bool,
}

/// Validation result for user input
#[derive(Debug)]
pub enum ValidationResult {
    Valid,
    Invalid(String),
}

impl InteractivePrompts {
    /// Create a new interactive prompts instance
    pub fn new() -> Result<Self> {
        let defaults_engine = DefaultsEngine::new()
            .context("Failed to initialize defaults engine")?;
        let session_manager = SessionManager::new()
            .context("Failed to initialize session manager")?;
        
        Ok(Self {
            defaults_engine,
            session_manager,
            theme: ColorfulTheme::default(),
        })
    }

    /// Run the complete interactive workflow with smart defaults
    pub fn run_interactive_workflow(&mut self, current_path: Option<&Path>) -> Result<UserChoices> {
        // Generate intelligent defaults based on current context
        let defaults = self.defaults_engine.generate_defaults(current_path)
            .context("Failed to generate intelligent defaults")?;

        // Display welcome and context information
        self.display_welcome(&defaults)?;

        // Collect user choices with smart defaults
        let repository = self.prompt_repository(&defaults)?;
        let branch = self.prompt_branch(&defaults)?;
        let author_mode = self.prompt_author_mode(&defaults)?;
        let years = self.prompt_years(&defaults)?;
        let hour = self.prompt_hour(&defaults)?;
        let github_username = self.prompt_github_username(&defaults)?;
        let github_token = self.prompt_github_token()?;
        let force_push = self.prompt_force_push()?;

        let choices = UserChoices {
            repository,
            branch,
            author_mode,
            years,
            hour,
            github_username,
            github_token,
            force_push,
        };

        // Learn from user choices for future sessions
        self.learn_from_choices(&choices)?;

        // Display summary and confirm
        self.display_summary_and_confirm(&choices)?;

        Ok(choices)
    }

    /// Display welcome message with context information
    fn display_welcome(&self, defaults: &IntelligentDefaults) -> Result<()> {
        println!("{}", "🕰️  Git Time Traveler".bright_blue().bold());
        println!("{}", "Travel back in time on your GitHub profile with ease!\n".cyan());

        // Show intelligent context if available
        if defaults.confidence > 0.6 {
            println!("{}", "Smart suggestions based on your context:".green());
            for reason in &defaults.reasoning {
                println!("  • {}", reason.dimmed());
            }
            println!();
        }

        Ok(())
    }

    /// Prompt for repository name with smart defaults
    fn prompt_repository(&self, defaults: &IntelligentDefaults) -> Result<String> {
        let prompt_text = "Repository name";
        
        match &defaults.repository {
            Some(suggested_repo) => {
                let input: String = Input::with_theme(&self.theme)
                    .with_prompt(format!("{} (suggested: {})", prompt_text, suggested_repo.bright_green()))
                    .default(suggested_repo.clone())
                    .validate_with(|input: &String| -> Result<(), String> {
                        match self.validate_repository_name(input) {
                            ValidationResult::Valid => Ok(()),
                            ValidationResult::Invalid(msg) => Err(msg),
                        }
                    })
                    .interact_text()
                    .context("Failed to get repository input")?;
                Ok(input)
            }
            None => {
                let input: String = Input::with_theme(&self.theme)
                    .with_prompt(prompt_text)
                    .validate_with(|input: &String| -> Result<(), String> {
                        match self.validate_repository_name(input) {
                            ValidationResult::Valid => Ok(()),
                            ValidationResult::Invalid(msg) => Err(msg),
                        }
                    })
                    .interact_text()
                    .context("Failed to get repository input")?;
                Ok(input)
            }
        }
    }

    /// Prompt for branch name with smart defaults
    fn prompt_branch(&self, defaults: &IntelligentDefaults) -> Result<String> {
        let input: String = Input::with_theme(&self.theme)
            .with_prompt(format!("Branch name (suggested: {})", defaults.branch.bright_green()))
            .default(defaults.branch.clone())
            .validate_with(|input: &String| -> Result<(), String> {
                match self.validate_branch_name(input) {
                    ValidationResult::Valid => Ok(()),
                    ValidationResult::Invalid(msg) => Err(msg),
                }
            })
            .interact_text()
            .context("Failed to get branch input")?;
        Ok(input)
    }

    /// Prompt for author mode with smart defaults
    fn prompt_author_mode(&self, defaults: &IntelligentDefaults) -> Result<AuthorMode> {
        let options = match &defaults.author_mode {
            AuthorMode::CurrentUser(identity) => {
                vec![
                    format!("Use your Git identity: {} <{}>", identity.name.bright_cyan(), identity.email.bright_cyan()),
                    "Use generic time traveler identity".to_string(),
                    "Specify custom author details".to_string(),
                    "Ask me each time".to_string(),
                ]
            }
            AuthorMode::TimeTraveler => {
                vec![
                    "Use generic time traveler identity".to_string(),
                    "Use your Git identity (if available)".to_string(),
                    "Specify custom author details".to_string(),
                    "Ask me each time".to_string(),
                ]
            }
            AuthorMode::Manual(identity) => {
                vec![
                    format!("Use custom identity: {} <{}>", identity.name.bright_cyan(), identity.email.bright_cyan()),
                    "Use generic time traveler identity".to_string(),
                    "Use your Git identity (if available)".to_string(),
                    "Ask me each time".to_string(),
                ]
            }
            AuthorMode::AskEachTime => {
                vec![
                    "Ask me each time".to_string(),
                    "Use generic time traveler identity".to_string(),
                    "Use your Git identity (if available)".to_string(),
                    "Specify custom author details".to_string(),
                ]
            }
        };

        let default_index = match &defaults.author_mode {
            AuthorMode::CurrentUser(_) => 0,
            AuthorMode::TimeTraveler => 0,
            AuthorMode::Manual(_) => 0,
            AuthorMode::AskEachTime => 0,
        };

        let selection = Select::with_theme(&self.theme)
            .with_prompt("How should commits be authored?")
            .items(&options)
            .default(default_index)
            .interact()
            .context("Failed to get author mode selection")?;

        let result = match (&defaults.author_mode, selection) {
            (AuthorMode::CurrentUser(identity), 0) => AuthorMode::CurrentUser(identity.clone()),
            (AuthorMode::Manual(identity), 0) => AuthorMode::Manual(identity.clone()),
            (_, sel) if options[sel].contains("time traveler") => AuthorMode::TimeTraveler,
            (_, sel) if options[sel].contains("custom author") || options[sel].contains("Specify custom") => {
                // Prompt for custom author details
                self.prompt_manual_author_details()?
            }
            (_, sel) if options[sel].contains("Ask me") => AuthorMode::AskEachTime,
            (AuthorMode::CurrentUser(identity), _) => AuthorMode::CurrentUser(identity.clone()),
            (AuthorMode::Manual(identity), _) => AuthorMode::Manual(identity.clone()),
            _ => AuthorMode::TimeTraveler,
        };

        Ok(result)
    }

    /// Prompt for manual author details
    fn prompt_manual_author_details(&self) -> Result<AuthorMode> {
        println!("\n{}", "📝 Custom Author Details".bright_blue().bold());
        println!("Specify the name and email for commit authorship:");

        let name: String = Input::with_theme(&self.theme)
            .with_prompt("Author name")
            .validate_with(|input: &String| -> Result<(), String> {
                let trimmed = input.trim();
                if trimmed.is_empty() {
                    Err("Author name cannot be empty".to_string())
                } else if trimmed.len() > 100 {
                    Err("Author name too long (max 100 characters)".to_string())
                } else {
                    Ok(())
                }
            })
            .interact_text()
            .context("Failed to get author name")?;

        let email: String = Input::with_theme(&self.theme)
            .with_prompt("Author email")
            .validate_with(|input: &String| -> Result<(), String> {
                match self.validate_email(input) {
                    ValidationResult::Valid => Ok(()),
                    ValidationResult::Invalid(msg) => Err(msg),
                }
            })
            .interact_text()
            .context("Failed to get author email")?;

        let identity = crate::git_context::GitIdentity {
            name: name.trim().to_string(),
            email: email.trim().to_string(),
        };

        Ok(AuthorMode::Manual(identity))
    }

    /// Prompt for years with smart defaults and validation
    fn prompt_years(&self, defaults: &IntelligentDefaults) -> Result<Vec<u32>> {
        let suggested_years = if defaults.suggested_years.len() == 1 {
            defaults.suggested_years[0].to_string()
        } else {
            format!("{}-{}", 
                defaults.suggested_years.iter().min().unwrap_or(&1990),
                defaults.suggested_years.iter().max().unwrap_or(&1990)
            )
        };

        println!("\n{}", "Year selection options:".bright_yellow());
        println!("  • Single year: {}", "1990".bright_green());
        println!("  • Year range: {}", "1990-1995".bright_green());
        println!("  • Multiple years: {}", "1990,1992,1994".bright_green());

        let input: String = Input::with_theme(&self.theme)
            .with_prompt(format!("Years for time travel (suggested: {})", suggested_years.bright_green()))
            .default(suggested_years)
            .validate_with(|input: &String| -> Result<(), String> {
                match self.validate_years_input(input) {
                    ValidationResult::Valid => Ok(()),
                    ValidationResult::Invalid(msg) => Err(msg),
                }
            })
            .interact_text()
            .context("Failed to get years input")?;

        self.parse_years_input(&input)
    }

    /// Prompt for hour with smart defaults
    fn prompt_hour(&self, defaults: &IntelligentDefaults) -> Result<u32> {
        let input: String = Input::with_theme(&self.theme)
            .with_prompt(format!("Hour for commits (0-23, suggested: {})", 
                format!("{}:00", defaults.suggested_hour).bright_green()))
            .default(defaults.suggested_hour.to_string())
            .validate_with(|input: &String| -> Result<(), String> {
                match self.validate_hour(input) {
                    ValidationResult::Valid => Ok(()),
                    ValidationResult::Invalid(msg) => Err(msg),
                }
            })
            .interact_text()
            .context("Failed to get hour input")?;

        input.parse::<u32>().context("Failed to parse hour")
    }

    /// Prompt for GitHub username with smart defaults
    fn prompt_github_username(&self, defaults: &IntelligentDefaults) -> Result<String> {
        let prompt_text = "GitHub username";
        
        match &defaults.github_username {
            Some(suggested_username) => {
                let input: String = Input::with_theme(&self.theme)
                    .with_prompt(format!("{} (suggested: {})", prompt_text, suggested_username.bright_green()))
                    .default(suggested_username.clone())
                    .validate_with(|input: &String| -> Result<(), String> {
                        match self.validate_github_username(input) {
                            ValidationResult::Valid => Ok(()),
                            ValidationResult::Invalid(msg) => Err(msg),
                        }
                    })
                    .interact_text()
                    .context("Failed to get GitHub username input")?;
                Ok(input)
            }
            None => {
                let input: String = Input::with_theme(&self.theme)
                    .with_prompt(prompt_text)
                    .validate_with(|input: &String| -> Result<(), String> {
                        match self.validate_github_username(input) {
                            ValidationResult::Valid => Ok(()),
                            ValidationResult::Invalid(msg) => Err(msg),
                        }
                    })
                    .interact_text()
                    .context("Failed to get GitHub username input")?;
                Ok(input)
            }
        }
    }

    /// Prompt for GitHub token (always secure input)
    fn prompt_github_token(&self) -> Result<String> {
        println!("\n{}", "GitHub Personal Access Token:".bright_yellow());
        println!("  • Create at: {}", "https://github.com/settings/tokens".bright_blue().underline());
        println!("  • Required permissions: {}", "repo (full control)".bright_green());

        let token: String = Password::with_theme(&self.theme)
            .with_prompt("GitHub token (input hidden)")
            .validate_with(|input: &String| -> Result<(), String> {
                match self.validate_github_token(input) {
                    ValidationResult::Valid => Ok(()),
                    ValidationResult::Invalid(msg) => Err(msg),
                }
            })
            .interact()
            .context("Failed to get GitHub token input")?;

        Ok(token)
    }

    /// Prompt for force push option
    fn prompt_force_push(&self) -> Result<bool> {
        println!("\n{}", "⚠️  Force Push Warning:".bright_red().bold());
        println!("Force push will overwrite remote history. Use with caution!");

        let force = Confirm::with_theme(&self.theme)
            .with_prompt("Enable force push?")
            .default(false)
            .interact()
            .context("Failed to get force push confirmation")?;

        Ok(force)
    }

    /// Display summary and get final confirmation
    fn display_summary_and_confirm(&self, choices: &UserChoices) -> Result<()> {
        println!("\n{}", "📋 Summary:".bright_blue().bold());
        println!("  Repository: {}", choices.repository.bright_green());
        println!("  Branch: {}", choices.branch.bright_cyan());
        
        let author_display = match &choices.author_mode {
            AuthorMode::CurrentUser(identity) => format!("Your identity: {} <{}>", identity.name, identity.email),
            AuthorMode::TimeTraveler => "Time Traveler".to_string(),
            AuthorMode::Manual(identity) => format!("Custom identity: {} <{}>", identity.name, identity.email),
            AuthorMode::AskEachTime => "Ask each time".to_string(),
        };
        println!("  Author: {}", author_display.bright_yellow());
        
        let years_display = if choices.years.len() == 1 {
            choices.years[0].to_string()
        } else {
            format!("{} years ({}-{})", 
                choices.years.len(),
                choices.years.iter().min().unwrap(),
                choices.years.iter().max().unwrap()
            )
        };
        println!("  Years: {}", years_display.bright_magenta());
        println!("  Time: {}:00", format!("{:02}", choices.hour).bright_white());
        println!("  Username: {}", choices.github_username.bright_green());
        println!("  Force push: {}", if choices.force_push { "Yes".red() } else { "No".green() });

        let proceed = Confirm::with_theme(&self.theme)
            .with_prompt("\nProceed with time travel?")
            .default(true)
            .interact()
            .context("Failed to get final confirmation")?;

        if !proceed {
            anyhow::bail!("Operation cancelled by user");
        }

        Ok(())
    }

    /// Learn from user choices for future sessions
    fn learn_from_choices(&mut self, choices: &UserChoices) -> Result<()> {
        // Learn repository preference
        self.session_manager.learn_from_choice("repository", &choices.repository);

        // Learn author mode preference
        let author_mode_str = match &choices.author_mode {
            AuthorMode::CurrentUser(_) => "current_user",
            AuthorMode::TimeTraveler => "time_traveler",
            AuthorMode::Manual(_) => "manual",
            AuthorMode::AskEachTime => "ask_each_time",
        };
        self.session_manager.learn_from_choice("author_mode", author_mode_str);

        // Learn year preferences
        for year in &choices.years {
            self.session_manager.learn_from_choice("year", &year.to_string());
        }

        // Learn hour preference
        self.session_manager.learn_from_choice("hour", &choices.hour.to_string());

        // Learn GitHub username
        self.session_manager.learn_from_choice("github_username", &choices.github_username);

        // Save session data
        self.session_manager.save()
            .context("Failed to save session data")?;

        Ok(())
    }

    // Validation methods

    fn validate_repository_name(&self, input: &str) -> ValidationResult {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return ValidationResult::Invalid("Repository name cannot be empty".to_string());
        }
        if trimmed.len() > 100 {
            return ValidationResult::Invalid("Repository name too long (max 100 characters)".to_string());
        }
        if !trimmed.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.') {
            return ValidationResult::Invalid("Repository name can only contain letters, numbers, hyphens, underscores, and dots".to_string());
        }
        ValidationResult::Valid
    }

    fn validate_branch_name(&self, input: &str) -> ValidationResult {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return ValidationResult::Invalid("Branch name cannot be empty".to_string());
        }
        if trimmed.starts_with('-') || trimmed.ends_with('-') {
            return ValidationResult::Invalid("Branch name cannot start or end with a hyphen".to_string());
        }
        if trimmed.contains("..") || trimmed.contains(' ') {
            return ValidationResult::Invalid("Branch name cannot contain spaces or consecutive dots".to_string());
        }
        ValidationResult::Valid
    }

    fn validate_years_input(&self, input: &str) -> ValidationResult {
        match self.parse_years_input(input) {
            Ok(years) => {
                if years.is_empty() {
                    ValidationResult::Invalid("At least one year must be specified".to_string())
                } else if years.len() > 50 {
                    ValidationResult::Invalid("Too many years specified (max 50)".to_string())
                } else {
                    ValidationResult::Valid
                }
            }
            Err(e) => ValidationResult::Invalid(e.to_string()),
        }
    }

    fn validate_hour(&self, input: &str) -> ValidationResult {
        match input.trim().parse::<u32>() {
            Ok(hour) if hour <= 23 => ValidationResult::Valid,
            Ok(_) => ValidationResult::Invalid("Hour must be between 0 and 23".to_string()),
            Err(_) => ValidationResult::Invalid("Please enter a valid number".to_string()),
        }
    }

    fn validate_github_username(&self, input: &str) -> ValidationResult {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return ValidationResult::Invalid("GitHub username cannot be empty".to_string());
        }
        if trimmed.len() > 39 {
            return ValidationResult::Invalid("GitHub username too long (max 39 characters)".to_string());
        }
        if !trimmed.chars().all(|c| c.is_alphanumeric() || c == '-') {
            return ValidationResult::Invalid("GitHub username can only contain letters, numbers, and hyphens".to_string());
        }
        if trimmed.starts_with('-') || trimmed.ends_with('-') {
            return ValidationResult::Invalid("GitHub username cannot start or end with a hyphen".to_string());
        }
        ValidationResult::Valid
    }

    fn validate_github_token(&self, input: &str) -> ValidationResult {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return ValidationResult::Invalid("GitHub token cannot be empty".to_string());
        }
        if trimmed.len() < 20 {
            return ValidationResult::Invalid("GitHub token seems too short".to_string());
        }
        ValidationResult::Valid
    }

    fn validate_email(&self, input: &str) -> ValidationResult {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return ValidationResult::Invalid("Email cannot be empty".to_string());
        }
        
        // Basic email validation - contains @ and has parts before and after
        if !trimmed.contains('@') {
            return ValidationResult::Invalid("Email must contain @ symbol".to_string());
        }
        
        let parts: Vec<&str> = trimmed.split('@').collect();
        if parts.len() != 2 {
            return ValidationResult::Invalid("Email must have exactly one @ symbol".to_string());
        }
        
        if parts[0].is_empty() || parts[1].is_empty() {
            return ValidationResult::Invalid("Email must have content before and after @".to_string());
        }
        
        if !parts[1].contains('.') {
            return ValidationResult::Invalid("Email domain must contain a dot".to_string());
        }
        
        if trimmed.len() > 254 {
            return ValidationResult::Invalid("Email too long (max 254 characters)".to_string());
        }
        
        ValidationResult::Valid
    }

    /// Parse years input (single year, range, or comma-separated list)
    fn parse_years_input(&self, input: &str) -> Result<Vec<u32>> {
        let trimmed = input.trim();
        
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
}



#[cfg(test)]
mod tests {
    use super::*;


    fn create_test_prompts() -> Result<InteractivePrompts> {
        InteractivePrompts::new()
    }

    #[test]
    fn test_validate_repository_name() -> Result<()> {
        let prompts = create_test_prompts()?;
        
        // Valid names
        assert!(matches!(prompts.validate_repository_name("my-repo"), ValidationResult::Valid));
        assert!(matches!(prompts.validate_repository_name("repo_name"), ValidationResult::Valid));
        assert!(matches!(prompts.validate_repository_name("repo.name"), ValidationResult::Valid));
        assert!(matches!(prompts.validate_repository_name("repo123"), ValidationResult::Valid));
        
        // Invalid names
        assert!(matches!(prompts.validate_repository_name(""), ValidationResult::Invalid(_)));
        assert!(matches!(prompts.validate_repository_name("   "), ValidationResult::Invalid(_)));
        assert!(matches!(prompts.validate_repository_name("repo name"), ValidationResult::Invalid(_)));
        assert!(matches!(prompts.validate_repository_name("repo@name"), ValidationResult::Invalid(_)));
        
        Ok(())
    }

    #[test]
    fn test_validate_branch_name() -> Result<()> {
        let prompts = create_test_prompts()?;
        
        // Valid names
        assert!(matches!(prompts.validate_branch_name("main"), ValidationResult::Valid));
        assert!(matches!(prompts.validate_branch_name("feature-branch"), ValidationResult::Valid));
        assert!(matches!(prompts.validate_branch_name("develop"), ValidationResult::Valid));
        
        // Invalid names
        assert!(matches!(prompts.validate_branch_name(""), ValidationResult::Invalid(_)));
        assert!(matches!(prompts.validate_branch_name("-main"), ValidationResult::Invalid(_)));
        assert!(matches!(prompts.validate_branch_name("main-"), ValidationResult::Invalid(_)));
        assert!(matches!(prompts.validate_branch_name("branch name"), ValidationResult::Invalid(_)));
        assert!(matches!(prompts.validate_branch_name("branch..name"), ValidationResult::Invalid(_)));
        
        Ok(())
    }

    #[test]
    fn test_validate_years_input() -> Result<()> {
        let prompts = create_test_prompts()?;
        
        // Valid inputs
        assert!(matches!(prompts.validate_years_input("1990"), ValidationResult::Valid));
        assert!(matches!(prompts.validate_years_input("1990-1995"), ValidationResult::Valid));
        assert!(matches!(prompts.validate_years_input("1990,1992,1994"), ValidationResult::Valid));
        
        // Invalid inputs
        assert!(matches!(prompts.validate_years_input(""), ValidationResult::Invalid(_)));
        assert!(matches!(prompts.validate_years_input("abc"), ValidationResult::Invalid(_)));
        assert!(matches!(prompts.validate_years_input("1969"), ValidationResult::Invalid(_)));
        assert!(matches!(prompts.validate_years_input("2031"), ValidationResult::Invalid(_)));
        
        Ok(())
    }

    #[test]
    fn test_validate_hour() -> Result<()> {
        let prompts = create_test_prompts()?;
        
        // Valid hours
        assert!(matches!(prompts.validate_hour("0"), ValidationResult::Valid));
        assert!(matches!(prompts.validate_hour("12"), ValidationResult::Valid));
        assert!(matches!(prompts.validate_hour("23"), ValidationResult::Valid));
        
        // Invalid hours
        assert!(matches!(prompts.validate_hour("24"), ValidationResult::Invalid(_)));
        assert!(matches!(prompts.validate_hour("-1"), ValidationResult::Invalid(_)));
        assert!(matches!(prompts.validate_hour("abc"), ValidationResult::Invalid(_)));
        
        Ok(())
    }

    #[test]
    fn test_validate_github_username() -> Result<()> {
        let prompts = create_test_prompts()?;
        
        // Valid usernames
        assert!(matches!(prompts.validate_github_username("user"), ValidationResult::Valid));
        assert!(matches!(prompts.validate_github_username("user-name"), ValidationResult::Valid));
        assert!(matches!(prompts.validate_github_username("user123"), ValidationResult::Valid));
        
        // Invalid usernames
        assert!(matches!(prompts.validate_github_username(""), ValidationResult::Invalid(_)));
        assert!(matches!(prompts.validate_github_username("-user"), ValidationResult::Invalid(_)));
        assert!(matches!(prompts.validate_github_username("user-"), ValidationResult::Invalid(_)));
        assert!(matches!(prompts.validate_github_username("user@name"), ValidationResult::Invalid(_)));
        
        Ok(())
    }

    #[test]
    fn test_parse_years_input() -> Result<()> {
        let prompts = create_test_prompts()?;
        
        // Single year
        assert_eq!(prompts.parse_years_input("1990")?, vec![1990]);
        
        // Year range
        assert_eq!(prompts.parse_years_input("1990-1992")?, vec![1990, 1991, 1992]);
        
        // Comma-separated list
        assert_eq!(prompts.parse_years_input("1990,1992,1994")?, vec![1990, 1992, 1994]);
        
        // Invalid inputs
        assert!(prompts.parse_years_input("1995-1990").is_err()); // Invalid range
        assert!(prompts.parse_years_input("1969").is_err()); // Year too early
        assert!(prompts.parse_years_input("2031").is_err()); // Year too late
        
        Ok(())
    }

    #[test]
    fn test_validate_email() -> Result<()> {
        let prompts = create_test_prompts()?;
        
        // Valid emails
        assert!(matches!(prompts.validate_email("user@example.com"), ValidationResult::Valid));
        assert!(matches!(prompts.validate_email("test.user@domain.co.uk"), ValidationResult::Valid));
        assert!(matches!(prompts.validate_email("user+tag@example.org"), ValidationResult::Valid));
        
        // Invalid emails
        assert!(matches!(prompts.validate_email(""), ValidationResult::Invalid(_)));
        assert!(matches!(prompts.validate_email("   "), ValidationResult::Invalid(_)));
        assert!(matches!(prompts.validate_email("user"), ValidationResult::Invalid(_)));
        assert!(matches!(prompts.validate_email("user@"), ValidationResult::Invalid(_)));
        assert!(matches!(prompts.validate_email("@example.com"), ValidationResult::Invalid(_)));
        assert!(matches!(prompts.validate_email("user@@example.com"), ValidationResult::Invalid(_)));
        assert!(matches!(prompts.validate_email("user@example"), ValidationResult::Invalid(_)));
        
        Ok(())
    }

    #[test]
    fn test_author_mode_variants() -> Result<()> {
        use crate::git_context::GitIdentity;
        
        // Test all AuthorMode variants
        let current_user_identity = GitIdentity {
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
        };
        
        let manual_identity = GitIdentity {
            name: "Custom Author".to_string(),
            email: "custom@example.com".to_string(),
        };
        
        let current_user_mode = AuthorMode::CurrentUser(current_user_identity.clone());
        let time_traveler_mode = AuthorMode::TimeTraveler;
        let manual_mode = AuthorMode::Manual(manual_identity.clone());
        let ask_each_time_mode = AuthorMode::AskEachTime;
        
        // Test that all variants can be created and compared
        assert_eq!(current_user_mode, AuthorMode::CurrentUser(current_user_identity));
        assert_eq!(time_traveler_mode, AuthorMode::TimeTraveler);
        assert_eq!(manual_mode, AuthorMode::Manual(manual_identity));
        assert_eq!(ask_each_time_mode, AuthorMode::AskEachTime);
        
        // Test that different variants are not equal
        assert_ne!(current_user_mode, time_traveler_mode);
        assert_ne!(manual_mode, time_traveler_mode);
        assert_ne!(ask_each_time_mode, manual_mode);
        
        Ok(())
    }
}