use anyhow::Result;
use git_timetraveler::{DateInput, DateParser, TimestampConfig, generate_timestamps};

fn main() -> Result<()> {
    println!("Git Time Traveler - Date Parsing Examples\n");

    let parser = DateParser::new()?;
    let config = TimestampConfig::default();

    // Example 1: Single year
    println!("Example 1: Single year");
    let date_input = parser.parse("1990")?;
    let timestamps = generate_timestamps(&date_input, &config)?;
    println!("Input: '1990'");
    println!("Generated {} timestamps:", timestamps.len());
    for (i, timestamp) in timestamps.iter().enumerate() {
        println!("  {}. {}", i + 1, timestamp.format("%Y-%m-%d %H:%M:%S UTC"));
    }
    println!();

    // Example 2: Year-month
    println!("Example 2: Year-month");
    let date_input = parser.parse("Jan 1990")?;
    let timestamps = generate_timestamps(&date_input, &config)?;
    println!("Input: 'Jan 1990'");
    println!("Generated {} timestamps:", timestamps.len());
    for (i, timestamp) in timestamps.iter().enumerate() {
        println!("  {}. {}", i + 1, timestamp.format("%Y-%m-%d %H:%M:%S UTC"));
    }
    println!();

    // Example 3: Full date
    println!("Example 3: Full date");
    let date_input = parser.parse("1990-01-01")?;
    let timestamps = generate_timestamps(&date_input, &config)?;
    println!("Input: '1990-01-01'");
    println!("Generated {} timestamps:", timestamps.len());
    for (i, timestamp) in timestamps.iter().enumerate() {
        println!("  {}. {}", i + 1, timestamp.format("%Y-%m-%d %H:%M:%S UTC"));
    }
    println!();

    // Example 4: Year range
    println!("Example 4: Year range");
    let date_input = parser.parse("1990-1992")?;
    let timestamps = generate_timestamps(&date_input, &config)?;
    println!("Input: '1990-1992'");
    println!("Generated {} timestamps:", timestamps.len());
    for (i, timestamp) in timestamps.iter().take(6).enumerate() {
        println!("  {}. {}", i + 1, timestamp.format("%Y-%m-%d %H:%M:%S UTC"));
    }
    if timestamps.len() > 6 {
        println!("  ... and {} more", timestamps.len() - 6);
    }
    println!();

    // Example 5: Year list
    println!("Example 5: Year list");
    let date_input = parser.parse("1990,1995,2000")?;
    let timestamps = generate_timestamps(&date_input, &config)?;
    println!("Input: '1990,1995,2000'");
    println!("Generated {} timestamps:", timestamps.len());
    for (i, timestamp) in timestamps.iter().take(8).enumerate() {
        println!("  {}. {}", i + 1, timestamp.format("%Y-%m-%d %H:%M:%S UTC"));
    }
    if timestamps.len() > 8 {
        println!("  ... and {} more", timestamps.len() - 8);
    }
    println!();

    // Example 6: Custom timestamp configuration
    println!("Example 6: Custom timestamp configuration");
    let mut custom_config = TimestampConfig::default();
    custom_config.default_hour = 9; // 9 AM
    custom_config.distribute_times = false; // All at same time
    custom_config.chronological_order = true;

    let date_input = parser.parse("1990")?;
    let timestamps = generate_timestamps(&date_input, &custom_config)?;
    println!("Input: '1990' with custom config (9 AM, no time distribution)");
    println!("Generated {} timestamps:", timestamps.len());
    for (i, timestamp) in timestamps.iter().enumerate() {
        println!("  {}. {}", i + 1, timestamp.format("%Y-%m-%d %H:%M:%S UTC"));
    }
    println!();

    // Example 7: Error handling
    println!("Example 7: Error handling");
    let invalid_inputs = vec![
        "1969",        // Year out of range
        "1990-13",     // Invalid month
        "1990-02-30",  // Invalid date
        "1995-1990",   // Invalid range
        "invalid",     // Invalid format
    ];

    for input in invalid_inputs {
        match parser.parse(input) {
            Ok(_) => println!("Input '{}': Parsed successfully (unexpected)", input),
            Err(e) => println!("Input '{}': Error - {}", input, e.to_string().lines().next().unwrap_or("Unknown error")),
        }
    }

    Ok(())
}