use anyhow::{Context, Result};
use chrono::DateTime;
use std::path::Path;

pub mod git_context;
pub mod git_operations;
pub mod github_client;
pub mod session;
pub mod defaults;
pub mod interactive;
pub mod date_parser;
pub mod errors;
pub mod dry_run;

pub use git_context::{GitContext, GitContextDetector, GitIdentity, GitRemote};
pub use git_operations::{GitOperations, TimeTravelCommitConfig, RepositoryConfig, GitCredentials, CommitResult, RepositoryResult};
pub use github_client::{GitHubClient, CreateRepositoryRequest, Repository, User, Branch, TokenInfo};
pub use session::{SessionManager, SessionData, SessionSuggestions, SessionStats, UserPreferences, RecentContext};
pub use defaults::{DefaultsEngine, IntelligentDefaults, AuthorMode, ContextAnalysis, DetectedPattern};
pub use interactive::{InteractivePrompts, UserChoices, ValidationResult};
pub use date_parser::{DateInput, DateParser, TimestampConfig, generate_timestamps};
pub use errors::{TimeTravelError, AuthError, RepoError, NetworkError, format_error_for_user};
pub use dry_run::{DryRunExecutor, DryRunConfig, DryRunPlan, display_and_confirm_dry_run};

/// Configuration for creating a time-traveled repository
#[derive(Debug, Clone)]
pub struct TimeTravelConfig {
    pub year: u32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub username: String,
    pub token: String,
    pub repo_name: Option<String>,
    pub branch: String,
    pub author: Option<GitIdentity>,
}

impl TimeTravelConfig {
    /// Create a new configuration with validation
    pub fn new(
        year: u32,
        month: u32,
        day: u32,
        hour: u32,
        username: String,
        token: String,
        repo_name: Option<String>,
        branch: String,
        author: Option<GitIdentity>,
    ) -> Result<Self> {
        use errors::validation::*;
        
        // Validate all inputs using the new validation system
        let validated_year = validate_year(year)
            .context("Invalid year provided")?;
        let validated_month = validate_month(month)
            .context("Invalid month provided")?;
        let validated_day = validate_day(day)
            .context("Invalid day provided")?;
        let validated_hour = validate_hour(hour)
            .context("Invalid hour provided")?;
        let validated_username = validate_username(&username)
            .context("Invalid username provided")?;
        let validated_token = validate_token(&token)
            .context("Invalid token provided")?;
        
        // Validate repository name if provided
        if let Some(ref name) = repo_name {
            errors::validation::validate_repository_name(name)
                .context("Invalid repository name provided")?;
        }
        
        Ok(Self {
            year: validated_year,
            month: validated_month,
            day: validated_day,
            hour: validated_hour,
            username: validated_username,
            token: validated_token,
            repo_name,
            branch,
            author,
        })
    }

    /// Get the repository name (custom or year)
    pub fn repo_name(&self) -> String {
        if let Some(ref name) = self.repo_name {
            name.clone()
        } else {
            self.year.to_string()
        }
    }

    /// Get the commit timestamp as an ISO 8601 string
    pub fn commit_timestamp(&self) -> Result<String> {
        let datetime_str = format!("{}-{:02}-{:02}T{:02}:00:00", 
                                  self.year, self.month, self.day, self.hour);
        let datetime = DateTime::parse_from_rfc3339(&format!("{}Z", datetime_str))
            .context("Failed to parse datetime")?;
        
        Ok(datetime.format("%Y-%m-%dT%H:%M:%S").to_string())
    }

    /// Get the formatted date string for display
    pub fn formatted_date(&self) -> String {
        format!("{}-{:02}-{:02} at {:02}:00:00", 
                self.year, self.month, self.day, self.hour)
    }
}

/// Progress callback trait for reporting progress
pub trait ProgressCallback {
    fn set_message(&self, message: &str);
    fn increment(&self);
    fn finish(&self, message: &str);
}

