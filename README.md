`envx` is a local-first CLI for working with `.env` files in Rust projects.
It covers the basic file operations, encryption support, diffing, template expansion, and shell completions.

## Features

- `init` creates a starter `.env` and updates `.gitignore`
- `list`, `get`, `set`, and `delete` manage keys
- `encrypt` and `decrypt` handle `.env.age`
- `diff` compares env files without printing values
- `view` opens an interactive TUI to view .env files
- `check` verifies `.env` against `.env.example`
- `merge` applies `.env.local` or another file on top of `.env`
- `expand` resolves `${VAR}` and `$VAR` references inside values
- `completions` generates bash, zsh, or fish completion scripts

## Build

```bash
cargo build
```

For a release binary:

```bash
cargo build --release
```

## Install

### Option 1: Cargo (Recommended)

If you have the Rust toolchain installed, you can build and install directly with Cargo:

```bash
cargo install local-envx 
```

### Option 2: Install Script

Use the included release script to compile the binary and copy it into your preferred directory (defaults to `~/.local/bin` if not specified):

```bash
INSTALL_DIR="$HOME/.local/bin" bash scripts/install.sh
```

## Usage

```bash
envx init
envx list
envx view
envx get DATABASE_URL
envx set API_KEY secret-value
envx delete API_KEY
envx diff .env .env.example
envx check
envx encrypt --delete
envx decrypt
```

### Smart Environment Handling & Security

`envx` has several intelligent behaviors designed for both seamless workflow and strict security:

- **Auto-Decryption for Reads**: If a plaintext `.env` is missing but an encrypted `.env.age` file exists, all read commands (`view`, `list`, `get`, `check`, `expand`, `diff`) will automatically detect it, prompt for your passphrase if needed, and decrypt it entirely in-memory. This means you can interact with your encrypted environments natively without ever risking writing a plaintext file to disk.
- **Write Guards**: If `.env` is missing but `.env.age` exists, modifying commands (`set`, `delete`, `merge`) will block and instruct you to decrypt the file first. This prevents you from accidentally creating a desynced plaintext file while your encrypted file is ignored.
- **Masked by Default**: `envx list` masks all secret values by default (`KEY=***`) to prevent shoulder-surfing. Use `envx list --reveal` to interactively unmask them.

### Advanced features

Merge a local override file into `.env`:

```bash
envx merge .env.local
```

Expand template references in a file:

```bash
envx expand .env --output .env.expanded
```

Template expansion resolves references in the same document, so values like these work:

```dotenv
HOST=localhost
PORT=5432
DATABASE_URL=postgres://${HOST}:${PORT}/app
```

### Shell completions

Generate a completion script for your shell:

```bash
envx completions bash
envx completions zsh
envx completions fish
```

Redirect the output into the location used by your shell completion system.

## Release Notes

The repository is set up for release builds through `cargo build --release` and the install script above.
For tagged releases, build the release binary and publish the generated artifact alongside the source tag.
