<p align="center">
  <h1 align="center">degu 🐹</h1>
  <p align="center">Infer JSON Schema, TypeScript, and Zod from sample JSON — stop hand-writing types for undocumented APIs.</p>
</p>

<p align="center">
  <a href="https://github.com/iamkorun/degu/actions/workflows/ci.yml"><img src="https://github.com/iamkorun/degu/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://crates.io/crates/degu"><img src="https://img.shields.io/crates/v/degu.svg" alt="crates.io"></a>
  <a href="https://github.com/iamkorun/degu/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
  <a href="https://github.com/iamkorun/degu/stargazers"><img src="https://img.shields.io/github/stars/iamkorun/degu?style=social" alt="Stars"></a>
  <a href="https://buymeacoffee.com/iamkorun"><img src="https://img.shields.io/badge/Buy%20Me%20a%20Coffee-ffdd00?logo=buy-me-a-coffee&logoColor=black" alt="Buy Me a Coffee"></a>
</p>

---

<!-- TODO: Add demo GIF -->

## The Problem

You hit a new third-party API. No OpenAPI spec, no TypeScript types, just a blob of JSON in the docs — or worse, only a `curl` example. You squint at it, hand-write a `User` interface, miss an optional field, ship a bug in production, repeat forever.

## The Solution

**degu** digs through your sample JSON and infers the full shape. Feed it one file, ten files, or a live `curl` pipe — it merges every observation, figures out which fields are optional, detects nullability, and emits clean types for your language of choice.

Named after the [degu](https://en.wikipedia.org/wiki/Degu) — a small South American rodent famous for digging elaborate burrows.

## Demo

Pipe an API response straight into a TypeScript interface:

```sh
$ curl -s https://api.github.com/users/octocat | degu -f typescript --name User
export interface User {
  login: string;
  id: number;
  node_id: string;
  avatar_url: string;
  gravatar_id: string;
  url: string;
  html_url: string;
  type: string;
  site_admin: boolean;
  name: string;
  company: string;
  ...
}
```

Merge two samples and degu figures out which fields are optional:

```sh
$ cat a.json
{"id": 1, "name": "Ada", "email": "ada@example.com"}

$ cat b.json
{"id": 2, "name": "Grace"}

$ degu -f typescript --name User a.json b.json
export interface User {
  id: number;
  name: string;
  email?: string;
}
```

Or emit a Zod schema:

```sh
$ echo '{"id":42,"name":"Ada","email":null,"tags":["rust","cli"]}' | degu -f zod --name User
import { z } from "zod";

export const User = z.object({
  id: z.number().int(),
  name: z.string(),
  email: z.unknown().nullable(),
  tags: z.array(z.string()),
});
export type User = z.infer<typeof User>;
```

## Quick Start

```sh
cargo install degu
curl -s https://api.example.com/users/1 | degu -f typescript --name User
```

## Installation

### From crates.io

```sh
cargo install degu
```

### From source

```sh
git clone https://github.com/iamkorun/degu.git
cd degu
cargo install --path .
```

### Binary releases

Pre-built binaries for Linux, macOS, and Windows are available on the [Releases](https://github.com/iamkorun/degu/releases) page.

## Usage

### Infer JSON Schema (default)

```sh
degu user.json
```

### Emit a TypeScript interface

```sh
degu -f typescript --name User user.json
```

### Emit a Zod schema

```sh
degu -f zod --name User user.json
```

### Merge multiple samples

Fields present in some samples but missing in others are marked optional:

```sh
degu -f typescript --name User sample1.json sample2.json sample3.json
```

### Read from stdin

```sh
curl -s https://api.example.com/users/1 | degu -f typescript --name User
```

### Strict mode

Treat every observed field as required and set `additionalProperties: false`:

```sh
degu --strict -f json-schema user.json
```

## Flags

| Flag | Short | Description |
|------|-------|-------------|
| `--format <FORMAT>` | `-f` | `json-schema` (default), `typescript`, or `zod` |
| `--strict` | | All fields required, `additionalProperties: false` |
| `--name <NAME>` | | Root type name for TypeScript / Zod output (default: `Root`) |
| `--quiet` | `-q` | Suppress informational stderr output |
| `--help` | `-h` | Print help |
| `--version` | `-V` | Print version |

Pass `-` as a filename to explicitly read from stdin. Omit all file arguments to do the same.

## Features

- **Merge semantics** — feed degu several samples, it detects which fields are optional
- **Nullability detection** — a field that's sometimes `null` becomes `T | null`
- **Type unions** — a field that's sometimes a string and sometimes a number becomes `string | number`
- **Integer vs. float** — integers stay `z.number().int()` in Zod output
- **Nested objects & arrays** — recursive inference all the way down
- **Three output formats** — JSON Schema (Draft 7), TypeScript interfaces, Zod schemas
- **Pipeable** — reads from files or stdin, writes to stdout
- **Strict mode** — for when you want to lock the shape down
- **No panics** — malformed JSON surfaces as a clean error with context
- **Single binary** — written in Rust, no runtime, `cargo install` and go

## Contributing

Contributions are welcome. Please open an issue first to discuss what you'd like to change.

```sh
git clone https://github.com/iamkorun/degu.git
cd degu
cargo test
```

## License

[MIT](LICENSE)

---

## Star History

<a href="https://star-history.com/#iamkorun/degu&Date">
  <img src="https://api.star-history.com/svg?repos=iamkorun/degu&type=Date" alt="Star History Chart" width="600">
</a>

---

<p align="center">
  <a href="https://buymeacoffee.com/iamkorun"><img src="https://cdn.buymeacoffee.com/buttons/v2/default-yellow.png" alt="Buy Me a Coffee" width="200"></a>
</p>