/// Create a time-traveled repository and push it to GitHub
pub async fn create_time_traveled_repo(
    config: &TimeTravelConfig,
    progress: Option<&dyn ProgressCallback>,
    force: bool,
) -> Result<()> {
    create_time_traveled_repo_with_options(config, progress, force, false).await
}

/// Create a time-traveled repository with dry-run support
pub async fn create_time_traveled_repo_with_options(
    config: &TimeTravelConfig,
    progress: Option<&dyn ProgressCallback>,
    force: bool,
    dry_run: bool,
) -> Result<()> {
    // Handle dry-run mode
    if dry_run {
        let dry_run_config = dry_run::DryRunConfig {
            show_detailed_operations: true,
            show_file_previews: true,
            show_risks: true,
            require_confirmation: false, // Don't require confirmation in dry-run
            interactive_confirmations: false,
        };
        
        let executor = dry_run::DryRunExecutor::new(dry_run_config);
        let plan = executor.create_plan(&[config.clone()])?;
        executor.display_plan(&plan)?;
        
        if let Some(p) = progress {
            p.finish("✅ Dry run complete - no changes made");
        }
        return Ok(());
    }

    let report_progress = |msg: &str| {
        if let Some(p) = &progress {
            p.set_message(msg);
            p.increment();
        }
    };

    // Initialize GitHub client and Git operations
    let github_client = GitHubClient::new(config.username.clone(), config.token.clone())
        .context("Failed to create GitHub client")?;
    let mut git_ops = GitOperations::new();

    report_progress("Validating GitHub token...");
    
    // Validate token and check permissions
    github_client.check_permissions().await
        .context("GitHub token validation failed")?;

    report_progress("Checking repository existence...");
    
    // Check if repository exists, create if it doesn't
    let repo_exists = github_client.repository_exists(&config.repo_name()).await
        .context("Failed to check repository existence")?;

    if !repo_exists {
        report_progress("Creating repository on GitHub...");
        
        let description = format!("Time travel repository for year {}", config.year);
        github_client.create_repository_with_defaults(
            &config.repo_name(),
            Some(&description),
            false, // public repository
        ).await.context("Failed to create repository on GitHub")?;
        
        // Wait a moment for repository to be fully initialized
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }

    report_progress("Cloning repository...");
    
    // Set up repository configuration
    let remote_url = format!("https://github.com/{}/{}.git", config.username, config.repo_name());
    let repo_config = RepositoryConfig {
        url: remote_url,
        branch: config.branch.clone(),
        local_path: None,
        credentials: Some(GitCredentials {
            username: config.username.clone(),
            token: config.token.clone(),
        }),
    };

    // Clone the repository
    let repo_result = git_ops.clone_repository(&repo_config)
        .context("Failed to clone repository")?;

    // Open the cloned repository
    let repo = git_ops.open_repository(&repo_result.repository_path)
        .context("Failed to open cloned repository")?;

    report_progress("Creating time travel content...");
    
    // Create time travel file
    let year_file = format!("timetravel-{}.md", config.year);
    let file_path = Path::new(&year_file);
    let file_content = git_ops.generate_time_travel_content(config.year, &config.repo_name());
    
    git_ops.create_file_with_content(&repo_result.repository_path, file_path, &file_content)
        .context("Failed to create time travel file")?;

    report_progress("Creating backdated commit...");
    
    // Parse timestamp for commit
    let timestamp_str = config.commit_timestamp()?;
    let timestamp = DateTime::parse_from_rfc3339(&format!("{}Z", timestamp_str))
        .context("Failed to parse timestamp")?
        .with_timezone(&chrono::Utc);

    // Set up commit configuration
    let author = config.author.clone().unwrap_or_else(|| GitIdentity {
        name: "Git Time Traveler".to_string(),
        email: "timetraveler@example.com".to_string(),
    });

    let commit_config = TimeTravelCommitConfig {
        timestamp,
        author: author.clone(),
        committer: author,
        message: format!("Time travel commit for {}", config.year),
        files_to_add: vec![file_path.to_path_buf()],
    };

    // Create the time travel commit
    let _commit_result = git_ops.create_time_travel_commit(&repo, &commit_config)
        .context("Failed to create time travel commit")?;

    report_progress("Pushing to GitHub...");
    
    // Push to remote
    let credentials = GitCredentials {
        username: config.username.clone(),
        token: config.token.clone(),
    };

    git_ops.push_to_remote(&repo, "origin", &config.branch, Some(&credentials), force)
        .context("Failed to push to GitHub")?;

    if let Some(p) = progress {
        p.finish("✅ Time travel complete!");
    }

    Ok(())
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_travel_config_validation() {
        // Valid config
        let config = TimeTravelConfig::new(
            1990, 1, 1, 18,
            "testuser".to_string(),
            "token123".to_string(),
            Some("testrepo".to_string()),
            "testbranch".to_string(),
            None
        );
        assert!(config.is_ok());

        // Invalid year
        let config = TimeTravelConfig::new(
            1969, 1, 1, 18,
            "testuser".to_string(),
            "token123".to_string(),
            Some("testrepo".to_string()),
            "testbranch".to_string(),
            None
        );
        assert!(config.is_err());

        // Invalid month
        let config = TimeTravelConfig::new(
            1990, 13, 1, 18,
            "testuser".to_string(),
            "token123".to_string(),
            Some("testrepo".to_string()),
            "testbranch".to_string(),
            None
        );
        assert!(config.is_err());

        // Empty username
        let config = TimeTravelConfig::new(
            1990, 1, 1, 18,
            "".to_string(),
            "token123".to_string(),
            Some("testrepo".to_string()),
            "testbranch".to_string(),
            None
        );
        assert!(config.is_err());
    }

    #[test]
    fn test_commit_timestamp() {
        let config = TimeTravelConfig::new(
            1990, 1, 1, 18,
            "testuser".to_string(),
            "token123".to_string(),
            Some("testrepo".to_string()),
            "testbranch".to_string(),
            None
        ).unwrap();

        let timestamp = config.commit_timestamp().unwrap();
        assert_eq!(timestamp, "1990-01-01T18:00:00");
    }

    #[test]
    fn test_formatted_date() {
        let config = TimeTravelConfig::new(
            1990, 1, 1, 18,
            "testuser".to_string(),
            "token123".to_string(),
            Some("testrepo".to_string()),
            "testbranch".to_string(),
            None
        ).unwrap();

        assert_eq!(config.formatted_date(), "1990-01-01 at 18:00:00");
    }

    #[test]
    fn test_repo_name() {
        let config = TimeTravelConfig::new(
            1990, 1, 1, 18,
            "testuser".to_string(),
            "token123".to_string(),
            Some("testrepo".to_string()),
            "testbranch".to_string(),
            None
        ).unwrap();

        assert_eq!(config.repo_name(), "testrepo");
    }

    #[test]
    fn test_author_configuration() {
        // Test with no author (should use default)
        let config_no_author = TimeTravelConfig::new(
            1990, 1, 1, 18,
            "testuser".to_string(),
            "token123".to_string(),
            Some("testrepo".to_string()),
            "testbranch".to_string(),
            None
        ).unwrap();
        assert!(config_no_author.author.is_none());

        // Test with custom author
        let custom_author = GitIdentity {
            name: "Custom Author".to_string(),
            email: "custom@example.com".to_string(),
        };
        let config_with_author = TimeTravelConfig::new(
            1990, 1, 1, 18,
            "testuser".to_string(),
            "token123".to_string(),
            Some("testrepo".to_string()),
            "testbranch".to_string(),
            Some(custom_author.clone())
        ).unwrap();
        
        assert!(config_with_author.author.is_some());
        let author = config_with_author.author.unwrap();
        assert_eq!(author.name, "Custom Author");
        assert_eq!(author.email, "custom@example.com");
    }
} 