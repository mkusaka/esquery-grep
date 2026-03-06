# esquery-grep

A Rust implementation of [ESQuery](https://github.com/estools/esquery) — CSS-like selector syntax for querying JavaScript and TypeScript ASTs.

Parses source code with [oxc](https://github.com/oxc-project/oxc), serializes to ESTree JSON, and matches against ESQuery selectors. Available as a CLI tool and a Rust library.

## CLI

```sh
# Install via npm
npm install -g esquery-grep

# Or via Cargo
cargo install --path crates/esquery-grep

# Find all identifiers in TypeScript files
eg 'src/**/*.ts' 'Identifier'

# Find binary expressions with + operator
eg 'src/**/*.js' 'BinaryExpression[operator="+"]'

# Find functions containing return statements
eg 'lib/**/*.tsx' 'FunctionDeclaration:has(ReturnStatement)'

# Force source type instead of inferring from extension
eg 'scripts/*' 'Identifier' --type ts
```

Output is grep-compatible (`path:line:col: text`):

```
src/index.ts:3:7: foo
src/index.ts:5:10: bar
```

Exit code is `0` when matches are found, `1` otherwise.

## Package Structure

| Package | Description |
|---------|-------------|
| [`npm`](npm/README.md) | npm package (WASM distribution) |
| `crates/esquery-grep` | CLI tool (`eg` binary) |
| `crates/esquery-selector` | ESQuery selector parser (winnow) |
| `crates/esquery-json` | Matcher for `serde_json::Value` ESTree ASTs |
| `crates/esquery-rs` | High-level API: source code → parse → query |

## Usage

### Rust

```rust
use esquery_rs::{query, JsSourceType};

let results = query("var x = 1 + 2;", "BinaryExpression", JsSourceType::Js);
for m in &results {
    println!("{}: {} ({}..{})", m.node_type, m.text, m.start, m.end);
}
// => BinaryExpression: 1 + 2 (8..13)
```


## Supported Selectors

| Selector | Example | Description |
|----------|---------|-------------|
| Node type | `Identifier` | Match by ESTree node type |
| Wildcard | `*` | Match any node |
| Attribute | `[name="x"]` | Match by node property |
| Attribute (comparison) | `[value>=10]` | `=`, `!=`, `>`, `>=`, `<`, `<=` |
| Attribute (regex) | `[name=/^on/]` | Match property with regex |
| Attribute (type) | `[value=type(number)]` | Match by value type |
| Descendant | `Function Identifier` | Ancestor-descendant relationship |
| Child | `Function > BlockStatement` | Direct parent-child |
| Sibling | `Decl ~ Decl` | General sibling |
| Adjacent | `Decl + Decl` | Immediately adjacent sibling |
| Class | `:statement`, `:expression` | AST node classification |
| `:has()` | `Function:has(Return)` | Contains matching descendant |
| `:not()` | `Literal:not([value=1])` | Negation |
| `:matches()` | `:matches(If, While)` | Union / or |
| `:nth-child()` | `:nth-child(2)` | Positional match |
| `:nth-last-child()` | `:nth-last-child(1)` | Positional from end |
| Field | `.value` | Match by parent field name |
| Compound | `Identifier[name="x"]` | Multiple conditions |
| Subject (`!`) | `!Function > Return` | Mark subject of match |

## Building

### Rust

```sh
cargo build --workspace
cargo test --workspace
```

### npm WASM artifact

`npm/eg.wasm` is built with Binaryen's `wasm-opt`, which must be available on `PATH`.
The CLI version defaults to the Rust crate version in `crates/esquery-grep/Cargo.toml`.
Set `EG_VERSION` if you want the built WASM to report a different version.

Local run steps:

1. Install `wasm-opt` from a Binaryen release, or otherwise make it available on `PATH`.
2. Confirm the command is available with `wasm-opt --version`.
3. Run the build script from the repository root.

Example using the same Binaryen release as CI:

```sh
BINARYEN_VERSION=version_125
archive="binaryen-${BINARYEN_VERSION}-<platform>.tar.gz"
curl -fsSLO "https://github.com/WebAssembly/binaryen/releases/download/${BINARYEN_VERSION}/${archive}"
tar -xzf "${archive}"
export PATH="$PWD/binaryen-${BINARYEN_VERSION}/bin:$PATH"
./scripts/build-wasm.sh
```

Override the version embedded in the WASM artifact:

```sh
EG_VERSION=0.1.5 ./scripts/build-wasm.sh
```

Smoke test after the build:

```sh
node npm/bin/eg.mjs --version
node npm/bin/eg.mjs 'crates/esquery-grep/tests/fixtures/app.js' 'Identifier'
```

## Known Limitations

- TypeScript-specific fields (e.g., `typeAnnotation`) are not traversed because the matcher uses estraverse visitor keys, which only cover standard ESTree node types. TS-specific top-level declarations (e.g., `TSInterfaceDeclaration`) are still found.

## License

MIT
