use anyhow::{Context, Result};
use reqwest::{Client, header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT}};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use crate::errors::{TimeTravelError, AuthError, RepoError, NetworkError};

/// GitHub API client for repository management
pub struct GitHubClient {
    client: Client,
    token: String,
    username: String,
}

/// Configuration for creating a new repository
#[derive(Debug, Clone, Serialize)]
pub struct CreateRepositoryRequest {
    pub name: String,
    pub description: Option<String>,
    pub private: bool,
    pub auto_init: bool,
    pub default_branch: Option<String>,
}

/// GitHub repository information
#[derive(Debug, Clone, Deserialize)]
pub struct Repository {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub private: bool,
    pub html_url: String,
    pub clone_url: String,
    pub ssh_url: String,
    pub default_branch: String,
    pub created_at: String,
    pub updated_at: String,
}

/// GitHub user information
#[derive(Debug, Clone, Deserialize)]
pub struct User {
    pub login: String,
    pub id: u64,
    pub name: Option<String>,
    pub email: Option<String>,
    pub public_repos: u32,
    pub followers: u32,
    pub following: u32,
}

/// GitHub branch information
#[derive(Debug, Clone, Deserialize)]
pub struct Branch {
    pub name: String,
    pub commit: BranchCommit,
    pub protected: bool,
}

/// Branch commit information
#[derive(Debug, Clone, Deserialize)]
pub struct BranchCommit {
    pub sha: String,
    pub url: String,
}

/// Token validation result
#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub valid: bool,
    pub scopes: Vec<String>,
    pub user: Option<User>,
    pub rate_limit_remaining: Option<u32>,
}

/// GitHub API error response
#[derive(Debug, Clone, Deserialize)]
pub struct GitHubError {
    pub message: String,
    pub documentation_url: Option<String>,
}

