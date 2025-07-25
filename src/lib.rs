use anyhow::{Context, Result};
use chrono::DateTime;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

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
    ) -> Result<Self> {
        // Validate year (reasonable range)
        if year < 1970 || year > 2030 {
            anyhow::bail!("Year must be between 1970 and 2030");
        }
        
        // Validate month
        if month < 1 || month > 12 {
            anyhow::bail!("Month must be between 1 and 12");
        }
        
        // Validate day
        if day < 1 || day > 31 {
            anyhow::bail!("Day must be between 1 and 31");
        }
        
        // Validate hour
        if hour > 23 {
            anyhow::bail!("Hour must be between 0 and 23");
        }
        
        // Validate username
        if username.trim().is_empty() {
            anyhow::bail!("Username cannot be empty");
        }
        
        // Validate token
        if token.trim().is_empty() {
            anyhow::bail!("Token cannot be empty");
        }
        
        Ok(Self {
            year,
            month,
            day,
            hour,
            username,
            token,
            repo_name,
            branch,
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
    let report_progress = |msg: &str| {
        if let Some(p) = &progress {
            p.set_message(msg);
            p.increment();
        }
    };

    report_progress("Cloning target branch...");
    let temp_dir = TempDir::new().context("Failed to create temporary directory")?;
    let repo_path = temp_dir.path().join(&config.repo_name());
    let remote_url = format!("https://{}:{}@github.com/{}/{}.git", config.username, config.token, config.username, config.repo_name());
    // Clone the target branch only
    run_git_command(temp_dir.path(), &["git", "clone", "--branch", &config.branch, "--single-branch", &remote_url, &config.repo_name()])?;

    // For each year, add a file and commit
    let year_file = format!("timetravel-{}.md", config.year);
    let file_path = repo_path.join(&year_file);
    let file_content = format!("# Time Travel Commit for {}\n\nThis file was created to show activity in the year {} on my GitHub profile.\n", config.year, config.year);
    fs::write(&file_path, file_content).context("Failed to write year file")?;

    report_progress("Staging files...");
    run_git_command(&repo_path, &["git", "add", &year_file])?;

    report_progress("Creating backdated commit...");
    let timestamp = config.commit_timestamp()?;
    let commit_message = format!("Time travel commit for {}", config.year);
    run_git_command_with_env(
        &repo_path,
        &["git", "commit", "-m", &commit_message],
        &[
            ("GIT_AUTHOR_DATE", &timestamp),
            ("GIT_COMMITTER_DATE", &timestamp),
            ("GIT_AUTHOR_NAME", "Git Time Traveler"),
            ("GIT_AUTHOR_EMAIL", "timetraveler@example.com"),
            ("GIT_COMMITTER_NAME", "Git Time Traveler"),
            ("GIT_COMMITTER_EMAIL", "timetraveler@example.com"),
        ],
    )?;

    report_progress("Pushing to GitHub...");
    push_to_github(&repo_path, &config.branch, force)?;

    if let Some(p) = progress {
        p.finish("✅ Time travel complete!");
    }

    Ok(())
}

fn run_git_command(repo_path: &Path, args: &[&str]) -> Result<()> {
    let output = Command::new(args[0])
        .current_dir(repo_path)
        .args(&args[1..])
        .output()
        .context("Failed to execute git command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Git command failed: {}", stderr);
    }

    Ok(())
}

fn run_git_command_with_env(
    repo_path: &Path,
    args: &[&str],
    env_vars: &[(&str, &str)],
) -> Result<()> {
    let mut cmd = Command::new(args[0]);
    cmd.current_dir(repo_path).args(&args[1..]);
    for (key, value) in env_vars {
        cmd.env(key, value);
    }
    let output = cmd.output().context("Failed to execute git command")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Git command failed: {}", stderr);
    }
    Ok(())
}

fn push_to_github(repo_path: &Path, branch: &str, force: bool) -> Result<()> {
    let mut args = vec!["git", "push", "origin", branch];
    if force {
        args.push("--force");
    }
    run_git_command(repo_path, &args)
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
            "testbranch".to_string()
        );
        assert!(config.is_ok());

        // Invalid year
        let config = TimeTravelConfig::new(
            1969, 1, 1, 18,
            "testuser".to_string(),
            "token123".to_string(),
            Some("testrepo".to_string()),
            "testbranch".to_string()
        );
        assert!(config.is_err());

        // Invalid month
        let config = TimeTravelConfig::new(
            1990, 13, 1, 18,
            "testuser".to_string(),
            "token123".to_string(),
            Some("testrepo".to_string()),
            "testbranch".to_string()
        );
        assert!(config.is_err());

        // Empty username
        let config = TimeTravelConfig::new(
            1990, 1, 1, 18,
            "".to_string(),
            "token123".to_string(),
            Some("testrepo".to_string()),
            "testbranch".to_string()
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
            "testbranch".to_string()
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
            "testbranch".to_string()
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
            "testbranch".to_string()
        ).unwrap();

        assert_eq!(config.repo_name(), "testrepo");
    }
} 