use anyhow::{Context, Result};
use bincode;
use serde::{Deserialize, Serialize};

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Session data that persists across npx executions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    /// User preferences learned over time
    pub user_preferences: UserPreferences,
    /// Recent repository contexts
    pub recent_contexts: Vec<RecentContext>,
    /// Session metadata
    pub metadata: SessionMetadata,
}

/// User preferences that are learned and stored
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    /// Preferred author mode (current user, time traveler, etc.)
    pub preferred_author_mode: Option<String>,
    /// Most commonly used years
    pub favorite_years: Vec<u32>,
    /// Preferred time of day for commits (hour)
    pub preferred_hour: Option<u32>,
    /// Preferred repositories
    pub favorite_repositories: Vec<String>,
    /// GitHub username if known
    pub github_username: Option<String>,
    /// Preferred branch names
    pub preferred_branches: Vec<String>,
}

/// Recent context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentContext {
    /// Working directory path
    pub working_directory: PathBuf,
    /// Repository name if detected
    pub repository_name: Option<String>,
    /// Branch name
    pub branch_name: Option<String>,
    /// User identity used
    pub user_identity: Option<String>,
    /// Timestamp of last use
    pub last_used: u64,
    /// Success count for this context
    pub success_count: u32,
}

/// Session metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    /// Total number of executions
    pub total_executions: u32,
    /// First use timestamp
    pub first_use: u64,
    /// Last use timestamp
    pub last_use: u64,
    /// Session format version for migration
    pub version: u32,
}

impl Default for SessionData {
    fn default() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            user_preferences: UserPreferences::default(),
            recent_contexts: Vec::new(),
            metadata: SessionMetadata {
                total_executions: 0,
                first_use: now,
                last_use: now,
                version: 1,
            },
        }
    }
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            preferred_author_mode: None,
            favorite_years: Vec::new(),
            preferred_hour: Some(18), // Default to 6 PM
            favorite_repositories: Vec::new(),
            github_username: None,
            preferred_branches: vec!["main".to_string(), "master".to_string()],
        }
    }
}

/// Session manager for handling persistence across npx executions
pub struct SessionManager {
    #[allow(dead_code)]
    pub session_dir: PathBuf,
    pub session_file: PathBuf,
    pub data: SessionData,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Result<Self> {
        let session_dir = Self::get_session_directory()?;
        let session_file = session_dir.join("session.bin");
        
        // Ensure session directory exists
        fs::create_dir_all(&session_dir)
            .context("Failed to create session directory")?;
        
        // Load existing session data or create new
        let data = Self::load_session_data(&session_file)
            .unwrap_or_else(|_| SessionData::default());
        
        Ok(Self {
            session_dir,
            session_file,
            data,
        })
    }

    /// Get the session directory path
    fn get_session_directory() -> Result<PathBuf> {
        let home_dir = dirs::home_dir()
            .context("Could not determine home directory")?;
        
        Ok(home_dir.join(".config").join("git-timetraveler"))
    }

    /// Load session data from file
    fn load_session_data(file_path: &Path) -> Result<SessionData> {
        let data = fs::read(file_path)
            .context("Failed to read session file")?;
        
        let session_data: SessionData = bincode::deserialize(&data)
            .context("Failed to deserialize session data")?;
        
        Ok(session_data)
    }

    /// Save session data to file
    pub fn save(&mut self) -> Result<()> {
        // Update metadata
        self.data.metadata.last_use = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.data.metadata.total_executions += 1;

        // Serialize data
        let serialized = bincode::serialize(&self.data)
            .context("Failed to serialize session data")?;
        
        // Write to file atomically
        let temp_file = self.session_file.with_extension("tmp");
        fs::write(&temp_file, serialized)
            .context("Failed to write session data to temp file")?;
        
        fs::rename(&temp_file, &self.session_file)
            .context("Failed to move temp file to session file")?;
        
        Ok(())
    }

    /// Get current session data
    pub fn get_data(&self) -> &SessionData {
        &self.data
    }

    /// Get mutable reference to session data
    pub fn get_data_mut(&mut self) -> &mut SessionData {
        &mut self.data
    }

    /// Update user preferences based on recent choices
    pub fn learn_from_choice(&mut self, choice_type: &str, value: &str) {
        match choice_type {
            "author_mode" => {
                self.data.user_preferences.preferred_author_mode = Some(value.to_string());
            }
            "year" => {
                if let Ok(year) = value.parse::<u32>() {
                    if !self.data.user_preferences.favorite_years.contains(&year) {
                        self.data.user_preferences.favorite_years.push(year);
                        // Keep only the most recent 10 years
                        if self.data.user_preferences.favorite_years.len() > 10 {
                            self.data.user_preferences.favorite_years.remove(0);
                        }
                    }
                }
            }
            "repository" => {
                let repo = value.to_string();
                if !self.data.user_preferences.favorite_repositories.contains(&repo) {
                    self.data.user_preferences.favorite_repositories.push(repo);
                    // Keep only the most recent 5 repositories
                    if self.data.user_preferences.favorite_repositories.len() > 5 {
                        self.data.user_preferences.favorite_repositories.remove(0);
                    }
                }
            }
            "github_username" => {
                self.data.user_preferences.github_username = Some(value.to_string());
            }
            "hour" => {
                if let Ok(hour) = value.parse::<u32>() {
                    if hour <= 23 {
                        self.data.user_preferences.preferred_hour = Some(hour);
                    }
                }
            }
            _ => {} // Unknown choice type
        }
    }

