use anyhow::Result;
use git2::{Repository, StatusOptions, Status, Signature, BranchType};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Git identity information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GitIdentity {
    pub name: String,
    pub email: String,
}

/// Git remote information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GitRemote {
    pub name: String,
    pub url: String,
    pub is_github: bool,
}

/// Comprehensive Git context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitContext {
    pub is_git_repo: bool,
    pub repo_path: Option<PathBuf>,
    pub current_branch: Option<String>,
    pub available_branches: Vec<String>,
    pub staged_files: Vec<PathBuf>,
    pub modified_files: Vec<PathBuf>,
    pub untracked_files: Vec<PathBuf>,
    pub user_identity: Option<GitIdentity>,
    pub remotes: Vec<GitRemote>,
    pub has_github_remote: bool,
    pub head_commit_id: Option<String>,
    pub is_bare: bool,
    pub workdir: Option<PathBuf>,
    /// Performance metrics
    pub detection_time_ms: u64,
}

impl Default for GitContext {
    fn default() -> Self {
        Self {
            is_git_repo: false,
            repo_path: None,
            current_branch: None,
            available_branches: Vec::new(),
            staged_files: Vec::new(),
            modified_files: Vec::new(),
            untracked_files: Vec::new(),
            user_identity: None,
            remotes: Vec::new(),
            has_github_remote: false,
            head_commit_id: None,
            is_bare: false,
            workdir: None,
            detection_time_ms: 0,
        }
    }
}

/// Ultra-fast Git context detector using libgit2
pub struct GitContextDetector {
    /// Cache for single execution session
    context_cache: Option<GitContext>,
}

impl GitContextDetector {
    pub fn new() -> Self {
        Self {
            context_cache: None,
        }
    }

    /// Detect Git context with sub-100ms performance target
    pub fn detect_context(&mut self, path: Option<&Path>) -> Result<GitContext> {
        let start_time = Instant::now();
        
        // Use cached context if available (for single execution session)
        if let Some(ref cached) = self.context_cache {
            return Ok(cached.clone());
        }

        let search_path = path.unwrap_or_else(|| Path::new("."));
        let mut context = GitContext::default();

        // Try to discover and open repository
        let repo = match Repository::discover(search_path) {
            Ok(repo) => {
                context.is_git_repo = true;
                context.repo_path = Some(repo.path().to_path_buf());
                context.is_bare = repo.is_bare();
                context.workdir = repo.workdir().map(|p| p.to_path_buf());
                Some(repo)
            }
            Err(_) => {
                // Not a git repository
                context.detection_time_ms = start_time.elapsed().as_millis() as u64;
                return Ok(context);
            }
        };

        if let Some(repo) = repo {
            // Get current branch (fast operation)
            context.current_branch = self.get_current_branch(&repo)?;

            // Get all branches (optimized)
            context.available_branches = self.get_all_branches(&repo)?;

            // Get file status (staged, modified, untracked)
            let (staged, modified, untracked) = self.get_file_status(&repo)?;
            context.staged_files = staged;
            context.modified_files = modified;
            context.untracked_files = untracked;

            // Get user identity from Git config
            context.user_identity = self.get_user_identity(&repo)?;

            // Get remotes information
            context.remotes = self.get_remotes(&repo)?;
            context.has_github_remote = context.remotes.iter().any(|r| r.is_github);

            // Get HEAD commit ID
            context.head_commit_id = self.get_head_commit_id(&repo)?;
        }

        context.detection_time_ms = start_time.elapsed().as_millis() as u64;
        
        // Cache the context for this execution session
        self.context_cache = Some(context.clone());
        
        Ok(context)
    }

    /// Get current branch name
    fn get_current_branch(&self, repo: &Repository) -> Result<Option<String>> {
        match repo.head() {
            Ok(head) => {
                if let Some(name) = head.shorthand() {
                    Ok(Some(name.to_string()))
                } else {
                    Ok(None)
                }
            }
            Err(_) => Ok(None), // Detached HEAD or no commits
        }
    }

    /// Get all available branches (local and remote)
    fn get_all_branches(&self, repo: &Repository) -> Result<Vec<String>> {
        let mut branches = Vec::new();
        
        // Get local branches
        let local_branches = repo.branches(Some(BranchType::Local))?;
        for branch_result in local_branches {
            let (branch, _) = branch_result?;
            if let Some(name) = branch.name()? {
                branches.push(name.to_string());
            }
        }

        // Get remote branches
        let remote_branches = repo.branches(Some(BranchType::Remote))?;
        for branch_result in remote_branches {
            let (branch, _) = branch_result?;
            if let Some(name) = branch.name()? {
                branches.push(name.to_string());
            }
        }

        Ok(branches)
    }

