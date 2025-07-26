use anyhow::{Context, Result};
use std::fmt;

/// Comprehensive error types for Git Time Traveler with actionable recovery suggestions
#[derive(Debug, Clone)]
pub enum TimeTravelError {
    /// User input validation errors
    InvalidInput {
        field: String,
        value: String,
        reason: String,
        suggestion: String,
    },
    
    /// Authentication and authorization errors
    Authentication {
        error_type: AuthError,
        message: String,
        help_url: Option<String>,
        recovery_steps: Vec<String>,
    },
    
    /// Repository-related errors
    Repository {
        error_type: RepoError,
        repository: String,
        message: String,
        recovery_steps: Vec<String>,
    },
    
    /// Git operation errors
    GitOperation {
        operation: String,
        details: String,
        recovery_steps: Vec<String>,
    },
    
    /// Network and API errors
    Network {
        service: String,
        error_type: NetworkError,
        retryable: bool,
        retry_after: Option<std::time::Duration>,
        recovery_steps: Vec<String>,
    },
    
    /// File system errors
    FileSystem {
        operation: String,
        path: String,
        details: String,
        recovery_steps: Vec<String>,
    },
    
    /// Configuration errors
    Configuration {
        setting: String,
        issue: String,
        recovery_steps: Vec<String>,
    },
}

#[derive(Debug, Clone)]
pub enum AuthError {
    InvalidToken,
    InsufficientPermissions,
    TokenExpired,
    RateLimited,
}

#[derive(Debug, Clone)]
pub enum RepoError {
    NotFound,
    AlreadyExists,
    AccessDenied,
    InvalidName,
    TooLarge,
}

#[derive(Debug, Clone)]
pub enum NetworkError {
    ConnectionFailed,
    Timeout,
    RateLimited,
    ServiceUnavailable,
    InvalidResponse,
}

impl TimeTravelError {
    /// Create an invalid input error with helpful suggestions
    pub fn invalid_input(field: &str, value: &str, reason: &str, suggestion: &str) -> Self {
        Self::InvalidInput {
            field: field.to_string(),
            value: value.to_string(),
            reason: reason.to_string(),
            suggestion: suggestion.to_string(),
        }
    }
    
    /// Create an authentication error with recovery steps
    pub fn authentication(error_type: AuthError, message: &str) -> Self {
        let (help_url, recovery_steps) = match error_type {
            AuthError::InvalidToken => (
                Some("https://github.com/settings/tokens".to_string()),
                vec![
                    "1. Go to https://github.com/settings/tokens".to_string(),
                    "2. Click 'Generate new token (classic)'".to_string(),
                    "3. Select 'repo' scope for full repository access".to_string(),
                    "4. Copy the generated token and try again".to_string(),
                ]
            ),
            AuthError::InsufficientPermissions => (
                Some("https://github.com/settings/tokens".to_string()),
                vec![
                    "Your token needs 'repo' permissions".to_string(),
                    "1. Go to your token settings".to_string(),
                    "2. Edit the token or create a new one".to_string(),
                    "3. Ensure 'repo' scope is selected".to_string(),
                ]
            ),
            AuthError::TokenExpired => (
                Some("https://github.com/settings/tokens".to_string()),
                vec![
                    "Your token has expired".to_string(),
                    "1. Go to https://github.com/settings/tokens".to_string(),
                    "2. Generate a new token with 'repo' scope".to_string(),
                    "3. Update your configuration with the new token".to_string(),
                ]
            ),
            AuthError::RateLimited => (
                Some("https://docs.github.com/en/rest/overview/resources-in-the-rest-api#rate-limiting".to_string()),
                vec![
                    "GitHub API rate limit exceeded".to_string(),
                    "1. Wait for the rate limit to reset (usually 1 hour)".to_string(),
                    "2. Consider using a personal access token for higher limits".to_string(),
                    "3. Try again later".to_string(),
                ]
            ),
        };
        
        Self::Authentication {
            error_type,
            message: message.to_string(),
            help_url,
            recovery_steps,
        }
    }
    
