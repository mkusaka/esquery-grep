# esquery-grep

A Rust implementation of [ESQuery](https://github.com/estools/esquery) — CSS-like selector syntax for querying JavaScript and TypeScript ASTs.

Parses source code with [oxc](https://github.com/oxc-project/oxc), serializes to ESTree JSON, and matches against ESQuery selectors. Available as both a Rust library and a Node.js native module (via NAPI).

## Crate Structure

| Crate | Description |
|-------|-------------|
| `esquery-selector` | ESQuery selector parser (winnow) |
| `esquery-json` | Matcher for `serde_json::Value` ESTree ASTs |
| `esquery-rs` | High-level API: source code → parse → query |
| `esquery-napi` | Node.js bindings via napi-rs |

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

### Node.js

```js
const { query } = require('esquery-rs');

const results = query('var x = 1 + 2;', 'BinaryExpression');
console.log(results);
// => [{ type: 'BinaryExpression', start: 8, end: 13, text: '1 + 2' }]
```

TypeScript types are auto-generated:

```ts
interface NapiMatchResult {
  type: string;
  start: number; // UTF-16 offset
  end: number;   // UTF-16 offset
  text: string;
}

function query(
  source: string,
  selector: string,
  sourceType?: 'js' | 'jsx' | 'ts' | 'tsx',
): NapiMatchResult[];
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

### Node.js (NAPI)

```sh
cd crates/esquery-napi
npm install
npm run build
```

## Known Limitations

- TypeScript-specific fields (e.g., `typeAnnotation`) are not traversed because the matcher uses estraverse visitor keys, which only cover standard ESTree node types. TS-specific top-level declarations (e.g., `TSInterfaceDeclaration`) are still found.
- The NAPI module converts UTF-8 byte offsets to UTF-16 code unit offsets for JavaScript compatibility. Rust API returns UTF-8 byte offsets.

## License

MIT
