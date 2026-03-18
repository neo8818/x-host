# x-hosts

`x-hosts` is a cross-platform desktop hosts file editor built with Tauri 2, TypeScript, and Rust. It combines structured entry management and direct raw editing in a desktop app designed for local hosts maintenance.

## Features

- Table view for enabling, disabling, adding, and removing hosts entries
- Raw text editor for direct hosts file editing
- Automatic backup creation before write operations
- In-app backup restore workflow
- DNS cache flush tools for supported platforms
- Remote sync using a user-specified hosts source URL
- Cross-platform desktop packaging for Windows, macOS, and Linux through GitHub Actions

## Tech stack

- Tauri 2
- TypeScript
- Vite
- Rust

## Prerequisites

- Node.js 20 or newer
- Rust stable toolchain
- Platform-specific Tauri system dependencies for your target OS

## Local development

Install dependencies:

```bash
npm ci
```

Start the Vite frontend development server:

```bash
npm run dev
```

Start the desktop app in Tauri development mode:

```bash
npm run tauri:dev
```

Build the frontend assets locally:

```bash
npm run build
```

Build a local desktop production bundle:

```bash
npm run tauri:build
```

`src-tauri/tauri.conf.json` runs `npm run dev` before Tauri development builds and `npm run build` before production bundles.

## Privileges and runtime notes

- Editing the real system hosts file typically requires administrator or root privileges.
- The application detects the default hosts file path per platform.
- `XHOSTS_HOSTS_PATH` can override the hosts file path.
- `XHOSTS_BACKUP_DIR` can override the backup directory.
- `XHOSTS_LOG_DIR` can override the diagnostic log directory.
- DNS cache flushing is best-effort on Linux because the required command depends on the local distribution and resolver setup.

## Remote sync

The built-in default remote sync URL is intentionally blank. `x-hosts` does not ship with a preconfigured GitHub or raw content source. To use remote sync, provide your own compatible URL inside the app.

## Release workflow

The repository includes `.github/workflows/build.yml` for cross-platform desktop packaging.

It can be triggered in two ways:

1. Manually with **Run workflow** in GitHub Actions.
2. Automatically by pushing a version tag such as `v0.1.0`.

The workflow builds and uploads desktop bundles for:

- Windows
- Linux
- macOS Apple Silicon
- macOS Intel

Before publishing a release locally, run at least these checks:

```bash
npm ci
npm run build
npm run tauri:build
```

The GitHub Actions release job creates a draft release and attaches the generated artifacts.

## Project structure

- `src/main.ts` - frontend UI logic and app state
- `src/style.css` - application styling
- `src-tauri/src/` - Rust backend commands and filesystem operations
- `src-tauri/tauri.conf.json` - Tauri app and bundle configuration
- `.github/workflows/build.yml` - CI packaging and release automation
- `scripts/` - local helper scripts

## License

This project is released under the [MIT License](LICENSE).
