#!/usr/bin/env bun
// scripts/fetch-bundled-model.ts - make the offline model available before a build
//
// The GigaAM v3 CTC model is ~151 MB, so it is not committed. This script
// downloads it, verifies the SHA-256, and extracts it into the Tauri resources
// directory so the bundler ships it inside the installer.
//
// It is idempotent: if the target directory already contains model.int8.onnx,
// the script exits without touching the network. That keeps rebuilds fast and
// makes an offline rebuild possible once the model has been fetched once.
//
// Run manually with: bun scripts/fetch-bundled-model.ts

import { createHash } from "node:crypto";
import { existsSync } from "node:fs";
import { mkdir, rename, rm, writeFile, readdir, stat } from "node:fs/promises";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const ARCHIVE_URL = "https://blob.handy.computer/giga-am-v3-int8.tar.gz";
const ARCHIVE_SHA256 =
  "d872462268430db140b69b72e0fc4b787b194c1dbe51b58de39444d55b6da45b";
const MODEL_DIR_NAME = "giga-am-v3-int8";
const MODEL_FILE = "model.int8.onnx";

const repoRoot = join(dirname(fileURLToPath(import.meta.url)), "..");
const modelsDir = join(repoRoot, "src-tauri", "resources", "models");
const targetDir = join(modelsDir, MODEL_DIR_NAME);
// Staging lives next to the target on purpose: renaming across volumes (a temp
// dir on another drive) fails with EXDEV on Windows.
const stagingDir = join(modelsDir, `.${MODEL_DIR_NAME}.staging`);

const sha256 = async (path: string): Promise<string> => {
  const data = await Bun.file(path).arrayBuffer();
  const hash = createHash("sha256").update(Buffer.from(data));
  return hash.digest("hex");
};

const isModelReady = async (): Promise<boolean> => {
  if (!existsSync(targetDir)) return false;
  const entries = await readdir(targetDir);
  return entries.includes(MODEL_FILE);
};

const extract = async (archive: string, dest: string): Promise<void> => {
  // The archive stores the model files at its root (no wrapping directory), so
  // it is extracted without --strip-components.
  // tar is present on macOS and Linux out of the box, and on Windows 10+
  // (bsdtar). No extra dependency for the common cases.
  const proc = Bun.spawn(["tar", "-xzf", archive, "-C", dest], {
    stdout: "inherit",
    stderr: "inherit",
  });
  const code = await proc.exited;
  if (code !== 0) {
    throw new Error(`tar exited with code ${code}`);
  }

  // The archive was produced on macOS and carries AppleDouble sidecars
  // (`._model.int8.onnx`). They are metadata forks, not model data, and must not
  // be shipped inside the installer.
  for (const entry of await readdir(dest)) {
    if (entry.startsWith("._")) {
      await rm(join(dest, entry), { force: true });
    }
  }
};

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

// Windows virus scanners routinely hold a handle on a freshly written 200 MB
// file for a moment, which surfaces as a transient EPERM/EBUSY from rename.
// Retrying briefly turns that into a success instead of a failed build.
const moveDir = async (
  from: string,
  to: string,
  attempts = 10,
): Promise<void> => {
  for (let attempt = 1; ; attempt += 1) {
    try {
      await rename(from, to);
      return;
    } catch (error) {
      if (attempt >= attempts) throw error;
      console.log(
        `fetch-bundled-model: rename failed (attempt ${attempt}), retrying`,
      );
      await sleep(1000);
    }
  }
};

const main = async () => {
  if (await isModelReady()) {
    console.log(
      `fetch-bundled-model: ${MODEL_DIR_NAME} already present, skipping download`,
    );
    return;
  }

  await mkdir(modelsDir, { recursive: true });
  const archive = join(modelsDir, ".model.tar.gz");

  try {
    console.log(`fetch-bundled-model: downloading ${ARCHIVE_URL}`);
    const response = await fetch(ARCHIVE_URL);
    if (!response.ok) {
      throw new Error(
        `download failed: HTTP ${response.status} ${response.statusText}`,
      );
    }
    await writeFile(archive, Buffer.from(await response.arrayBuffer()));

    console.log("fetch-bundled-model: verifying checksum");
    const actual = await sha256(archive);
    if (actual !== ARCHIVE_SHA256) {
      throw new Error(
        `checksum mismatch: expected ${ARCHIVE_SHA256}, got ${actual}`,
      );
    }

    // Extract to a staging directory, then move into place, so an interrupted
    // run never leaves a half-written model that later looks "already present".
    await rm(stagingDir, { recursive: true, force: true });
    await mkdir(stagingDir, { recursive: true });
    await extract(archive, stagingDir);

    if (!existsSync(join(stagingDir, MODEL_FILE))) {
      throw new Error(`archive did not contain ${MODEL_FILE}`);
    }

    await rm(targetDir, { recursive: true, force: true });
    await moveDir(stagingDir, targetDir);
    const size = (await stat(join(targetDir, MODEL_FILE))).size;
    console.log(
      `fetch-bundled-model: installed ${MODEL_DIR_NAME}/${MODEL_FILE} (${Math.round(size / 1024 / 1024)} MB)`,
    );
  } finally {
    await rm(archive, { force: true });
    await rm(stagingDir, { recursive: true, force: true });
  }
};

await main();
