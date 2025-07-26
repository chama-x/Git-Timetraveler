use anyhow::Result;
use colored::*;
use dialoguer::{theme::ColorfulTheme, Confirm};
use std::collections::HashMap;
use crate::{TimeTravelConfig, GitIdentity};

/// Represents an operation that would be performed during time travel
#[derive(Debug, Clone)]
pub enum PlannedOperation {
    /// Validate GitHub token and permissions
    ValidateToken {
        username: String,
    },
    /// Check if repository exists
    CheckRepository {
        repository: String,
        username: String,
    },
    /// Create a new repository
    CreateRepository {
        repository: String,
        description: String,
        private: bool,
    },
    /// Clone repository to local temporary directory
    CloneRepository {
        repository: String,
        branch: String,
        url: String,
    },
    /// Create time travel file
    CreateFile {
        filename: String,
        content_preview: String,
    },
    /// Create backdated commit
    CreateCommit {
        year: u32,
        timestamp: String,
        author: GitIdentity,
        message: String,
        files: Vec<String>,
    },
    /// Push commit to remote repository
    PushCommit {
        repository: String,
        branch: String,
        force: bool,
    },
    /// Clean up temporary files
    Cleanup {
        temp_path: String,
    },
}

/// Dry run execution plan with detailed operation breakdown
#[derive(Debug, Clone)]
pub struct DryRunPlan {
    pub operations: Vec<PlannedOperation>,
    pub summary: DryRunSummary,
    pub risks: Vec<String>,
    pub confirmations_needed: Vec<String>,
}

/// Summary of what would be done in the dry run
#[derive(Debug, Clone)]
pub struct DryRunSummary {
    pub total_operations: usize,
    pub years_to_process: Vec<u32>,
    pub repositories_affected: Vec<String>,
    pub files_to_create: Vec<String>,
    pub commits_to_create: usize,
    pub estimated_duration: std::time::Duration,
}

/// Configuration for dry run and confirmation behavior
#[derive(Debug, Clone)]
pub struct DryRunConfig {
    pub show_detailed_operations: bool,
    pub show_file_previews: bool,
    pub show_risks: bool,
    pub require_confirmation: bool,
    pub interactive_confirmations: bool,
}

impl Default for DryRunConfig {
    fn default() -> Self {
        Self {
            show_detailed_operations: true,
            show_file_previews: true,
            show_risks: true,
            require_confirmation: true,
            interactive_confirmations: true,
        }
    }
}

/// Dry run executor that analyzes and displays planned operations
pub struct DryRunExecutor {
    config: DryRunConfig,
}

impl DryRunExecutor {
    /// Create a new dry run executor
    pub fn new(config: DryRunConfig) -> Self {
        Self { config }
    }

    /// Create a dry run plan for multiple years
    pub fn create_plan(&self, configs: &[TimeTravelConfig]) -> Result<DryRunPlan> {
        let mut operations = Vec::new();
        let mut repositories = std::collections::HashSet::new();
        let mut files_to_create = Vec::new();

        // Add initial validation operations
        if let Some(first_config) = configs.first() {
            operations.push(PlannedOperation::ValidateToken {
                username: first_config.username.clone(),
            });
        }

        // Group operations by repository
        let mut repo_configs: HashMap<String, Vec<&TimeTravelConfig>> = HashMap::new();
        for config in configs {
            let repo_name = config.repo_name();
            repo_configs.entry(repo_name).or_default().push(config);
        }

        // Process each repository
        for (repo_name, repo_configs_list) in repo_configs {
            repositories.insert(repo_name.clone());
            
            // Check repository existence
            operations.push(PlannedOperation::CheckRepository {
                repository: repo_name.clone(),
                username: repo_configs_list[0].username.clone(),
            });

            // Clone repository
            operations.push(PlannedOperation::CloneRepository {
                repository: repo_name.clone(),
                branch: repo_configs_list[0].branch.clone(),
                url: format!("https://github.com/{}/{}.git", 
                    repo_configs_list[0].username, repo_name),
            });

            // Process each year for this repository
            for config in repo_configs_list {
                let filename = format!("timetravel-{}.md", config.year);
                files_to_create.push(filename.clone());

                // Create file
                operations.push(PlannedOperation::CreateFile {
                    filename: filename.clone(),
                    content_preview: self.generate_file_preview(config),
                });

                // Create commit
                let author = config.author.clone().unwrap_or_else(|| GitIdentity {
                    name: "Git Time Traveler".to_string(),
                    email: "timetraveler@example.com".to_string(),
                });

                operations.push(PlannedOperation::CreateCommit {
                    year: config.year,
                    timestamp: config.commit_timestamp()?,
                    author,
                    message: format!("Time travel commit for {}", config.year),
                    files: vec![filename],
                });

                // Push commit
                operations.push(PlannedOperation::PushCommit {
                    repository: repo_name.clone(),
                    branch: config.branch.clone(),
                    force: false, // This would be determined by actual config
                });
            }
        }

        // Add cleanup operation
        operations.push(PlannedOperation::Cleanup {
            temp_path: "/tmp/git-timetraveler-*".to_string(),
        });

        // Create summary
        let years: Vec<u32> = configs.iter().map(|c| c.year).collect();
        let summary = DryRunSummary {
            total_operations: operations.len(),
            years_to_process: years.clone(),
            repositories_affected: repositories.into_iter().collect(),
            files_to_create,
            commits_to_create: configs.len(),
            estimated_duration: std::time::Duration::from_secs((configs.len() as u64) * 10), // Rough estimate
        };

        // Identify risks
        let risks = self.identify_risks(configs, &operations);
        let confirmations_needed = self.identify_confirmations_needed(configs, &operations);

        Ok(DryRunPlan {
            operations,
            summary,
            risks,
            confirmations_needed,
        })
    }

