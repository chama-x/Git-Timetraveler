#!/usr/bin/env node

const { execSync } = require("child_process");
const fs = require("fs");
const path = require("path");
const https = require("https");

const GITHUB_REPO = "chama-x/Git-Timetraveler";
const VERSION = require("./package.json").version;

function getPlatformInfo() {
  const platform = process.platform;
  const arch = process.arch;

  let target;
  let extension;
  let binaryName = "git-timetraveler";

  if (platform === "win32") {
    target =
      arch === "arm64" ? "aarch64-pc-windows-msvc" : "x86_64-pc-windows-msvc";
    extension = ".tar.gz";
    binaryName += ".exe";
  } else if (platform === "darwin") {
    target = arch === "arm64" ? "aarch64-apple-darwin" : "x86_64-apple-darwin";
    extension = ".tar.gz";
  } else if (platform === "linux") {
    target =
      arch === "arm64"
        ? "aarch64-unknown-linux-gnu"
        : "x86_64-unknown-linux-gnu";
    extension = ".tar.gz";
  } else {
    throw new Error(`Unsupported platform: ${platform}-${arch}`);
  }

  return { target, extension, binaryName };
}

function downloadFile(url, dest) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(dest);

    https
      .get(url, (response) => {
        if (response.statusCode === 302 || response.statusCode === 301) {
          // Follow redirect
          return downloadFile(response.headers.location, dest)
            .then(resolve)
            .catch(reject);
        }

        if (response.statusCode !== 200) {
          reject(
            new Error(
              `Failed to download: ${response.statusCode} ${response.statusMessage}`,
            ),
          );
          return;
        }

        response.pipe(file);

        file.on("finish", () => {
          file.close();
          resolve();
        });

        file.on("error", (err) => {
          fs.unlink(dest, () => {}); // Delete the file on error
          reject(err);
        });
      })
      .on("error", reject);
  });
}

function extractArchive(archivePath, extractDir) {
  const platform = process.platform;

  if (platform === "win32") {
    // Use PowerShell to extract on Windows
    execSync(
      `powershell -command "Expand-Archive -Path '${archivePath}' -DestinationPath '${extractDir}' -Force"`,
      { stdio: "inherit" },
    );
  } else {
    // Use tar for .tar.gz files on Unix-like systems
    execSync(`tar -xzf "${archivePath}" -C "${extractDir}"`, {
      stdio: "inherit",
    });
  }
}

async function install() {
  try {
    console.log("🚀 Installing Git Time Traveler...");

    const { target, extension, binaryName } = getPlatformInfo();
    const version = `v${VERSION}`;
    const filename = `git-timetraveler-${target}${extension}`;
    const downloadUrl = `https://github.com/${GITHUB_REPO}/releases/download/${version}/${filename}`;

    const binaryDir = path.join(__dirname, "binary");
    const binDir = path.join(__dirname, "bin");
    const tempDir = path.join(__dirname, "temp");

    // Create directories
    fs.mkdirSync(binaryDir, { recursive: true });
    fs.mkdirSync(binDir, { recursive: true });
    fs.mkdirSync(tempDir, { recursive: true });

    // Check if we already have a binary in the package
    const packageBinaryPath = path.join(binaryDir, binaryName);
    const binBinaryPath = path.join(binDir, binaryName);

    if (fs.existsSync(packageBinaryPath)) {
      console.log("✅ Found bundled binary, using it instead of downloading");

      // Copy to bin directory
      fs.copyFileSync(packageBinaryPath, binBinaryPath);

      // Make executable on Unix-like systems
      if (process.platform !== "win32") {
        fs.chmodSync(packageBinaryPath, "755");
        fs.chmodSync(binBinaryPath, "755");
      }

      console.log(
        "✅ Git Time Traveler installed successfully using bundled binary!",
      );
      return;
    }

    console.log(`📦 Downloading ${filename}...`);
    console.log(`🔗 URL: ${downloadUrl}`);

    const archivePath = path.join(tempDir, filename);

    try {
      // Download the archive
      await downloadFile(downloadUrl, archivePath);
    } catch (err) {
      console.log(`⚠️ Failed to download from GitHub: ${err.message}`);
      if (fs.existsSync(binBinaryPath)) {
        console.log("✅ Using existing binary");
        return;
      } else {
        throw new Error(`No binary available: ${err.message}`);
      }
    }

    console.log("📂 Extracting archive...");

    // Extract the archive
    extractArchive(archivePath, tempDir);

    // Find the binary in the extracted files
    const extractedFiles = fs.readdirSync(tempDir, { recursive: true });

    const binaryPath = extractedFiles
      .map((file) => path.join(tempDir, file))
      .find((file) => {
        const basename = path.basename(file);
        return basename === binaryName && fs.statSync(file).isFile();
      });

    if (!binaryPath) {
      throw new Error(
        `Binary ${binaryName} not found in extracted files. Found: ${extractedFiles.join(", ")}`,
      );
    }

    // Move binary to binary directory
    const finalBinaryPath = path.join(binaryDir, binaryName);
    fs.copyFileSync(binaryPath, finalBinaryPath);

    // Make executable on Unix-like systems
    if (process.platform !== "win32") {
      fs.chmodSync(finalBinaryPath, "755");
    }

    console.log(`✅ Binary installed to: ${finalBinaryPath}`);

    // Clean up
    fs.rmSync(tempDir, { recursive: true, force: true });

    // Copy to bin directory if not already done
    if (!fs.existsSync(path.join(binDir, binaryName))) {
      fs.copyFileSync(finalBinaryPath, path.join(binDir, binaryName));
      if (process.platform !== "win32") {
        fs.chmodSync(path.join(binDir, binaryName), "755");
      }
    }

    console.log("✅ Git Time Traveler installed successfully!");
    console.log("");
    console.log("Usage:");
    console.log("  npx git-timetraveler --help");
    console.log("  npx git-timetraveler --year 1990");
    console.log("  npx git-timetraveler  # Interactive mode");
  } catch (error) {
    console.error("❌ Installation failed:", error.message);
    console.error("");
    console.error("You can try:");
    console.error("1. Downloading the binary manually from:");
    console.error(`   https://github.com/${GITHUB_REPO}/releases`);
    console.error("2. Building from source:");
    console.error(`   git clone https://github.com/${GITHUB_REPO}.git`);
    console.error("   cd Git-Timetraveler");
    console.error("   cargo build --release");

    process.exit(1);
  }
}

if (require.main === module) {
  install();
}

module.exports = { install };
