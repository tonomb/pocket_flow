# Engineering Spec: macOS Build & Release Pipeline with Code Signing

**Ticket ID:** NOJIRA-001  
**Status:** In Progress  
**Target Audience:** Junior Developers  
**Last Updated:** January 30, 2025

---

## Description

Implement a complete, automated build and release pipeline for Pocket Flow that enables:
- Professional code signing with Apple Developer ID certificates
- Automated DMG (disk image) creation for macOS distribution
- GitHub Actions workflow triggered by git tags
- One-command release process for developers
- Downloadable signed app bundles from GitHub Releases

After implementation, releasing a new version will be as simple as:
```bash
./scripts/release.sh 0.2.0
```

The app will automatically build, sign, package, and publish to GitHub Releases without manual intervention.

---

## Technical Context

**Current State:**
- Rust project using eframe/egui for GUI
- Builds locally with `cargo build --release`
- No automated distribution mechanism
- Users must run `cargo run` to test the app

**Target State:**
- Professional DMG installer for macOS
- Code-signed with Developer ID certificate
- Automated builds via GitHub Actions
- GitHub Releases page with downloadable DMG files
- Support for tagged semantic versioning (v1.0.0, v0.2.1, etc.)

**Technology Stack:**
- **Build Tool:** Cargo (Rust)
- **CI/CD:** GitHub Actions
- **Code Signing:** macOS codesign utility + Apple Developer ID
- **Packaging:** create-dmg (brew tool)
- **Distribution:** GitHub Releases API

**Architecture:**
```
Developer pushes tag v1.0.0
  ↓
GitHub Actions triggered
  ↓
Build: cargo build --release
  ↓
Bundle: Create .app directory structure
  ↓
Sign: codesign with Developer ID
  ↓
Package: create-dmg tool
  ↓
Release: Upload DMG to GitHub Releases
  ↓
Users download DMG from releases page
```

---

## Implementation Details

### Code Signing Certificate Management
- Developers create Apple Developer ID certificate in Xcode (one-time setup)
- Certificate exported as .p12 file and base64-encoded
- Stored securely as GitHub repository secrets (not in code)
- GitHub Actions retrieves secrets and uses them for signing during build

### Build Script Architecture
- **build-app-bundle.sh:** Creates proper macOS .app bundle structure
  - Copies binary to `MacOS/` directory
  - Creates `Contents/Info.plist` with app metadata
  - Copies assets (fonts, etc.) to resources directory
  - Result: `Pocket Flow.app` directory

- **create-dmg.sh:** Creates disk image from .app bundle
  - Uses `create-dmg` tool for professional packaging
  - Creates `.dmg` file suitable for distribution
  - Includes app icon and drag-to-install experience

- **release.sh:** Helper script for creating releases
  - Validates git state (clean working directory)
  - Creates git tag
  - Pushes to GitHub (triggers Actions)
  - Provides user feedback

### GitHub Actions Workflow
Triggered on push of git tags matching pattern `v*` (e.g., v0.1.0, v1.0.0)

**Execution Steps:**
1. Checkout code
2. Install Rust toolchain
3. Build release binary with `cargo build --release`
4. Install `create-dmg` tool
5. Create .app bundle using build script
6. Code sign .app with Developer ID certificate (from secrets)
7. Create DMG using create-dmg script
8. Create GitHub Release and upload DMG artifact

**Total Execution Time:** 5-10 minutes

### Version Management
- Uses semantic versioning: `MAJOR.MINOR.PATCH` (e.g., 0.1.0, 1.2.3)
- Version stored in `Cargo.toml`
- Git tags match version: `v0.1.0`, `v1.2.3`, etc.
- Each release creates one GitHub release with one DMG file

---

## Acceptance Criteria

1. Apple Developer ID certificate can be created and stored securely
2. Build script successfully creates proper .app bundle with all assets included
3. DMG creation script produces installable disk image
4. GitHub Actions workflow automatically triggers on git tag push
5. Workflow successfully code signs the app without manual intervention
6. DMG file is uploaded to GitHub Releases with correct naming convention
7. Users can download DMG from releases page and install app without errors
8. App launches successfully after installation from DMG
9. Release helper script guides user through the release process
10. Complete documentation enables junior developers to implement and maintain the system

---

## Testing Considerations

