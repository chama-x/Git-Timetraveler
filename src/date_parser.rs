use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Datelike, NaiveDate, TimeZone, Timelike, Utc};
use regex::Regex;
use std::collections::HashSet;

/// Represents different types of date input that can be parsed
#[derive(Debug, Clone, PartialEq)]
pub enum DateInput {
    /// Single year (e.g., "1990")
    Year(u32),
    /// Year and month (e.g., "1990-01" or "Jan 1990")
    YearMonth(u32, u32),
    /// Full date (e.g., "1990-01-01")
    FullDate(NaiveDate),
    /// Range of years (e.g., "1990-1995")
    Range(u32, u32),
    /// List of years (e.g., "1990,1992,1994")
    List(Vec<u32>),
}

/// Configuration for timestamp generation
#[derive(Debug, Clone)]
pub struct TimestampConfig {
    /// Default hour for commits (0-23)
    pub default_hour: u32,
    /// Whether to distribute commits across the day
    pub distribute_times: bool,
    /// Whether to ensure chronological ordering
    pub chronological_order: bool,
}

impl Default for TimestampConfig {
    fn default() -> Self {
        Self {
            default_hour: 18, // 6 PM
            distribute_times: true,
            chronological_order: true,
        }
    }
}

/// Parser for flexible date input formats
pub struct DateParser {
    year_regex: Regex,
    year_month_regex: Regex,
    full_date_regex: Regex,
    range_regex: Regex,
    month_names: Vec<(&'static str, u32)>,
}

impl DateParser {
    /// Create a new date parser with compiled regexes
    pub fn new() -> Result<Self> {
        let year_regex = Regex::new(r"^\s*(\d{4})\s*$")
            .context("Failed to compile year regex")?;
        
        let year_month_regex = Regex::new(r"^\s*(\d{4})-(\d{1,2})\s*$|^\s*([A-Za-z]{3,9})\s+(\d{4})\s*$")
            .context("Failed to compile year-month regex")?;
        
        let full_date_regex = Regex::new(r"^\s*(\d{4})-(\d{1,2})-(\d{1,2})\s*$")
            .context("Failed to compile full date regex")?;
        
        let range_regex = Regex::new(r"^\s*(\d{4})\s*-\s*(\d{4})\s*$")
            .context("Failed to compile range regex")?;

        let month_names = vec![
            ("january", 1), ("jan", 1),
            ("february", 2), ("feb", 2),
            ("march", 3), ("mar", 3),
            ("april", 4), ("apr", 4),
            ("may", 5),
            ("june", 6), ("jun", 6),
            ("july", 7), ("jul", 7),
            ("august", 8), ("aug", 8),
            ("september", 9), ("sep", 9), ("sept", 9),
            ("october", 10), ("oct", 10),
            ("november", 11), ("nov", 11),
            ("december", 12), ("dec", 12),
        ];

        Ok(Self {
            year_regex,
            year_month_regex,
            full_date_regex,
            range_regex,
            month_names,
        })
    }

    /// Parse a date input string into a DateInput enum
    pub fn parse(&self, input: &str) -> Result<DateInput> {
        let input = input.trim();
        
        if input.is_empty() {
            return Err(anyhow!("Date input cannot be empty"));
        }

        // Check for comma-separated list first
        if input.contains(',') {
            return self.parse_list(input);
        }

        // Try to parse as range
        if let Some(captures) = self.range_regex.captures(input) {
            let start_year: u32 = captures[1].parse()
                .context("Invalid start year in range")?;
            let end_year: u32 = captures[2].parse()
                .context("Invalid end year in range")?;
            
            self.validate_year_range(start_year, end_year)?;
            return Ok(DateInput::Range(start_year, end_year));
        }

        // Try to parse as full date
        if let Some(captures) = self.full_date_regex.captures(input) {
            let year: u32 = captures[1].parse()
                .context("Invalid year in full date")?;
            let month: u32 = captures[2].parse()
                .context("Invalid month in full date")?;
            let day: u32 = captures[3].parse()
                .context("Invalid day in full date")?;
            
            let date = self.create_naive_date(year, month, day)?;
            return Ok(DateInput::FullDate(date));
        }

        // Try to parse as year-month
        if let Some(captures) = self.year_month_regex.captures(input) {
            if let (Some(year_match), Some(month_match)) = (captures.get(1), captures.get(2)) {
                // Format: YYYY-MM
                let year: u32 = year_match.as_str().parse()
                    .context("Invalid year in year-month format")?;
                let month: u32 = month_match.as_str().parse()
                    .context("Invalid month in year-month format")?;
                
                self.validate_year(year)?;
                self.validate_month(month)?;
                return Ok(DateInput::YearMonth(year, month));
            } else if let (Some(month_match), Some(year_match)) = (captures.get(3), captures.get(4)) {
                // Format: Month YYYY
                let year: u32 = year_match.as_str().parse()
                    .context("Invalid year in month-year format")?;
                let month = self.parse_month_name(month_match.as_str())?;
                
                self.validate_year(year)?;
                return Ok(DateInput::YearMonth(year, month));
            }
        }

        // Try to parse as single year
        if let Some(captures) = self.year_regex.captures(input) {
            let year: u32 = captures[1].parse()
                .context("Invalid year format")?;
            
            self.validate_year(year)?;
            return Ok(DateInput::Year(year));
        }

        // If nothing matches, provide helpful error message
        Err(anyhow!(
            "Invalid date format: '{}'\n\nSupported formats:\n\
            • Single year: 1990\n\
            • Year-month: 1990-01 or Jan 1990\n\
            • Full date: 1990-01-01\n\
            • Year range: 1990-1995\n\
            • Year list: 1990,1992,1994",
            input
        ))
    }

