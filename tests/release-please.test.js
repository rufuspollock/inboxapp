import assert from "node:assert/strict";
import test from "node:test";
import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const configPath = path.join(root, ".github", "release-please-config.json");

function normalizePath(value) {
  return value.replace(/\\/g, "/");
}

test("release-please config includes extra files for tauri versioning", async () => {
  const raw = await fs.readFile(configPath, "utf-8");
  const config = JSON.parse(raw);
  assert.equal(config["release-type"], "node");
  assert.equal(config["package-name"], "tauri-app");
  const extraFiles = config["extra-files"].map((file) => normalizePath(file));
  assert.ok(extraFiles.includes("src-tauri/tauri.conf.json"));
  assert.ok(extraFiles.includes("src-tauri/Cargo.toml"));
});
