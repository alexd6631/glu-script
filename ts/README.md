# glu-script-ts (experiment)

A TypeScript port of the glu-script interpreter. This is an experiment alongside the Rust/napi implementation — the Rust sources and the published npm entrypoints are untouched.

## Design

The port mirrors the Rust architecture module by module (`scanner` → `parser` → `compiler` → `evaluator` → `stdlib`), with one deliberate difference: **all data models (AST, values, tokens) are tagged unions** (discriminated object unions with a `tag` field) rather than TS `enum`s, which gives exhaustiveness checking via `switch` and keeps the shape close to Rust's `enum` semantics.

Behavior is ported faithfully (same error messages, truthiness rules, 256-frame call-depth limit, stdlib set: `hello`, `map`, `filter`, `fold`, `reduce`, `any`, `every`, `find`), with small ergonomic adaptations for JS consumers:

- evaluation errors are thrown as `GluEvalError` instead of returned as `Result`
- parse failures throw `GluParseError`
- the public API converts values to plain JS (`toJs`), matching `lib.rs`

## Usage

```ts
import { evaluate, parse, scan } from './ts/src/index'

evaluate('{ let x = 5; x * 2 }') // 10
evaluate('map([1, 2, 3], |x| x * 2)') // [2, 4, 6]
```

## Dev

From the repository root (reuses the existing toolchain: `typescript`, `ava`, `oxlint`, `oxfmt`, `@oxc-node/core`):

```sh
cd ts && ../node_modules/.bin/tsc --noEmit        # typecheck
OXC_TSCONFIG_PATH=./ts/tsconfig.json ./node_modules/.bin/ava ts/__test__/index.spec.ts  # tests
./node_modules/.bin/oxlint ts/                    # lint
./node_modules/.bin/oxfmt --check ts/             # format
```
