#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

function updateVersion(newVersion) {
  console.log(`🔄 Updating version to ${newVersion}...`);
  
  // Update root package.json
  const rootPackage = JSON.parse(fs.readFileSync('package.json', 'utf8'));
  rootPackage.version = newVersion;
  fs.writeFileSync('package.json', JSON.stringify(rootPackage, null, 2) + '\n');
  console.log('✅ Updated package.json');
  
  // Update npm/package.json
  const npmPackage = JSON.parse(fs.readFileSync('npm/package.json', 'utf8'));
  npmPackage.version = newVersion;
  fs.writeFileSync('npm/package.json', JSON.stringify(npmPackage, null, 2) + '\n');
  console.log('✅ Updated npm/package.json');
  
  // Update Cargo.toml
  let cargoToml = fs.readFileSync('Cargo.toml', 'utf8');
  cargoToml = cargoToml.replace(/version = "[^"]+"/, `version = "${newVersion}"`);
  fs.writeFileSync('Cargo.toml', cargoToml);
  console.log('✅ Updated Cargo.toml');
  
  console.log(`\n🎉 Version updated to ${newVersion} across all files!`);
}

const newVersion = process.argv[2];
if (!newVersion) {
  console.error('Usage: node scripts/update-version.js <version>');
  console.error('Example: node scripts/update-version.js 1.0.0');
  process.exit(1);
}

// Validate semver format
if (!/^\d+\.\d+\.\d+$/.test(newVersion)) {
  console.error('Error: Version must be in semver format (e.g., 1.0.0)');
  process.exit(1);
}

updateVersion(newVersion);