const fs = require("fs");
const path = require("path");
const https = require("https");
const { execSync } = require("child_process");

const pkg = require("./package.json");

const BIN_NAME = "jira-rs";
const REPO = "rafael-arreola/mcp-jira-rs";
const VERSION = `v${pkg.version}`;

// Mapeo de arquitectura y sistema operativo a target triple de Rust
function getTarget() {
  const arch = process.arch;
  const platform = process.platform;

  if (platform === "darwin") {
    if (arch === "x64") return "x86_64-apple-darwin";
    if (arch === "arm64") return "aarch64-apple-darwin";
  } else if (platform === "win32") {
    if (arch === "x64") return "x86_64-pc-windows-msvc";
    if (arch === "arm64") return "aarch64-pc-windows-msvc";
  } else if (platform === "linux") {
    if (arch === "x64") return "x86_64-unknown-linux-gnu";
    if (arch === "arm64") return "aarch64-unknown-linux-gnu";
  }

  throw new Error(`Unsupported platform: ${platform} ${arch}`);
}

const target = getTarget();
const ext = process.platform === "win32" ? "zip" : "tar.gz";
const filename = `${BIN_NAME}-${VERSION}-${target}.${ext}`;
const url = `https://github.com/${REPO}/releases/download/${VERSION}/${filename}`;
const binDir = path.join(__dirname, "bin");
const binFile = process.platform === "win32" ? `${BIN_NAME}.exe` : BIN_NAME;
const finalBinPath = path.join(binDir, binFile);

console.log(`Detected platform: ${process.platform} ${process.arch}`);
console.log(`Target: ${target}`);
console.log(`Downloading from: ${url}`);

if (!fs.existsSync(binDir)) {
  fs.mkdirSync(binDir);
}

const tempFile = path.join(binDir, filename);

const file = fs.createWriteStream(tempFile);

https
  .get(url, (response) => {
    if (response.statusCode === 302 || response.statusCode === 301) {
      https.get(response.headers.location, (res) => {
        download(res);
      });
    } else {
      download(response);
    }
  })
  .on("error", (err) => {
    console.error(`Error downloading file: ${err.message}`);
    process.exit(1);
  });

function download(response) {
  if (response.statusCode !== 200) {
    console.error(`Failed to download: ${response.statusCode}`);
    process.exit(1);
  }

  response.pipe(file);

  file.on("finish", () => {
    file.close(extract);
  });
}

function extract() {
  console.log("Extracting...");
  try {
    if (ext === "zip") {
      // En Windows moderno, tar suele estar disponible y maneja zip.
      // Si no, podríamos usar powershell.
      // Intentamos tar primero.
      try {
        execSync(`tar -xf "${tempFile}" -C "${binDir}"`);
      } catch (e) {
        console.log("tar failed, trying powershell...");
        execSync(
          `powershell -Command "Expand-Archive -Path '${tempFile}' -DestinationPath '${binDir}' -Force"`,
        );
      }
    } else {
      execSync(`tar -xzf "${tempFile}" -C "${binDir}"`);
    }

    fs.unlinkSync(tempFile);

    if (!fs.existsSync(finalBinPath)) {
      // Buscar en subdirectorios
      const findFile = (dir) => {
        const files = fs.readdirSync(dir);
        for (const f of files) {
          const fullPath = path.join(dir, f);
          const stat = fs.statSync(fullPath);
          if (stat.isDirectory()) {
            const found = findFile(fullPath);
            if (found) return found;
          } else if (f === binFile) {
            return fullPath;
          }
        }
        return null;
      };

      const foundPath = findFile(binDir);
      if (foundPath) {
        fs.renameSync(foundPath, finalBinPath);
      } else {
        console.error("Could not find binary in extracted files.");
        process.exit(1);
      }
    }

    if (process.platform !== "win32") {
      fs.chmodSync(finalBinPath, 0o755);
    }

    console.log(`Successfully installed to ${finalBinPath}`);
  } catch (err) {
    console.error(`Error extracting file: ${err.message}`);
    process.exit(1);
  }
}