    /// Display the dry run plan to the user
    pub fn display_plan(&self, plan: &DryRunPlan) -> Result<()> {
        self.display_header();
        self.display_summary(&plan.summary);
        
        if self.config.show_risks && !plan.risks.is_empty() {
            self.display_risks(&plan.risks);
        }

        if !plan.confirmations_needed.is_empty() {
            self.display_confirmations_needed(&plan.confirmations_needed);
        }

        if self.config.show_detailed_operations {
            self.display_operations(&plan.operations);
        }

        Ok(())
    }

    /// Ask for user confirmation before proceeding
    pub fn confirm_execution(&self, plan: &DryRunPlan) -> Result<bool> {
        if !self.config.require_confirmation {
            return Ok(true);
        }

        if !self.config.interactive_confirmations {
            println!("\n{}", "⚠️  Confirmation required but running in non-interactive mode".yellow());
            println!("{}", "Use --yes flag to skip confirmations or run interactively".cyan());
            return Ok(false);
        }

        println!("\n{}", "⚠️  Confirmation Required".yellow().bold());
        
        // Show critical warnings
        if !plan.risks.is_empty() {
            println!("\n{}", "🚨 Potential Risks:".red().bold());
            for risk in &plan.risks {
                println!("  {} {}", "•".red(), risk.yellow());
            }
        }

        // Ask for confirmation
        let confirm = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Do you want to proceed with these operations?")
            .default(false)
            .interact()?;

        if confirm {
            println!("\n{} {}", "✅".green(), "Proceeding with time travel operations...".green());
        } else {
            println!("\n{} {}", "❌".red(), "Operation cancelled by user".red());
        }

        Ok(confirm)
    }

    /// Generate a preview of file content
    fn generate_file_preview(&self, config: &TimeTravelConfig) -> String {
        let content = format!(
            "# Time Travel Commit for {}\n\n\
            This file was created to show activity in the year {} on my GitHub profile.\n\n\
            Repository: {}\n\
            Generated: [timestamp]\n\n\
            ## About Time Travel Commits\n\n\
            This commit was backdated to create historical activity on GitHub.\n\
            The actual creation time may differ from the commit timestamp.\n",
            config.year,
            config.year,
            config.repo_name()
        );

        // Return first few lines for preview
        content.lines().take(3).collect::<Vec<_>>().join("\n") + "\n..."
    }

    /// Identify potential risks in the operation
    fn identify_risks(&self, configs: &[TimeTravelConfig], operations: &[PlannedOperation]) -> Vec<String> {
        let mut risks = Vec::new();

        // Check for force push operations
        if operations.iter().any(|op| matches!(op, PlannedOperation::PushCommit { force: true, .. })) {
            risks.push("Force push will overwrite remote history - this cannot be undone".to_string());
        }

        // Check for multiple years (potential rate limiting)
        if configs.len() > 10 {
            risks.push(format!("Processing {} years may trigger GitHub rate limits", configs.len()));
        }

        // Check for very old years
        let old_years: Vec<u32> = configs.iter()
            .map(|c| c.year)
            .filter(|&year| year < 1990)
            .collect();
        if !old_years.is_empty() {
            risks.push(format!("Very old years ({:?}) may look suspicious on your profile", old_years));
        }

        // Check for repository creation
        if operations.iter().any(|op| matches!(op, PlannedOperation::CreateRepository { .. })) {
            risks.push("New repositories will be created on your GitHub account".to_string());
        }

        risks
    }

