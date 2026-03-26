#!/bin/bash
set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Log functions
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

# Function to run a Docker container and test installation
test_in_container() {
    local distro=$1
    local package_type=$2
    local package_path=$3
    
    log "Testing on $distro using $package_type..."
    
    case "$package_type" in
        "deb")
            docker run --rm -v "$(pwd)/packages:/packages" $distro bash -c "
                echo 'Installing $package_path...' &&
                apt-get update -y &&
                apt-get install -y /packages/$(basename $package_path) &&
                up-man --version &&
                echo 'Testing run command...' &&
                up-man validate
            "
            ;;
        "rpm")
            docker run --rm -v "$(pwd)/packages:/packages" $distro bash -c "
                echo 'Installing $package_path...' &&
                dnf install -y /packages/$(basename $package_path) &&
                up-man --version &&
                echo 'Testing run command...' &&
                up-man validate
            "
            ;;
        "appimage")
            docker run --rm -v "$(pwd)/packages:/packages" $distro bash -c "
                echo 'Testing AppImage...' &&
                chmod +x /packages/$(basename $package_path) &&
                /packages/$(basename $package_path) --version &&
                echo 'Testing run command...' &&
                /packages/$(basename $package_path) validate
            "
            ;;
        "binary")
            docker run --rm -v "$(pwd)/packages:/packages" $distro bash -c "
                echo 'Testing binary...' &&
                tar -xzf /packages/$(basename $package_path) -C /tmp &&
                chmod +x /tmp/up-man &&
                /tmp/up-man --version &&
                echo 'Testing run command...' &&
                /tmp/up-man validate
            "
            ;;
        *)
            error "Unknown package type: $package_type"
            return 1
            ;;
    esac
    
    if [ $? -eq 0 ]; then
        success "Test successful on $distro using $package_type"
    else
        error "Test failed on $distro using $package_type"
        return 1
    fi
}

# Main function
main() {
    log "Starting package testing on multiple platforms..."
    
    # Make sure we have Docker
    if ! command -v docker &> /dev/null; then
        error "Docker is required for testing. Please install Docker and try again."
        exit 1
    fi
    
    # Check if packages directory exists
    if [ ! -d "packages" ]; then
        error "Packages directory not found. Run build-release.sh first."
        exit 1
    fi
    
    # Find packages
    deb_pkg=$(find packages -name "*.deb" | head -n 1)
    rpm_pkg=$(find packages -name "*.rpm" | head -n 1)
    appimage_pkg=$(find packages -name "*.AppImage" | head -n 1)
    tar_pkg=$(find packages -name "*.tar.gz" | head -n 1)
    
    # Test on Debian-based distributions
    if [ -n "$deb_pkg" ]; then
        test_in_container "ubuntu:latest" "deb" "$deb_pkg"
        test_in_container "debian:stable" "deb" "$deb_pkg"
    else
        warn "No .deb package found to test"
    fi
    
    # Test on RPM-based distributions
    if [ -n "$rpm_pkg" ]; then
        test_in_container "fedora:latest" "rpm" "$rpm_pkg"
        test_in_container "almalinux:latest" "rpm" "$rpm_pkg"
    else
        warn "No .rpm package found to test"
    fi
    
    # Test AppImage
    if [ -n "$appimage_pkg" ]; then
        test_in_container "ubuntu:latest" "appimage" "$appimage_pkg"
        test_in_container "fedora:latest" "appimage" "$appimage_pkg"
    else
        warn "No AppImage package found to test"
    fi
    
    # Test tarball with static binary
    if [ -n "$tar_pkg" ]; then
        test_in_container "alpine:latest" "binary" "$tar_pkg"
    else
        warn "No .tar.gz package found to test"
    fi
    
    log "Package testing completed!"
}

# Run main function
main "$@"
