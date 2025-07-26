use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use git2::{Repository, Signature, Commit, PushOptions, RemoteCallbacks, Cred};
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use crate::git_context::GitIdentity;
use crate::errors::TimeTravelError;

/// Enhanced Git operations for time travel functionality
pub struct GitOperations {
    /// Optional temporary directory for cloned repositories
    temp_dir: Option<TempDir>,
}

/// Configuration for creating time travel commits
#[derive(Debug, Clone)]
pub struct TimeTravelCommitConfig {
    pub timestamp: DateTime<Utc>,
    pub author: GitIdentity,
    pub committer: GitIdentity,
    pub message: String,
    pub files_to_add: Vec<PathBuf>,
}

/// Configuration for repository operations
#[derive(Debug, Clone)]
pub struct RepositoryConfig {
    pub url: String,
    pub branch: String,
    pub local_path: Option<PathBuf>,
    pub credentials: Option<GitCredentials>,
}

/// Git credentials for authentication
#[derive(Debug, Clone)]
pub struct GitCredentials {
    pub username: String,
    pub token: String,
}

/// Result of a time travel commit operation
#[derive(Debug)]
pub struct CommitResult {
    pub commit_id: String,
    pub timestamp: DateTime<Utc>,
    pub files_added: Vec<PathBuf>,
    pub message: String,
}

/// Result of a repository operation
#[derive(Debug)]
pub struct RepositoryResult {
    pub repository_path: PathBuf,
    pub current_branch: String,
    pub head_commit: Option<String>,
}

impl GitOperations {
    /// Create a new GitOperations instance
    pub fn new() -> Self {
        Self {
            temp_dir: None,
        }
    }

    /// Clone a repository to a temporary directory
    pub fn clone_repository(&mut self, config: &RepositoryConfig) -> Result<RepositoryResult> {
        // Create temporary directory if not exists
        if self.temp_dir.is_none() {
            self.temp_dir = Some(TempDir::new()
                .map_err(|e| TimeTravelError::file_system(
                    "create",
                    "temporary directory",
                    &e.to_string()
                ))
                .context("Failed to create temporary directory")?);
        }

        let temp_dir = self.temp_dir.as_ref().unwrap();
        let repo_name = self.extract_repo_name_from_url(&config.url)?;
        let local_path = temp_dir.path().join(&repo_name);

        // Set up clone options with credentials if provided
        let mut builder = git2::build::RepoBuilder::new();
        
        if let Some(ref creds) = config.credentials {
            let mut callbacks = RemoteCallbacks::new();
            let username = creds.username.clone();
            let token = creds.token.clone();
            
            callbacks.credentials(move |_url, _username_from_url, _allowed_types| {
                Cred::userpass_plaintext(&username, &token)
            });
            
            let mut fetch_options = git2::FetchOptions::new();
            fetch_options.remote_callbacks(callbacks);
            builder.fetch_options(fetch_options);
        }

        // Clone the repository
        let repo = builder
            .branch(&config.branch)
            .clone(&config.url, &local_path)
            .map_err(|e| {
                let error_msg = e.to_string();
                if error_msg.contains("authentication") || error_msg.contains("401") {
                    TimeTravelError::git_operation("clone", "Authentication failed - check your GitHub token")
                } else if error_msg.contains("not found") || error_msg.contains("404") {
                    TimeTravelError::git_operation("clone", "Repository not found - check the URL and permissions")
                } else if error_msg.contains("network") || error_msg.contains("timeout") {
                    TimeTravelError::git_operation("clone", "Network error - check your internet connection")
                } else {
                    TimeTravelError::git_operation("clone", &error_msg)
                }
            })
            .context("Failed to clone repository")?;

        // Get current branch and head commit
        let current_branch = self.get_current_branch_name(&repo)?;
        let head_commit = self.get_head_commit_id(&repo)?;

        Ok(RepositoryResult {
            repository_path: local_path,
            current_branch,
            head_commit,
        })
    }

    /// Open an existing repository
    pub fn open_repository(&self, path: &Path) -> Result<Repository> {
        Repository::open(path).context("Failed to open repository")
    }