    /// Create a repository error with context-specific recovery steps
    pub fn repository(error_type: RepoError, repository: &str, message: &str) -> Self {
        let recovery_steps = match error_type {
            RepoError::NotFound => vec![
                format!("Repository '{}' was not found", repository),
                "1. Check the repository name for typos".to_string(),
                "2. Ensure you have access to the repository".to_string(),
                "3. Use --create-repo flag to create it automatically".to_string(),
            ],
            RepoError::AlreadyExists => vec![
                format!("Repository '{}' already exists", repository),
                "1. Choose a different repository name".to_string(),
                "2. Use --force flag to overwrite (use with caution)".to_string(),
                "3. Delete the existing repository first".to_string(),
            ],
            RepoError::AccessDenied => vec![
                format!("Access denied to repository '{}'", repository),
                "1. Check your GitHub token permissions".to_string(),
                "2. Ensure you own the repository or have write access".to_string(),
                "3. Verify the repository name is correct".to_string(),
            ],
            RepoError::InvalidName => vec![
                format!("Invalid repository name: '{}'", repository),
                "1. Repository names must be 1-100 characters".to_string(),
                "2. Use only letters, numbers, hyphens, and underscores".to_string(),
                "3. Cannot start or end with hyphens".to_string(),
            ],
            RepoError::TooLarge => vec![
                "Repository operation would exceed size limits".to_string(),
                "1. Consider using fewer years or smaller date ranges".to_string(),
                "2. Split the operation into multiple repositories".to_string(),
                "3. Clean up existing repository content first".to_string(),
            ],
        };
        
        Self::Repository {
            error_type,
            repository: repository.to_string(),
            message: message.to_string(),
            recovery_steps,
        }
    }
    
    /// Create a Git operation error with recovery suggestions
    pub fn git_operation(operation: &str, details: &str) -> Self {
        let recovery_steps = match operation {
            "clone" => vec![
                "Failed to clone repository".to_string(),
                "1. Check your internet connection".to_string(),
                "2. Verify the repository URL is correct".to_string(),
                "3. Ensure your GitHub token has access".to_string(),
                "4. Try again in a few moments".to_string(),
            ],
            "commit" => vec![
                "Failed to create commit".to_string(),
                "1. Check if files were properly staged".to_string(),
                "2. Verify Git configuration is correct".to_string(),
                "3. Ensure the repository is not corrupted".to_string(),
            ],
            "push" => vec![
                "Failed to push to remote repository".to_string(),
                "1. Check your internet connection".to_string(),
                "2. Verify your GitHub token permissions".to_string(),
                "3. Try using --force flag if safe to do so".to_string(),
                "4. Check if the branch is protected".to_string(),
            ],
            _ => vec![
                format!("Git operation '{}' failed", operation),
                "1. Check your Git configuration".to_string(),
                "2. Verify repository permissions".to_string(),
                "3. Try the operation again".to_string(),
            ],
        };
        
        Self::GitOperation {
            operation: operation.to_string(),
            details: details.to_string(),
            recovery_steps,
        }
    }
    
    /// Create a network error with retry information
    pub fn network(service: &str, error_type: NetworkError, retryable: bool) -> Self {
        let (retry_after, recovery_steps) = match error_type {
            NetworkError::ConnectionFailed => (
                None,
                vec![
                    format!("Failed to connect to {}", service),
                    "1. Check your internet connection".to_string(),
                    "2. Verify the service is accessible".to_string(),
                    "3. Try again in a few moments".to_string(),
                ]
            ),
            NetworkError::Timeout => (
                Some(std::time::Duration::from_secs(30)),
                vec![
                    format!("Request to {} timed out", service),
                    "1. Check your internet connection speed".to_string(),
                    "2. Try again with a more stable connection".to_string(),
                    "3. The service might be experiencing high load".to_string(),
                ]
            ),
            NetworkError::RateLimited => (
                Some(std::time::Duration::from_secs(3600)),
                vec![
                    format!("{} rate limit exceeded", service),
                    "1. Wait for the rate limit to reset".to_string(),
                    "2. Consider using authentication for higher limits".to_string(),
                    "3. Reduce the frequency of requests".to_string(),
                ]
            ),
            NetworkError::ServiceUnavailable => (
                Some(std::time::Duration::from_secs(300)),
                vec![
                    format!("{} is temporarily unavailable", service),
                    "1. The service is experiencing issues".to_string(),
                    "2. Check the service status page".to_string(),
                    "3. Try again in a few minutes".to_string(),
                ]
            ),
            NetworkError::InvalidResponse => (
                None,
                vec![
                    format!("Received invalid response from {}", service),
                    "1. The service might be experiencing issues".to_string(),
                    "2. Try again in a few moments".to_string(),
                    "3. Check if you're using the latest version".to_string(),
                ]
            ),
        };
        
        Self::Network {
            service: service.to_string(),
            error_type,
            retryable,
            retry_after,
            recovery_steps,
        }
    }
    