    /// Get file status (staged, modified, untracked) efficiently
    fn get_file_status(&self, repo: &Repository) -> Result<(Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>)> {
        let mut staged = Vec::new();
        let mut modified = Vec::new();
        let mut untracked = Vec::new();

        let mut opts = StatusOptions::new();
        opts.include_untracked(true);
        opts.include_ignored(false);

        let statuses = repo.statuses(Some(&mut opts))?;

        for entry in statuses.iter() {
            let status = entry.status();
            let path = PathBuf::from(entry.path().unwrap_or(""));

            if status.contains(Status::INDEX_NEW) 
                || status.contains(Status::INDEX_MODIFIED) 
                || status.contains(Status::INDEX_DELETED) 
                || status.contains(Status::INDEX_RENAMED) 
                || status.contains(Status::INDEX_TYPECHANGE) {
                staged.push(path.clone());
            }

            if status.contains(Status::WT_MODIFIED) 
                || status.contains(Status::WT_DELETED) 
                || status.contains(Status::WT_TYPECHANGE) 
                || status.contains(Status::WT_RENAMED) {
                modified.push(path.clone());
            }

            if status.contains(Status::WT_NEW) {
                untracked.push(path);
            }
        }

        Ok((staged, modified, untracked))
    }

    /// Get user identity from Git configuration
    fn get_user_identity(&self, repo: &Repository) -> Result<Option<GitIdentity>> {
        let config = repo.config()?;
        
        let name = config.get_string("user.name").ok();
        let email = config.get_string("user.email").ok();

        match (name, email) {
            (Some(name), Some(email)) => Ok(Some(GitIdentity { name, email })),
            _ => Ok(None),
        }
    }

    /// Get remote information
    fn get_remotes(&self, repo: &Repository) -> Result<Vec<GitRemote>> {
        let mut remotes = Vec::new();
        let remote_names = repo.remotes()?;

        for name in remote_names.iter() {
            if let Some(name) = name {
                if let Ok(remote) = repo.find_remote(name) {
                    if let Some(url) = remote.url() {
                        let is_github = url.contains("github.com");
                        remotes.push(GitRemote {
                            name: name.to_string(),
                            url: url.to_string(),
                            is_github,
                        });
                    }
                }
            }
        }

        Ok(remotes)
    }

    /// Get HEAD commit ID
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

    /// Clear the context cache (useful for testing or when context might change)
    pub fn clear_cache(&mut self) {
        self.context_cache = None;
    }
}

impl GitContext {
    /// Check if there are any staged files
    pub fn has_staged_files(&self) -> bool {
        !self.staged_files.is_empty()
    }

    /// Check if there are any modified files
    pub fn has_modified_files(&self) -> bool {
        !self.modified_files.is_empty()
    }

    /// Check if there are any untracked files
    pub fn has_untracked_files(&self) -> bool {
        !self.untracked_files.is_empty()
    }

    /// Get the primary GitHub remote (usually 'origin')
    pub fn get_github_remote(&self) -> Option<&GitRemote> {
        self.remotes.iter().find(|r| r.is_github && r.name == "origin")
            .or_else(|| self.remotes.iter().find(|r| r.is_github))
    }

    /// Check if the repository is in a clean state (no staged, modified, or untracked files)
    pub fn is_clean(&self) -> bool {
        self.staged_files.is_empty() && self.modified_files.is_empty() && self.untracked_files.is_empty()
    }

    /// Get a summary of the repository state
    pub fn summary(&self) -> String {
        if !self.is_git_repo {
            return "Not a Git repository".to_string();
        }

        let branch = self.current_branch.as_deref().unwrap_or("(detached HEAD)");
        let staged_count = self.staged_files.len();
        let modified_count = self.modified_files.len();
        let untracked_count = self.untracked_files.len();

        format!(
            "Branch: {} | Staged: {} | Modified: {} | Untracked: {} | Detection: {}ms",
            branch, staged_count, modified_count, untracked_count, self.detection_time_ms
        )
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
    fn test_detect_non_git_directory() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let mut detector = GitContextDetector::new();
        
        let context = detector.detect_context(Some(temp_dir.path()))?;
        
        assert!(!context.is_git_repo);
        assert!(context.repo_path.is_none());
        assert!(context.current_branch.is_none());
        assert!(context.available_branches.is_empty());
        assert!(context.user_identity.is_none());
        assert!(context.remotes.is_empty());
        assert!(!context.has_github_remote);
        
        Ok(())
    }

