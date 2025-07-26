#!/usr/bin/env node

const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

const TARGETS = [
  'x86_64-unknown-linux-gnu',
  'aarch64-unknown-linux-gnu',
  'x86_64-apple-darwin',
  'aarch64-apple-darwin',
  'x86_64-pc-windows-msvc',
  'aarch64-pc-windows-msvc'
];

function buildTarget(target) {
  console.log(`🔨 Building for ${target}...`);
  
  try {
    execSync(`cargo build --release --target ${target}`, { 
      stdio: 'inherit',
      env: { ...process.env }
    });
    
    const binaryName = target.includes('windows') ? 'git-timetraveler.exe' : 'git-timetraveler';
    const sourcePath = path.join('target', target, 'release', binaryName);
    const destDir = path.join('binary', target);
    const destPath = path.join(destDir, binaryName);
    
    // Create destination directory
    fs.mkdirSync(destDir, { recursive: true });
    
    // Copy binary
    if (fs.existsSync(sourcePath)) {
      fs.copyFileSync(sourcePath, destPath);
      console.log(`✅ Built ${target} -> ${destPath}`);
    } else {
      console.error(`❌ Binary not found at ${sourcePath}`);
      return false;
    }
    
    return true;
  } catch (error) {
    console.error(`❌ Failed to build ${target}:`, error.message);
    return false;
  }
}

function main() {
  console.log('🚀 Building Git Time Traveler for all platforms...');
  
  // Ensure binary directory exists
  fs.mkdirSync('binary', { recursive: true });
  
  let successCount = 0;
  let totalCount = TARGETS.length;
  
  for (const target of TARGETS) {
    if (buildTarget(target)) {
      successCount++;
    }
  }
  
  console.log('');
  console.log(`📊 Build Summary: ${successCount}/${totalCount} targets built successfully`);
  
  if (successCount === totalCount) {
    console.log('✅ All builds completed successfully!');
    process.exit(0);
  } else {
    console.log('⚠️  Some builds failed. Check the output above for details.');
    process.exit(1);
  }
}

if (require.main === module) {
  main();
}

module.exports = { buildTarget, TARGETS };