# Development & CI/CD

## Local Development

```bash
cd app
npm install
npm run tauri:dev    # Runs both Vite dev server + Tauri window
```

Requires:
- Node.js 22+
- Rust (install via rustup.rs)
- macOS: Xcode Command Line Tools
- Windows: Visual Studio Build Tools with C++ workload

## CI/CD Pipeline

### On every push/PR to main (`ci.yml`):
- **Frontend**: TypeScript check, lint, Vite build
- **Backend**: Cargo check, Clippy warnings, format check

### On version tag (`build-release.yml`):
- Builds installers for:
  - macOS ARM (Apple Silicon) — `.dmg`
  - macOS x64 (Intel) — `.dmg`
  - Windows x64 — `.msi` + `.exe`
- Creates a draft GitHub Release with all installers attached

### How to publish a release:

```bash
# 1. Bump version in app/src-tauri/tauri.conf.json and app/package.json
# 2. Commit the version bump
git add -A && git commit -m "release: v0.1.0"

# 3. Tag and push
git tag v0.1.0
git push origin main --tags
```

This triggers the build workflow. After it finishes (~10-15 min):
1. Go to GitHub → Releases
2. Find the draft release
3. Edit release notes if needed
4. Click "Publish release"

Users can then download the installer for their platform from the Releases page.

## Code Signing (Optional but Recommended)

Without signing, users get "unidentified developer" warnings on Mac and SmartScreen warnings on Windows.

### macOS Signing

Add these GitHub repository secrets:

| Secret | Description |
|--------|-------------|
| `APPLE_CERTIFICATE` | Base64-encoded `.p12` Developer ID certificate |
| `APPLE_CERTIFICATE_PASSWORD` | Password for the .p12 file |
| `APPLE_SIGNING_IDENTITY` | e.g. "Developer ID Application: Your Name (TEAMID)" |
| `APPLE_ID` | Your Apple ID email |
| `APPLE_PASSWORD` | App-specific password (not your Apple ID password) |
| `APPLE_TEAM_ID` | Your Apple Developer Team ID |
| `KEYCHAIN_PASSWORD` | Any random string (for the temp CI keychain) |

To get a Developer ID certificate:
1. Enroll in [Apple Developer Program](https://developer.apple.com/programs/) ($99/year)
2. In Xcode → Settings → Accounts → Manage Certificates
3. Create a "Developer ID Application" certificate
4. Export as .p12 and base64-encode it: `base64 -i certificate.p12 | pbcopy`

### Windows Signing

For Windows EV code signing (removes SmartScreen warnings):
1. Purchase an EV code signing certificate from a CA (DigiCert, Sectigo, etc.)
2. Set `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` secrets

Without signing, the app still works — users just need to click "More info" → "Run anyway" on the SmartScreen prompt.

## Auto-Updates (Future)

Tauri has built-in updater support. When ready:
1. Add `tauri-plugin-updater` to the Rust dependencies
2. Configure the update endpoint in `tauri.conf.json`
3. Host update manifests alongside releases (or use a simple JSON endpoint)

Users would then get in-app update notifications for new versions.