    #[test]
    fn test_detect_empty_git_repo() -> Result<()> {
        let (_temp_dir, repo) = create_test_repo()?;
        let mut detector = GitContextDetector::new();
        
        let context = detector.detect_context(Some(repo.workdir().unwrap()))?;
        
        assert!(context.is_git_repo);
        assert!(context.repo_path.is_some());
        assert!(context.current_branch.is_none()); // No commits yet
        assert!(context.user_identity.is_some());
        assert_eq!(context.user_identity.unwrap().name, "Test User");
        assert!(context.remotes.is_empty());
        assert!(!context.has_github_remote);
        
        Ok(())
    }

    #[test]
    fn test_detect_repo_with_files() -> Result<()> {
        let (_temp_dir, repo) = create_test_repo()?;
        let workdir = repo.workdir().unwrap();
        
        // Create some files
        fs::write(workdir.join("modified.txt"), "original content")?;
        fs::write(workdir.join("untracked.txt"), "untracked content")?;
        
        // Create initial commit for modified.txt
        let mut index = repo.index()?;
        index.add_path(Path::new("modified.txt"))?;
        index.write()?;
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;
        let signature = Signature::now("Test User", "test@example.com")?;
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "Initial commit",
            &tree,
            &[],
        )?;
        
        // Modify the file and create a new staged file
        fs::write(workdir.join("modified.txt"), "modified content")?;
        fs::write(workdir.join("staged.txt"), "staged content")?;
        
        // Stage the new file
        let mut index = repo.index()?;
        index.add_path(Path::new("staged.txt"))?;
        index.write()?;
        
        let mut detector = GitContextDetector::new();
        let context = detector.detect_context(Some(workdir))?;
        
        assert!(context.is_git_repo);
        assert!(context.current_branch.is_some());
        // Git might use "main" as default branch in newer versions
        let branch_name = context.current_branch.as_ref().unwrap();
        assert!(branch_name == "master" || branch_name == "main");
        assert!(context.has_staged_files());
        assert!(context.has_modified_files());
        assert!(context.has_untracked_files());
        assert!(!context.is_clean());
        
        Ok(())
    }

    #[test]
    fn test_performance_target() -> Result<()> {
        let (_temp_dir, repo) = create_test_repo()?;
        let mut detector = GitContextDetector::new();
        
        let context = detector.detect_context(Some(repo.workdir().unwrap()))?;
        
        // Performance target: sub-100ms detection
        assert!(context.detection_time_ms < 100, 
                "Detection took {}ms, should be under 100ms", context.detection_time_ms);
        
        Ok(())
    }

    #[test]
    fn test_context_caching() -> Result<()> {
        let (_temp_dir, repo) = create_test_repo()?;
        let mut detector = GitContextDetector::new();
        let workdir = repo.workdir().unwrap();
        
        // First detection
        let context1 = detector.detect_context(Some(workdir))?;
        let _first_time = context1.detection_time_ms;
        
        // Second detection should use cache (much faster)
        let context2 = detector.detect_context(Some(workdir))?;
        
        // Should be the same context
        assert_eq!(context1.is_git_repo, context2.is_git_repo);
        assert_eq!(context1.repo_path, context2.repo_path);
        
        // Cache should make subsequent calls much faster
        // (though this might not always be true in tests due to timing variations)
        
        Ok(())
    }

    #[test]
    fn test_github_remote_detection() -> Result<()> {
        let (_temp_dir, repo) = create_test_repo()?;
        
        // Add a GitHub remote
        repo.remote("origin", "https://github.com/user/repo.git")?;
        repo.remote("upstream", "https://gitlab.com/user/repo.git")?;
        
        let mut detector = GitContextDetector::new();
        let context = detector.detect_context(Some(repo.workdir().unwrap()))?;
        
        assert!(context.has_github_remote);
        assert_eq!(context.remotes.len(), 2);
        
        let github_remote = context.get_github_remote();
        assert!(github_remote.is_some());
        assert_eq!(github_remote.unwrap().name, "origin");
        assert!(github_remote.unwrap().is_github);
        
        Ok(())
    }

    #[test]
    fn test_context_summary() -> Result<()> {
        let (_temp_dir, repo) = create_test_repo()?;
        let mut detector = GitContextDetector::new();
        
        let context = detector.detect_context(Some(repo.workdir().unwrap()))?;
        let summary = context.summary();
        
        assert!(summary.contains("Branch:"));
        assert!(summary.contains("Staged:"));
        assert!(summary.contains("Modified:"));
        assert!(summary.contains("Untracked:"));
        assert!(summary.contains("Detection:"));
        assert!(summary.contains("ms"));
        
        Ok(())
    }
}