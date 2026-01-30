# Pocket Flow: Build & Release Specification

## Overview

This document provides step-by-step instructions for setting up automated builds and releases of Pocket Flow for macOS. The goal is to create a professional DMG installer that users can download from GitHub Releases.

**Target Audience:** Junior developers or anyone unfamiliar with macOS app distribution

**Outcome:** When complete, pushing a git tag will automatically build and release a signed macOS app bundle as a DMG file.

---

## Architecture Overview

```
Developer pushes git tag v1.0.0
         ↓
GitHub Actions Workflow triggered
         ↓
Build Rust release binary
         ↓
Create .app bundle (macOS application format)
         ↓
Code sign with Developer ID certificate
         ↓
Create DMG disk image (installer)
         ↓
Upload to GitHub Releases page
         ↓
Users download DMG and install app
```

---

## Prerequisites

Before starting, ensure you have:

- [ ] macOS 11.0 (Big Sur) or later
- [ ] Xcode Command Line Tools (`xcode-select --install`)
- [ ] Rust toolchain installed (`rustup`)
- [ ] GitHub repository access (already set up: `git@github.com:tonomb/pocket_flow.git`)
- [ ] Apple Developer Account (for code signing certificate)
- [ ] GitHub account with write access to the repository

---

## Part 1: Code Signing Certificate Setup

This section establishes the code signing identity needed to sign your app. Do this **once** on your development Mac.

### Task 1.1: Create Apple Developer ID Certificate

**Objective:** Create a Developer ID Application certificate for code signing

**Steps:**

1. **Open Xcode Preferences**
   ```bash
   open /Applications/Xcode.app/Contents/Preferences/Accounts
   ```

2. **Add Apple ID** (if not already present)
   - Click the `+` button at bottom left
   - Select "Apple ID"
   - Sign in with your Apple Developer account
   - Wait for it to load

3. **Download Development Certificate**
   - Select your Apple ID from the left panel
   - Click "Manage Certificates..." button
   - Click `+` button to add new certificate
   - Select "Developer ID Application"
   - Click "Create"
   - Wait for certificate to generate
   - Click "Done"

4. **Verify Certificate Installation**
   ```bash
   security find-identity -v -p codesigning
   ```
   
   **Expected Output:**
   ```
   1) <40 character hash> "Developer ID Application: Your Name (TEAMID)"
   ```
   
   **Note:** The `TEAMID` in parentheses is important - you'll need it later.

5. **Record Your Certificate Identity**
   
   Run this command to get the full certificate name:
   ```bash
   security find-identity -v -p codesigning | grep "Developer ID Application"
   ```
   
   **Save the output.** It should look like:
   ```
   "Developer ID Application: Antonio Martinez (ABC123DE45)"
   ```
   
   **Store this value** - you'll need it for the GitHub Actions workflow.

**Troubleshooting:**

- **No certificates found?** Make sure you signed in with the correct Apple Developer account
- **Certificate revoked?** Regenerate it by repeating steps 1-3
- **Still having issues?** Check Apple's [Developer ID certificate guide](https://help.apple.com/xcode/mac/current/#/dev54d690f09)

**Next:** Proceed to Task 1.2

---

### Task 1.2: Create GitHub Repository Secret for Code Signing

**Objective:** Store your certificate securely in GitHub so the Actions workflow can use it

**Prerequisites:** You need:
- Your Developer ID certificate (created in Task 1.1)
- The certificate password
- GitHub repo write access

**Steps:**

1. **Export Certificate to File**
   
   On your local Mac, open Keychain Access:
   ```bash
   open /Applications/Utilities/Keychain\ Access.app
   ```

