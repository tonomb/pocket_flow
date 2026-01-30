# Build & Release Specifications

This directory contains comprehensive documentation for setting up automated builds and releases of Pocket Flow for macOS.

## Documents

### 1. **BUILD_AND_RELEASE_SPEC.md** (Main Guide - START HERE)
**Type:** Complete Implementation Guide
**Length:** ~1,100 lines
**Audience:** Junior developers, first-time implementers

**What it covers:**
- Complete architecture overview
- Step-by-step instructions for all 6 phases
- Code signing certificate setup
- GitHub Actions workflow creation
- Build script creation
- First release walkthrough
- Troubleshooting guide (Appendix A)
- File checklist (Appendix B)

**When to use:**
- First time setting up the build pipeline
- Understanding how the system works
- Detailed troubleshooting

**Time to complete:** 1-2 hours (includes testing)

---

### 2. **QUICK_REFERENCE.md** (Cheat Sheet)
**Type:** Quick Reference Guide
**Length:** ~150 lines
**Audience:** Developers familiar with the system

**What it covers:**
- Quick 3-step release process
- Common commands
- Quick troubleshooting table
- Version numbering guide
- GitHub secrets checklist
- File structure overview

**When to use:**
- After setup is complete
- Creating releases (bookmark this!)
- Quick command lookup
- Fast troubleshooting

**Time to reference:** 30 seconds - 2 minutes

---

### 3. **IMPLEMENTATION_CHECKLIST.md** (Progress Tracker)
**Type:** Step-by-Step Checklist
**Length:** ~200 lines
**Audience:** Anyone implementing the spec

**What it covers:**
- Checkbox for each task in the spec
- Organized by part and task
- Verification steps
- Status tracking
- Reference commands for each step

**When to use:**
- Track progress through setup
- Ensure nothing is missed
- Mark completion
- Archive when done

---

## Getting Started

### For First-Time Setup

1. **Start with:** `BUILD_AND_RELEASE_SPEC.md`
   - Read overview
   - Follow Part 1: Code Signing (30 min)
   - Follow Part 2: Build Scripts (20 min)
   - Follow Part 3: GitHub Actions (10 min)
   - Follow Part 4: Release Script (5 min)
   - Follow Part 5: First Release (15 min + waiting)
   - Follow Part 6: Understand ongoing process (10 min)

2. **Track progress with:** `IMPLEMENTATION_CHECKLIST.md`
   - Check off each task as completed
   - Use reference commands to verify

3. **Test everything:** Create first test release before going live

---

### For Ongoing Development

1. **Bookmark:** `QUICK_REFERENCE.md`
   - Use for all release commands
   - Quick lookup for troubleshooting

2. **Standard release process:**
   ```bash
   # 1. Make changes and commit
   git add .
   git commit -m "feat: add new feature"
   
   # 2. Update version in Cargo.toml
   # Edit: version = "0.2.0"
   git add Cargo.toml
   git commit -m "chore: bump version to 0.2.0"
   git push origin main
   
   # 3. Create release (GitHub Actions handles rest!)
   ./scripts/release.sh 0.2.0
   ```

3. **Done!** App automatically builds and releases on GitHub

---

## Key Concepts

### Architecture

```
Developer: Makes changes, pushes git tag
    ↓
GitHub Actions: Automatically triggered by tag
    ↓
Workflow steps:
  1. Build Rust binary (cargo build --release)
  2. Create .app bundle (proper macOS format)
  3. Code sign with Developer ID
  4. Create DMG installer
  5. Upload to GitHub Releases
    ↓
Users: Download DMG from releases page and install
```

### What Gets Created

After setup, the following files are created:

```
.github/workflows/build-and-release.yml   ← Automated build workflow
scripts/build-app-bundle.sh               ← Creates .app bundle
scripts/create-dmg.sh                     ← Creates DMG installer
scripts/release.sh                        ← Helper to create releases
```

### Version Numbering

Uses Semantic Versioning (MAJOR.MINOR.PATCH):

- `0.1.0` → Initial development
- `0.2.0` → New feature (bump MINOR)
- `0.2.1` → Bug fix (bump PATCH)
- `1.0.0` → First stable release

---

## Prerequisites

You must have:

- ✅ macOS 11.0 (Big Sur) or later
- ✅ Xcode Command Line Tools (`xcode-select --install`)
- ✅ Rust (`rustup`)
- ✅ GitHub account with repo access
- ✅ Apple Developer account (for code signing)