    /// Parse a comma-separated list of years
    fn parse_list(&self, input: &str) -> Result<DateInput> {
        let mut years = Vec::new();
        let mut seen_years = HashSet::new();

        for part in input.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            // Each part should be a single year
            if let Some(captures) = self.year_regex.captures(part) {
                let year: u32 = captures[1].parse()
                    .with_context(|| format!("Invalid year in list: '{}'", part))?;
                
                self.validate_year(year)?;
                
                if seen_years.contains(&year) {
                    return Err(anyhow!("Duplicate year in list: {}", year));
                }
                
                seen_years.insert(year);
                years.push(year);
            } else {
                return Err(anyhow!(
                    "Invalid year format in list: '{}'\n\
                    Each item in a comma-separated list must be a 4-digit year (e.g., 1990,1992,1994)",
                    part
                ));
            }
        }

        if years.is_empty() {
            return Err(anyhow!("No valid years found in list"));
        }

        Ok(DateInput::List(years))
    }

    /// Parse month name to number
    fn parse_month_name(&self, month_str: &str) -> Result<u32> {
        let month_lower = month_str.to_lowercase();
        
        for (name, number) in &self.month_names {
            if *name == month_lower {
                return Ok(*number);
            }
        }

        Err(anyhow!(
            "Invalid month name: '{}'\n\
            Supported month names: Jan, Feb, Mar, Apr, May, Jun, Jul, Aug, Sep, Oct, Nov, Dec\n\
            (Full names like January, February, etc. are also supported)",
            month_str
        ))
    }

    /// Create a NaiveDate with validation
    fn create_naive_date(&self, year: u32, month: u32, day: u32) -> Result<NaiveDate> {
        self.validate_year(year)?;
        self.validate_month(month)?;
        self.validate_day(day)?;

        NaiveDate::from_ymd_opt(year as i32, month, day)
            .ok_or_else(|| anyhow!(
                "Invalid date: {}-{:02}-{:02}\n\
                Please check that the day is valid for the given month and year.",
                year, month, day
            ))
    }

    /// Validate year is in reasonable range
    fn validate_year(&self, year: u32) -> Result<()> {
        if year < 1970 || year > 2030 {
            return Err(anyhow!(
                "Year {} is out of range\n\
                Supported years: 1970-2030\n\
                (This range covers the typical Git timestamp range)",
                year
            ));
        }
        Ok(())
    }

    /// Validate month is 1-12
    fn validate_month(&self, month: u32) -> Result<()> {
        if month < 1 || month > 12 {
            return Err(anyhow!(
                "Month {} is invalid\n\
                Months must be between 1 and 12",
                month
            ));
        }
        Ok(())
    }

    /// Validate day is 1-31 (basic validation)
    fn validate_day(&self, day: u32) -> Result<()> {
        if day < 1 || day > 31 {
            return Err(anyhow!(
                "Day {} is invalid\n\
                Days must be between 1 and 31",
                day
            ));
        }
        Ok(())
    }