2. **Find and Export Certificate**
   - In the search box, type: `Developer ID Application`
   - Right-click the certificate
   - Select "Export..."
   - Save as: `developer-id.p12`
   - When prompted for password, enter something memorable (you'll need it)
   - Save in a temporary location (e.g., Desktop)

3. **Encode Certificate as Base64**
   
   In Terminal:
   ```bash
   base64 -i ~/Desktop/developer-id.p12 | pbcopy
   ```
   
   This copies the encoded certificate to your clipboard.

4. **Add to GitHub Secrets**
   
   - Go to your GitHub repository: https://github.com/tonomb/pocket_flow
   - Click **Settings** tab
   - Click **Secrets and variables** → **Actions** (left sidebar)
   - Click **New repository secret**
   - Name: `MACOS_CERTIFICATE`
   - Value: Paste from clipboard (Cmd+V)
   - Click **Add secret**

5. **Add Certificate Password as Secret**
   
   - Click **New repository secret** again
   - Name: `MACOS_CERTIFICATE_PWD`
   - Value: The password you entered in Step 2
   - Click **Add secret**

6. **Add Developer Team ID as Secret**
   
   - Extract the team ID from your certificate (from Task 1.1)
   - Example: If certificate is "Developer ID Application: Your Name (ABC123DE45)", the ID is `ABC123DE45`
   - Click **New repository secret** again
   - Name: `DEVELOPER_ID_APPLICATION`
   - Value: The full certificate identity string (e.g., `"Developer ID Application: Antonio Martinez (ABC123DE45)"`)
   - Click **Add secret**

7. **Verify Secrets Added**
   
   Go back to **Settings** → **Secrets and variables** → **Actions**
   
   You should see three secrets listed:
   - `MACOS_CERTIFICATE`
   - `MACOS_CERTIFICATE_PWD`
   - `DEVELOPER_ID_APPLICATION`

8. **Clean Up**
   
   Delete the temporary certificate file:
   ```bash
   rm ~/Desktop/developer-id.p12
   ```

**Next:** Proceed to Part 2

---

## Part 2: Create Build Scripts

This section creates helper scripts that build the app and create the DMG installer.

### Task 2.1: Create App Bundle Build Script

**Objective:** Create a shell script that builds the .app bundle

**File Location:** `scripts/build-app-bundle.sh`

**Steps:**

1. **Create Scripts Directory**
   ```bash
   mkdir -p /Users/antomb/Desktop/make/pocket_flow/scripts
   ```

2. **Create the Script File**
   
   Create file: `/Users/antomb/Desktop/make/pocket_flow/scripts/build-app-bundle.sh`
   
   Copy the following content:

```bash
#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
APP_NAME="Pocket Flow"
BUNDLE_ID="com.pocket-flow.app"
EXECUTABLE_NAME="pocket_flow"
VERSION="${1:-0.1.0}"
BINARY_PATH="target/release/$EXECUTABLE_NAME"
ASSETS_DIR="assets"

echo -e "${YELLOW}Building $APP_NAME macOS app bundle...${NC}"
echo "Version: $VERSION"
echo "Binary: $BINARY_PATH"

# Step 1: Check if binary exists
if [ ! -f "$BINARY_PATH" ]; then
    echo -e "${RED}Error: Binary not found at $BINARY_PATH${NC}"
    echo "Run 'cargo build --release' first"
    exit 1
fi

# Step 2: Create app bundle structure
APP_BUNDLE="target/release/$APP_NAME.app"
CONTENTS_DIR="$APP_BUNDLE/Contents"
MACOS_DIR="$CONTENTS_DIR/MacOS"
RESOURCES_DIR="$CONTENTS_DIR/Resources"

echo -e "${YELLOW}Creating app bundle structure...${NC}"

rm -rf "$APP_BUNDLE"
mkdir -p "$MACOS_DIR"
mkdir -p "$RESOURCES_DIR"

# Step 3: Copy binary to app bundle
echo -e "${YELLOW}Copying binary...${NC}"
cp "$BINARY_PATH" "$MACOS_DIR/$EXECUTABLE_NAME"
chmod +x "$MACOS_DIR/$EXECUTABLE_NAME"

# Step 4: Copy assets (fonts, etc.)
if [ -d "$ASSETS_DIR" ]; then
    echo -e "${YELLOW}Copying assets...${NC}"
    cp -r "$ASSETS_DIR" "$RESOURCES_DIR/"
fi

# Step 5: Create Info.plist
echo -e "${YELLOW}Creating Info.plist...${NC}"
cat > "$CONTENTS_DIR/Info.plist" << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleExecutable</key>
    <string>pocket_flow</string>
    <key>CFBundleIdentifier</key>
    <string>com.pocket-flow.app</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>Pocket Flow</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>LSMinimumSystemVersion</key>
    <string>11.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSHumanReadableCopyright</key>
    <string>Copyright © 2025 Pocket Flow. All rights reserved.</string>
    <key>NSMainNibFile</key>
    <string></string>
    <key>NSPrincipalClass</key>
    <string>NSApplication</string>
</dict>
</plist>
EOF

echo -e "${GREEN}✓ App bundle created at: $APP_BUNDLE${NC}"
echo -e "${GREEN}✓ Ready for code signing${NC}"
```

3. **Make Script Executable**
   ```bash
   chmod +x /Users/antomb/Desktop/make/pocket_flow/scripts/build-app-bundle.sh
   ```

4. **Test the Script**
   ```bash
   cd /Users/antomb/Desktop/make/pocket_flow
   cargo build --release
   ./scripts/build-app-bundle.sh 0.1.0
   ```
   
   **Expected Output:**
   ```
   ✓ App bundle created at: target/release/Pocket Flow.app
   ✓ Ready for code signing
   ```

**Troubleshooting:**

- **Script not found?** Make sure you created it in the correct location and ran `chmod +x`
- **Binary not found?** Run `cargo build --release` first
- **Permission denied?** Run `chmod +x scripts/build-app-bundle.sh`

**Next:** Proceed to Task 2.2

---

### Task 2.2: Create DMG Creation Script

**Objective:** Create a shell script that creates the DMG installer

**File Location:** `scripts/create-dmg.sh`

**Steps:**

1. **Create the Script File**
   
   Create file: `/Users/antomb/Desktop/make/pocket_flow/scripts/create-dmg.sh`
   
   Copy the following content:

```bash
#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
APP_NAME="Pocket Flow"
VERSION="${1:-0.1.0}"
APP_BUNDLE="target/release/$APP_NAME.app"
DMG_PATH="target/release/Pocket_Flow_${VERSION}.dmg"

echo -e "${YELLOW}Creating DMG installer...${NC}"
echo "App: $APP_NAME"
echo "Version: $VERSION"

# Step 1: Check if app bundle exists
if [ ! -d "$APP_BUNDLE" ]; then
    echo -e "${RED}Error: App bundle not found at $APP_BUNDLE${NC}"
    echo "Run './scripts/build-app-bundle.sh' first"
    exit 1
fi

# Step 2: Check if create-dmg is installed
if ! command -v create-dmg &> /dev/null; then
    echo -e "${YELLOW}Installing create-dmg tool...${NC}"
    brew install create-dmg
fi

# Step 3: Clean up any existing DMG
if [ -f "$DMG_PATH" ]; then
    echo -e "${YELLOW}Removing existing DMG...${NC}"
    rm "$DMG_PATH"
fi

# Step 4: Create temporary directory for DMG contents
TEMP_DMG_DIR=$(mktemp -d)
echo -e "${YELLOW}Creating DMG in temporary directory...${NC}"
cp -r "$APP_BUNDLE" "$TEMP_DMG_DIR/"

# Step 5: Create DMG using create-dmg
echo -e "${YELLOW}Building DMG image...${NC}"
create-dmg \
  --volname "Pocket Flow" \
  --background /dev/null \
  --window-pos 200 120 \
  --window-size 400 300 \
  --icon-size 120 \
  --icon "Pocket Flow.app" 100 150 \
  --hide-extension "Pocket Flow.app" \
  "$DMG_PATH" \
  "$TEMP_DMG_DIR/"

# Step 6: Clean up temporary directory
rm -rf "$TEMP_DMG_DIR"

echo -e "${GREEN}✓ DMG created at: $DMG_PATH${NC}"
echo -e "${GREEN}✓ Ready for distribution${NC}"
```

2. **Make Script Executable**
   ```bash
   chmod +x /Users/antomb/Desktop/make/pocket_flow/scripts/create-dmg.sh
   ```

3. **Install create-dmg Tool** (if not already installed)
   ```bash
   brew install create-dmg
   ```

4. **Test the Script**
   ```bash
   cd /Users/antomb/Desktop/make/pocket_flow
   ./scripts/create-dmg.sh 0.1.0
   ```
   
   **Expected Output:**
   ```
   ✓ DMG created at: target/release/Pocket_Flow_0.1.0.dmg
   ✓ Ready for distribution
   ```

5. **Verify DMG Creation**
   ```bash
   ls -lh target/release/Pocket_Flow_0.1.0.dmg
   ```

**Troubleshooting:**

- **create-dmg not found?** Install it: `brew install create-dmg`
- **App bundle not found?** Run `./scripts/build-app-bundle.sh 0.1.0` first
- **Permission denied?** Run `chmod +x scripts/create-dmg.sh`

**Next:** Proceed to Part 3

---

## Part 3: Create GitHub Actions Workflow

This section creates the automated build and release workflow that runs when you push a git tag.

### Task 3.1: Create GitHub Actions Workflow File

**Objective:** Create the automated build workflow

**File Location:** `.github/workflows/build-and-release.yml`

**Steps:**

1. **Create GitHub Workflows Directory**
   ```bash
   mkdir -p /Users/antomb/Desktop/make/pocket_flow/.github/workflows
   ```

2. **Create Workflow File**
   
   Create file: `/Users/antomb/Desktop/make/pocket_flow/.github/workflows/build-and-release.yml`
   
   Copy the following content:

```yaml
name: Build and Release macOS App

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    runs-on: macos-latest
    
    steps:
    - name: Checkout Repository
      uses: actions/checkout@v4

    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable

    - name: Cache Cargo
      uses: Swatinem/rust-cache@v2

    - name: Build Release Binary
      run: cargo build --release

    - name: Install DMG Creation Tool
      run: brew install create-dmg

    - name: Create App Bundle
      run: |
        mkdir -p scripts
        # Copy build script (will be created in next task)
        ./scripts/build-app-bundle.sh ${{ github.ref_name }}
      env:
        VERSION: ${{ github.ref_name }}

    - name: Code Sign App Bundle
      env:
        MACOS_CERTIFICATE: ${{ secrets.MACOS_CERTIFICATE }}
        MACOS_CERTIFICATE_PWD: ${{ secrets.MACOS_CERTIFICATE_PWD }}
        DEVELOPER_ID_APPLICATION: ${{ secrets.DEVELOPER_ID_APPLICATION }}
      run: |
        # Create keychain
        security create-keychain -p "$MACOS_CERTIFICATE_PWD" build.keychain
        security default-keychain -s build.keychain
        security unlock-keychain -p "$MACOS_CERTIFICATE_PWD" build.keychain
        
        # Import certificate
        echo $MACOS_CERTIFICATE | base64 --decode > certificate.p12
        security import certificate.p12 -k build.keychain -P "$MACOS_CERTIFICATE_PWD" -T /usr/bin/codesign
        security set-key-partition-list -S apple-tool:,apple:,codesign: -k "$MACOS_CERTIFICATE_PWD" build.keychain
        
        # Sign the app
        codesign --force --verbose --sign "$DEVELOPER_ID_APPLICATION" "target/release/Pocket Flow.app"
        
        # Verify signature
        codesign -v "target/release/Pocket Flow.app"

    - name: Create DMG
      run: ./scripts/create-dmg.sh ${{ github.ref_name }}

    - name: Create GitHub Release
      uses: softprops/action-gh-release@v1
      with:
        files: target/release/Pocket_Flow_${{ github.ref_name }}.dmg
        draft: false
        prerelease: false
      env:
        GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}

    - name: Clean Up Keychain
      if: always()
      run: |
        security delete-keychain build.keychain || true
```

3. **Verify File Created**
   ```bash
   ls -la /Users/antomb/Desktop/make/pocket_flow/.github/workflows/
   ```
   
   Should show: `build-and-release.yml`

**Understanding the Workflow:**

The workflow does the following when triggered by a git tag:

1. **Checkout** - Downloads your code from GitHub
2. **Install Rust** - Sets up Rust toolchain
3. **Build Release** - Compiles your app with `cargo build --release`
4. **Install Tools** - Installs `create-dmg` for packaging
5. **Create App Bundle** - Runs your build script to create .app
6. **Code Sign** - Signs the app with your Developer ID certificate
7. **Create DMG** - Packages the signed app as a DMG installer
8. **Create Release** - Uploads DMG to GitHub Releases page
9. **Cleanup** - Removes temporary signing keychain

**Next:** Proceed to Task 3.2

---

### Task 3.2: Verify GitHub Secrets

**Objective:** Confirm all required secrets are properly stored

**Steps:**

1. **Go to GitHub Repository Settings**
   
   - Navigate to: https://github.com/tonomb/pocket_flow/settings/secrets/actions
   - You should see the three secrets created in Part 1, Task 1.2:
     - `MACOS_CERTIFICATE`
     - `MACOS_CERTIFICATE_PWD`
     - `DEVELOPER_ID_APPLICATION`

2. **Verify Each Secret**
   
   Each secret should show:
   - Name (e.g., `MACOS_CERTIFICATE`)
   - A "Created" date
   - An "Updated" date
   - Last used status (empty if not used yet)

3. **Test Secret Access** (Optional)
   
   You can test if secrets are accessible by creating a simple test workflow, but this isn't necessary for production.

**Next:** Proceed to Part 4

---

## Part 4: Create Release Script

This section creates a convenient local script for tagging and pushing releases.

### Task 4.1: Create Release Helper Script

**Objective:** Create a script to simplify the release process

**File Location:** `scripts/release.sh`

**Steps:**

1. **Create the Script File**
   
   Create file: `/Users/antomb/Desktop/make/pocket_flow/scripts/release.sh`
   
   Copy the following content:

```bash
#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
REPO_URL="https://github.com/tonomb/pocket_flow"
RELEASES_PAGE="$REPO_URL/releases"

# Function to print usage
usage() {
    cat << EOF
${BLUE}Pocket Flow Release Helper${NC}

Usage: ./scripts/release.sh [version]

Examples:
  ./scripts/release.sh 0.1.0    # Release version 0.1.0
  ./scripts/release.sh 1.0.0    # Release version 1.0.0

This script will:
1. Verify the working directory is clean
2. Update Cargo.toml with the new version
3. Create a git tag
4. Push to GitHub (triggering automated build)
5. Provide link to monitor the build

EOF
    exit 1
}

# Validate input
if [ -z "$1" ]; then
    echo -e "${RED}Error: Version required${NC}"
    usage
fi

VERSION=$1
TAG="v$VERSION"

# Validate version format
if ! [[ $VERSION =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo -e "${RED}Error: Invalid version format. Use semantic versioning (e.g., 0.1.0)${NC}"
    exit 1
fi

echo -e "${BLUE}═══════════════════════════════════════${NC}"
echo -e "${BLUE}Pocket Flow Release: $TAG${NC}"
echo -e "${BLUE}═══════════════════════════════════════${NC}"

# Step 1: Check git status
echo -e "${YELLOW}Checking git status...${NC}"
if [ -n "$(git status --porcelain)" ]; then
    echo -e "${RED}Error: Working directory has uncommitted changes${NC}"
    echo "Please commit or stash changes before releasing"
    echo ""
    echo "Current changes:"
    git status --short
    exit 1
fi
echo -e "${GREEN}✓ Working directory clean${NC}"

# Step 2: Check if tag already exists
echo -e "${YELLOW}Checking if tag already exists...${NC}"
if git rev-parse "$TAG" &>/dev/null; then
    echo -e "${RED}Error: Tag $TAG already exists${NC}"
    echo "Either:"
    echo "  1. Use a different version number"
    echo "  2. Delete the tag: git tag -d $TAG && git push origin :refs/tags/$TAG"
    exit 1
fi
echo -e "${GREEN}✓ Tag available${NC}"

# Step 3: Confirm version bump
echo -e "${YELLOW}Current version in Cargo.toml:${NC}"
grep '^version = ' Cargo.toml
echo ""
echo -e "${YELLOW}New version will be: $VERSION${NC}"
read -p "Continue with release? (y/n) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo -e "${RED}Release cancelled${NC}"
    exit 1
fi

# Step 4: Create git tag
echo -e "${YELLOW}Creating git tag...${NC}"
git tag -a "$TAG" -m "Release version $VERSION"
echo -e "${GREEN}✓ Tag created: $TAG${NC}"

# Step 5: Push tag to GitHub
echo -e "${YELLOW}Pushing tag to GitHub...${NC}"
git push origin "$TAG"
echo -e "${GREEN}✓ Tag pushed to GitHub${NC}"

# Step 6: Display next steps
echo ""
echo -e "${GREEN}═══════════════════════════════════════${NC}"
echo -e "${GREEN}✓ Release $TAG initiated!${NC}"
echo -e "${GREEN}═══════════════════════════════════════${NC}"
echo ""
echo -e "${BLUE}Next Steps:${NC}"
echo "1. Monitor the automated build:"
echo -e "   ${BLUE}$RELEASES_PAGE${NC}"
echo ""
echo "2. Build progress:"
echo -e "   Go to: ${BLUE}$REPO_URL/actions${NC}"
echo "   Look for workflow: 'Build and Release macOS App'"
echo ""
echo "3. Download the app:"
echo "   Once workflow completes, download the DMG from the Releases page"
echo ""
echo -e "${YELLOW}Build typically takes 5-10 minutes${NC}"
```

2. **Make Script Executable**
   ```bash
   chmod +x /Users/antomb/Desktop/make/pocket_flow/scripts/release.sh
   ```

3. **Test the Script (Dry Run)**
   
   Don't actually release yet, but verify the script works:
   ```bash
   # This will show usage and exit
   ./scripts/release.sh
   ```

**Next:** Proceed to Part 5

---

## Part 5: Testing and First Release

This section walks through creating your first test release.

### Task 5.1: Update Cargo.toml Version

**Objective:** Prepare the project for the first release

**Steps:**

1. **Open Cargo.toml**
   ```bash
   open /Users/antomb/Desktop/make/pocket_flow/Cargo.toml
   ```

2. **Update Version**
   
   Change line 3 from:
   ```toml
   version = "0.1.0"
   ```
   
   To:
   ```toml
   version = "0.1.0"
   ```
   
   (Or update to `0.2.0`, `1.0.0`, etc. if you prefer a different initial version)

3. **Verify and Save**
   ```bash
   grep '^version = ' /Users/antomb/Desktop/make/pocket_flow/Cargo.toml
   ```
   
   Should output: `version = "0.1.0"`

**Next:** Proceed to Task 5.2

---

### Task 5.2: Create First Test Release

**Objective:** Execute the complete release workflow

**Prerequisites:**
- All previous tasks completed
- GitHub secrets configured
- Scripts created and tested
- Cargo.toml updated

**Steps:**

1. **Commit Cargo.toml Changes**
   ```bash
   cd /Users/antomb/Desktop/make/pocket_flow
   git add Cargo.toml
   git commit -m "chore: update version to 0.1.0 for release"
   git push origin main
   ```

2. **Execute Release Script**
   ```bash
   ./scripts/release.sh 0.1.0
   ```
   
   **Expected Output:**
   ```
   ═══════════════════════════════════════
   Pocket Flow Release: v0.1.0
   ═══════════════════════════════════════
   Checking git status...
   ✓ Working directory clean
   Checking if tag already exists...
   ✓ Tag available
   Current version in Cargo.toml: version = "0.1.0"
   New version will be: 0.1.0
   Continue with release? (y/n) y
   Creating git tag...
   ✓ Tag created: v0.1.0
   Pushing tag to GitHub...
   ✓ Tag pushed to GitHub
   ═══════════════════════════════════════
   ✓ Release v0.1.0 initiated!
   ═══════════════════════════════════════
   
   Next Steps:
   1. Monitor the automated build:
      https://github.com/tonomb/pocket_flow/releases
   ...
   ```

3. **Monitor Build Progress**
   
   - Go to: https://github.com/tonomb/pocket_flow/actions
   - Look for the "Build and Release macOS App" workflow
   - Click on the running job to see detailed logs
   - Wait for all steps to complete (typically 5-10 minutes)

4. **Verify Release**
   
   Once the workflow completes:
   - Go to: https://github.com/tonomb/pocket_flow/releases
   - You should see a new release: `v0.1.0`
   - The DMG file should be attached as a release asset

5. **Download and Test the DMG**
   
   - Download `Pocket_Flow_v0.1.0.dmg` from the release page
   - Open the DMG file
   - You should see the Pocket Flow app
   - Drag the app to Applications folder
   - Launch the app from Applications

**Troubleshooting:**

**Workflow Failed - Signing Error?**
- Check if secrets are correctly configured in GitHub Settings
- Verify certificate is installed locally: `security find-identity -v -p codesigning`

**Workflow Failed - Build Error?**
- Check the detailed workflow logs on GitHub Actions page
- Look for specific error messages in the "Build Release Binary" step
- Run `cargo build --release` locally to debug

**DMG Opens but App Won't Launch?**
- Check macOS Gatekeeper warning
- You may need to allow the app: System Preferences → Security & Privacy
- Or bypass Gatekeeper: `sudo xattr -d com.apple.quarantine /Applications/Pocket\ Flow.app`

**Next:** Proceed to Part 6

---

## Part 6: Ongoing Release Process

This section describes how to create future releases.

### Task 6.1: Standard Release Workflow

**Objective:** Document the process for releasing new versions

**Steps for Each New Release:**

1. **Make Changes to Code**
   ```bash
   # Edit files, add features, fix bugs
   git add .
   git commit -m "feat: add new feature"
   ```

2. **Update Version in Cargo.toml**
   ```toml
   version = "0.2.0"  # Update to new version
   ```

3. **Commit Version Update**
   ```bash
   git add Cargo.toml
   git commit -m "chore: bump version to 0.2.0"
   git push origin main
   ```

4. **Create Release**
   ```bash
   ./scripts/release.sh 0.2.0
   ```

5. **Wait for Workflow**
   - Monitor on: https://github.com/tonomb/pocket_flow/actions
   - Build typically takes 5-10 minutes

6. **Verify Release**
   - Check: https://github.com/tonomb/pocket_flow/releases
   - Download and test the DMG

### Task 6.2: Version Numbering Guidelines

**Objective:** Understand versioning conventions

**Semantic Versioning Format:** `MAJOR.MINOR.PATCH`

**Examples:**
- `0.1.0` → Initial development release
- `0.2.0` → Add new feature (minor version bump)
- `0.2.1` → Bug fix (patch bump)
- `1.0.0` → First stable release
- `1.1.0` → New features for stable version
- `1.1.1` → Bug fix for stable version

**Decision Guide:**
- **PATCH (0.0.X)**: Bug fixes, minor improvements
- **MINOR (0.X.0)**: New features, non-breaking changes
- **MAJOR (X.0.0)**: Breaking changes, major new version

---

## Appendix A: Troubleshooting Guide

### Common Issues and Solutions

**Issue: "security find-identity" returns 0 identities**

*Cause:* Certificate not installed or wrong account

*Solution:*
1. Verify you're logged into correct Apple Developer account in Xcode
2. Regenerate certificate in Xcode
3. Run: `security find-identity -v -p codesigning`

---

**Issue: GitHub Actions workflow fails with "certificate not found"**

*Cause:* Secrets not properly configured

*Solution:*
1. Go to GitHub Settings → Secrets and Variables → Actions
2. Verify all three secrets exist:
   - `MACOS_CERTIFICATE`
   - `MACOS_CERTIFICATE_PWD`
   - `DEVELOPER_ID_APPLICATION`
3. Re-create any missing secrets

---

**Issue: App won't launch after extracting from DMG**

*Cause:* Gatekeeper blocking unsigned/untrusted app

*Solution:*
1. Open System Preferences → Security & Privacy
2. Click the lock icon to unlock settings
3. Allow the app to run
4. Or, bypass Gatekeeper:
   ```bash
   sudo xattr -d com.apple.quarantine /Applications/Pocket\ Flow.app
   ```

---

**Issue: Build script says "Binary not found"**

*Cause:* Haven't run `cargo build --release` yet

*Solution:*
1. Run: `cargo build --release`
2. Wait for compilation to complete
3. Then run the build script again

---

**Issue: create-dmg command not found**

*Cause:* Tool not installed

*Solution:*
```bash
brew install create-dmg
```

---

**Issue: Tag already exists error**

*Cause:* You're trying to create a release with a version that's already tagged

*Solution:*
1. Use a different version number
2. Or delete the old tag:
   ```bash
   git tag -d v0.1.0
   git push origin :refs/tags/v0.1.0
   ```

---

### Getting Help

If you encounter issues not covered in this guide:

1. **Check Workflow Logs**
   - Go to: https://github.com/tonomb/pocket_flow/actions
   - Click the failed workflow run
   - Look for detailed error messages

2. **Local Debugging**
   - Run scripts locally to identify issues
   - Check each script output step-by-step

3. **Common Resources**
   - [GitHub Actions Documentation](https://docs.github.com/en/actions)
   - [Apple Code Signing Guide](https://help.apple.com/xcode/mac/current/#/dev54d690f09)
   - [create-dmg Documentation](https://github.com/create-dmg/create-dmg)

---

## Appendix B: File Checklist

After completing all tasks, you should have created these files:

```
pocket_flow/
├── .github/
│   └── workflows/
│       └── build-and-release.yml         ✓ Created in Task 3.1
├── scripts/
│   ├── build-app-bundle.sh              ✓ Created in Task 2.1
│   ├── create-dmg.sh                    ✓ Created in Task 2.2
│   └── release.sh                       ✓ Created in Task 4.1
├── docs/
│   └── specs/
│       └── BUILD_AND_RELEASE_SPEC.md    ✓ This file
└── Cargo.toml                           ✓ Updated in Task 5.1
```

---

## Summary

Congratulations! You've successfully set up:

✅ Code signing certificate with Apple Developer ID
✅ GitHub secrets for secure credential storage
✅ Automated build and packaging scripts
✅ GitHub Actions workflow for continuous deployment
✅ Release helper script for easy version management
✅ Complete testing workflow

**Result:** You can now release new versions by simply running:
```bash
./scripts/release.sh 0.2.0
```

The app will automatically build, sign, package as DMG, and upload to GitHub Releases!

---

**Document Version:** 1.0
**Last Updated:** January 30, 2025
**Audience:** Junior Developers
**Status:** Ready for Implementation
