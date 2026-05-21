# lsuser

List system users in a clean, columnar layout.

Most operating systems do not provide a straightforward, consistent API for enumerating local user accounts. `lsuser` solves this by using raw POSIX `libc` calls (`getpwent`/`setpwent`/`endpwent`) to read the password database and present users in a readable table — or structured JSON for scripting.

## Features

- **Cross-platform** — Works on macOS, Linux, and other Unix-likes via libc.
- **Smart defaults** — Filters out system accounts by default (UID < 500 on macOS, UID < 1000 or 65534 on Linux).
- **Show all accounts** — Use `-a` / `--all` to include system daemon users.
- **Customizable columns** — Choose which fields to display: `USER`, `UID`, `GID`, `REAL_NAME`, `HOME`, `SHELL`.
- **JSON output** — Use `-J` / `--json` for machine-readable output.
- **No external commands** — Does not parse `/etc/passwd` directly or shell out to `dscl`/`getent`.

## Installation

```sh
cargo install lsuser
```

Or build from source:

```sh
git clone https://github.com/your-username/lsuser.git
cd lsuser
cargo build --release
cp target/release/lsuser ~/.local/bin/
```

## Usage

```
lsuser [OPTIONS]
```

| Flag                    | Description                                    |
|-------------------------|------------------------------------------------|
| `-a`, `--all`           | List all users including system accounts       |
| `-n`, `--noheadings`    | Omit the header row                            |
| `-o`, `--output`        | Comma-separated columns (USER,UID,GID,REAL_NAME,HOME,SHELL) |
| `-O`, `--output-all`    | Show all available columns                     |
| `-J`, `--json`          | Output as JSON                                 |

### Examples

```sh
lsuser                     # list human users (default columns: USER, UID, HOME, SHELL)
lsuser -a                  # include system accounts
lsuser -o USER,UID,REAL_NAME  # custom columns
lsuser -O -J               # all columns as JSON
```

## Release

CI and releases are automated through one GitHub Actions workflow that runs only for version tags.

1. Add a crates.io token as the repository secret `CARGO_REGISTRY_TOKEN`.
2. Bump the version in `Cargo.toml`.
3. Create and push a matching version tag:

```sh
git tag v0.4.3
git push origin v0.4.3
```

When a version tag is pushed, the workflow validates formatting, clippy, tests, and crate packaging, builds macOS and Linux binaries, then publishes to crates.io and creates a GitHub Release in parallel.
