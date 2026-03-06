#!/usr/bin/env node

import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { dirname, resolve, relative, isAbsolute } from "node:path";
import { parseArgs } from "node:util";
import { WASI } from "node:wasi";

const __dirname = dirname(fileURLToPath(import.meta.url));
const wasmPath = resolve(__dirname, "..", "eg.wasm");
const isBun = typeof Bun !== "undefined";

// Normalize the first positional file pattern argument for WASI compatibility.
// Other arguments (selector, flags) are passed through as-is.
// - Bun: preopens {"/":"/"} doubles absolute paths (oven-sh/bun#27724),
//   so convert to relative from CWD with preopens {".":"."}
// - Node: preopens {"/":"/"} works, but WASI guest CWD is "/" so relative
//   paths must be resolved to absolute
const cwd = process.cwd();
const userArgs = process.argv.slice(2);
const wasiArgs = [...userArgs];
const { tokens } = parseArgs({
  args: wasiArgs,
  options: {
    type: {
      type: "string",
      short: "t",
    },
  },
  allowPositionals: true,
  strict: false,
  tokens: true,
});
const patternArgIndex = tokens.find((token) => token.kind === "positional")?.index ?? -1;
if (patternArgIndex >= 0) {
  const pattern = wasiArgs[patternArgIndex];
  if (isBun) {
    wasiArgs[patternArgIndex] = isAbsolute(pattern)
      ? relative(cwd, pattern)
      : pattern;
  } else {
    wasiArgs[patternArgIndex] = isAbsolute(pattern)
      ? pattern
      : resolve(cwd, pattern);
  }
}

const wasi = new WASI({
  version: "preview1",
  args: ["eg", ...wasiArgs],
  preopens: isBun ? { ".": "." } : { "/": "/" },
});

const buf = await readFile(wasmPath);
const mod = await WebAssembly.compile(buf);
// Bun lacks getImportObject() (oven-sh/bun#27204)
const imports =
  typeof wasi.getImportObject === "function"
    ? wasi.getImportObject()
    : { wasi_snapshot_preview1: wasi.wasiImport };
const instance = await WebAssembly.instantiate(mod, imports);
const exitCode = wasi.start(instance);
if (exitCode != null) process.exitCode = exitCode;