    /// Validate year range
    fn validate_year_range(&self, start_year: u32, end_year: u32) -> Result<()> {
        self.validate_year(start_year)?;
        self.validate_year(end_year)?;

        if start_year > end_year {
            return Err(anyhow!(
                "Invalid year range: {}-{}\n\
                Start year must be less than or equal to end year",
                start_year, end_year
            ));
        }

        let range_size = end_year - start_year + 1;
        if range_size > 50 {
            return Err(anyhow!(
                "Year range too large: {} years ({}-{})\n\
                Maximum supported range is 50 years to prevent excessive commits",
                range_size, start_year, end_year
            ));
        }

        Ok(())
    }
}

/// Generate timestamps for commits based on date input and configuration
pub fn generate_timestamps(date_input: &DateInput, config: &TimestampConfig) -> Result<Vec<DateTime<Utc>>> {
    match date_input {
        DateInput::Year(year) => {
            generate_year_timestamps(*year, config)
        }
        DateInput::YearMonth(year, month) => {
            generate_month_timestamps(*year, *month, config)
        }
        DateInput::FullDate(date) => {
            generate_single_timestamp(date, config)
        }
        DateInput::Range(start_year, end_year) => {
            generate_range_timestamps(*start_year, *end_year, config)
        }
        DateInput::List(years) => {
            generate_list_timestamps(years, config)
        }
    }
}

/// Generate timestamps for a single year
fn generate_year_timestamps(year: u32, config: &TimestampConfig) -> Result<Vec<DateTime<Utc>>> {
    let mut timestamps = Vec::new();
    
    // Generate 3-5 commits spread throughout the year
    let commit_count = 4;
    let days_in_year = if is_leap_year(year) { 366 } else { 365 };
    
    for i in 0..commit_count {
        let day_offset = (days_in_year / (commit_count + 1)) * (i + 1);
        let start_of_year = NaiveDate::from_ymd_opt(year as i32, 1, 1)
            .ok_or_else(|| anyhow!("Invalid year: {}", year))?;
        
        let target_date = start_of_year + chrono::Duration::days(day_offset.min(days_in_year - 1) as i64);
        
        let hour = if config.distribute_times {
            // Distribute across working hours (9 AM to 9 PM)
            9 + (i * 3) % 13
        } else {
            config.default_hour
        };
        
        let timestamp = create_timestamp(&target_date, hour as u32)?;
        timestamps.push(timestamp);
    }
    
    if config.chronological_order {
        timestamps.sort();
    }
    
    Ok(timestamps)
}

/// Generate timestamps for a specific month
fn generate_month_timestamps(year: u32, month: u32, config: &TimestampConfig) -> Result<Vec<DateTime<Utc>>> {
    let mut timestamps = Vec::new();
    
    // Generate 2-3 commits in the month
    let commit_count = 2;
    let days_in_month = days_in_month(year, month)?;
    
    for i in 0..commit_count {
        let day = ((days_in_month / (commit_count + 1)) * (i + 1)).max(1);
        let target_date = NaiveDate::from_ymd_opt(year as i32, month, day)
            .ok_or_else(|| anyhow!("Invalid date: {}-{:02}-{:02}", year, month, day))?;
        
        let hour = if config.distribute_times {
            config.default_hour + (i * 2) % 8 // Vary by a few hours
        } else {
            config.default_hour
        };
        
        let timestamp = create_timestamp(&target_date, hour)?;
        timestamps.push(timestamp);
    }
    
    if config.chronological_order {
        timestamps.sort();
    }
    
    Ok(timestamps)
}

/// Generate timestamp for a specific date
fn generate_single_timestamp(date: &NaiveDate, config: &TimestampConfig) -> Result<Vec<DateTime<Utc>>> {
    let timestamp = create_timestamp(date, config.default_hour)?;
    Ok(vec![timestamp])
}

/// Generate timestamps for a range of years
fn generate_range_timestamps(start_year: u32, end_year: u32, config: &TimestampConfig) -> Result<Vec<DateTime<Utc>>> {
    let mut all_timestamps = Vec::new();
    
    for year in start_year..=end_year {
        let year_timestamps = generate_year_timestamps(year, config)?;
        all_timestamps.extend(year_timestamps);
    }
    
    if config.chronological_order {
        all_timestamps.sort();
    }
    
    Ok(all_timestamps)
}

/// Generate timestamps for a list of years
fn generate_list_timestamps(years: &[u32], config: &TimestampConfig) -> Result<Vec<DateTime<Utc>>> {
    let mut all_timestamps = Vec::new();
    
    for &year in years {
        let year_timestamps = generate_year_timestamps(year, config)?;
        all_timestamps.extend(year_timestamps);
    }
    
    if config.chronological_order {
        all_timestamps.sort();
    }
    
    Ok(all_timestamps)
}

