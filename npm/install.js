#!/usr/bin/env node

const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');
const https = require('https');

const GITHUB_REPO = 'chama-x/Git-Timetraveler';
const VERSION = require('./package.json').version;

function getPlatformInfo() {
  const platform = process.platform;
  const arch = process.arch;
  
  let target;
  let extension;
  
  if (platform === 'win32') {
    target = 'x86_64-pc-windows-msvc';
    extension = '.zip';
  } else if (platform === 'darwin') {
    if (arch === 'arm64') {
      target = 'aarch64-apple-darwin';
    } else {
      target = 'x86_64-apple-darwin';
    }
    extension = '.tar.gz';
  } else if (platform === 'linux') {
    target = 'x86_64-unknown-linux-gnu';
    extension = '.tar.gz';
  } else {
    throw new Error(`Unsupported platform: ${platform}-${arch}`);
  }
  
  return { target, extension };
}

function downloadFile(url, dest) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(dest);
    
    https.get(url, (response) => {
      if (response.statusCode === 302 || response.statusCode === 301) {
        // Follow redirect
        return downloadFile(response.headers.location, dest).then(resolve).catch(reject);
      }
      
      if (response.statusCode !== 200) {
        reject(new Error(`Failed to download: ${response.statusCode} ${response.statusMessage}`));
        return;
      }
      
      response.pipe(file);
      
      file.on('finish', () => {
        file.close();
        resolve();
      });
      
      file.on('error', (err) => {
        fs.unlink(dest, () => {}); // Delete the file on error
        reject(err);
      });
    }).on('error', reject);
  });
}

function extractArchive(archivePath, extractDir) {
  const platform = process.platform;
  
  if (platform === 'win32') {
    // Use PowerShell to extract zip on Windows
    execSync(`powershell -command "Expand-Archive -Path '${archivePath}' -DestinationPath '${extractDir}' -Force"`, { stdio: 'inherit' });
  } else {
    // Use tar for .tar.gz files on Unix-like systems
    execSync(`tar -xzf "${archivePath}" -C "${extractDir}"`, { stdio: 'inherit' });
  }
}

async function install() {
  try {
    console.log('🚀 Installing Git Time Traveler...');
    
    const { target, extension } = getPlatformInfo();
    const version = `v${VERSION}`;
    const filename = `git-timetraveler-${target}${extension}`;
    const downloadUrl = `https://github.com/${GITHUB_REPO}/releases/download/${version}/${filename}`;
    
    console.log(`📦 Downloading ${filename}...`);
    
    const binDir = path.join(__dirname, 'bin');
    const tempDir = path.join(__dirname, 'temp');
    
    // Create directories
    fs.mkdirSync(binDir, { recursive: true });
    fs.mkdirSync(tempDir, { recursive: true });
    
    const archivePath = path.join(tempDir, filename);
    
    // Download the archive
    await downloadFile(downloadUrl, archivePath);
    
    console.log('📂 Extracting archive...');
    
    // Extract the archive
    extractArchive(archivePath, tempDir);
    
    // Find the binary in the extracted files
    const extractedFiles = fs.readdirSync(tempDir, { recursive: true });
    let binaryName = 'git-timetraveler';
    if (process.platform === 'win32') {
      binaryName += '.exe';
    }
    
    const binaryPath = extractedFiles
      .map(file => path.join(tempDir, file))
      .find(file => {
        const basename = path.basename(file);
        return basename === binaryName && fs.statSync(file).isFile();
      });
    
    if (!binaryPath) {
      throw new Error(`Binary ${binaryName} not found in extracted files`);
    }
    
    // Move binary to bin directory
    const finalBinaryPath = path.join(binDir, binaryName);
    fs.copyFileSync(binaryPath, finalBinaryPath);
    
    // Make executable on Unix-like systems
    if (process.platform !== 'win32') {
      fs.chmodSync(finalBinaryPath, '755');
    }
    
    // Create a wrapper script for cross-platform execution
    const wrapperPath = path.join(binDir, 'git-timetraveler');
    if (process.platform === 'win32') {
      // Create .cmd wrapper for Windows
      fs.writeFileSync(wrapperPath + '.cmd', `@echo off\n"%~dp0\\git-timetraveler.exe" %*\n`);
    } else {
      // The binary is already executable on Unix-like systems
      if (finalBinaryPath.endsWith('.exe')) {
        // This shouldn't happen, but just in case
        fs.renameSync(finalBinaryPath, wrapperPath);
      }
    }
    
    // Clean up
    fs.rmSync(tempDir, { recursive: true, force: true });
    
    console.log('✅ Git Time Traveler installed successfully!');
    console.log('');
    console.log('Usage:');
    console.log('  npx @git-timetraveler/cli --help');
    console.log('  npx @git-timetraveler/cli --year 1990');
    
  } catch (error) {
    console.error('❌ Installation failed:', error.message);
    console.error('');
    console.error('You can try:');
    console.error('1. Downloading the binary manually from:');
    console.error(`   https://github.com/${GITHUB_REPO}/releases`);
    console.error('2. Building from source:');
    console.error(`   git clone https://github.com/${GITHUB_REPO}.git`);
    console.error('   cd git-timetraveler');
    console.error('   cargo build --release');
    
    process.exit(1);
  }
}

if (require.main === module) {
  install();
}

module.exports = { install }; 