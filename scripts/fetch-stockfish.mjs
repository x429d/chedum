#!/usr/bin/env node
// Downloads the pinned Stockfish build for the current platform and stores it
// under src-tauri/binaries with the Rust target triple in the file name.
// Tauri's `externalBin` expects exactly that name; the binary is not in git.
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import { chmod, copyFile, mkdir, mkdtemp, readdir, rm, stat, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const VERSION = "sf_19";
const RELEASE = `https://github.com/official-stockfish/Stockfish/releases/download/${VERSION}`;

// asset, sha256, executable inside the archive, Rust target triple
const PLATFORMS = {
  "darwin-arm64": [
    "stockfish-macos-universal.tar.gz",
    "a1f0e3bcc5a6927a11fe6fc8e54a779754645f3c2bae2cf13420fd1957adaa77",
    "stockfish-macos-universal",
    "aarch64-apple-darwin",
  ],
  "darwin-x64": [
    "stockfish-macos-universal.tar.gz",
    "a1f0e3bcc5a6927a11fe6fc8e54a779754645f3c2bae2cf13420fd1957adaa77",
    "stockfish-macos-universal",
    "x86_64-apple-darwin",
  ],
  "linux-x64": [
    "stockfish-linux-x86-64-universal.tar.gz",
    "9defc0d4e55d49c65a6d042f3e571a39fcea499ade6dbe741b53b8c65e03611f",
    "stockfish-linux-x86-64-universal",
    "x86_64-unknown-linux-gnu",
  ],
  "linux-arm64": [
    "stockfish-linux-arm64-universal.tar.gz",
    "fe26cfd1d9db4c8af3d21e24d9ff34cacb31c1f940085a7583da11796f2bac01",
    "stockfish-linux-arm64-universal",
    "aarch64-unknown-linux-gnu",
  ],
  "linux-riscv64": [
    "stockfish-linux-riscv64-universal.tar.gz",
    "d4b4d118c003bc669a939d9fb208e3ef6b367730fd79a70dbc278b18475df3d1",
    "stockfish-linux-riscv64-universal",
    "riscv64gc-unknown-linux-gnu",
  ],
  "win32-x64": [
    "stockfish-windows-x86-64-universal.zip",
    "3c8bf1f9ea66a09350a40df4f632288285ac206d99f33ab5842c408fc30b48a7",
    "stockfish-windows-x86-64-universal.exe",
    "x86_64-pc-windows-msvc",
  ],
  "win32-arm64": [
    "stockfish-windows-arm64-universal.zip",
    "8372ad3f0d7276deb2c70f801f541ec7db463219fc6d9c7592864e542aa4f401",
    "stockfish-windows-arm64-universal.exe",
    "aarch64-pc-windows-msvc",
  ],
};

const platform = `${process.platform}-${process.arch}`;
const entry = PLATFORMS[platform];

if (!entry) {
  console.error(
    `Unsupported platform: ${platform}. Supported: ${Object.keys(PLATFORMS).join(", ")}.`,
  );
  process.exit(1);
}

const [asset, sha256, executableName, hostTarget] = entry;
const target = requestedTarget(hostTarget);
const isWindows = process.platform === "win32";

// Cross-compiling (for example an Intel bundle built on Apple Silicon) needs the
// binary for the target triple, which only works when the release asset is a
// universal build for the same operating system.
function requestedTarget(defaultTarget) {
  const index = process.argv.indexOf("--target");
  if (index === -1) {
    return defaultTarget;
  }
  const value = process.argv[index + 1];
  if (!value) {
    console.error("Missing value for --target.");
    process.exit(1);
  }
  if (value === defaultTarget) {
    return value;
  }
  const suffix = defaultTarget.slice(defaultTarget.indexOf("-"));
  const universal = asset.includes("-universal");
  if (!universal || !value.endsWith(suffix)) {
    console.error(
      `Cannot use ${asset} for target ${value}; only universal builds for the same OS support cross-compilation.`,
    );
    process.exit(1);
  }
  return value;
}

const binaryDirectory = path.join(ROOT, "src-tauri", "binaries");
const suffix = isWindows ? ".exe" : "";
const destination = path.join(binaryDirectory, `chedum-stockfish-${target}${suffix}`);
const legacy = path.join(binaryDirectory, `stockfish-${target}${suffix}`);

try {
  const info = await stat(destination);
  if (info.size > 0) {
    console.log(`Stockfish already present: ${path.relative(ROOT, destination)}`);
    process.exit(0);
  }
} catch {
  // Not fetched yet.
}

// Older checkouts used the plain `stockfish-<target>` name. Rename it instead
// of downloading the same 80 MB archive again.
try {
  const info = await stat(legacy);
  if (info.size > 0) {
    await mkdir(binaryDirectory, { recursive: true });
    await copyFile(legacy, destination);
    await chmod(destination, 0o755);
    await rm(legacy, { force: true });
    console.log(`Stockfish moved: ${path.relative(ROOT, destination)}`);
    process.exit(0);
  }
} catch {
  // No legacy binary.
}

const workdir = await mkdtemp(path.join(tmpdir(), "chedum-stockfish-"));
const archive = path.join(workdir, asset);

try {
  const url = `${RELEASE}/${asset}`;
  console.log(`Downloading ${url}…`);
  const response = await fetch(url, { redirect: "follow" });
  if (!response.ok) {
    throw new Error(`Download failed with HTTP ${response.status}`);
  }
  const bytes = Buffer.from(await response.arrayBuffer());
  const digest = createHash("sha256").update(bytes).digest("hex");
  if (digest !== sha256) {
    throw new Error(`Checksum mismatch: expected ${sha256}, got ${digest}`);
  }
  await writeFile(archive, bytes);

  // `tar` is available on macOS, Linux and Windows 10+ (bsdtar reads zip too).
  const extracted = spawnSync("tar", ["-xf", archive, "-C", workdir], { stdio: "inherit" });
  if (extracted.error) {
    throw new Error(`Could not run tar: ${extracted.error.message}`);
  }
  if (extracted.status !== 0) {
    throw new Error(`tar exited with code ${extracted.status}`);
  }

  const binary = await findBinary(workdir, executableName);
  if (!binary) {
    throw new Error(`Could not find ${executableName} in ${asset}`);
  }

  await mkdir(binaryDirectory, { recursive: true });
  await copyFile(binary, destination);
  await chmod(destination, 0o755);
  console.log(`Stockfish installed: ${path.relative(ROOT, destination)}`);
} finally {
  await rm(workdir, { recursive: true, force: true });
}

async function findBinary(directory, name) {
  const entries = await readdir(directory, { withFileTypes: true });
  for (const entry of entries) {
    const full = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      const found = await findBinary(full, name);
      if (found) {
        return found;
      }
    } else if (entry.name === name) {
      return full;
    }
  }
  return null;
}
