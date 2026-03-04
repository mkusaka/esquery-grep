# esquery-grep

Grep JavaScript and TypeScript files using [ESQuery](https://github.com/estools/esquery) selectors.

Powered by [oxc](https://github.com/oxc-project/oxc) parser. Distributed as a WASM binary — no native dependencies required.

## Install

```sh
npm install -g esquery-grep
```

Or run directly:

```sh
npx esquery-grep 'src/**/*.ts' 'Identifier'
```

## Usage

```sh
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

## Requirements

- Node.js >= 20.0.0 or Bun

## License

MIT