    /// Add or update a recent context
    pub fn update_context(&mut self, working_dir: &Path, repo_name: Option<&str>, 
                         branch_name: Option<&str>, user_identity: Option<&str>, 
                         success: bool) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Find existing context or create new one
        let mut found = false;
        for context in &mut self.data.recent_contexts {
            if context.working_directory == working_dir {
                context.last_used = now;
                if success {
                    context.success_count += 1;
                }
                // Update other fields if provided
                if let Some(repo) = repo_name {
                    context.repository_name = Some(repo.to_string());
                }
                if let Some(branch) = branch_name {
                    context.branch_name = Some(branch.to_string());
                }
                if let Some(identity) = user_identity {
                    context.user_identity = Some(identity.to_string());
                }
                found = true;
                break;
            }
        }

        if !found {
            let new_context = RecentContext {
                working_directory: working_dir.to_path_buf(),
                repository_name: repo_name.map(|s| s.to_string()),
                branch_name: branch_name.map(|s| s.to_string()),
                user_identity: user_identity.map(|s| s.to_string()),
                last_used: now,
                success_count: if success { 1 } else { 0 },
            };
            self.data.recent_contexts.push(new_context);
        }

        // Keep only the most recent 10 contexts
        if self.data.recent_contexts.len() > 10 {
            // Sort by last_used and keep the most recent
            self.data.recent_contexts.sort_by(|a, b| b.last_used.cmp(&a.last_used));
            self.data.recent_contexts.truncate(10);
        }
    }

    /// Get suggestions based on current context and history
    pub fn get_suggestions(&self, current_dir: &Path) -> SessionSuggestions {
        let mut suggestions = SessionSuggestions::default();

        // Find matching recent context
        let matching_context = self.data.recent_contexts.iter()
            .find(|ctx| ctx.working_directory == current_dir);

        if let Some(context) = matching_context {
            suggestions.suggested_repository = context.repository_name.clone();
            suggestions.suggested_branch = context.branch_name.clone();
            suggestions.suggested_user_identity = context.user_identity.clone();
        }

        // Add general preferences
        suggestions.suggested_author_mode = self.data.user_preferences.preferred_author_mode.clone();
        suggestions.suggested_hour = self.data.user_preferences.preferred_hour;
        suggestions.suggested_years = self.data.user_preferences.favorite_years.clone();
        suggestions.suggested_repositories = self.data.user_preferences.favorite_repositories.clone();
        suggestions.github_username = self.data.user_preferences.github_username.clone();

        suggestions
    }

    /// Clean up old session data to prevent storage bloat
    pub fn cleanup(&mut self) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Remove contexts older than 30 days
        let thirty_days_ago = now - (30 * 24 * 60 * 60);
        self.data.recent_contexts.retain(|ctx| ctx.last_used > thirty_days_ago);

        // Limit favorite years to reasonable number
        if self.data.user_preferences.favorite_years.len() > 20 {
            self.data.user_preferences.favorite_years.truncate(20);
        }

        // Limit favorite repositories
        if self.data.user_preferences.favorite_repositories.len() > 10 {
            self.data.user_preferences.favorite_repositories.truncate(10);
        }

        Ok(())
    }

    /// Get session statistics
    pub fn get_stats(&self) -> SessionStats {
        SessionStats {
            total_executions: self.data.metadata.total_executions,
            days_since_first_use: {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                (now - self.data.metadata.first_use) / (24 * 60 * 60)
            },
            recent_contexts_count: self.data.recent_contexts.len(),
            favorite_years_count: self.data.user_preferences.favorite_years.len(),
        }
    }
}

/// Suggestions based on session history
#[derive(Debug, Clone, Default)]
pub struct SessionSuggestions {
    pub suggested_repository: Option<String>,
    pub suggested_branch: Option<String>,
    pub suggested_user_identity: Option<String>,
    pub suggested_author_mode: Option<String>,
    pub suggested_hour: Option<u32>,
    pub suggested_years: Vec<u32>,
    pub suggested_repositories: Vec<String>,
    pub github_username: Option<String>,
}

/// Session statistics
#[derive(Debug, Clone)]
pub struct SessionStats {
    pub total_executions: u32,
    pub days_since_first_use: u64,
    pub recent_contexts_count: usize,
    pub favorite_years_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;


    fn create_test_session_manager() -> Result<(TempDir, SessionManager)> {
        let temp_dir = TempDir::new()?;
        
        // Override the session directory for testing
        let session_dir = temp_dir.path().join(".config").join("git-timetraveler");
        fs::create_dir_all(&session_dir)?;
        
        let session_file = session_dir.join("session.bin");
        let data = SessionData::default();
        
        let manager = SessionManager {
            session_dir,
            session_file,
            data,
        };
        
        Ok((temp_dir, manager))
    }

