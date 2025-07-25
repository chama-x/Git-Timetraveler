# Design Document

## Overview

The redesigned Git Timetraveler embodies the Apple design philosophy: profound simplicity that reveals depth when needed. Like the best Apple products, it works beautifully for first-time users while offering sophisticated capabilities for power users. The interface learns from user behavior, suggests intelligent defaults, but always keeps the user in complete control.

## Architecture

### Core Design Philosophy (Inspired by Jony Ive's Principles)

1. **Simplicity First**: The primary interface is so simple it needs no explanation
2. **Progressive Disclosure**: Advanced features emerge naturally as users need them
3. **Intelligent Anticipation**: The tool learns user patterns and suggests next actions
4. **User Agency**: Suggestions are offered, never imposed - users remain in control
5. **Invisible Complexity**: Sophisticated operations feel effortless
6. **Contextual Awareness**: The tool adapts to the user's current environment and history

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    CLI Interface Layer                      │
├─────────────────────────────────────────────────────────────┤
│  Command Parser  │  Interactive UI  │  Progress Display    │
├─────────────────────────────────────────────────────────────┤
│                   Configuration Manager                     │
├─────────────────────────────────────────────────────────────┤
│   Git Context   │  GitHub Client   │   Repository Manager  │
├─────────────────────────────────────────────────────────────┤
│              Time Travel Engine (Core Logic)               │
├─────────────────────────────────────────────────────────────┤
│   Date Parser   │  Commit Builder  │   Safety Validator    │
└─────────────────────────────────────────────────────────────┘
```

## Components and Interfaces

### 1. CLI Interface Layer

**Command Structure:**
```bash
# Primary commands
git-timetraveler                    # Interactive mode with smart defaults
git-timetraveler commit <date>      # Commit staged files to specific date
git-timetraveler create <date>      # Create new historical activity
git-timetraveler config             # Manage configuration
git-timetraveler auth               # Manage GitHub authentication

# Quick operations
git-timetraveler 1990               # Shorthand for creating activity in 1990
git-timetraveler 1990-1995          # Create activity across range
git-timetraveler --as-me 1990       # Commit as current user identity
```

**Progressive Disclosure Interface:**

**Level 1 - First Time User (Absolute Simplicity):**
```
$ git-timetraveler

🕰️  When should this commit appear?
❯ 1990
```

**Level 2 - After First Use (Learned Preferences):**
```
$ git-timetraveler

🕰️  Git Time Traveler

Based on your history, you usually:
❯ Commit as yourself to 'my-project' repository

