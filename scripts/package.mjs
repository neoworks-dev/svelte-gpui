#!/usr/bin/env bun
// Packages an app into one executable, with the bundle and the files beside it embedded.
//
//   bun scripts/package.mjs --app examples/demo [--out dist/package] [--no-tar] [--no-strip]
//
// The app describes itself in the "svelte-gpui" key of its package.json, or in a
// svelte-gpui.json beside it (docs/packaging.md). Steps:
//   1. run the app's build script;
//   2. compile the bundle to bytecode with the host's `--compile-only`;
//   3. write what gets embedded to target/package/APP/embed;
//   4. build the executable: the app's own crate, or a small generated one;
//   5. collect the executable, a .desktop file and the icon in OUT/APP, and a tarball.

import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { cpSync, existsSync, mkdirSync, readdirSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { arch } from "node:os";
import { basename, dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");

function fail(message) {
  console.error(`package: ${message}`);
  process.exit(1);
}

function parseArguments(argv) {
  const options = { out: join(root, "dist", "package"), tar: true, strip: true };
  for (let index = 0; index < argv.length; index++) {
    const argument = argv[index];
    if (argument === "--app") options.app = resolve(argv[++index] ?? fail("--app needs a directory"));
    else if (argument === "--out") options.out = resolve(argv[++index] ?? fail("--out needs a directory"));
    else if (argument === "--no-tar") options.tar = false;
    else if (argument === "--no-strip") options.strip = false;
    else fail(`unknown argument ${argument}`);
  }
  if (options.app === undefined) fail("usage: bun scripts/package.mjs --app examples/demo [--out dist/package] [--no-tar] [--no-strip]");
  return options;
}

/** The app's metadata, with the defaults filled in and the required parts checked. */
function readMetadata(app) {
  const packagePath = join(app, "package.json");
  const standalone = join(app, "svelte-gpui.json");
  const manifest = existsSync(packagePath) ? JSON.parse(readFileSync(packagePath, "utf8")) : {};
  const found = existsSync(standalone) ? JSON.parse(readFileSync(standalone, "utf8")) : manifest["svelte-gpui"];
  if (found === undefined) fail(`${app} has no "svelte-gpui" key in package.json and no svelte-gpui.json`);
  const metadata = { name: manifest.name, version: manifest.version ?? "0.0.0", build: "build.mjs", ...found };
  for (const key of ["name", "id", "bundle"]) {
    if (typeof metadata[key] !== "string" || metadata[key] === "") fail(`the metadata needs "${key}"`);
  }
  if (!/^[A-Za-z][A-Za-z0-9_-]*(\.[A-Za-z][A-Za-z0-9_-]*)+$/.test(metadata.id)) {
    fail(`"id" must be a reverse-domain name such as dev.example.my-app, not ${metadata.id}`);
  }
  metadata.executable ??= metadata.name.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "");
  return metadata;
}

function run(command, args, options = {}) {
  console.log(`$ ${command} ${args.join(" ")}`);
  const result = spawnSync(command, args, { stdio: "inherit", ...options });
  if (result.status !== 0) fail(`${command} failed`);
}

/**
 * Where cargo puts what it builds: `CARGO_TARGET_DIR` when it is set, otherwise this checkout's own
 * `target/`, seeded with the GPUI artifacts by `scripts/host.sh` so they are not compiled again.
 */
function cargoEnvironment() {
  if (process.env.CARGO_TARGET_DIR !== undefined) {
    return process.env;
  }
  run("bash", [join(root, "scripts", "host.sh"), "--seed"]);
  return { ...process.env, CARGO_TARGET_DIR: join(root, "target") };
}

/** Builds with cargo, and returns the path of the executable it made. */
function cargoBuild(manifest, extraArguments, extraEnvironment = {}) {
  const args = ["build", "--release", "--manifest-path", manifest, "--message-format=json-render-diagnostics", ...extraArguments];
  console.log(`$ cargo ${args.join(" ")}`);
  const result = spawnSync("cargo", args, {
    env: { ...cargoEnvironment(), ...extraEnvironment },
    stdio: ["inherit", "pipe", "inherit"],
    maxBuffer: 1 << 28,
  });
  if (result.status !== 0) fail("cargo failed");
  const artifacts = result.stdout
    .toString()
    .split("\n")
    .filter((line) => line.startsWith("{"))
    .map((line) => JSON.parse(line))
    .filter((message) => message.reason === "compiler-artifact" && message.executable && resolve(message.manifest_path) === resolve(manifest));
  if (artifacts.length === 0) fail(`cargo built no executable from ${manifest}`);
  return artifacts[artifacts.length - 1].executable;
}

/** Runs the app's build script, and returns the bundle's path. */
function buildBundle(app, metadata) {
  if (metadata.build !== false) run("bun", [resolve(app, metadata.build)], { cwd: app });
  const bundle = resolve(app, metadata.bundle);
  if (!existsSync(bundle)) fail(`the build did not produce ${bundle}`);
  return bundle;
}

/** Compiles the bundle with the host's own QuickJS, so the bytecode matches the executable's. */
function compileBytecode(bundle, output) {
  const host = cargoBuild(join(root, "host", "Cargo.toml"), ["--bin", "svelte-gpui"]);
  const result = spawnSync(host, ["--compile-only", bundle, output], { stdio: "inherit" });
  if (result.status !== 0) {
    console.warn("package: the bundle did not compile to bytecode; embedding its source instead");
    return false;
  }
  return true;
}

/** The files that travel with the bundle: everything in its directory except build leftovers. */
function filesBesideBundle(bundle) {
  const directory = dirname(bundle);
  const found = [];
  const walk = (current) => {
    for (const entry of readdirSync(current, { withFileTypes: true })) {
      const path = join(current, entry.name);
      if (entry.isDirectory()) walk(path);
      else if (path !== bundle && !/\.(qbc|map|tmp)$|\.meta\.json$/.test(entry.name)) found.push(path);
    }
  };
  walk(directory);
  return found.sort().map((path) => ({ path: relative(directory, path).split("/").join("/"), data: readFileSync(path) }));
}

/** The archive host/src/embedded.rs reads: per file a u32 path length, the path, a u64 size, the bytes. */
function makeArchive(files) {
  const parts = [];
  for (const { path, data } of files) {
    const name = Buffer.from(path, "utf8");
    const head = Buffer.alloc(4);
    head.writeUInt32LE(name.length);
    const size = Buffer.alloc(8);
    size.writeBigUInt64LE(BigInt(data.length));
    parts.push(head, name, size, data);
  }
  return Buffer.concat(parts);
}

/** Writes metadata.json, script.bin and files.bin, which `embed!()` includes. */
function writeEmbedded(directory, metadata, bundle) {
  rmSync(directory, { recursive: true, force: true });
  mkdirSync(directory, { recursive: true });
  const bytecode = compileBytecode(bundle, join(directory, "script.bin"));
  if (!bytecode) writeFileSync(join(directory, "script.bin"), readFileSync(bundle));
  const archive = makeArchive(filesBesideBundle(bundle));
  writeFileSync(join(directory, "files.bin"), archive);
  const hash = createHash("sha256").update(archive).digest("hex").slice(0, 16);
  const embedded = { name: metadata.name, id: metadata.id, version: metadata.version, title: metadata.title, bytecode, hash };
  writeFileSync(join(directory, "metadata.json"), JSON.stringify(embedded, null, 2));
}

/** A crate whose whole program is `run(Options::embedded(embed!()))`, for an app with no Rust of its own. */
function generateCrate(directory, metadata) {
  const version = /^\d+\.\d+\.\d+/.test(metadata.version) ? metadata.version : "0.0.0";
  const host = join(root, "host");
  mkdirSync(join(directory, "src"), { recursive: true });
  writeFileSync(
    join(directory, "Cargo.toml"),
    `[package]\nname = ${JSON.stringify(metadata.executable)}\nversion = ${JSON.stringify(version)}\nedition = "2024"\n\n` +
      `[dependencies]\nsvelte_gpui_host = { path = ${JSON.stringify(host)} }\n\n` +
      `[patch.crates-io]\nxattr = { path = ${JSON.stringify(join(host, "vendor", "xattr"))} }\n` +
      `gpui = { path = ${JSON.stringify(join(host, "vendor", "gpui"))} }\n`,
  );
  writeFileSync(
    join(directory, "src", "main.rs"),
    "fn main() {\n    svelte_gpui::run(svelte_gpui::Options::embedded(svelte_gpui::embed!()));\n}\n",
  );
  // The host's lock pins the versions GPUI was built with.
  if (!existsSync(join(directory, "Cargo.lock"))) cpSync(join(host, "Cargo.lock"), join(directory, "Cargo.lock"));
}

function buildExecutable(app, metadata, work, embedDirectory) {
  const environment = { SVELTE_GPUI_EMBED_DIR: embedDirectory };
  if (metadata.crate !== undefined) {
    const features = metadata.crate.features ?? [];
    const manifest = join(app, metadata.crate.manifest ?? "Cargo.toml");
    return cargoBuild(manifest, features.length > 0 ? ["--features", features.join(",")] : [], environment);
  }
  const crate = join(work, "crate");
  generateCrate(crate, metadata);
  return cargoBuild(join(crate, "Cargo.toml"), [], environment);
}

function desktopEntry(metadata) {
  const lines = [
    "[Desktop Entry]",
    "Type=Application",
    `Name=${metadata.name}`,
    ...(metadata.description ? [`Comment=${metadata.description}`] : []),
    `Exec=${metadata.executable}`,
    `Icon=${metadata.id}`,
    "Terminal=false",
    `Categories=${metadata.categories ?? "Utility;"}`,
    // The window's application id is the metadata's id, which is how the desktop finds this file.
    `StartupWMClass=${metadata.id}`,
  ];
  return `${lines.join("\n")}\n`;
}

/** Collects the executable, the .desktop file and the icon in `directory`. */
function collect(app, metadata, executable, directory, options) {
  rmSync(directory, { recursive: true, force: true });
  mkdirSync(directory, { recursive: true });
  const target = join(directory, metadata.executable);
  cpSync(executable, target);
  if (options.strip) spawnSync("strip", [target], { stdio: "inherit" });
  writeFileSync(join(directory, `${metadata.id}.desktop`), desktopEntry(metadata));
  if (metadata.icon !== undefined) {
    const icon = resolve(app, metadata.icon);
    if (!existsSync(icon)) fail(`the icon ${icon} does not exist`);
    cpSync(icon, join(directory, `${metadata.id}${icon.slice(icon.lastIndexOf("."))}`));
  }
}

function makeTarball(directory, metadata, work) {
  const machine = { x64: "x86_64", arm64: "aarch64" }[arch()] ?? arch();
  const name = `${metadata.executable}-${metadata.version}-linux-${machine}`;
  const staging = join(work, "tarball");
  rmSync(staging, { recursive: true, force: true });
  mkdirSync(join(staging, name), { recursive: true });
  for (const file of readdirSync(directory)) {
    if (!file.endsWith(".tar.gz")) cpSync(join(directory, file), join(staging, name, file));
  }
  const tarball = join(directory, `${name}.tar.gz`);
  run("tar", ["-czf", tarball, "-C", staging, name]);
  return tarball;
}

function size(path) {
  const bytes = statSync(path).size;
  return bytes >= 1 << 20 ? `${(bytes / 1048576).toFixed(1)} MB` : `${(bytes / 1024).toFixed(1)} KB`;
}

const options = parseArguments(process.argv.slice(2));
const metadata = readMetadata(options.app);
const work = join(root, "target", "package", basename(options.app));
const embedDirectory = join(work, "embed");
const output = join(options.out, basename(options.app));

mkdirSync(work, { recursive: true });
const bundle = buildBundle(options.app, metadata);
writeEmbedded(embedDirectory, metadata, bundle);
const executable = buildExecutable(options.app, metadata, work, embedDirectory);
collect(options.app, metadata, executable, output, options);
const tarball = options.tar ? makeTarball(output, metadata, work) : undefined;

console.log(`\npackaged ${metadata.name} ${metadata.version}`);
for (const file of readdirSync(output)) console.log(`  ${join(relative(root, output), file)}  ${size(join(output, file))}`);
if (tarball === undefined) console.log("  (no tarball)");