/// Create a UTC timestamp from date and hour
fn create_timestamp(date: &NaiveDate, hour: u32) -> Result<DateTime<Utc>> {
    let hour = hour.min(23); // Ensure hour is valid
    let minute = 0;
    let second = 0;
    
    let naive_datetime = date.and_hms_opt(hour, minute, second)
        .ok_or_else(|| anyhow!("Invalid time: {:02}:00:00", hour))?;
    
    Ok(Utc.from_utc_datetime(&naive_datetime))
}

/// Check if a year is a leap year
fn is_leap_year(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Get the number of days in a month
fn days_in_month(year: u32, month: u32) -> Result<u32> {
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => if is_leap_year(year) { 29 } else { 28 },
        _ => return Err(anyhow!("Invalid month: {}", month)),
    };
    Ok(days)
}

impl Default for DateParser {
    fn default() -> Self {
        Self::new().expect("Failed to create default DateParser")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_year() {
        let parser = DateParser::new().unwrap();
        
        assert_eq!(parser.parse("1990").unwrap(), DateInput::Year(1990));
        assert_eq!(parser.parse(" 2000 ").unwrap(), DateInput::Year(2000));
        
        // Invalid years
        assert!(parser.parse("1969").is_err());
        assert!(parser.parse("2031").is_err());
        assert!(parser.parse("abc").is_err());
    }

    #[test]
    fn test_parse_year_month() {
        let parser = DateParser::new().unwrap();
        
        assert_eq!(parser.parse("1990-01").unwrap(), DateInput::YearMonth(1990, 1));
        assert_eq!(parser.parse("Jan 1990").unwrap(), DateInput::YearMonth(1990, 1));
        assert_eq!(parser.parse("january 1990").unwrap(), DateInput::YearMonth(1990, 1));
        assert_eq!(parser.parse("Dec 2000").unwrap(), DateInput::YearMonth(2000, 12));
        
        // Invalid months
        assert!(parser.parse("1990-13").is_err());
        assert!(parser.parse("1990-00").is_err());
        assert!(parser.parse("Xyz 1990").is_err());
    }

    #[test]
    fn test_parse_full_date() {
        let parser = DateParser::new().unwrap();
        
        let expected_date = NaiveDate::from_ymd_opt(1990, 1, 1).unwrap();
        assert_eq!(parser.parse("1990-01-01").unwrap(), DateInput::FullDate(expected_date));
        
        // Invalid dates
        assert!(parser.parse("1990-02-30").is_err()); // February 30th doesn't exist
        assert!(parser.parse("1990-13-01").is_err()); // Month 13 doesn't exist
        assert!(parser.parse("1990-01-32").is_err()); // Day 32 doesn't exist
    }

    #[test]
    fn test_parse_range() {
        let parser = DateParser::new().unwrap();
        
        assert_eq!(parser.parse("1990-1995").unwrap(), DateInput::Range(1990, 1995));
        assert_eq!(parser.parse(" 2000 - 2005 ").unwrap(), DateInput::Range(2000, 2005));
        
        // Invalid ranges
        assert!(parser.parse("1995-1990").is_err()); // Start > end
        assert!(parser.parse("1970-2025").is_err()); // Range too large
    }

    #[test]
    fn test_parse_list() {
        let parser = DateParser::new().unwrap();
        
        assert_eq!(parser.parse("1990,1992,1994").unwrap(), DateInput::List(vec![1990, 1992, 1994]));
        assert_eq!(parser.parse(" 1990 , 1995 ").unwrap(), DateInput::List(vec![1990, 1995]));
        
        // Invalid lists
        assert!(parser.parse("1990,1990").is_err()); // Duplicate years
        assert!(parser.parse("1990,abc").is_err()); // Invalid year in list
        assert!(parser.parse(",").is_err()); // Empty list
    }

    #[test]
    fn test_error_messages() {
        let parser = DateParser::new().unwrap();
        
        let err = parser.parse("invalid").unwrap_err();
        assert!(err.to_string().contains("Supported formats"));
        
        let err = parser.parse("1969").unwrap_err();
        assert!(err.to_string().contains("out of range"));
        
        let err = parser.parse("1990-13").unwrap_err();
        assert!(err.to_string().contains("invalid"));
    }

    #[test]
    fn test_generate_year_timestamps() {
        let config = TimestampConfig::default();
        let date_input = DateInput::Year(1990);
        
        let timestamps = generate_timestamps(&date_input, &config).unwrap();
        assert_eq!(timestamps.len(), 4); // Should generate 4 commits for a year
        
        // All timestamps should be in 1990
        for timestamp in &timestamps {
            assert_eq!(timestamp.year(), 1990);
        }
        
        // Should be chronologically ordered
        for i in 1..timestamps.len() {
            assert!(timestamps[i] >= timestamps[i-1]);
        }
    }

    #[test]
    fn test_generate_month_timestamps() {
        let config = TimestampConfig::default();
        let date_input = DateInput::YearMonth(1990, 6);
        
        let timestamps = generate_timestamps(&date_input, &config).unwrap();
        assert_eq!(timestamps.len(), 2); // Should generate 2 commits for a month
        
        // All timestamps should be in June 1990
        for timestamp in &timestamps {
            assert_eq!(timestamp.year(), 1990);
            assert_eq!(timestamp.month(), 6);
        }
    }

    #[test]
    fn test_generate_single_timestamp() {
        let config = TimestampConfig::default();
        let date = NaiveDate::from_ymd_opt(1990, 1, 1).unwrap();
        let date_input = DateInput::FullDate(date);
        
        let timestamps = generate_timestamps(&date_input, &config).unwrap();
        assert_eq!(timestamps.len(), 1);
        
        let timestamp = &timestamps[0];
        assert_eq!(timestamp.year(), 1990);
        assert_eq!(timestamp.month(), 1);
        assert_eq!(timestamp.day(), 1);
        assert_eq!(timestamp.hour(), config.default_hour);
    }

    #[test]
    fn test_generate_range_timestamps() {
        let config = TimestampConfig::default();
        let date_input = DateInput::Range(1990, 1992);
        
        let timestamps = generate_timestamps(&date_input, &config).unwrap();
        assert_eq!(timestamps.len(), 12); // 4 commits per year * 3 years
        
        // Should span from 1990 to 1992
        assert_eq!(timestamps.first().unwrap().year(), 1990);
        assert_eq!(timestamps.last().unwrap().year(), 1992);
    }

    #[test]
    fn test_generate_list_timestamps() {
        let config = TimestampConfig::default();
        let date_input = DateInput::List(vec![1990, 1995]);
        
        let timestamps = generate_timestamps(&date_input, &config).unwrap();
        assert_eq!(timestamps.len(), 8); // 4 commits per year * 2 years
        
        // Should have commits in both 1990 and 1995
        let years: Vec<i32> = timestamps.iter().map(|t| t.year()).collect();
        assert!(years.contains(&1990));
        assert!(years.contains(&1995));
    }

    #[test]
    fn test_timestamp_distribution() {
        let mut config = TimestampConfig::default();
        config.distribute_times = true;
        
        let date_input = DateInput::Year(1990);
        let timestamps = generate_timestamps(&date_input, &config).unwrap();
        
        // Should have different hours when distribution is enabled
        let hours: Vec<u32> = timestamps.iter().map(|t| t.hour()).collect();
        let unique_hours: std::collections::HashSet<u32> = hours.into_iter().collect();
        assert!(unique_hours.len() > 1, "Should have different hours when distribution is enabled");
    }

    #[test]
    fn test_chronological_ordering() {
        let mut config = TimestampConfig::default();
        config.chronological_order = true;
        
        let date_input = DateInput::Range(1990, 1992);
        let timestamps = generate_timestamps(&date_input, &config).unwrap();
        
        // Should be in chronological order
        for i in 1..timestamps.len() {
            assert!(timestamps[i] >= timestamps[i-1], "Timestamps should be chronologically ordered");
        }
    }

    #[test]
    fn test_leap_year() {
        assert!(is_leap_year(2000)); // Divisible by 400
        assert!(!is_leap_year(1900)); // Divisible by 100 but not 400
        assert!(is_leap_year(2004)); // Divisible by 4 but not 100
        assert!(!is_leap_year(2001)); // Not divisible by 4
    }

    #[test]
    fn test_days_in_month() {
        assert_eq!(days_in_month(2000, 1).unwrap(), 31); // January
        assert_eq!(days_in_month(2000, 2).unwrap(), 29); // February (leap year)
        assert_eq!(days_in_month(2001, 2).unwrap(), 28); // February (non-leap year)
        assert_eq!(days_in_month(2000, 4).unwrap(), 30); // April
        assert!(days_in_month(2000, 13).is_err()); // Invalid month
    }
}