---

## The Complete Release Process

### Step 1: Code Signing Setup (Part 1 - 30 min)
- Create Developer ID certificate in Xcode
- Export certificate to .p12 file
- Store securely in GitHub Secrets

### Step 2: Build Scripts (Part 2 - 20 min)
- Create `build-app-bundle.sh` - packages code as .app
- Create `create-dmg.sh` - creates DMG installer

### Step 3: GitHub Actions (Part 3 - 10 min)
- Create workflow file that runs on git tag
- Workflow automates: build → sign → package → release

### Step 4: Release Helper (Part 4 - 5 min)
- Create `release.sh` script for easy release creation
- Makes releasing as simple as: `./scripts/release.sh 0.2.0`

### Step 5: Test First Release (Part 5 - 30 min + waiting)
- Create first release tag
- Monitor GitHub Actions build (5-10 min)
- Download and test the DMG

### Step 6: Ongoing Process (Part 6 - ongoing)
- For each new release: update version, commit, run release script
- Everything else is automated!

---

## Common Tasks

### Create a Release
```bash
# 1. Update version in Cargo.toml
# 2. Commit and push
git add Cargo.toml
git commit -m "chore: bump version to 0.2.0"
git push origin main

# 3. Create release
./scripts/release.sh 0.2.0

# 4. Wait 5-10 minutes for GitHub Actions to build
# 5. Download DMG from releases page
```

### Check Certificate
```bash
security find-identity -v -p codesigning
```

### Monitor a Build
```
https://github.com/tonomb/pocket_flow/actions
```

### Download a Release
```
https://github.com/tonomb/pocket_flow/releases
```

---

## Troubleshooting

### Common Issues

| Issue | Quick Fix |
|-------|-----------|
| "security find-identity" returns 0 | Check Xcode → Accounts → Manage Certificates |
| GitHub Actions fails on signing | Verify 3 secrets in GitHub Settings |
| DMG won't open on another Mac | Check Apple Gatekeeper security settings |
| Build script says "Binary not found" | Run `cargo build --release` first |
| "Tag already exists" error | Use different version or delete old tag |

### Detailed Help

For comprehensive troubleshooting, see:
- **BUILD_AND_RELEASE_SPEC.md** → Appendix A (Common Issues)
- **QUICK_REFERENCE.md** → Troubleshooting section

---

## File Organization

```
docs/specs/
├── README.md                          ← You are here
├── BUILD_AND_RELEASE_SPEC.md          ← Full detailed guide (START HERE)
├── QUICK_REFERENCE.md                 ← Cheat sheet (bookmark this!)
└── IMPLEMENTATION_CHECKLIST.md        ← Progress tracker

scripts/ (created during setup)
├── build-app-bundle.sh                ← Creates .app bundle
├── create-dmg.sh                      ← Creates DMG
└── release.sh                         ← Release helper

.github/workflows/ (created during setup)
└── build-and-release.yml              ← GitHub Actions workflow
```

---

## Useful Links

| Link | Purpose |
|------|---------|
| https://github.com/tonomb/pocket_flow/releases | Download releases |
| https://github.com/tonomb/pocket_flow/actions | View build status |
| https://github.com/tonomb/pocket_flow/settings/secrets/actions | Manage GitHub secrets |
| https://help.apple.com/xcode/mac/current/#/dev54d690f09 | Apple code signing guide |

---

## Questions?

### For Implementation Questions
- Check **BUILD_AND_RELEASE_SPEC.md** first
- Read the specific section for your issue

### For Quick Lookup
- Use **QUICK_REFERENCE.md**
- Find your command in the "Common Commands" section

### For Tracking Progress
- Use **IMPLEMENTATION_CHECKLIST.md**
- Check off completed tasks

### Still Stuck?
- See **BUILD_AND_RELEASE_SPEC.md** → Appendix A
- Check GitHub Actions logs: https://github.com/tonomb/pocket_flow/actions

---

## Summary

What you'll have after completing these specs:

✅ Automated builds on every git tag
✅ Code signed DMG installers
✅ Professional GitHub Releases
✅ Users can download directly without running cargo
✅ One-command release process: `./scripts/release.sh X.Y.Z`
✅ Complete documentation for future developers

**Estimated Setup Time:** 1-2 hours
**Estimated Release Time (after setup):** 5-10 minutes + any waiting

---

**Document Version:** 1.0
**Last Updated:** January 30, 2025
**Status:** Ready for Implementation
