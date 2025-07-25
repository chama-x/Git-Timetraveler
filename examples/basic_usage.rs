use anyhow::Result;
use git_timetraveler::{create_time_traveled_repo, ProgressCallback, TimeTravelConfig};

struct SimpleProgress;

impl ProgressCallback for SimpleProgress {
    fn set_message(&self, message: &str) {
        println!("📋 {}", message);
    }

    fn increment(&self) {
        // Simple progress - just print dots
        print!(".");
    }

    fn finish(&self, message: &str) {
        println!("\n✅ {}", message);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Example: Create a repository for the year 1985
    let config = TimeTravelConfig::new(
        1985,                           // year
        10,                            // month (October)
        26,                            // day (Back to the Future Day!)
        9,                             // hour (9 AM)
        "your-github-username".to_string(),
        "your-github-token".to_string(),
    )?;

    println!("🚀 Creating time-traveled repository for {}", config.formatted_date());
    
    let progress = SimpleProgress;
    create_time_traveled_repo(&config, Some(&progress)).await?;

    println!("🎉 Repository created successfully!");
    println!("Check your profile: https://github.com/{}", config.username);

    Ok(())
} 