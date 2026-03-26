#!/bin/bash
set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Log function
log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Check dependencies
check_deps() {
    log "Checking dependencies..."
    
    missing_deps=()
    
    # Required tools
    for tool in cargo rustc tar dpkg-deb rpm; do
        if ! command -v $tool &> /dev/null; then
            missing_deps+=("$tool")
        fi
    done
    
    # Optional tools
    for tool in cargo-deb cargo-rpm; do
        if ! command -v $tool &> /dev/null; then
            warn "$tool not found. Installing..."
            cargo install $tool
        fi
    done
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        error "Missing required dependencies: ${missing_deps[*]}"
        error "Please install them and try again."
        exit 1
    fi
    
    success "All dependencies satisfied"
}

# Update changelog
update_changelog() {
    log "Updating CHANGELOG.md..."
    
    version=$(grep '^version' Cargo.toml | head -n 1 | awk -F '"' '{print $2}')
    date=$(date +"%Y-%m-%d")
    
    if ! grep -q "## \[$version\]" CHANGELOG.md; then
        # Create a new entry if it doesn't exist
        temp_file=$(mktemp)
        changelog_header=$(head -n 2 CHANGELOG.md)
        changelog_rest=$(tail -n +3 CHANGELOG.md)
        
        echo "$changelog_header" > "$temp_file"
        echo -e "\n## [$version] - $date\n" >> "$temp_file"
        echo "### Added" >> "$temp_file"
        echo "- " >> "$temp_file"
        echo "### Changed" >> "$temp_file"
        echo "- " >> "$temp_file"
        echo "### Fixed" >> "$temp_file"
        echo "- " >> "$temp_file"
        echo -e "\n$changelog_rest" >> "$temp_file"
        
        mv "$temp_file" CHANGELOG.md
        
        warn "CHANGELOG.md updated with new version $version entry."
        warn "Please edit CHANGELOG.md to add details before continuing."
        read -p "Press enter to continue after editing CHANGELOG.md..."
    else
        success "CHANGELOG.md already contains version $version"
    fi
}

# Build for various targets
build_release() {
    log "Building release version..."
    cargo clean
    cargo build --release
    success "Release build completed: target/release/up-man"
    
    # Optional: Build for musl (static linking)
    if command -v rustup &> /dev/null; then
        log "Building statically linked version with musl..."
        rustup target add x86_64-unknown-linux-musl
        cargo build --release --target x86_64-unknown-linux-musl
        success "Static build completed: target/x86_64-unknown-linux-musl/release/up-man"
    else
        warn "rustup not found, skipping musl build"
    fi
}

# Create distribution packages
create_packages() {
    log "Creating distribution packages..."
    
    # Create directory for packages
    mkdir -p packages
    
    # Get version
    version=$(grep '^version' Cargo.toml | head -n 1 | awk -F '"' '{print $2}')
    
    # Create tarball
    log "Creating tarball..."
    # Copy necessary files to target/release for tarball creation
    cp LICENSE README.md CHANGELOG.md target/release/
    cp -r resources target/release/
    tar -czf "packages/up-man-${version}.tar.gz" -C target/release up-man LICENSE README.md CHANGELOG.md resources
    # Clean up copied files (optional)
    rm target/release/LICENSE target/release/README.md target/release/CHANGELOG.md
    rm -r target/release/resources
    success "Tarball created"
    
    # Create .deb package
    log "Creating Debian package..."
    if command -v cargo-deb &> /dev/null; then
        cargo deb
        mv target/debian/*.deb packages/
        success "Debian package created"
    else
        error "cargo-deb not installed, skipping Debian package"
    fi
    
    # Create .rpm package
    log "Creating RPM package..."
    cargo rpm build -vv --target x86_64-unknown-linux-gnu # Use -vv for very verbose output
    if [ $? -ne 0 ]; then
        echo "[ERROR] RPM package creation failed"
        exit 1
    fi
    
    # Create AppImage
    log "Creating AppImage package..."
    if command -v appimagetool &> /dev/null; then
        # Create AppDir structure
        mkdir -p packages/AppDir/{usr/bin,usr/share/applications,usr/share/icons/hicolor/scalable/apps}
        
        # Copy binary and resources
        cp target/release/up-man packages/AppDir/usr/bin/
        
        # Create desktop file
        cat > packages/AppDir/usr/share/applications/up-man.desktop << EOF
[Desktop Entry]
Name=Universal Package Manager
Comment=Universal Package Manager Updater
Exec=up-man
Icon=up-man
Type=Application
Categories=Utility;
Terminal=true
EOF
        
        # Create simple icon (placeholder - replace with a proper icon)
        echo '<svg xmlns="http://www.w3.org/2000/svg" width="256" height="256">
<rect width="256" height="256" fill="#3498db"/>
<text x="128" y="128" font-family="sans-serif" font-size="80" text-anchor="middle" fill="white">UP</text>
</svg>' > packages/AppDir/usr/share/icons/hicolor/scalable/apps/up-man.svg
        
        # Create AppRun script
        cat > packages/AppDir/AppRun << EOF
#!/bin/bash
cd "\$(dirname "\$0")"
exec usr/bin/up-man "\$@"
EOF
        chmod +x packages/AppDir/AppRun
        
        # Build AppImage
        ARCH=$(uname -m)
        appimagetool packages/AppDir "packages/up-man-${version}-${ARCH}.AppImage"
        success "AppImage package created"
    else
        warn "appimagetool not found, skipping AppImage creation"
        warn "Install it with: sudo apt install appimagetool or visit https://github.com/AppImage/AppImageKit"
    fi
    
    success "All packages created in ./packages directory"
}

# Show summary of created artifacts
show_summary() {
    log "Build Summary:"
    echo "------------------------------------"
    echo "Artifacts created:"
    ls -lh packages/
    
    echo ""
    echo "To install the Debian package:"
    echo "  sudo dpkg -i packages/up-man_*.deb"
    
    echo ""
    echo "To install the RPM package:"
    echo "  sudo rpm -i packages/up-man-*.rpm"
    
    echo ""
    echo "To install from tarball:"
    echo "  tar -xzf packages/up-man-*.tar.gz"
    echo "  sudo cp up-man /usr/local/bin/"
    
    echo ""
    echo "To install the static binary:"
    echo "  sudo cp target/x86_64-unknown-linux-musl/release/up-man /usr/local/bin/"
    echo "------------------------------------"
}

# Main function
main() {
    log "Starting build process for up-man..."
    
    check_deps
    update_changelog
    build_release
    create_packages
    show_summary
    
    success "Build process completed successfully!"
}

# Run main function
main