    /// Create a file system error with recovery steps
    pub fn file_system(operation: &str, path: &str, details: &str) -> Self {
        let recovery_steps = match operation {
            "read" => vec![
                format!("Failed to read file: {}", path),
                "1. Check if the file exists".to_string(),
                "2. Verify you have read permissions".to_string(),
                "3. Ensure the file is not locked by another process".to_string(),
            ],
            "write" => vec![
                format!("Failed to write file: {}", path),
                "1. Check if you have write permissions".to_string(),
                "2. Ensure the directory exists".to_string(),
                "3. Verify there's enough disk space".to_string(),
            ],
            "create" => vec![
                format!("Failed to create file: {}", path),
                "1. Check if the parent directory exists".to_string(),
                "2. Verify you have write permissions".to_string(),
                "3. Ensure the filename is valid".to_string(),
            ],
            _ => vec![
                format!("File system operation '{}' failed for: {}", operation, path),
                "1. Check file permissions".to_string(),
                "2. Verify the path is correct".to_string(),
                "3. Ensure sufficient disk space".to_string(),
            ],
        };
        
        Self::FileSystem {
            operation: operation.to_string(),
            path: path.to_string(),
            details: details.to_string(),
            recovery_steps,
        }
    }
    
    /// Create a configuration error with helpful guidance
    pub fn configuration(setting: &str, issue: &str) -> Self {
        let recovery_steps = vec![
            format!("Configuration issue with {}: {}", setting, issue),
            "1. Check your configuration file syntax".to_string(),
            "2. Verify all required settings are present".to_string(),
            "3. Use --help to see valid options".to_string(),
            "4. Reset to defaults if needed".to_string(),
        ];
        
        Self::Configuration {
            setting: setting.to_string(),
            issue: issue.to_string(),
            recovery_steps,
        }
    }
    
    /// Get user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            Self::InvalidInput { field, value, reason, .. } => {
                format!("Invalid {}: '{}' - {}", field, value, reason)
            }
            Self::Authentication { message, .. } => message.clone(),
            Self::Repository { message, .. } => message.clone(),
            Self::GitOperation { operation, details, .. } => {
                format!("Git {} operation failed: {}", operation, details)
            }
            Self::Network { service, error_type, .. } => {
                match error_type {
                    NetworkError::ConnectionFailed => format!("Cannot connect to {}", service),
                    NetworkError::Timeout => format!("Request to {} timed out", service),
                    NetworkError::RateLimited => format!("{} rate limit exceeded", service),
                    NetworkError::ServiceUnavailable => format!("{} is unavailable", service),
                    NetworkError::InvalidResponse => format!("Invalid response from {}", service),
                }
            }
            Self::FileSystem { operation, path, details, .. } => {
                format!("File {} failed for '{}': {}", operation, path, details)
            }
            Self::Configuration { setting, issue, .. } => {
                format!("Configuration error in {}: {}", setting, issue)
            }
        }
    }
    
    /// Get recovery suggestions
    pub fn recovery_suggestions(&self) -> &[String] {
        match self {
            Self::InvalidInput { .. } => &[],
            Self::Authentication { recovery_steps, .. } => recovery_steps,
            Self::Repository { recovery_steps, .. } => recovery_steps,
            Self::GitOperation { recovery_steps, .. } => recovery_steps,
            Self::Network { recovery_steps, .. } => recovery_steps,
            Self::FileSystem { recovery_steps, .. } => recovery_steps,
            Self::Configuration { recovery_steps, .. } => recovery_steps,
        }
    }
    
    /// Get help URL if available
    pub fn help_url(&self) -> Option<&str> {
        match self {
            Self::Authentication { help_url, .. } => help_url.as_deref(),
            _ => None,
        }
    }
    
    /// Check if the error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Network { retryable, .. } => *retryable,
            Self::GitOperation { operation, .. } => {
                matches!(operation.as_str(), "clone" | "push")
            }
            _ => false,
        }
    }
    
    /// Get suggested retry delay
    pub fn retry_after(&self) -> Option<std::time::Duration> {
        match self {
            Self::Network { retry_after, .. } => *retry_after,
            _ => None,
        }
    }
    
    /// Get suggestion for the invalid input error
    pub fn suggestion(&self) -> Option<&str> {
        match self {
            Self::InvalidInput { suggestion, .. } => Some(suggestion),
            _ => None,
        }
    }
}

