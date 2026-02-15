# Run Scripts

Scripts for managing Git History Editor development and deployment.

## Quick Start

### Development
```bash
./run/start.sh          # Start dev server on http://localhost:1420
./run/stop.sh           # Stop dev server
```

### Production
```bash
./run/build.sh          # Build app for current architecture
INSTALL=1 ./run/build.sh # Build and install to ~/Applications
./run/launch.sh         # Launch installed app
```

---

## Scripts

### `start.sh` - Start development server

Starts the Tauri dev environment with hot-reload and devtools.

```bash
./run/start.sh          # Tauri dev mode (recommended)
./run/start.sh --build  # Full production build
```

**Features:**
- Live reload on file changes
- Built-in devtools for debugging
- Accessible at http://localhost:1420

---

### `stop.sh` - Stop all running processes

Kills the dev server, Vite, and any running app instances.

```bash
./run/stop.sh
```

**Kills:**
- Tauri dev processes
- Vite dev server
- Git History Editor app instances

---

### `build.sh` - Build production app

Builds an unsigned .app bundle for macOS.

**Basic build (current architecture):**
```bash
./run/build.sh
```

**Build universal binary (aarch64 + x86_64):**
```bash
UNIVERSAL=1 ./run/build.sh
```

**Build and install to ~/Applications:**
```bash
INSTALL=1 ./run/build.sh
```

**Build, sign, and install:**
```bash
SIGN_IDENTITY="Developer ID Application: Your Name (TEAMID)" INSTALL=1 ./run/build.sh
```

**Environment Variables:**

| Variable | Default | Description |
|----------|---------|-------------|
| `UNIVERSAL` | `0` | Set to `1` for universal binary (slower) |
| `INSTALL` | `0` | Set to `1` to auto-install to ~/Applications |
| `SIGN_IDENTITY` | (none) | Code signing identity for notarization |

---

### `launch.sh` - Launch installed app

Opens the app from ~/Applications.

```bash
./run/launch.sh
```

Requires the app to be built and installed first.

---

## Workflow Examples

### Local Development
```bash
./run/start.sh          # Terminal 1: Start dev server
# ... make changes ...
./run/stop.sh           # Terminal 2: When done
```

### Build for Testing
```bash
./run/build.sh          # Builds to src-tauri/target/release/bundle/macos/
# Result: ~/path/to/.../Git History Editor.app
```

### Install to Applications
```bash
INSTALL=1 ./run/build.sh    # Builds and installs
./run/launch.sh             # Launch it
```

### Sign and Distribute
```bash
SIGN_IDENTITY="Developer ID Application: Your Name (TEAMID)" \
INSTALL=1 \
UNIVERSAL=1 \
./run/build.sh
```

---

## Troubleshooting

### App not found after build
```
ERROR: App not found at /Users/pweiss/Applications/Git History Editor.app
```

The app was built to `src-tauri/target/release/bundle/macos/`. Either:
1. Use `INSTALL=1 ./run/build.sh` to auto-copy to ~/Applications
2. Manually copy: `cp -r src-tauri/target/release/bundle/macos/*.app ~/Applications/`

### Port 1420 already in use
```
./run/stop.sh  # Kill existing dev servers
./run/start.sh # Try again
```

### Code signing issues
For development, you don't need to sign. For distribution, you need:
- Apple Developer Program membership
- "Developer ID Application" certificate in Keychain

List available identities:
```bash
security find-identity -v -p codesigning
```

---

## File Locations

- **Source:** `/Users/pweiss/dev/git-history-editor/`
- **Dev build:** `src-tauri/target/release/bundle/macos/Git History Editor.app`
- **Installed:** `~/Applications/Git History Editor.app`
- **Vite server:** http://localhost:1420
