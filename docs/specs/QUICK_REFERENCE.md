# Pocket Flow: Build & Release Quick Reference

Quick cheat sheet for common operations.

## First-Time Setup

1. **Complete the full spec:** Read `BUILD_AND_RELEASE_SPEC.md` (takes ~1 hour)
2. **Verify everything works:** Test all scripts locally before first release

## Creating a Release

### Quick 3-Step Process

```bash
# 1. Update version in Cargo.toml
# Edit Cargo.toml, change version = "X.Y.Z"

# 2. Commit and push
git add Cargo.toml
git commit -m "chore: bump version to X.Y.Z"
git push origin main

# 3. Create release (automated workflow handles the rest!)
./scripts/release.sh X.Y.Z
```

**Then:**
- ✅ Monitor build: https://github.com/tonomb/pocket_flow/actions
- ✅ Download app: https://github.com/tonomb/pocket_flow/releases
- ✅ Test the DMG

## Common Commands

### Build Locally
```bash
# Build release binary
cargo build --release

# Create .app bundle
./scripts/build-app-bundle.sh 0.1.0

# Create DMG
./scripts/create-dmg.sh 0.1.0
```

### Check Signing Certificate
```bash
security find-identity -v -p codesigning
```

### Test Release Manually
```bash
# Simulate what GitHub Actions does
cargo build --release
./scripts/build-app-bundle.sh 0.1.0
./scripts/create-dmg.sh 0.1.0

# Now test the DMG
open target/release/Pocket_Flow_0.1.0.dmg
```

### View GitHub Actions Logs
```bash
# Open in browser
open https://github.com/tonomb/pocket_flow/actions
```

### Download Latest Release
```bash
# Go to releases page
open https://github.com/tonomb/pocket_flow/releases
```

## Troubleshooting Quick Fixes

| Problem | Fix |
|---------|-----|
| "security find-identity" returns 0 | Check Xcode → Accounts → Developer ID certs |
| DMG won't open on another Mac | Check Gatekeeper: System Prefs → Security & Privacy |
| App won't launch from DMG | `sudo xattr -d com.apple.quarantine /Applications/Pocket\ Flow.app` |
| Build fails locally | Run `cargo build --release` first |
| Tag already exists | Use different version or: `git tag -d v0.1.0 && git push origin :refs/tags/v0.1.0` |

## Version Numbering

**Format:** `MAJOR.MINOR.PATCH`

- `0.1.0` → Initial dev release
- `0.2.0` → New feature (bump MINOR)
- `0.2.1` → Bug fix (bump PATCH)
- `1.0.0` → First stable release

## GitHub Secrets Checklist

Must be set in GitHub Settings → Secrets and Variables → Actions:

- [ ] `MACOS_CERTIFICATE` (base64 encoded .p12 file)
- [ ] `MACOS_CERTIFICATE_PWD` (certificate password)
- [ ] `DEVELOPER_ID_APPLICATION` (full cert name, e.g., "Developer ID Application: Name (TEAMID)")

## Release Checklist

Before running `./scripts/release.sh X.Y.Z`:

- [ ] All changes committed and pushed to `main`
- [ ] Version updated in `Cargo.toml`
- [ ] No uncommitted changes: `git status` is clean
- [ ] Version tag doesn't already exist: `git tag | grep vX.Y.Z` is empty

## GitHub Actions Workflow Stages

When you push a tag, GitHub Actions automatically:

1. ✅ **Checkout** code
2. ✅ **Build** release binary with Rust
3. ✅ **Create** macOS .app bundle
4. ✅ **Code Sign** with Developer ID
5. ✅ **Create** DMG installer
6. ✅ **Release** on GitHub (users can download DMG)

Total time: ~5-10 minutes

## File Structure After Setup

```
.github/workflows/
└── build-and-release.yml       ← GitHub Actions workflow

scripts/
├── build-app-bundle.sh         ← Create .app bundle
├── create-dmg.sh               ← Create DMG from .app
└── release.sh                  ← Helper to tag and push

docs/specs/
├── BUILD_AND_RELEASE_SPEC.md   ← Full detailed guide
└── QUICK_REFERENCE.md          ← This file

Cargo.toml                       ← Update version here before release
```

## Helpful Links

- **Releases:** https://github.com/tonomb/pocket_flow/releases
- **Actions:** https://github.com/tonomb/pocket_flow/actions
- **Settings:** https://github.com/tonomb/pocket_flow/settings/secrets/actions
- **Apple Code Signing:** https://help.apple.com/xcode/mac/current/#/dev54d690f09

---

For detailed step-by-step instructions, see `BUILD_AND_RELEASE_SPEC.md`
