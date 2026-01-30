# Build & Release Implementation Checklist

Use this checklist to track your progress through the BUILD_AND_RELEASE_SPEC.md setup.

---

## Part 1: Code Signing Certificate Setup

### Task 1.1: Create Apple Developer ID Certificate

- [ ] Open Xcode Preferences
- [ ] Add Apple ID (or verify existing)
- [ ] Download Development Certificate
  - [ ] Create new "Developer ID Application" certificate
  - [ ] Wait for generation
- [ ] Verify certificate installed:
  ```bash
  security find-identity -v -p codesigning
  ```
- [ ] Record certificate identity string:
  - [ ] Save the output somewhere safe
  - [ ] Note the format: `"Developer ID Application: Name (TEAMID)"`

### Task 1.2: Create GitHub Repository Secrets

- [ ] Export certificate to .p12 file
  - [ ] Open Keychain Access
  - [ ] Find "Developer ID Application"
  - [ ] Right-click → Export
  - [ ] Save as `developer-id.p12`
  - [ ] Set a memorable password

- [ ] Encode certificate as Base64:
  ```bash
  base64 -i ~/Desktop/developer-id.p12 | pbcopy
  ```

- [ ] Add GitHub Secrets (https://github.com/tonomb/pocket_flow/settings/secrets/actions):
  - [ ] `MACOS_CERTIFICATE` = Base64 encoded .p12 file
  - [ ] `MACOS_CERTIFICATE_PWD` = The password you entered
  - [ ] `DEVELOPER_ID_APPLICATION` = Full cert name (e.g., "Developer ID Application: Antonio Martinez (ABC123DE45)")

- [ ] Verify secrets added (should see 3 secrets listed)

- [ ] Clean up temporary file:
  ```bash
  rm ~/Desktop/developer-id.p12
  ```

---

## Part 2: Create Build Scripts

### Task 2.1: Create App Bundle Build Script

- [ ] Create directory: `scripts/`
- [ ] Create file: `scripts/build-app-bundle.sh`
- [ ] Copy content from spec (section "Task 2.1")
- [ ] Make executable:
  ```bash
  chmod +x scripts/build-app-bundle.sh
  ```
- [ ] Test locally:
  ```bash
  cargo build --release
  ./scripts/build-app-bundle.sh 0.1.0
  ```
- [ ] Verify output:
  ```bash
  ls -la target/release/Pocket\ Flow.app/
  ```

### Task 2.2: Create DMG Creation Script

- [ ] Create file: `scripts/create-dmg.sh`
- [ ] Copy content from spec (section "Task 2.2")
- [ ] Make executable:
  ```bash
  chmod +x scripts/create-dmg.sh
  ```
- [ ] Install create-dmg tool:
  ```bash
  brew install create-dmg
  ```
- [ ] Test locally:
  ```bash
  ./scripts/create-dmg.sh 0.1.0
  ```
- [ ] Verify DMG created:
  ```bash
  ls -lh target/release/Pocket_Flow_0.1.0.dmg
  ```

---

## Part 3: Create GitHub Actions Workflow

### Task 3.1: Create GitHub Actions Workflow File

- [ ] Create directory: `.github/workflows/`
- [ ] Create file: `.github/workflows/build-and-release.yml`
- [ ] Copy content from spec (section "Task 3.1")
- [ ] Verify file created:
  ```bash
  cat .github/workflows/build-and-release.yml | head -20
  ```

### Task 3.2: Verify GitHub Secrets

- [ ] Go to: https://github.com/tonomb/pocket_flow/settings/secrets/actions
- [ ] Verify all 3 secrets exist:
  - [ ] `MACOS_CERTIFICATE`
  - [ ] `MACOS_CERTIFICATE_PWD`
  - [ ] `DEVELOPER_ID_APPLICATION`

---

## Part 4: Create Release Script

### Task 4.1: Create Release Helper Script

- [ ] Create file: `scripts/release.sh`
- [ ] Copy content from spec (section "Task 4.1")
- [ ] Make executable:
  ```bash
  chmod +x scripts/release.sh
  ```
- [ ] Test help output (should not crash):
  ```bash
  ./scripts/release.sh
  ```

---

## Part 5: Testing and First Release

### Task 5.1: Update Cargo.toml Version

- [ ] Open `Cargo.toml`
- [ ] Verify version is set (should be "0.1.0" or similar):
  ```bash
  grep '^version = ' Cargo.toml
  ```
- [ ] Note the version for the next task

### Task 5.2: Create First Test Release

- [ ] Commit any uncommitted changes:
  ```bash
  git status
  ```
  Should show: `nothing to commit, working tree clean`

- [ ] Run release script:
  ```bash
  ./scripts/release.sh 0.1.0
  ```
  (Or use whatever version is in Cargo.toml)

- [ ] Answer prompt: `Continue with release? (y/n)` → Press `y`

- [ ] Verify tag was pushed:
  ```bash
  git tag | grep v0.1.0
  ```

- [ ] Monitor GitHub Actions:
  - [ ] Go to: https://github.com/tonomb/pocket_flow/actions
  - [ ] Wait for "Build and Release macOS App" workflow to complete
  - [ ] Check each step for green checkmarks
  - [ ] Typical time: 5-10 minutes

- [ ] Verify release created:
  - [ ] Go to: https://github.com/tonomb/pocket_flow/releases
  - [ ] Should see new release `v0.1.0`
  - [ ] Should see DMG file attached

- [ ] Download and test DMG:
  - [ ] Download `Pocket_Flow_v0.1.0.dmg`
  - [ ] Open the DMG file
  - [ ] See Pocket Flow app icon
  - [ ] Drag app to Applications folder
  - [ ] Launch from Applications
  - [ ] App should open without errors

---

## Part 6: Ongoing Release Process

### Task 6.1: Standard Release Workflow

- [ ] Understand the 6-step release process:
  - [ ] Make code changes
  - [ ] Update version in Cargo.toml
  - [ ] Commit and push to main
  - [ ] Run `./scripts/release.sh X.Y.Z`
  - [ ] Monitor GitHub Actions
  - [ ] Test downloaded DMG

### Task 6.2: Version Numbering Guidelines

- [ ] Understand semantic versioning (MAJOR.MINOR.PATCH)
- [ ] Understand when to bump:
  - [ ] PATCH for bug fixes (0.1.1, 0.1.2)
  - [ ] MINOR for new features (0.2.0, 0.3.0)
  - [ ] MAJOR for breaking changes (1.0.0, 2.0.0)

---

## Verification Steps

Run these commands to verify everything is set up correctly:

```bash
# Check all scripts exist and are executable
ls -la scripts/
# Should show: build-app-bundle.sh, create-dmg.sh, release.sh (all with 'x' permission)

# Check GitHub Actions workflow exists
cat .github/workflows/build-and-release.yml | head -5
# Should show: "name: Build and Release macOS App"

# Check certificate is installed
security find-identity -v -p codesigning
# Should show at least 1 "Developer ID Application" certificate

# Check GitHub secrets are set (requires GitHub CLI or web interface)
# Go to: https://github.com/tonomb/pocket_flow/settings/secrets/actions
# Should see 3 secrets listed
```

---

## Troubleshooting

If any step fails, refer to:

1. **BUILD_AND_RELEASE_SPEC.md** - Detailed instructions for each task
2. **Appendix A** in spec - Common issues and solutions
3. **QUICK_REFERENCE.md** - Quick troubleshooting table

---

## Next Steps After Completion

Once this checklist is complete:

- [ ] Archive this checklist (mark as complete)
- [ ] Review QUICK_REFERENCE.md for ongoing use
- [ ] Follow "Part 6: Ongoing Release Process" for future releases
- [ ] Automate your release process with the helper script

---

**Status:** Not Started ▢ | In Progress ▢ | Completed ▢

**Date Started:** _______________

**Date Completed:** _______________

**Notes:**
```



```

---

## Quick Summary

What you've created:
- ✅ **BUILD_AND_RELEASE_SPEC.md** (1,104 lines) - Complete detailed guide
- ✅ **QUICK_REFERENCE.md** - Cheat sheet for common operations
- ✅ **IMPLEMENTATION_CHECKLIST.md** - Step-by-step tracking checklist

These documents are designed so a junior developer can:
1. Read the full spec once to understand the architecture
2. Use the checklist to track progress
3. Reference the quick guide for common operations after setup is complete
