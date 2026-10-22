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
  const extraFiles = config["extra-files"].map((file) => normalizePath(typeof file === "string" ? file : file.path));
  assert.ok(extraFiles.includes("src-tauri/tauri.conf.json"));
  assert.ok(extraFiles.includes("src-tauri/Cargo.toml"));
});

test("manifest releases include the application at the repository root", async () => {
  const config = JSON.parse(await fs.readFile(configPath, "utf-8"));
  assert.ok(Object.hasOwn(config.packages ?? {}, "."), "Root package must be configured or Release Please silently releases nothing");
});

test("Rust package version updater targets the package table", async () => {
  const config = JSON.parse(await fs.readFile(configPath, "utf-8"));
  const cargo = config["extra-files"].find(file => typeof file === "object" && file.path === "src-tauri/Cargo.toml");
  assert.equal(cargo?.jsonpath, "$.package.version");
});