    /// Create a time travel commit with custom timestamp
    pub fn create_time_travel_commit(
        &self,
        repo: &Repository,
        config: &TimeTravelCommitConfig,
    ) -> Result<CommitResult> {
        // Add files to the index
        let mut index = repo.index().context("Failed to get repository index")?;
        
        for file_path in &config.files_to_add {
            index.add_path(file_path)
                .with_context(|| format!("Failed to add file to index: {:?}", file_path))?;
        }
        
        index.write().context("Failed to write index")?;

        // Create tree from index
        let tree_id = index.write_tree().context("Failed to write tree")?;
        let tree = repo.find_tree(tree_id).context("Failed to find tree")?;

        // Get parent commit (if any)
        let parent_commit = match repo.head() {
            Ok(head) => {
                let oid = head.target().context("Failed to get HEAD target")?;
                Some(repo.find_commit(oid).context("Failed to find parent commit")?)
            }
            Err(_) => None, // No parent commit (initial commit)
        };

        // Create signatures with custom timestamp
        let author_sig = Signature::new(
            &config.author.name,
            &config.author.email,
            &git2::Time::new(config.timestamp.timestamp(), 0),
        ).context("Failed to create author signature")?;

        let committer_sig = Signature::new(
            &config.committer.name,
            &config.committer.email,
            &git2::Time::new(config.timestamp.timestamp(), 0),
        ).context("Failed to create committer signature")?;

        // Create the commit
        let parents: Vec<&Commit> = parent_commit.as_ref().map(|c| vec![c]).unwrap_or_default();
        let commit_id = repo.commit(
            Some("HEAD"),
            &author_sig,
            &committer_sig,
            &config.message,
            &tree,
            &parents,
        ).context("Failed to create commit")?;

        Ok(CommitResult {
            commit_id: commit_id.to_string(),
            timestamp: config.timestamp,
            files_added: config.files_to_add.clone(),
            message: config.message.clone(),
        })
    }

    /// Push commits to remote repository
    pub fn push_to_remote(
        &self,
        repo: &Repository,
        remote_name: &str,
        branch: &str,
        credentials: Option<&GitCredentials>,
        force: bool,
    ) -> Result<()> {
        // Find the remote
        let mut remote = repo.find_remote(remote_name)
            .map_err(|e| TimeTravelError::git_operation(
                "push",
                &format!("Remote '{}' not found: {}", remote_name, e)
            ))
            .with_context(|| format!("Failed to find remote: {}", remote_name))?;

        // Set up push options with credentials if provided
        let mut push_options = PushOptions::new();
        
        if let Some(creds) = credentials {
            let mut callbacks = RemoteCallbacks::new();
            let username = creds.username.clone();
            let token = creds.token.clone();
            
            callbacks.credentials(move |_url, _username_from_url, _allowed_types| {
                Cred::userpass_plaintext(&username, &token)
            });
            
            push_options.remote_callbacks(callbacks);
        }

        // Create refspec for push
        let refspec = if force {
            format!("+refs/heads/{}:refs/heads/{}", branch, branch)
        } else {
            format!("refs/heads/{}:refs/heads/{}", branch, branch)
        };

        // Push to remote
        remote.push(&[&refspec], Some(&mut push_options))
            .map_err(|e| {
                let error_msg = e.to_string();
                if error_msg.contains("authentication") || error_msg.contains("401") {
                    TimeTravelError::git_operation("push", "Authentication failed - check your GitHub token permissions")
                } else if error_msg.contains("403") || error_msg.contains("forbidden") {
                    TimeTravelError::git_operation("push", "Push forbidden - check repository permissions or branch protection")
                } else if error_msg.contains("non-fast-forward") {
                    TimeTravelError::git_operation("push", "Push rejected - use --force flag to overwrite remote history (use with caution)")
                } else if error_msg.contains("network") || error_msg.contains("timeout") {
                    TimeTravelError::git_operation("push", "Network error during push - check your internet connection")
                } else {
                    TimeTravelError::git_operation("push", &error_msg)
                }
            })
            .context("Failed to push to remote")?;

        Ok(())
    }

    /// Create a new file with content in the repository
    pub fn create_file_with_content(
        &self,
        repo_path: &Path,
        file_path: &Path,
        content: &str,
    ) -> Result<()> {
        let full_path = repo_path.join(file_path);
        
        // Create parent directories if they don't exist
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create parent directories")?;
        }

        // Write file content
        std::fs::write(&full_path, content)
            .with_context(|| format!("Failed to write file: {:?}", full_path))?;

        Ok(())
    }

    /// Generate time travel content for a specific year
    pub fn generate_time_travel_content(&self, year: u32, repo_name: &str) -> String {
        format!(
            "# Time Travel Commit for {}\n\n\
            This file was created to show activity in the year {} on my GitHub profile.\n\n\
            Repository: {}\n\
            Generated: {}\n\n\
            ## About Time Travel Commits\n\n\
            This commit was backdated to create historical activity on GitHub.\n\
            The actual creation time may differ from the commit timestamp.\n",
            year,
            year,
            repo_name,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        )
    }

    /// Check if a repository exists and is accessible
    pub fn check_repository_exists(&self, path: &Path) -> bool {
        Repository::open(path).is_ok()
    }

    /// Get the current branch name
    fn get_current_branch_name(&self, repo: &Repository) -> Result<String> {
        match repo.head() {
            Ok(head) => {
                if let Some(name) = head.shorthand() {
                    Ok(name.to_string())
                } else {
                    Ok("HEAD".to_string()) // Detached HEAD
                }
            }
            Err(_) => Ok("main".to_string()), // Default for new repository
        }
    }

    /// Get the HEAD commit ID
    fn get_head_commit_id(&self, repo: &Repository) -> Result<Option<String>> {
        match repo.head() {
            Ok(head) => {
                if let Some(oid) = head.target() {
                    Ok(Some(oid.to_string()))
                } else {
                    Ok(None)
                }
            }
            Err(_) => Ok(None),
        }
    }

    /// Extract repository name from URL
    fn extract_repo_name_from_url(&self, url: &str) -> Result<String> {
        let url = url.trim_end_matches(".git");
        let parts: Vec<&str> = url.split('/').collect();
        
        if let Some(name) = parts.last() {
            if !name.is_empty() {
                Ok(name.to_string())
            } else {
                anyhow::bail!("Invalid repository URL: cannot extract name")
            }
        } else {
            anyhow::bail!("Invalid repository URL: no name found")
        }
    }

    /// Clean up temporary directory
    pub fn cleanup(&mut self) {
        self.temp_dir = None;
    }
}