    #[test]
    fn test_session_data_serialization() -> Result<()> {
        let data = SessionData::default();
        let serialized = bincode::serialize(&data)?;
        let deserialized: SessionData = bincode::deserialize(&serialized)?;
        
        assert_eq!(data.metadata.version, deserialized.metadata.version);
        assert_eq!(data.user_preferences.preferred_hour, deserialized.user_preferences.preferred_hour);
        
        Ok(())
    }

    #[test]
    fn test_session_save_and_load() -> Result<()> {
        let (_temp_dir, mut manager) = create_test_session_manager()?;
        
        // Modify some data
        manager.learn_from_choice("year", "1990");
        manager.learn_from_choice("repository", "test-repo");
        
        // Save the session
        manager.save()?;
        
        // Load a new manager from the same file
        let loaded_data = SessionManager::load_session_data(&manager.session_file)?;
        
        assert!(loaded_data.user_preferences.favorite_years.contains(&1990));
        assert!(loaded_data.user_preferences.favorite_repositories.contains(&"test-repo".to_string()));
        assert_eq!(loaded_data.metadata.total_executions, 1);
        
        Ok(())
    }

    #[test]
    fn test_learn_from_choice() -> Result<()> {
        let (_temp_dir, mut manager) = create_test_session_manager()?;
        
        manager.learn_from_choice("author_mode", "current_user");
        manager.learn_from_choice("year", "1995");
        manager.learn_from_choice("repository", "my-project");
        manager.learn_from_choice("github_username", "testuser");
        manager.learn_from_choice("hour", "14");
        
        let prefs = &manager.data.user_preferences;
        assert_eq!(prefs.preferred_author_mode, Some("current_user".to_string()));
        assert!(prefs.favorite_years.contains(&1995));
        assert!(prefs.favorite_repositories.contains(&"my-project".to_string()));
        assert_eq!(prefs.github_username, Some("testuser".to_string()));
        assert_eq!(prefs.preferred_hour, Some(14));
        
        Ok(())
    }

    #[test]
    fn test_context_updates() -> Result<()> {
        let (_temp_dir, mut manager) = create_test_session_manager()?;
        let test_dir = Path::new("/test/dir");
        
        // Add a context
        manager.update_context(test_dir, Some("test-repo"), Some("main"), Some("testuser"), true);
        
        assert_eq!(manager.data.recent_contexts.len(), 1);
        let context = &manager.data.recent_contexts[0];
        assert_eq!(context.working_directory, test_dir);
        assert_eq!(context.repository_name, Some("test-repo".to_string()));
        assert_eq!(context.branch_name, Some("main".to_string()));
        assert_eq!(context.success_count, 1);
        
        // Update the same context
        manager.update_context(test_dir, Some("test-repo"), Some("develop"), Some("testuser"), true);
        
        assert_eq!(manager.data.recent_contexts.len(), 1); // Should still be 1
        let context = &manager.data.recent_contexts[0];
        assert_eq!(context.branch_name, Some("develop".to_string()));
        assert_eq!(context.success_count, 2);
        
        Ok(())
    }

    #[test]
    fn test_suggestions() -> Result<()> {
        let (_temp_dir, mut manager) = create_test_session_manager()?;
        let test_dir = Path::new("/test/dir");
        
        // Set up some preferences and context
        manager.learn_from_choice("author_mode", "current_user");
        manager.learn_from_choice("year", "1990");
        manager.update_context(test_dir, Some("test-repo"), Some("main"), Some("testuser"), true);
        
        let suggestions = manager.get_suggestions(test_dir);
        
        assert_eq!(suggestions.suggested_author_mode, Some("current_user".to_string()));
        assert_eq!(suggestions.suggested_repository, Some("test-repo".to_string()));
        assert_eq!(suggestions.suggested_branch, Some("main".to_string()));
        assert!(suggestions.suggested_years.contains(&1990));
        
        Ok(())
    }

    #[test]
    fn test_cleanup() -> Result<()> {
        let (_temp_dir, mut manager) = create_test_session_manager()?;
        
        // Add many years to test cleanup
        for year in 1980..2010 {
            manager.learn_from_choice("year", &year.to_string());
        }
        
        // Add many repositories
        for i in 0..15 {
            manager.learn_from_choice("repository", &format!("repo-{}", i));
        }
        
        manager.cleanup()?;
        
        assert!(manager.data.user_preferences.favorite_years.len() <= 20);
        assert!(manager.data.user_preferences.favorite_repositories.len() <= 10);
        
        Ok(())
    }

    #[test]
    fn test_session_stats() -> Result<()> {
        let (_temp_dir, mut manager) = create_test_session_manager()?;
        
        manager.learn_from_choice("year", "1990");
        manager.update_context(Path::new("/test"), Some("repo"), None, None, true);
        manager.save()?;
        
        let stats = manager.get_stats();
        assert_eq!(stats.total_executions, 1);
        assert_eq!(stats.recent_contexts_count, 1);
        assert_eq!(stats.favorite_years_count, 1);
        
        Ok(())
    }
}