impl fmt::Display for TimeTravelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.user_message())
    }
}

impl std::error::Error for TimeTravelError {}

/// Helper trait for converting common errors to TimeTravelError
pub trait IntoTimeTravelError<T> {
    fn with_context_tt(self, error: TimeTravelError) -> Result<T>;
}

impl<T, E> IntoTimeTravelError<T> for std::result::Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn with_context_tt(self, error: TimeTravelError) -> Result<T> {
        self.with_context(|| error)
    }
}

/// Enhanced error formatting for user-friendly display
pub fn format_error_for_user(error: &anyhow::Error) -> String {
    // Try to downcast to TimeTravelError first
    if let Some(tt_error) = error.downcast_ref::<TimeTravelError>() {
        format_time_travel_error(tt_error)
    } else {
        // Fallback to generic error formatting
        format_generic_error(error)
    }
}

/// Format TimeTravelError with colors and suggestions
fn format_time_travel_error(error: &TimeTravelError) -> String {
    use colored::*;
    
    let mut output = String::new();
    
    // Main error message
    output.push_str(&format!("{} {}\n", "❌".red(), error.user_message().red().bold()));
    
    // Recovery suggestions
    let suggestions = error.recovery_suggestions();
    if !suggestions.is_empty() {
        output.push_str(&format!("\n{}\n", "💡 How to fix this:".yellow().bold()));
        for suggestion in suggestions {
            output.push_str(&format!("  {}\n", suggestion.cyan()));
        }
    }
    
    // Help URL if available
    if let Some(url) = error.help_url() {
        output.push_str(&format!("\n{} {}\n", "📖 More info:".blue().bold(), url.blue().underline()));
    }
    
    // Retry information
    if error.is_retryable() {
        output.push_str(&format!("\n{} {}\n", "🔄".green(), "This operation can be retried".green()));
        if let Some(delay) = error.retry_after() {
            output.push_str(&format!("   Wait {} seconds before retrying\n", delay.as_secs()));
        }
    }
    
    output
}

/// Format generic errors with basic styling
fn format_generic_error(error: &anyhow::Error) -> String {
    use colored::*;
    
    let mut output = String::new();
    output.push_str(&format!("{} {}\n", "❌".red(), error.to_string().red().bold()));
    
    // Show error chain if available
    let mut current = error.source();
    if current.is_some() {
        output.push_str(&format!("\n{}\n", "🔍 Error details:".yellow().bold()));
        let mut level = 1;
        while let Some(err) = current {
            output.push_str(&format!("  {}: {}\n", level, err.to_string().dimmed()));
            current = err.source();
            level += 1;
        }
    }
    
    output
}

/// Validation helpers for common input types
pub mod validation {
    use super::*;
    
    /// Validate year input
    pub fn validate_year(year: u32) -> Result<u32, TimeTravelError> {
        if year < 1970 || year > 2030 {
            Err(TimeTravelError::invalid_input(
                "year",
                &year.to_string(),
                "must be between 1970 and 2030",
                "Choose a year within the valid range (1970-2030)"
            ))
        } else {
            Ok(year)
        }
    }
    
    /// Validate month input
    pub fn validate_month(month: u32) -> Result<u32, TimeTravelError> {
        if month < 1 || month > 12 {
            Err(TimeTravelError::invalid_input(
                "month",
                &month.to_string(),
                "must be between 1 and 12",
                "Use a valid month number (1-12)"
            ))
        } else {
            Ok(month)
        }
    }
    
    /// Validate day input
    pub fn validate_day(day: u32) -> Result<u32, TimeTravelError> {
        if day < 1 || day > 31 {
            Err(TimeTravelError::invalid_input(
                "day",
                &day.to_string(),
                "must be between 1 and 31",
                "Use a valid day number (1-31)"
            ))
        } else {
            Ok(day)
        }
    }
    
