#!/usr/bin/env node

const { execSync } = require('child_process');
const fs = require('fs');

function createRelease() {
  const version = JSON.parse(fs.readFileSync('package.json', 'utf8')).version;
  const tagName = `v${version}`;
  
  console.log(`🏷️  Creating release ${tagName}...`);
  
  // Check if we're in a git repository
  try {
    execSync('git status', { stdio: 'pipe' });
  } catch (error) {
    console.error('❌ Not in a git repository');
    process.exit(1);
  }
  
  // Check if there are uncommitted changes
  try {
    const status = execSync('git status --porcelain', { encoding: 'utf8' });
    if (status.trim()) {
      console.error('❌ There are uncommitted changes. Please commit or stash them first.');
      console.log('Uncommitted changes:');
      console.log(status);
      process.exit(1);
    }
  } catch (error) {
    console.error('❌ Failed to check git status');
    process.exit(1);
  }
  
  // Check if tag already exists
  try {
    execSync(`git rev-parse ${tagName}`, { stdio: 'pipe' });
    console.error(`❌ Tag ${tagName} already exists`);
    process.exit(1);
  } catch (error) {
    // Tag doesn't exist, which is what we want
  }
  
  console.log('✅ Pre-flight checks passed');
  
  // Create and push tag
  try {
    console.log(`🏷️  Creating tag ${tagName}...`);
    execSync(`git tag -a ${tagName} -m "Release ${tagName}"`, { stdio: 'inherit' });
    
    console.log(`⬆️  Pushing tag ${tagName}...`);
    execSync(`git push origin ${tagName}`, { stdio: 'inherit' });
    
    console.log('✅ Tag created and pushed successfully!');
    console.log('');
    console.log('🚀 GitHub Actions will now:');
    console.log('  1. Build binaries for all platforms');
    console.log('  2. Create GitHub release with assets');
    console.log('  3. Upload cross-platform binaries');
    console.log('');
    console.log('📋 Next steps:');
    console.log('  1. Wait for GitHub Actions to complete');
    console.log('  2. Verify release at: https://github.com/chama-x/Git-Timetraveler/releases');
    console.log('  3. Publish to NPM: cd npm && npm publish');
    console.log('  4. Test published package: npx git-timetraveler@latest --help');
    
  } catch (error) {
    console.error('❌ Failed to create or push tag:', error.message);
    process.exit(1);
  }
}

function main() {
  console.log('🚀 Git Time Traveler Release Creator\n');
  
  // Run pre-publish check first
  console.log('🔍 Running pre-publish validation...');
  try {
    execSync('node scripts/pre-publish-check.js', { stdio: 'inherit' });
  } catch (error) {
    console.error('❌ Pre-publish check failed. Fix issues before creating release.');
    process.exit(1);
  }
  
  console.log('\n📋 Ready to create release!');
  console.log('This will:');
  console.log('  1. Create a git tag');
  console.log('  2. Push the tag to GitHub');
  console.log('  3. Trigger GitHub Actions to build and release');
  console.log('');
  
  // Simple confirmation
  const readline = require('readline');
  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout
  });
  
  rl.question('Continue? (y/N): ', (answer) => {
    rl.close();
    
    if (answer.toLowerCase() === 'y' || answer.toLowerCase() === 'yes') {
      createRelease();
    } else {
      console.log('❌ Release cancelled');
      process.exit(0);
    }
  });
}

if (require.main === module) {
  main();
}