    /// Identify operations that need explicit confirmation
    fn identify_confirmations_needed(&self, configs: &[TimeTravelConfig], operations: &[PlannedOperation]) -> Vec<String> {
        let mut confirmations = Vec::new();

        // Repository creation
        let repo_creations: Vec<_> = operations.iter()
            .filter_map(|op| match op {
                PlannedOperation::CreateRepository { repository, .. } => Some(repository.clone()),
                _ => None,
            })
            .collect();
        
        if !repo_creations.is_empty() {
            confirmations.push(format!("Create {} new repositories: {}", 
                repo_creations.len(), 
                repo_creations.join(", ")));
        }

        // Multiple years processing
        if configs.len() > 5 {
            confirmations.push(format!("Process {} years of commits", configs.len()));
        }

        // Force push operations
        let force_pushes: Vec<_> = operations.iter()
            .filter_map(|op| match op {
                PlannedOperation::PushCommit { repository, force: true, .. } => Some(repository.clone()),
                _ => None,
            })
            .collect();
        
        if !force_pushes.is_empty() {
            confirmations.push(format!("Force push to repositories: {}", force_pushes.join(", ")));
        }

        confirmations
    }

    /// Display the dry run header
    fn display_header(&self) {
        println!("{}", "🔍 Dry Run Mode - Preview of Operations".bright_blue().bold());
        println!("{}", "No changes will be made to your repositories".dimmed());
        println!();
    }

    /// Display the operation summary
    fn display_summary(&self, summary: &DryRunSummary) {
        println!("{}", "📋 Operation Summary".bright_yellow().bold());
        println!("  {} {}", "Total Operations:".cyan(), summary.total_operations.to_string().bright_white());
        println!("  {} {}", "Years to Process:".cyan(), 
            if summary.years_to_process.len() == 1 {
                summary.years_to_process[0].to_string()
            } else {
                format!("{} years ({}-{})", 
                    summary.years_to_process.len(),
                    summary.years_to_process.iter().min().unwrap(),
                    summary.years_to_process.iter().max().unwrap())
            }.bright_magenta());
        println!("  {} {}", "Repositories:".cyan(), summary.repositories_affected.join(", ").bright_green());
        println!("  {} {}", "Files to Create:".cyan(), summary.files_to_create.len().to_string().bright_white());
        println!("  {} {}", "Commits to Create:".cyan(), summary.commits_to_create.to_string().bright_white());
        println!("  {} ~{} seconds", "Estimated Duration:".cyan(), summary.estimated_duration.as_secs().to_string().bright_white());
        println!();
    }

    /// Display identified risks
    fn display_risks(&self, risks: &[String]) {
        println!("{}", "⚠️  Potential Risks".yellow().bold());
        for risk in risks {
            println!("  {} {}", "•".yellow(), risk.red());
        }
        println!();
    }

    /// Display confirmations needed
    fn display_confirmations_needed(&self, confirmations: &[String]) {
        println!("{}", "✋ Confirmations Needed".bright_yellow().bold());
        for confirmation in confirmations {
            println!("  {} {}", "•".yellow(), confirmation.cyan());
        }
        println!();
    }