impl GitHubClient {
    /// Create a new GitHub client
    pub fn new(username: String, token: String) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("token {}", token))
                .context("Failed to create authorization header")?,
        );
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static("git-timetraveler/0.1.0"),
        );

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .default_headers(headers)
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            token,
            username,
        })
    }

    /// Validate the GitHub token and get user information
    pub async fn validate_token(&self) -> Result<TokenInfo> {
        let response = self
            .client
            .get("https://api.github.com/user")
            .send()
            .await
            .context("Failed to send token validation request")?;

        // Get rate limit info from headers
        let rate_limit_remaining = response
            .headers()
            .get("x-ratelimit-remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok());

        // Get scopes from headers
        let scopes = response
            .headers()
            .get("x-oauth-scopes")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.split(',').map(|scope| scope.trim().to_string()).collect())
            .unwrap_or_default();

        if response.status().is_success() {
            let user: User = response
                .json()
                .await
                .context("Failed to parse user response")?;

            Ok(TokenInfo {
                valid: true,
                scopes,
                user: Some(user),
                rate_limit_remaining,
            })
        } else {
            Ok(TokenInfo {
                valid: false,
                scopes,
                user: None,
                rate_limit_remaining,
            })
        }
    }

    /// Check if a repository exists
    pub async fn repository_exists(&self, repo_name: &str) -> Result<bool> {
        let url = format!("https://api.github.com/repos/{}/{}", self.username, repo_name);
        
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    TimeTravelError::network("GitHub API", NetworkError::Timeout, true)
                } else if e.is_connect() {
                    TimeTravelError::network("GitHub API", NetworkError::ConnectionFailed, true)
                } else {
                    TimeTravelError::network("GitHub API", NetworkError::InvalidResponse, false)
                }
            })
            .context("Failed to check repository existence")?;

        match response.status().as_u16() {
            200 => Ok(true),
            404 => Ok(false),
            401 => Err(TimeTravelError::authentication(
                AuthError::InvalidToken,
                "Authentication failed while checking repository"
            ).into()),
            403 => Err(TimeTravelError::repository(
                RepoError::AccessDenied,
                repo_name,
                "Access denied to repository"
            ).into()),
            429 => Err(TimeTravelError::network("GitHub API", NetworkError::RateLimited, true).into()),
            500..=599 => Err(TimeTravelError::network("GitHub API", NetworkError::ServiceUnavailable, true).into()),
            _ => Ok(false), // Treat other errors as "not found"
        }
    }

    /// Get repository information
    pub async fn get_repository(&self, repo_name: &str) -> Result<Repository> {
        let url = format!("https://api.github.com/repos/{}/{}", self.username, repo_name);
        
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to get repository information")?;

        if response.status().is_success() {
            let repo: Repository = response
                .json()
                .await
                .context("Failed to parse repository response")?;
            Ok(repo)
        } else {
            let error: GitHubError = response
                .json()
                .await
                .context("Failed to parse error response")?;
            anyhow::bail!("GitHub API error: {}", error.message);
        }
    }

    /// Create a new repository
    pub async fn create_repository(&self, request: &CreateRepositoryRequest) -> Result<Repository> {
        let url = "https://api.github.com/user/repos";
        
        let response = self
            .client
            .post(url)
            .json(request)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    TimeTravelError::network("GitHub API", NetworkError::Timeout, true)
                } else if e.is_connect() {
                    TimeTravelError::network("GitHub API", NetworkError::ConnectionFailed, true)
                } else {
                    TimeTravelError::network("GitHub API", NetworkError::InvalidResponse, false)
                }
            })
            .context("Failed to send repository creation request")?;

        if response.status().is_success() {
            let repo: Repository = response
                .json()
                .await
                .map_err(|_| TimeTravelError::network("GitHub API", NetworkError::InvalidResponse, false))
                .context("Failed to parse repository creation response")?;
            Ok(repo)
        } else {
            let status = response.status();
            
            match status.as_u16() {
                422 => {
                    Err(TimeTravelError::repository(
                        RepoError::AlreadyExists,
                        &request.name,
                        &format!("Repository '{}' already exists or name is invalid", request.name)
                    ).into())
                }
                401 => {
                    Err(TimeTravelError::authentication(
                        AuthError::InvalidToken,
                        "Authentication failed - check your GitHub token"
                    ).into())
                }
                403 => {
                    Err(TimeTravelError::authentication(
                        AuthError::InsufficientPermissions,
                        "Insufficient permissions to create repository"
                    ).into())
                }
                429 => {
                    Err(TimeTravelError::network("GitHub API", NetworkError::RateLimited, true).into())
                }
                500..=599 => {
                    Err(TimeTravelError::network("GitHub API", NetworkError::ServiceUnavailable, true).into())
                }
                _ => {
                    let error_text = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    Err(TimeTravelError::repository(
                        RepoError::InvalidName,
                        &request.name,
                        &format!("Failed to create repository: {} - {}", status, error_text)
                    ).into())
                }
            }
        }
    }

    /// List branches for a repository
    pub async fn list_branches(&self, repo_name: &str) -> Result<Vec<Branch>> {
        let url = format!("https://api.github.com/repos/{}/{}/branches", self.username, repo_name);
        
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to list branches")?;

        if response.status().is_success() {
            let branches: Vec<Branch> = response
                .json()
                .await
                .context("Failed to parse branches response")?;
            Ok(branches)
        } else {
            let error: GitHubError = response
                .json()
                .await
                .context("Failed to parse error response")?;
            anyhow::bail!("GitHub API error: {}", error.message);
        }
    }

    /// Delete a repository (use with caution)
    pub async fn delete_repository(&self, repo_name: &str) -> Result<()> {
        let url = format!("https://api.github.com/repos/{}/{}", self.username, repo_name);
        
        let response = self
            .client
            .delete(&url)
            .send()
            .await
            .context("Failed to delete repository")?;

        if response.status().is_success() {
            Ok(())
        } else {
            let error: GitHubError = response
                .json()
                .await
                .context("Failed to parse error response")?;
            anyhow::bail!("GitHub API error: {}", error.message);
        }
    }

    /// Check if the token has required permissions for repository operations
    pub async fn check_permissions(&self) -> Result<Vec<String>> {
        let token_info = self.validate_token().await
            .context("Failed to validate GitHub token")?;
        
        if !token_info.valid {
            return Err(TimeTravelError::authentication(
                AuthError::InvalidToken,
                "GitHub token validation failed"
            ).into());
        }

        let required_scopes = vec!["repo".to_string()];
        let missing_scopes: Vec<String> = required_scopes
            .into_iter()
            .filter(|scope| !token_info.scopes.contains(scope))
            .collect();

        if !missing_scopes.is_empty() {
            return Err(TimeTravelError::authentication(
                AuthError::InsufficientPermissions,
                &format!("Missing required permissions: {}", missing_scopes.join(", "))
            ).into());
        }

        Ok(token_info.scopes)
    }

    /// Get the authenticated user's information
    pub async fn get_user(&self) -> Result<User> {
        let token_info = self.validate_token().await?;
        
        if let Some(user) = token_info.user {
            Ok(user)
        } else {
            anyhow::bail!("Failed to get user information - invalid token");
        }
    }

    /// Create a repository with smart defaults
    pub async fn create_repository_with_defaults(
        &self,
        name: &str,
        description: Option<&str>,
        private: bool,
    ) -> Result<Repository> {
        let request = CreateRepositoryRequest {
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            private,
            auto_init: true, // Initialize with README
            default_branch: Some("main".to_string()),
        };

        self.create_repository(&request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require a valid GitHub token and should be run with caution
    // They are disabled by default to avoid accidental API calls

    #[tokio::test]
    #[ignore] // Remove this to run the test with a real token
    async fn test_validate_token() -> Result<()> {
        let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN not set");
        let username = std::env::var("GITHUB_USERNAME").expect("GITHUB_USERNAME not set");
        
        let client = GitHubClient::new(username, token)?;
        let token_info = client.validate_token().await?;
        
        assert!(token_info.valid);
        assert!(token_info.user.is_some());
        
        Ok(())
    }

    #[tokio::test]
    #[ignore] // Remove this to run the test with a real token
    async fn test_check_permissions() -> Result<()> {
        let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN not set");
        let username = std::env::var("GITHUB_USERNAME").expect("GITHUB_USERNAME not set");
        
        let client = GitHubClient::new(username, token)?;
        let scopes = client.check_permissions().await?;
        
        assert!(scopes.contains(&"repo".to_string()));
        
        Ok(())
    }

    #[test]
    fn test_create_repository_request() {
        let request = CreateRepositoryRequest {
            name: "test-repo".to_string(),
            description: Some("Test repository".to_string()),
            private: false,
            auto_init: true,
            default_branch: Some("main".to_string()),
        };

        assert_eq!(request.name, "test-repo");
        assert_eq!(request.description, Some("Test repository".to_string()));
        assert!(!request.private);
        assert!(request.auto_init);
        assert_eq!(request.default_branch, Some("main".to_string()));
    }
}