#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

console.log('🔍 Pre-publish validation checklist\n');

let errors = [];
let warnings = [];

// Check version consistency
function checkVersionConsistency() {
  console.log('📋 Checking version consistency...');
  
  const rootPackage = JSON.parse(fs.readFileSync('package.json', 'utf8'));
  const npmPackage = JSON.parse(fs.readFileSync('npm/package.json', 'utf8'));
  const cargoToml = fs.readFileSync('Cargo.toml', 'utf8');
  
  const cargoVersionMatch = cargoToml.match(/version = "([^"]+)"/);
  const cargoVersion = cargoVersionMatch ? cargoVersionMatch[1] : null;
  
  if (rootPackage.version !== npmPackage.version) {
    errors.push(`Version mismatch: root package.json (${rootPackage.version}) vs npm/package.json (${npmPackage.version})`);
  }
  
  if (rootPackage.version !== cargoVersion) {
    errors.push(`Version mismatch: package.json (${rootPackage.version}) vs Cargo.toml (${cargoVersion})`);
  }
  
  if (errors.length === 0) {
    console.log(`✅ All versions consistent: ${rootPackage.version}`);
  }
}

// Check required files exist
function checkRequiredFiles() {
  console.log('📁 Checking required files...');
  
  const requiredFiles = [
    'package.json',
    'npm/package.json',
    'npm/install.js',
    'npm/bin/git-timetraveler',
    'install.js',
    'bin/git-timetraveler',
    'Cargo.toml',
    'README.md',
    'LICENSE'
  ];
  
  for (const file of requiredFiles) {
    if (!fs.existsSync(file)) {
      errors.push(`Missing required file: ${file}`);
    }
  }
  
  if (errors.length === 0) {
    console.log('✅ All required files present');
  }
}

// Check binary exists
function checkBinary() {
  console.log('🔧 Checking binary...');
  
  const binaryPath = 'binary/git-timetraveler';
  if (!fs.existsSync(binaryPath)) {
    errors.push('Binary not found at binary/git-timetraveler - run cargo build --release first');
  } else {
    const stats = fs.statSync(binaryPath);
    console.log(`✅ Binary exists (${(stats.size / 1024 / 1024).toFixed(1)}MB)`);
  }
}

// Test npm pack
function testNpmPack() {
  console.log('📦 Testing npm pack...');
  
  try {
    const output = execSync('npm pack --dry-run', { 
      cwd: 'npm',
      encoding: 'utf8',
      stdio: 'pipe'
    });
    
    // Parse the output to check package size
    const lines = output.split('\n');
    const sizeLine = lines.find(line => line.includes('package size:'));
    if (sizeLine) {
      console.log(`✅ Package size: ${sizeLine.split('package size:')[1].trim()}`);
    }
    
    // Check if all expected files are included
    const expectedFiles = ['README.md', 'bin/git-timetraveler', 'binary/git-timetraveler', 'install.js', 'package.json'];
    for (const file of expectedFiles) {
      if (!output.includes(file)) {
        warnings.push(`Expected file ${file} not found in npm pack output`);
      }
    }
    
  } catch (error) {
    errors.push(`npm pack test failed: ${error.message}`);
  }
}

// Test binary execution
function testBinaryExecution() {
  console.log('🚀 Testing binary execution...');
  
  try {
    const output = execSync('node bin/git-timetraveler --no-menu --help', { 
      encoding: 'utf8',
      stdio: 'pipe'
    });
    
    if (output.includes('Git Time Traveler')) {
      console.log('✅ Binary executes correctly');
    } else {
      errors.push('Binary execution test failed - unexpected output');
    }
  } catch (error) {
    errors.push(`Binary execution test failed: ${error.message}`);
  }
}

// Run tests
function runTests() {
  console.log('🧪 Running integration tests...');
  
  try {
    execSync('cargo test --test integration_tests', { 
      stdio: 'pipe'
    });
    console.log('✅ All integration tests pass');
  } catch (error) {
    errors.push('Integration tests failed - fix tests before publishing');
  }
}

// Check package.json metadata
function checkPackageMetadata() {
  console.log('📝 Checking package metadata...');
  
  const pkg = JSON.parse(fs.readFileSync('npm/package.json', 'utf8'));
  
  const requiredFields = ['name', 'version', 'description', 'keywords', 'homepage', 'repository', 'license', 'author'];
  for (const field of requiredFields) {
    if (!pkg[field]) {
      errors.push(`Missing required package.json field: ${field}`);
    }
  }
  
  if (pkg.keywords && pkg.keywords.length < 3) {
    warnings.push('Consider adding more keywords for better discoverability');
  }
  
  if (!pkg.description || pkg.description.length < 20) {
    warnings.push('Description should be more descriptive');
  }
  
  console.log('✅ Package metadata looks good');
}

// Main execution
function main() {
  checkVersionConsistency();
  checkRequiredFiles();
  checkBinary();
  checkPackageMetadata();
  testNpmPack();
  testBinaryExecution();
  runTests();
  
  console.log('\n📊 Validation Summary:');
  
  if (errors.length > 0) {
    console.log('\n❌ Errors (must fix before publishing):');
    errors.forEach(error => console.log(`  • ${error}`));
  }
  
  if (warnings.length > 0) {
    console.log('\n⚠️  Warnings (recommended to fix):');
    warnings.forEach(warning => console.log(`  • ${warning}`));
  }
  
  if (errors.length === 0) {
    console.log('\n✅ Ready for publication!');
    console.log('\nNext steps:');
    console.log('1. npm publish (from npm/ directory)');
    console.log('2. Create GitHub release');
    console.log('3. Test published package');
    process.exit(0);
  } else {
    console.log('\n❌ Fix errors before publishing');
    process.exit(1);
  }
}

if (require.main === module) {
  main();
}

module.exports = { main };