    /// Display detailed operations
    fn display_operations(&self, operations: &[PlannedOperation]) {
        println!("{}", "🔧 Detailed Operations".bright_blue().bold());
        
        for (i, operation) in operations.iter().enumerate() {
            let step_num = format!("{}.", i + 1);
            match operation {
                PlannedOperation::ValidateToken { username } => {
                    println!("  {} {} Validate GitHub token for user '{}'", 
                        step_num.bright_white(), "🔑".blue(), username.bright_green());
                }
                PlannedOperation::CheckRepository { repository, username } => {
                    println!("  {} {} Check if repository '{}/{}' exists", 
                        step_num.bright_white(), "🔍".blue(), username.dimmed(), repository.bright_green());
                }
                PlannedOperation::CreateRepository { repository, description, private } => {
                    println!("  {} {} Create {} repository '{}' with description: '{}'", 
                        step_num.bright_white(), "📁".green(), 
                        if *private { "private" } else { "public" },
                        repository.bright_green(), description.dimmed());
                }
                PlannedOperation::CloneRepository { repository, branch, url } => {
                    println!("  {} {} Clone repository '{}' (branch: {}) from {}", 
                        step_num.bright_white(), "⬇️".blue(), repository.bright_green(), 
                        branch.bright_cyan(), url.dimmed());
                }
                PlannedOperation::CreateFile { filename, content_preview } => {
                    println!("  {} {} Create file '{}'", 
                        step_num.bright_white(), "📄".green(), filename.bright_yellow());
                    if self.config.show_file_previews {
                        for line in content_preview.lines() {
                            println!("      {}", line.dimmed());
                        }
                    }
                }
                PlannedOperation::CreateCommit { year, timestamp, author, message, files } => {
                    println!("  {} {} Create backdated commit for year {}", 
                        step_num.bright_white(), "💾".green(), year.to_string().bright_yellow());
                    println!("      {} {}", "Timestamp:".dimmed(), timestamp.bright_white());
                    println!("      {} {} <{}>", "Author:".dimmed(), author.name.bright_cyan(), author.email.dimmed());
                    println!("      {} {}", "Message:".dimmed(), message.bright_white());
                    println!("      {} {}", "Files:".dimmed(), files.join(", ").bright_yellow());
                }
                PlannedOperation::PushCommit { repository, branch, force } => {
                    println!("  {} {} {} push to '{}/{}' (branch: {})", 
                        step_num.bright_white(), "⬆️".blue(),
                        if *force { "Force".red() } else { "Push".green() },
                        repository.bright_green(), branch.bright_cyan(), branch.bright_cyan());
                }
                PlannedOperation::Cleanup { temp_path } => {
                    println!("  {} {} Clean up temporary files: {}", 
                        step_num.bright_white(), "🧹".yellow(), temp_path.dimmed());
                }
            }
        }
        println!();
    }
}

/// Helper function to create a dry run plan for a single configuration
pub fn create_single_config_plan(config: &TimeTravelConfig, dry_run_config: DryRunConfig) -> Result<DryRunPlan> {
    let executor = DryRunExecutor::new(dry_run_config);
    executor.create_plan(&[config.clone()])
}

/// Helper function to display and confirm a dry run for multiple configurations
pub fn display_and_confirm_dry_run(configs: &[TimeTravelConfig], interactive: bool) -> Result<bool> {
    let dry_run_config = DryRunConfig {
        show_detailed_operations: true,
        show_file_previews: false, // Keep it concise for multiple configs
        show_risks: true,
        require_confirmation: true,
        interactive_confirmations: interactive,
    };

    let executor = DryRunExecutor::new(dry_run_config);
    let plan = executor.create_plan(configs)?;
    
    executor.display_plan(&plan)?;
    executor.confirm_execution(&plan)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> TimeTravelConfig {
        TimeTravelConfig::new(
            1990, 1, 1, 18,
            "testuser".to_string(),
            "ghp_test123".to_string(),
            Some("testrepo".to_string()),
            "main".to_string(),
            None,
        ).unwrap()
    }

    #[test]
    fn test_create_plan() {
        let config = create_test_config();
        let dry_run_config = DryRunConfig::default();
        let executor = DryRunExecutor::new(dry_run_config);
        
        let plan = executor.create_plan(&[config]).unwrap();
        
        assert!(!plan.operations.is_empty());
        assert_eq!(plan.summary.years_to_process, vec![1990]);
        assert_eq!(plan.summary.commits_to_create, 1);
    }

    #[test]
    fn test_identify_risks() {
        let config = create_test_config();
        let dry_run_config = DryRunConfig::default();
        let executor = DryRunExecutor::new(dry_run_config);
        
        let operations = vec![
            PlannedOperation::PushCommit {
                repository: "test".to_string(),
                branch: "main".to_string(),
                force: true,
            }
        ];
        
        let risks = executor.identify_risks(&[config], &operations);
        assert!(!risks.is_empty());
        assert!(risks[0].contains("Force push"));
    }

    #[test]
    fn test_file_preview_generation() {
        let config = create_test_config();
        let dry_run_config = DryRunConfig::default();
        let executor = DryRunExecutor::new(dry_run_config);
        
        let preview = executor.generate_file_preview(&config);
        assert!(preview.contains("1990"));
        assert!(preview.contains("testrepo"));
        assert!(preview.contains("..."));
    }
}