    /// Validate hour input
    pub fn validate_hour(hour: u32) -> Result<u32, TimeTravelError> {
        if hour > 23 {
            Err(TimeTravelError::invalid_input(
                "hour",
                &hour.to_string(),
                "must be between 0 and 23",
                "Use 24-hour format (0-23)"
            ))
        } else {
            Ok(hour)
        }
    }
    
    /// Validate username input
    pub fn validate_username(username: &str) -> Result<String, TimeTravelError> {
        let trimmed = username.trim();
        if trimmed.is_empty() {
            Err(TimeTravelError::invalid_input(
                "username",
                username,
                "cannot be empty",
                "Provide your GitHub username"
            ))
        } else if trimmed.len() > 39 {
            Err(TimeTravelError::invalid_input(
                "username",
                username,
                "too long (max 39 characters)",
                "GitHub usernames must be 39 characters or less"
            ))
        } else {
            Ok(trimmed.to_string())
        }
    }
    
    /// Validate token input
    pub fn validate_token(token: &str) -> Result<String, TimeTravelError> {
        let trimmed = token.trim();
        if trimmed.is_empty() {
            Err(TimeTravelError::invalid_input(
                "token",
                "***",
                "cannot be empty",
                "Provide a valid GitHub personal access token"
            ))
        } else if !trimmed.starts_with("ghp_") && !trimmed.starts_with("github_pat_") {
            Err(TimeTravelError::invalid_input(
                "token",
                "***",
                "invalid format",
                "GitHub tokens should start with 'ghp_' or 'github_pat_'"
            ))
        } else {
            Ok(trimmed.to_string())
        }
    }
    
    /// Validate repository name
    pub fn validate_repository_name(name: &str) -> Result<String, TimeTravelError> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            Err(TimeTravelError::invalid_input(
                "repository name",
                name,
                "cannot be empty",
                "Provide a valid repository name"
            ))
        } else if trimmed.len() > 100 {
            Err(TimeTravelError::invalid_input(
                "repository name",
                name,
                "too long (max 100 characters)",
                "Repository names must be 100 characters or less"
            ))
        } else if trimmed.starts_with('-') || trimmed.ends_with('-') {
            Err(TimeTravelError::invalid_input(
                "repository name",
                name,
                "cannot start or end with hyphens",
                "Remove leading/trailing hyphens from the name"
            ))
        } else if !trimmed.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.') {
            Err(TimeTravelError::invalid_input(
                "repository name",
                name,
                "contains invalid characters",
                "Use only letters, numbers, hyphens, underscores, and periods"
            ))
        } else {
            Ok(trimmed.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_invalid_input_error() {
        let error = TimeTravelError::invalid_input(
            "year",
            "1969",
            "must be between 1970 and 2030",
            "Choose a year within the valid range"
        );
        
        assert_eq!(error.user_message(), "Invalid year: '1969' - must be between 1970 and 2030");
        assert_eq!(error.suggestion().unwrap(), "Choose a year within the valid range");
    }
    
    #[test]
    fn test_authentication_error() {
        let error = TimeTravelError::authentication(
            AuthError::InvalidToken,
            "Token validation failed"
        );
        
        assert_eq!(error.user_message(), "Token validation failed");
        assert!(error.help_url().is_some());
        assert!(!error.recovery_suggestions().is_empty());
    }
    
    #[test]
    fn test_validation_helpers() {
        use validation::*;
        
        // Valid inputs
        assert!(validate_year(1990).is_ok());
        assert!(validate_month(6).is_ok());
        assert!(validate_day(15).is_ok());
        assert!(validate_hour(18).is_ok());
        
        // Invalid inputs
        assert!(validate_year(1969).is_err());
        assert!(validate_month(13).is_err());
        assert!(validate_day(32).is_err());
        assert!(validate_hour(24).is_err());
    }
    
    #[test]
    fn test_repository_name_validation() {
        use validation::validate_repository_name;
        
        // Valid names
        assert!(validate_repository_name("my-repo").is_ok());
        assert!(validate_repository_name("my_repo").is_ok());
        assert!(validate_repository_name("my.repo").is_ok());
        assert!(validate_repository_name("myrepo123").is_ok());
        
        // Invalid names
        assert!(validate_repository_name("").is_err());
        assert!(validate_repository_name("-invalid").is_err());
        assert!(validate_repository_name("invalid-").is_err());
        assert!(validate_repository_name("invalid@name").is_err());
    }
}