impl Drop for GitOperations {
    fn drop(&mut self) {
        self.cleanup();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_repo() -> Result<(TempDir, Repository)> {
        let temp_dir = TempDir::new()?;
        let repo = Repository::init(temp_dir.path())?;
        
        // Set up basic config
        let mut config = repo.config()?;
        config.set_str("user.name", "Test User")?;
        config.set_str("user.email", "test@example.com")?;
        
        Ok((temp_dir, repo))
    }

    #[test]
    fn test_create_file_with_content() -> Result<()> {
        let (_temp_dir, repo) = create_test_repo()?;
        let git_ops = GitOperations::new();
        let repo_path = repo.workdir().unwrap();
        
        let file_path = Path::new("test.txt");
        let content = "Hello, World!";
        
        git_ops.create_file_with_content(repo_path, file_path, content)?;
        
        let full_path = repo_path.join(file_path);
        assert!(full_path.exists());
        
        let read_content = fs::read_to_string(&full_path)?;
        assert_eq!(read_content, content);
        
        Ok(())
    }

    #[test]
    fn test_generate_time_travel_content() -> Result<()> {
        let git_ops = GitOperations::new();
        let content = git_ops.generate_time_travel_content(1990, "test-repo");
        
        assert!(content.contains("1990"));
        assert!(content.contains("test-repo"));
        assert!(content.contains("Time Travel Commit"));
        
        Ok(())
    }

    #[test]
    fn test_create_time_travel_commit() -> Result<()> {
        let (_temp_dir, repo) = create_test_repo()?;
        let git_ops = GitOperations::new();
        let repo_path = repo.workdir().unwrap();
        
        // Create a test file
        let file_path = Path::new("timetravel-1990.md");
        let content = git_ops.generate_time_travel_content(1990, "test-repo");
        git_ops.create_file_with_content(repo_path, file_path, &content)?;
        
        // Create time travel commit config
        let timestamp = DateTime::parse_from_rfc3339("1990-01-01T18:00:00Z")?.with_timezone(&Utc);
        let author = GitIdentity {
            name: "Time Traveler".to_string(),
            email: "timetraveler@example.com".to_string(),
        };
        
        let config = TimeTravelCommitConfig {
            timestamp,
            author: author.clone(),
            committer: author,
            message: "Time travel commit for 1990".to_string(),
            files_to_add: vec![file_path.to_path_buf()],
        };
        
        // Create the commit
        let result = git_ops.create_time_travel_commit(&repo, &config)?;
        
        assert!(!result.commit_id.is_empty());
        assert_eq!(result.timestamp, timestamp);
        assert_eq!(result.files_added.len(), 1);
        assert_eq!(result.message, "Time travel commit for 1990");
        
        // Verify the commit exists
        let commit_oid = git2::Oid::from_str(&result.commit_id)?;
        let commit = repo.find_commit(commit_oid)?;
        assert_eq!(commit.message().unwrap(), "Time travel commit for 1990");
        
        Ok(())
    }

    #[test]
    fn test_extract_repo_name_from_url() -> Result<()> {
        let git_ops = GitOperations::new();
        
        // Test various URL formats
        assert_eq!(
            git_ops.extract_repo_name_from_url("https://github.com/user/repo.git")?,
            "repo"
        );
        assert_eq!(
            git_ops.extract_repo_name_from_url("https://github.com/user/repo")?,
            "repo"
        );
        assert_eq!(
            git_ops.extract_repo_name_from_url("git@github.com:user/repo.git")?,
            "repo"
        );
        
        Ok(())
    }

    #[test]
    fn test_check_repository_exists() -> Result<()> {
        let (_temp_dir, repo) = create_test_repo()?;
        let git_ops = GitOperations::new();
        
        // Should exist
        assert!(git_ops.check_repository_exists(repo.path()));
        
        // Should not exist
        let non_existent = Path::new("/non/existent/path");
        assert!(!git_ops.check_repository_exists(non_existent));
        
        Ok(())
    }
}