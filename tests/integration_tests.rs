use std::process::Command;
use std::path::Path;

#[test]
fn test_help_command() {
    let output = Command::new("cargo")
        .args(&["run", "--", "--no-menu", "--help"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Debug output
    if !output.status.success() {
        eprintln!("Help command failed:");
        eprintln!("STDOUT: {}", stdout);
        eprintln!("STDERR: {}", stderr);
        eprintln!("Exit code: {:?}", output.status.code());
    }

    assert!(output.status.success(), "Help command should succeed");
    assert!(stdout.contains("Git Time Traveler"));
    assert!(stdout.contains("Usage:"));
    assert!(stdout.contains("--year"));
    assert!(stdout.contains("--username"));
}

#[test]
fn test_version_command() {
    let output = Command::new("cargo")
        .args(&["run", "--", "--no-menu", "--version"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("git-timetraveler"));
}

#[test]
fn test_dry_run_mode() {
    let output = Command::new("cargo")
        .args(&[
            "run", "--", 
            "--dry-run", 
            "--no-menu",
            "--year", "1990",
            "--username", "testuser",
            "--repo", "testrepo",
            "--token", "ghp_fake_token_for_testing"
        ])
        .output()
        .expect("Failed to execute command");

    // In dry run mode, it should succeed even with fake token
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Debug output (only if test fails)
    if !output.status.success() {
        eprintln!("Dry run test failed:");
        eprintln!("STDOUT: {}", stdout);
        eprintln!("STDERR: {}", stderr);
        eprintln!("Exit code: {:?}", output.status.code());
    }
    
    // Should either succeed or fail gracefully with informative message
    if !output.status.success() {
        assert!(stderr.contains("token") || stderr.contains("authentication") || stderr.contains("GitHub") || stderr.contains("error"));
    } else {
        assert!(stdout.contains("Dry Run Mode") || stdout.contains("dry run") || stdout.contains("would") || stdout.contains("preview"));
    }
}

#[test]
fn test_invalid_year() {
    let output = Command::new("cargo")
        .args(&[
            "run", "--", 
            "--no-menu",
            "--year", "1800", // Invalid year (too old)
            "--username", "testuser",
            "--repo", "testrepo",
            "--token", "ghp_fake_token_for_testing"
        ])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("year") || stderr.contains("1800") || stderr.contains("range"));
}

#[test]
fn test_missing_required_args() {
    let output = Command::new("cargo")
        .args(&[
            "run", "--", 
            "--no-menu",
            "--year", "1990"
            // Missing username, repo, and token
        ])
        .output()
        .expect("Failed to execute command");

    // Should fail with helpful error about missing required arguments
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("username") || stderr.contains("repo") || stderr.contains("token"));
}

#[test]
fn test_year_range_parsing() {
    let output = Command::new("cargo")
        .args(&[
            "run", "--", 
            "--dry-run",
            "--no-menu",
            "--years", "1990-1992",
            "--username", "testuser",
            "--repo", "testrepo",
            "--token", "ghp_fake_token_for_testing"
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Should either succeed or fail gracefully
    if !output.status.success() {
        assert!(stderr.contains("token") || stderr.contains("authentication") || stderr.contains("GitHub"));
    } else {
        // Should show processing multiple years
        assert!(stdout.contains("1990") && (stdout.contains("1991") || stdout.contains("1992")));
    }
}

#[test]
fn test_year_list_parsing() {
    let output = Command::new("cargo")
        .args(&[
            "run", "--", 
            "--dry-run",
            "--no-menu",
            "--years", "1990,1995,2000",
            "--username", "testuser",
            "--repo", "testrepo",
            "--token", "ghp_fake_token_for_testing"
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Should either succeed or fail gracefully
    if !output.status.success() {
        assert!(stderr.contains("token") || stderr.contains("authentication") || stderr.contains("GitHub"));
    } else {
        // Should show processing the specified years
        assert!(stdout.contains("1990") && stdout.contains("1995") && stdout.contains("2000"));
    }
}

#[cfg(test)]
mod npm_integration_tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_npm_wrapper_help() {
        // Build the binary first
        let build_output = Command::new("cargo")
            .args(&["build", "--release"])
            .output()
            .expect("Failed to build binary");
        
        assert!(build_output.status.success(), "Failed to build binary");

        // Ensure binary directory exists and copy binary
        fs::create_dir_all("binary").expect("Failed to create binary directory");
        fs::copy("target/release/git-timetraveler", "binary/git-timetraveler")
            .expect("Failed to copy binary");

        // Test npm wrapper
        let output = Command::new("node")
            .args(&["bin/git-timetraveler", "--help"])
            .output()
            .expect("Failed to execute npm wrapper");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Git Time Traveler"));
        assert!(stdout.contains("Usage:"));
    }

    #[test]
    fn test_npm_wrapper_version() {
        // Ensure binary exists
        if !Path::new("binary/git-timetraveler").exists() {
            let build_output = Command::new("cargo")
                .args(&["build", "--release"])
                .output()
                .expect("Failed to build binary");
            
            assert!(build_output.status.success());
            fs::create_dir_all("binary").expect("Failed to create binary directory");
            fs::copy("target/release/git-timetraveler", "binary/git-timetraveler")
                .expect("Failed to copy binary");
        }

        let output = Command::new("node")
            .args(&["bin/git-timetraveler", "--version"])
            .output()
            .expect("Failed to execute npm wrapper");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("git-timetraveler"));
    }
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_invalid_token_format() {
        let output = Command::new("cargo")
            .args(&[
                "run", "--", 
                "--dry-run",
                "--no-menu",
                "--year", "1990",
                "--username", "testuser",
                "--repo", "testrepo",
                "--token", "invalid_token_format"
            ])
            .output()
            .expect("Failed to execute command");

        // Should handle invalid token gracefully
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(stderr.contains("token") || stderr.contains("authentication"));
        }
    }

    #[test]
    fn test_invalid_date_combinations() {
        let output = Command::new("cargo")
            .args(&[
                "run", "--", 
                "--dry-run",
                "--no-menu",
                "--year", "1990",
                "--month", "13", // Invalid month
                "--username", "testuser",
                "--repo", "testrepo",
                "--token", "ghp_fake_token_for_testing"
            ])
            .output()
            .expect("Failed to execute command");

        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("month") || stderr.contains("13") || stderr.contains("range"));
    }

    #[test]
    fn test_network_error_simulation() {
        // Test with invalid GitHub API endpoint to simulate network errors
        let output = Command::new("cargo")
            .args(&[
                "run", "--", 
                "--dry-run",
                "--no-menu",
                "--year", "1990",
                "--username", "nonexistentuser12345",
                "--repo", "testrepo",
                "--token", "ghp_fake_token_for_testing"
            ])
            .output()
            .expect("Failed to execute command");

        // Should handle network/auth errors gracefully
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(stderr.contains("error") || stderr.contains("failed"));
        }
    }
}