When should this appear?
❯ 1990 (like last time)
  1995 (you haven't used this year yet)
  Different year...
```

**Level 3 - Advanced User (Full Control):**
```
$ git-timetraveler

🕰️  Git Time Traveler

Quick actions based on your patterns:
❯ Commit staged files as John Doe to 1990
  Create new activity in my-project for 1990-1995
  
Or customize:
  📁 Different repository (current: my-project)
  👤 Different author (current: John Doe)
  📅 Different date pattern
  ⚙️  Advanced options
```

**Expert Mode (Command Line):**
```bash
# The tool remembers and suggests, but expert users can bypass everything
git-timetraveler 1990 --as-me --repo my-project --staged
```

### 2. Intelligent Preference Learning

**Learning Hierarchy (Apple-style Adaptive Interface):**
1. **Immediate Context**: What the user is doing right now
2. **Session Patterns**: What they've done in this session
3. **Historical Preferences**: Their established patterns over time
4. **Smart Defaults**: Sensible fallbacks that feel natural

**Preference Storage (Invisible to User):**
```toml
# ~/.config/timetraveler/learned_preferences.toml
[patterns]
most_used_years = ["1990", "1995", "2000"]
preferred_author_mode = "current_user"  # learned from choices
typical_time_of_day = "18:00"  # learned from patterns
favorite_repositories = ["my-project", "portfolio"]

[context_memory]
last_repository = "my-project"
last_branch = "main"
last_author_choice = "current_user"
recent_date_patterns = ["1990", "1990-1995"]

[suggestions]
# The tool learns what to suggest based on success patterns
suggest_repo_creation = true  # user said yes before
suggest_staged_files = true   # user often has staged files
```

**Progressive Disclosure in Configuration:**
- **Beginner**: No configuration needed, everything just works
- **Intermediate**: Gentle suggestions appear: "Would you like me to remember this choice?"
- **Advanced**: Full configuration access through `git-timetraveler config`
- **Expert**: Direct file editing and environment variable overrides

### 3. Contextual Intelligence Engine

**Multi-Layer Context Awareness:**
```rust
pub struct ContextualIntelligence {
    pub current_context: GitContext,
    pub user_patterns: UserPatterns,
    pub session_history: SessionHistory,
    pub environmental_hints: EnvironmentalHints,
}

pub struct GitContext {
    pub is_git_repo: bool,
    pub current_branch: Option<String>,
    pub available_branches: Vec<String>,
    pub staged_files: Vec<PathBuf>,
    pub user_identity: Option<GitIdentity>,
    pub remotes: Vec<GitRemote>,
    pub has_github_remote: bool,
}

pub struct UserPatterns {
    pub preferred_years: Vec<u32>,
    pub typical_author_choice: AuthorPreference,
    pub repository_preferences: HashMap<String, RepositoryPreference>,
    pub time_of_day_patterns: Vec<TimePattern>,
}

impl ContextualIntelligence {
    pub fn suggest_next_action(&self) -> ActionSuggestion {
        // Apple-style: anticipate what user wants to do
        // Based on context + patterns + current state
    }
    
    pub fn progressive_options(&self, user_experience_level: ExperienceLevel) -> Vec<Option> {
        // Show appropriate level of complexity
        match user_experience_level {
            ExperienceLevel::FirstTime => vec![most_obvious_action()],
            ExperienceLevel::Familiar => vec![learned_preferences(), alternatives()],
            ExperienceLevel::Advanced => vec![quick_actions(), customization_options()],
            ExperienceLevel::Expert => vec![all_options_with_shortcuts()],
        }
    }
}
```

**Contextual Suggestions (Like iOS Shortcuts):**
- **In Git Repo with Staged Files**: "Commit these 3 staged files to 1990?"
- **Empty Repository**: "Create your first historical commit for 1990?"
- **After Previous Session**: "Continue with 1991? (You did 1990 yesterday)"
- **Pattern Recognition**: "You usually do ranges - try 1990-1995?"

### 4. GitHub Integration

**GitHub Client Features:**
- Token validation and permission checking
- Repository creation with proper initialization
- Branch management and protection
- Secure credential storage using OS keychain

**Repository Management:**
```rust
pub struct GitHubClient {
    token: SecureString,
    client: reqwest::Client,
}

impl GitHubClient {
    pub async fn validate_token(&self) -> Result<TokenInfo>;
    pub async fn create_repository(&self, name: &str, options: CreateRepoOptions) -> Result<Repository>;
    pub async fn get_branches(&self, repo: &str) -> Result<Vec<Branch>>;
    pub async fn check_repository_exists(&self, repo: &str) -> Result<bool>;
}
```

### 5. Time Travel Engine

**Core Operations:**
```rust
pub struct TimeTravelEngine {
    git_context: GitContext,
    github_client: GitHubClient,
    config: Configuration,
}

impl TimeTravelEngine {
    pub async fn commit_staged_files(&self, target_date: DateTime, options: CommitOptions) -> Result<CommitResult>;
    pub async fn create_historical_activity(&self, date_range: DateRange, options: ActivityOptions) -> Result<ActivityResult>;
    pub async fn preview_operation(&self, operation: Operation) -> Result<OperationPreview>;
}
```

### 6. Date and Time Parsing

**Flexible Date Input:**
```rust
pub enum DateInput {
    Year(u32),                    // "1990"
    YearMonth(u32, u32),         // "1990-01" or "Jan 1990"
    FullDate(NaiveDate),         // "1990-01-01"
    Range(DateRange),            // "1990-1995"
    List(Vec<DateInput>),        // "1990,1992,1994"
}

pub struct DateParser;

impl DateParser {
    pub fn parse(input: &str) -> Result<DateInput>;
    pub fn generate_timestamps(input: DateInput, time_preference: TimePreference) -> Vec<DateTime>;
}
```

## Data Models

### User Experience Models

```rust
#[derive(Serialize, Deserialize)]
pub struct UserExperience {
    pub experience_level: ExperienceLevel,
    pub interaction_patterns: InteractionPatterns,
    pub preferences: LearnedPreferences,
    pub context_memory: ContextMemory,
}

#[derive(Serialize, Deserialize)]
pub enum ExperienceLevel {
    FirstTime,      // Show only the essential action
    Familiar,       // Show learned preferences + one alternative
    Advanced,       // Show quick actions + customization
    Expert,         // Show everything + shortcuts
}

#[derive(Serialize, Deserialize)]
pub struct InteractionPatterns {
    pub total_uses: u32,
    pub successful_patterns: Vec<ActionPattern>,
    pub preferred_interface_style: InterfaceStyle,
    pub typical_session_length: Duration,
}

#[derive(Serialize, Deserialize)]
pub struct LearnedPreferences {
    pub author_preference: AuthorPreference,
    pub repository_patterns: Vec<RepositoryPattern>,
    pub date_patterns: Vec<DatePattern>,
    pub workflow_preferences: WorkflowPreferences,
}

#[derive(Serialize, Deserialize)]
pub enum AuthorPreference {
    AlwaysCurrentUser,
    AlwaysTimeTraveler,
    ContextDependent,  // Different choice based on repo/situation
    AskEachTime,
}

#[derive(Serialize, Deserialize)]
pub struct WorkflowPreferences {
    pub prefers_staged_files: bool,
    pub likes_batch_operations: bool,
    pub wants_confirmations: bool,
    pub prefers_detailed_feedback: bool,
}
```

### Progressive Disclosure Models

```rust
pub struct ProgressiveInterface {
    pub current_level: ExperienceLevel,
    pub available_actions: Vec<Action>,
    pub suggested_actions: Vec<SuggestedAction>,
    pub hidden_advanced_options: Vec<AdvancedOption>,
}

pub struct SuggestedAction {
    pub action: Action,
    pub confidence: f32,  // How sure we are this is what they want
    pub reasoning: String,  // "Based on your last 3 sessions"
    pub one_click_executable: bool,
}

impl ProgressiveInterface {
    pub fn adapt_to_user(&mut self, user_experience: &UserExperience) {
        // Apple-style adaptation: interface evolves with user
        match user_experience.experience_level {
            ExperienceLevel::FirstTime => self.show_only_essential(),
            ExperienceLevel::Familiar => self.show_learned_preferences(),
            ExperienceLevel::Advanced => self.show_quick_actions_and_customization(),
            ExperienceLevel::Expert => self.show_everything_with_shortcuts(),
        }
    }
    
    pub fn suggest_next_level(&self) -> Option<ExperienceLevel> {
        // Gently suggest when user might be ready for more features
        // Like iOS suggesting new features after you've mastered basics
    }
}
```

### Operation Models

```rust
pub struct CommitOptions {
    pub author: AuthorChoice,
    pub message: Option<String>,
    pub repository: RepositoryTarget,
    pub branch: String,
    pub force: bool,
}

pub struct ActivityOptions {
    pub pattern: ActivityPattern,
    pub author: AuthorChoice,
    pub repository: RepositoryTarget,
    pub file_content: ContentStrategy,
}

pub enum ActivityPattern {
    Sparse,      // Few commits, spread out
    Moderate,    // Regular activity
    Heavy,       // Frequent commits
    Custom(Vec<DateTime>),
}

pub enum RepositoryTarget {
    Current,
    Named(String),
    CreateNew(String),
}
```

## Error Handling

### Error Categories

1. **User Input Errors**: Invalid dates, missing required information
2. **Authentication Errors**: Invalid tokens, insufficient permissions
3. **Git Errors**: Repository issues, merge conflicts, network problems
4. **GitHub API Errors**: Rate limiting, repository access issues

### Error Response Strategy

```rust
pub enum TimeTravelError {
    InvalidDate { input: String, suggestion: String },
    AuthenticationFailed { reason: String, help_url: String },
    RepositoryNotFound { name: String, create_suggestion: bool },
    InsufficientPermissions { required: Vec<String>, current: Vec<String> },
    GitOperationFailed { operation: String, details: String },
    NetworkError { retryable: bool, retry_after: Option<Duration> },
}

impl TimeTravelError {
    pub fn user_friendly_message(&self) -> String;
    pub fn suggested_actions(&self) -> Vec<String>;
    pub fn help_url(&self) -> Option<String>;
}
```

## Testing Strategy

### Unit Testing
- Date parsing logic with various input formats
- Configuration management and defaults
- Git context detection
- Error handling and user message generation

### Integration Testing
- GitHub API interactions with test repositories
- Git operations in temporary repositories
- End-to-end workflows with different scenarios

### User Experience Testing
- Interactive flow testing with different terminal environments
- Performance testing with large date ranges
- Error recovery and user guidance validation

## Security Considerations

### Token Management
- Use OS keychain for secure token storage
- Never log or display tokens in plain text
- Implement token validation before storage
- Provide clear token permission requirements

### Safe Operations
- Always validate repository permissions before operations
- Implement dry-run mode for all destructive operations
- Create backups or use safe branching for existing repositories
- Validate all user inputs to prevent injection attacks

### Privacy
- Minimize data collection and storage
- Clear documentation of what data is stored where
- Easy cleanup and reset options
- No telemetry or usage tracking without explicit consent

## Performance Optimization

### Efficient Git Operations
- Use libgit2 bindings for better performance than shell commands
- Implement parallel processing for bulk operations where safe
- Cache repository information to avoid repeated API calls
- Optimize network requests with proper batching

### User Experience Performance
- Lazy loading of configuration and context information
- Progressive disclosure of options to avoid overwhelming users
- Intelligent caching of GitHub API responses
- Responsive progress indicators for long-running operations

## Accessibility and Internationalization

### Terminal Compatibility
- Graceful degradation for terminals without color support
- Proper handling of different terminal sizes
- Screen reader compatibility for progress indicators
- Keyboard-only navigation for interactive modes

### Localization Readiness
- Externalized user-facing strings
- Date format localization
- Error message localization framework
- Cultural considerations for default behaviors