### Unit/Integration Testing
- Test build scripts locally before first release
- Verify .app bundle structure is correct
- Verify Info.plist contains required keys
- Test DMG creation and installation on clean macOS system

### Workflow Testing
- Create test tag (v0.1.0-test) to verify GitHub Actions execution
- Monitor workflow logs for any failures
- Verify DMG file appears in GitHub Releases
- Download DMG on different macOS versions to test compatibility

### Release Testing
- Test release script with dry-run (check git state without pushing)
- Perform complete release workflow end-to-end
- Install app from DMG on another Mac
- Verify app functionality after installation

---

## Dependencies

### External Dependencies
- **Xcode:** Required for Developer ID certificate creation
- **Apple Developer Account:** Required for code signing (one-time cost ~$99/year)
- **Rust toolchain:** Must be installed for local builds
- **create-dmg:** Brew package, installed during workflow
- **GitHub Repository:** Must have write access for secrets and releases

### Prerequisite Work
- None (self-contained feature)

### Blocking Work
- None (can be implemented independently)

---

## Resources

**Apple & Code Signing:**
- [Apple Code Signing Guide](https://help.apple.com/xcode/mac/current/#/dev54d690f09)
- [Developer ID Certificate Documentation](https://developer.apple.com/help/account/develop-with-xcode/install-an-app-you-have-uploaded-to-app-store-connect/)
- [Gatekeeper Overview](https://support.apple.com/en-us/102445)

**GitHub Actions:**
- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Workflow Syntax Reference](https://docs.github.com/en/actions/using-workflows/workflow-syntax-for-github-actions)

**Rust & Distribution:**
- [eframe/egui Documentation](https://docs.rs/eframe/latest/eframe/)
- [Cargo Build Documentation](https://doc.rust-lang.org/cargo/commands/cargo-build.html)

**Tools:**
- [create-dmg GitHub](https://github.com/create-dmg/create-dmg)
- [macOS App Bundle Structure](https://developer.apple.com/library/archive/documentation/CoreFoundation/Conceptual/CFBundles/BundleTypes/BundleTypes.html)

---

## Relevant Files

**To Be Created:**
- `.github/workflows/build-and-release.yml` - GitHub Actions workflow definition
- `scripts/build-app-bundle.sh` - Create .app bundle script
- `scripts/create-dmg.sh` - Create DMG installer script
- `scripts/release.sh` - Release helper script

**To Be Modified:**
- `Cargo.toml` - Ensure version field is set correctly (metadata)

**To Be Updated (Docs):**
- `docs/specs/NOJIRA-001-spec-build-and-release.md` - This spec (living document)

**Reference Files (Not Modified):**
- `src/main.rs` - Application code (used for building)
- `assets/` - App assets like fonts (bundled in .app)

---

## Implementation Tasks - Phase 1 (Parent Tasks)

Based on the requirements above, here are the high-level tasks in logical development order:

- [ ] 1.0 **Set Up Code Signing Infrastructure**
  - Establish Apple Developer ID certificate
  - Securely store credentials in GitHub
  - Verify certificate is ready for use

- [ ] 2.0 **Create Build and Packaging Scripts**
  - Create shell script to build .app bundle
  - Create shell script to generate DMG
  - Test scripts locally with manual builds

- [ ] 3.0 **Create Release Helper Script**
  - Create shell script to manage git tags
  - Add validation and user prompts
  - Enable easy release creation

- [ ] 4.0 **Build GitHub Actions Workflow**
  - Define workflow triggers (git tags)
  - Implement build steps (cargo, bundle, sign)
  - Configure release creation

- [ ] 5.0 **Test Complete Pipeline**
  - Create first test release with v0.1.0 tag
  - Monitor workflow execution
  - Verify DMG creation and GitHub Releases

- [ ] 6.0 **Create Comprehensive Documentation**
  - Document complete setup process
  - Create quick reference guide
  - Create implementation checklist

---

## Implementation Tasks - Phase 2 (Detailed Sub-Tasks)

### 1.0 Set Up Code Signing Infrastructure

- [ ] 1.1 **Create Apple Developer ID Certificate**
  - [ ] 1.1.1 Open Xcode and navigate to Preferences → Accounts
  - [ ] 1.1.2 Add or select Apple ID from Developer Account
  - [ ] 1.1.3 Click "Manage Certificates..." button
  - [ ] 1.1.4 Create new "Developer ID Application" certificate
  - [ ] 1.1.5 Wait for certificate generation to complete
  - [ ] 1.1.6 Run `security find-identity -v -p codesigning` to verify installation
  - [ ] 1.1.7 Record the certificate identity string (e.g., "Developer ID Application: Name (TEAMID)")
  - [ ] 1.1.8 Document the TEAMID for later use in GitHub secrets

- [ ] 1.2 **Export Certificate to .p12 File**
  - [ ] 1.2.1 Open Keychain Access application
  - [ ] 1.2.2 Search for "Developer ID Application"
  - [ ] 1.2.3 Right-click certificate and select "Export..."
  - [ ] 1.2.4 Save file as `developer-id.p12` to temporary location (e.g., Desktop)
  - [ ] 1.2.5 Create and remember a strong password for the .p12 file
  - [ ] 1.2.6 Verify file was created: `ls -la ~/Desktop/developer-id.p12`

- [ ] 1.3 **Encode Certificate and Store GitHub Secrets**
  - [ ] 1.3.1 Base64 encode the certificate: `base64 -i ~/Desktop/developer-id.p12 | pbcopy`
  - [ ] 1.3.2 Go to GitHub repo Settings → Secrets and variables → Actions
  - [ ] 1.3.3 Create secret `MACOS_CERTIFICATE` with base64 content
  - [ ] 1.3.4 Create secret `MACOS_CERTIFICATE_PWD` with certificate password
  - [ ] 1.3.5 Create secret `DEVELOPER_ID_APPLICATION` with full cert name string
  - [ ] 1.3.6 Verify all 3 secrets appear in GitHub Settings
  - [ ] 1.3.7 Delete temporary certificate file: `rm ~/Desktop/developer-id.p12`

- [ ] 1.4 **Verify Code Signing Setup**
  - [ ] 1.4.1 Run `security find-identity -v -p codesigning` locally
  - [ ] 1.4.2 Confirm at least 1 "Developer ID Application" certificate is listed
  - [ ] 1.4.3 Run `codesign --version` to confirm codesign utility is available
  - [ ] 1.4.4 Document that setup is ready for workflow use

---

### 2.0 Create Build and Packaging Scripts

- [ ] 2.1 **Create build-app-bundle.sh Script**
  - [ ] 2.1.1 Create `scripts/` directory: `mkdir -p scripts`
  - [ ] 2.1.2 Create file `scripts/build-app-bundle.sh` with content from spec section "Task 2.1"
  - [ ] 2.1.3 Make script executable: `chmod +x scripts/build-app-bundle.sh`
  - [ ] 2.1.4 Review script logic:
    - [ ] Validates binary exists at correct path
    - [ ] Creates app bundle directory structure
    - [ ] Copies binary to `MacOS/` subdirectory
    - [ ] Copies assets folder to resources
    - [ ] Creates proper Info.plist with app metadata
  - [ ] 2.1.5 Test script locally: `cargo build --release && ./scripts/build-app-bundle.sh 0.1.0`
  - [ ] 2.1.6 Verify .app bundle created: `ls -la target/release/Pocket\ Flow.app/`
  - [ ] 2.1.7 Verify .app structure has: `Contents/MacOS/pocket_flow`, `Contents/Info.plist`, `Contents/Resources/assets/`

- [ ] 2.2 **Create create-dmg.sh Script**
  - [ ] 2.2.1 Create file `scripts/create-dmg.sh` with content from spec section "Task 2.2"
  - [ ] 2.2.2 Make script executable: `chmod +x scripts/create-dmg.sh`
  - [ ] 2.2.3 Install create-dmg tool: `brew install create-dmg`
  - [ ] 2.2.4 Review script logic:
    - [ ] Validates .app bundle exists
    - [ ] Checks for create-dmg tool availability
    - [ ] Creates temporary directory for DMG contents
    - [ ] Runs create-dmg with proper parameters
    - [ ] Cleans up temporary files
    - [ ] Outputs DMG to target/release/ directory
  - [ ] 2.2.5 Test script locally: `./scripts/create-dmg.sh 0.1.0`
  - [ ] 2.2.6 Verify DMG created: `ls -lh target/release/Pocket_Flow_0.1.0.dmg`
  - [ ] 2.2.7 Test DMG mounting: `open target/release/Pocket_Flow_0.1.0.dmg`
  - [ ] 2.2.8 Verify app is visible in mounted DMG

- [ ] 2.3 **Test Build Scripts End-to-End**
  - [ ] 2.3.1 Clean build artifacts: `cargo clean`
  - [ ] 2.3.2 Build release binary: `cargo build --release`
  - [ ] 2.3.3 Create .app bundle: `./scripts/build-app-bundle.sh 0.1.0`
  - [ ] 2.3.4 Create DMG: `./scripts/create-dmg.sh 0.1.0`
  - [ ] 2.3.5 Verify final DMG file exists and is larger than 50MB
  - [ ] 2.3.6 Document that local build process works correctly

---

### 3.0 Create Release Helper Script

- [ ] 3.1 **Create release.sh Script**
  - [ ] 3.1.1 Create file `scripts/release.sh` with content from spec section "Task 4.1"
  - [ ] 3.1.2 Make script executable: `chmod +x scripts/release.sh`
  - [ ] 3.1.3 Review script logic:
    - [ ] Accepts version number as parameter
    - [ ] Validates version format (semantic versioning)
    - [ ] Checks git working directory is clean
    - [ ] Verifies tag doesn't already exist
    - [ ] Creates annotated git tag
    - [ ] Pushes tag to GitHub
    - [ ] Provides user feedback and next steps
  - [ ] 3.1.4 Test help output: `./scripts/release.sh` (should show usage)
  - [ ] 3.1.5 Test version validation: `./scripts/release.sh invalid` (should reject)
  - [ ] 3.1.6 Test dry-run without actually pushing (verify prompt works)

- [ ] 3.2 **Add Release Script to Version Control**
  - [ ] 3.2.1 Verify script exists: `ls -la scripts/release.sh`
  - [ ] 3.2.2 Add to git: `git add scripts/release.sh`
  - [ ] 3.2.3 Commit: `git commit -m "feat: add release helper script"`
  - [ ] 3.2.4 Push to main: `git push origin main`

---

### 4.0 Build GitHub Actions Workflow

- [ ] 4.1 **Create GitHub Actions Workflow File**
  - [ ] 4.1.1 Create directory: `mkdir -p .github/workflows`
  - [ ] 4.1.2 Create file `.github/workflows/build-and-release.yml` with workflow content
  - [ ] 4.1.3 Review workflow structure:
    - [ ] Trigger: `on: push: tags: 'v*'` (matches semver tags)
    - [ ] Job runs on: `macos-latest`
    - [ ] Step 1: Checkout code
    - [ ] Step 2: Install Rust toolchain
    - [ ] Step 3: Cache Cargo dependencies
    - [ ] Step 4: Build release binary
    - [ ] Step 5: Install create-dmg
    - [ ] Step 6: Create .app bundle
    - [ ] Step 7: Code sign app (using secrets)
    - [ ] Step 8: Create DMG
    - [ ] Step 9: Create GitHub Release
    - [ ] Step 10: Clean up keychain

- [ ] 4.2 **Configure Code Signing in Workflow**
  - [ ] 4.2.1 Review keychain creation step in workflow
  - [ ] 4.2.2 Review certificate import step in workflow
  - [ ] 4.2.3 Review codesign command in workflow
  - [ ] 4.2.4 Review signature verification step in workflow
  - [ ] 4.2.5 Verify all steps use correct GitHub secrets references
  - [ ] 4.2.6 Verify cleanup step runs even on failure

- [ ] 4.3 **Configure Release Artifact Upload**
  - [ ] 4.3.1 Review `softprops/action-gh-release` configuration
  - [ ] 4.3.2 Verify `files` parameter matches DMG output path
  - [ ] 4.3.3 Verify `GITHUB_TOKEN` is available (built-in, no secret needed)
  - [ ] 4.3.4 Test that workflow will create releases (don't test yet, just review)

- [ ] 4.4 **Add Workflow to Version Control**
  - [ ] 4.4.1 Verify workflow file exists: `cat .github/workflows/build-and-release.yml | head -20`
  - [ ] 4.4.2 Add to git: `git add .github/workflows/build-and-release.yml`
  - [ ] 4.4.3 Commit: `git commit -m "feat: add GitHub Actions build and release workflow"`
  - [ ] 4.4.4 Push to main: `git push origin main`

---

### 5.0 Test Complete Pipeline

- [ ] 5.1 **Prepare for First Release**
  - [ ] 5.1.1 Verify Cargo.toml version is set: `grep '^version = ' Cargo.toml`
  - [ ] 5.1.2 Note the current version (e.g., 0.1.0)
  - [ ] 5.1.3 Verify no uncommitted changes: `git status` should be clean
  - [ ] 5.1.4 Verify all build scripts are executable:
    - [ ] `ls -la scripts/build-app-bundle.sh | grep 'x'`
    - [ ] `ls -la scripts/create-dmg.sh | grep 'x'`
    - [ ] `ls -la scripts/release.sh | grep 'x'`

- [ ] 5.2 **Create First Test Release**
  - [ ] 5.2.1 Run release script: `./scripts/release.sh 0.1.0`
  - [ ] 5.2.2 Answer the prompt: respond with "y" to confirm release
  - [ ] 5.2.3 Verify tag was created locally: `git tag | grep v0.1.0`
  - [ ] 5.2.4 Verify tag was pushed to GitHub: monitor terminal output
  - [ ] 5.2.5 Wait 30 seconds for GitHub Actions to register the push

- [ ] 5.3 **Monitor GitHub Actions Workflow**
  - [ ] 5.3.1 Go to: https://github.com/tonomb/pocket_flow/actions
  - [ ] 5.3.2 Look for "Build and Release macOS App" workflow
  - [ ] 5.3.3 Click the running job to see detailed logs
  - [ ] 5.3.4 Monitor each step for completion:
    - [ ] Checkout - should complete in <1 min
    - [ ] Install Rust - should complete in 1-2 min
    - [ ] Build Release - should complete in 3-5 min
    - [ ] Create App Bundle - should complete in <1 min
    - [ ] Code Sign - should complete in <1 min
    - [ ] Create DMG - should complete in 1-2 min
    - [ ] Create Release - should complete in <1 min
  - [ ] 5.3.5 Total workflow time should be 5-10 minutes
  - [ ] 5.3.6 All steps should show green checkmarks (success)

- [ ] 5.4 **Verify GitHub Release Created**
  - [ ] 5.4.1 Go to: https://github.com/tonomb/pocket_flow/releases
  - [ ] 5.4.2 Look for new release titled "v0.1.0"
  - [ ] 5.4.3 Verify DMG file is attached as an asset:
    - [ ] Filename should be: `Pocket_Flow_v0.1.0.dmg`
    - [ ] File size should be 50-100+ MB
  - [ ] 5.4.4 Click download link to verify file downloads successfully

- [ ] 5.5 **Test DMG Installation on Local Mac**
  - [ ] 5.5.1 Open Downloads folder where DMG was downloaded
  - [ ] 5.5.2 Double-click DMG to mount it
  - [ ] 5.5.3 Verify DMG window opens showing the app
  - [ ] 5.5.4 Drag "Pocket Flow.app" to Applications folder
  - [ ] 5.5.5 Eject the DMG
  - [ ] 5.5.6 Go to Applications folder
  - [ ] 5.5.7 Double-click "Pocket Flow" app to launch it
  - [ ] 5.5.8 Verify app launches without errors
  - [ ] 5.5.9 Verify app is responsive (can see UI, buttons work)
  - [ ] 5.5.10 Quit the app
  - [ ] 5.5.11 Document that installation works correctly

- [ ] 5.6 **Test on Different Mac (Optional but Recommended)**
  - [ ] 5.6.1 Download DMG on a different macOS system (if available)
  - [ ] 5.6.2 Repeat installation steps 5.5.2-5.5.11
  - [ ] 5.6.3 Verify app works on different hardware/OS version
  - [ ] 5.6.4 Note any Gatekeeper warnings and verify they're expected

- [ ] 5.7 **Verify First Release Success**
  - [ ] 5.7.1 All workflow steps completed successfully
  - [ ] 5.7.2 DMG file created and uploaded to GitHub
  - [ ] 5.7.3 App launches successfully from DMG
  - [ ] 5.7.4 No runtime errors or warnings
  - [ ] 5.7.5 Document that complete pipeline works end-to-end

---

### 6.0 Create Comprehensive Documentation

- [ ] 6.1 **Create Main Documentation Guide**
  - [ ] 6.1.1 Create file: `docs/specs/BUILD_AND_RELEASE_SPEC.md`
  - [ ] 6.1.2 Include sections:
    - [ ] Overview and purpose
    - [ ] Prerequisites checklist
    - [ ] Architecture overview with diagrams
    - [ ] Part 1-6: Detailed implementation steps
    - [ ] Part 1: Code Signing (Tasks 1.1-1.2 with step-by-step instructions)
    - [ ] Part 2: Build Scripts (Tasks 2.1-2.2 with full script content)
    - [ ] Part 3: GitHub Actions (Tasks 3.1-3.2 with full workflow content)
    - [ ] Part 4: Release Script (Task 4.1 with full script content)
    - [ ] Part 5: Testing (Tasks 5.1-5.2 with detailed verification steps)
    - [ ] Part 6: Ongoing Process (Tasks 6.1-6.2)
    - [ ] Appendix A: Troubleshooting guide
    - [ ] Appendix B: File checklist
  - [ ] 6.1.3 Include code blocks for all scripts
  - [ ] 6.1.4 Ensure instructions are clear enough for junior developers

- [ ] 6.2 **Create Quick Reference Guide**
  - [ ] 6.2.1 Create file: `docs/specs/QUICK_REFERENCE.md`
  - [ ] 6.2.2 Include sections:
    - [ ] Quick 3-step release process
    - [ ] Common commands (10+ examples)
    - [ ] Troubleshooting quick fixes table
    - [ ] Version numbering guide
    - [ ] GitHub secrets checklist
    - [ ] Release checklist
    - [ ] File structure overview
    - [ ] Helpful links

- [ ] 6.3 **Create Implementation Checklist**
  - [ ] 6.3.1 Create file: `docs/specs/IMPLEMENTATION_CHECKLIST.md`
  - [ ] 6.3.2 Include checkbox for every task (1.1, 1.2, etc.)
  - [ ] 6.3.3 Organize by part and task
  - [ ] 6.3.4 Include verification commands for each major task
  - [ ] 6.3.5 Include status tracking fields (started, completed dates)

- [ ] 6.4 **Create Documentation Index**
  - [ ] 6.4.1 Create file: `docs/specs/README.md`
  - [ ] 6.4.2 Provide clear navigation between all documentation
  - [ ] 6.4.3 Explain what each document covers
  - [ ] 6.4.4 Recommend reading order for different audiences
  - [ ] 6.4.5 Include FAQ section

- [ ] 6.5 **Create Documentation Index Master**
  - [ ] 6.5.1 Create file: `docs/specs/INDEX.md`
  - [ ] 6.5.2 High-level overview of all documentation
  - [ ] 6.5.3 Use cases and which docs to read
  - [ ] 6.5.4 Quick command reference
  - [ ] 6.5.5 FAQ and troubleshooting links

- [ ] 6.6 **Review and Polish All Documentation**
  - [ ] 6.6.1 Read through all docs for clarity
  - [ ] 6.6.2 Verify code examples are correct and tested
  - [ ] 6.6.3 Ensure links are valid
  - [ ] 6.6.4 Add table of contents where appropriate
  - [ ] 6.6.5 Verify formatting is consistent across all docs

- [ ] 6.7 **Commit Documentation**
  - [ ] 6.7.1 Add all documentation files: `git add docs/specs/`
  - [ ] 6.7.2 Commit: `git commit -m "docs: add comprehensive build and release documentation"`
  - [ ] 6.7.3 Push: `git push origin main`

---

## Open Questions

1. **Notarization:** Should the workflow include Apple notarization for newer macOS versions, or is basic code signing sufficient for your target users?

2. **Multiple Architectures:** Should the workflow build for both Intel (x86_64) and Apple Silicon (ARM64) Macs? Currently set to `macos-latest` which handles both.

3. **Auto-Update Mechanism:** Beyond scope for now, but should we document where in the code the auto-update license check would integrate?

4. **Minimum macOS Version:** Current spec assumes macOS 11.0 (Big Sur) - confirm this is acceptable?

---

**Document Status:** Awaiting confirmation to generate detailed sub-tasks
