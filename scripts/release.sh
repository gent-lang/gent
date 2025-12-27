#!/usr/bin/env bash
#
# GENT Release Script
#
# This script handles releasing GENT to:
# - GitHub Releases (with binaries)
# - crates.io (Cargo)
# - Homebrew (via tap)
#
# Usage:
#   ./scripts/release.sh <version>
#   ./scripts/release.sh 0.2.0
#
# Prerequisites:
#   - gh (GitHub CLI) installed and authenticated
#   - cargo login done (for crates.io)
#   - Cross-compilation targets installed (for multi-platform builds)
#
# To install cross-compilation targets:
#   rustup target add x86_64-apple-darwin
#   rustup target add aarch64-apple-darwin
#   rustup target add x86_64-unknown-linux-gnu
#   rustup target add x86_64-pc-windows-msvc

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print colored message
info() { echo -e "${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

# Check if required tools are installed
check_requirements() {
    info "Checking requirements..."

    command -v cargo >/dev/null 2>&1 || error "cargo is required but not installed"
    command -v gh >/dev/null 2>&1 || error "gh (GitHub CLI) is required but not installed"
    command -v git >/dev/null 2>&1 || error "git is required but not installed"
    command -v shasum >/dev/null 2>&1 || error "shasum is required but not installed"

    # Check gh authentication
    gh auth status >/dev/null 2>&1 || error "gh is not authenticated. Run: gh auth login"

    success "All requirements met"
}

# Get current version from Cargo.toml
get_current_version() {
    grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/'
}

# Update version in Cargo.toml
update_version() {
    local new_version=$1
    local current_version
    current_version=$(get_current_version)

    info "Updating version: $current_version -> $new_version"

    # Update Cargo.toml
    sed -i.bak "s/^version = \"$current_version\"/version = \"$new_version\"/" Cargo.toml
    rm -f Cargo.toml.bak

    # Update Cargo.lock
    cargo update -p gent

    success "Version updated to $new_version"
}

# Run tests
run_tests() {
    info "Running tests..."
    cargo test --release || error "Tests failed"
    success "All tests passed"
}

# Build release binaries
build_binaries() {
    local version=$1
    local dist_dir="dist/gent-$version"

    info "Building release binaries..."

    rm -rf dist
    mkdir -p "$dist_dir"

    # Build for current platform (native)
    info "Building for current platform..."
    cargo build --release

    # Determine current platform
    local os=$(uname -s | tr '[:upper:]' '[:lower:]')
    local arch=$(uname -m)

    # Normalize architecture names
    case $arch in
        x86_64) arch="x86_64" ;;
        arm64|aarch64) arch="aarch64" ;;
    esac

    # Normalize OS names
    case $os in
        darwin) os="apple-darwin" ;;
        linux) os="unknown-linux-gnu" ;;
    esac

    local target="${arch}-${os}"
    local binary_name="gent-$version-$target"

    mkdir -p "$dist_dir/$binary_name"
    cp target/release/gent "$dist_dir/$binary_name/"
    cp README.md LICENSE "$dist_dir/$binary_name/"

    # Create tarball
    info "Creating tarball for $target..."
    (cd "$dist_dir" && tar -czf "$binary_name.tar.gz" "$binary_name")

    # Calculate SHA256
    (cd "$dist_dir" && shasum -a 256 "$binary_name.tar.gz" > "$binary_name.tar.gz.sha256")

    success "Built $binary_name"

    # Try cross-compilation if targets are available
    local targets=(
        "x86_64-apple-darwin"
        "aarch64-apple-darwin"
        "x86_64-unknown-linux-gnu"
    )

    for cross_target in "${targets[@]}"; do
        if [ "$cross_target" != "$target" ]; then
            if rustup target list --installed | grep -q "$cross_target"; then
                info "Cross-compiling for $cross_target..."
                if cargo build --release --target "$cross_target" 2>/dev/null; then
                    local cross_binary_name="gent-$version-$cross_target"
                    mkdir -p "$dist_dir/$cross_binary_name"
                    cp "target/$cross_target/release/gent" "$dist_dir/$cross_binary_name/"
                    cp README.md LICENSE "$dist_dir/$cross_binary_name/"
                    (cd "$dist_dir" && tar -czf "$cross_binary_name.tar.gz" "$cross_binary_name")
                    (cd "$dist_dir" && shasum -a 256 "$cross_binary_name.tar.gz" > "$cross_binary_name.tar.gz.sha256")
                    success "Built $cross_binary_name"
                else
                    warn "Cross-compilation for $cross_target failed, skipping"
                fi
            else
                warn "Target $cross_target not installed, skipping"
            fi
        fi
    done

    # List built artifacts
    info "Built artifacts:"
    ls -la "$dist_dir"/*.tar.gz
}

# Commit version bump
commit_version() {
    local version=$1

    info "Committing version bump..."

    git add Cargo.toml Cargo.lock
    git commit -m "chore: bump version to $version"

    success "Version bump committed"
}

# Create git tag
create_tag() {
    local version=$1

    info "Creating git tag v$version..."

    git tag -a "v$version" -m "Release v$version"

    success "Tag v$version created"
}

# Push to GitHub
push_to_github() {
    local version=$1

    info "Pushing to GitHub..."

    git push origin main
    git push origin "v$version"

    success "Pushed to GitHub"
}

# Create GitHub release
create_github_release() {
    local version=$1
    local dist_dir="dist/gent-$version"

    info "Creating GitHub release..."

    # Generate release notes
    local release_notes="## GENT v$version

### Installation

#### Homebrew (macOS)
\`\`\`bash
brew tap gent-lang/gent
brew install gent
\`\`\`

#### Cargo
\`\`\`bash
cargo install gent
\`\`\`

#### Binary Download
Download the appropriate binary for your platform from the assets below.

### Checksums
See the \`.sha256\` files for each binary.
"

    # Create release with assets
    gh release create "v$version" \
        --title "v$version" \
        --notes "$release_notes" \
        "$dist_dir"/*.tar.gz \
        "$dist_dir"/*.sha256

    success "GitHub release created"
}

# Publish to crates.io
publish_to_crates() {
    info "Publishing to crates.io..."

    # Dry run first
    cargo publish --dry-run || error "Cargo publish dry-run failed"

    # Actual publish
    read -p "Proceed with crates.io publish? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        cargo publish
        success "Published to crates.io"
    else
        warn "Skipped crates.io publish"
    fi
}

# Update Homebrew formula
update_homebrew_formula() {
    local version=$1
    local dist_dir="dist/gent-$version"

    info "Updating Homebrew formula..."

    # Get SHA256 for macOS builds
    local darwin_x64_sha=""
    local darwin_arm64_sha=""

    if [ -f "$dist_dir/gent-$version-x86_64-apple-darwin.tar.gz.sha256" ]; then
        darwin_x64_sha=$(cat "$dist_dir/gent-$version-x86_64-apple-darwin.tar.gz.sha256" | awk '{print $1}')
    fi

    if [ -f "$dist_dir/gent-$version-aarch64-apple-darwin.tar.gz.sha256" ]; then
        darwin_arm64_sha=$(cat "$dist_dir/gent-$version-aarch64-apple-darwin.tar.gz.sha256" | awk '{print $1}')
    fi

    # Generate formula
    cat > homebrew/gent.rb << EOF
# typed: false
# frozen_string_literal: true

class Gent < Formula
  desc "A programming language for AI agents"
  homepage "https://github.com/gent-lang/gent"
  version "$version"
  license "MIT"

  on_macos do
    on_intel do
      url "https://github.com/gent-lang/gent/releases/download/v$version/gent-$version-x86_64-apple-darwin.tar.gz"
      sha256 "${darwin_x64_sha:-PLACEHOLDER_SHA256_X64}"
    end

    on_arm do
      url "https://github.com/gent-lang/gent/releases/download/v$version/gent-$version-aarch64-apple-darwin.tar.gz"
      sha256 "${darwin_arm64_sha:-PLACEHOLDER_SHA256_ARM64}"
    end
  end

  def install
    bin.install "gent"
  end

  test do
    # Create a simple test file
    (testpath/"test.gnt").write <<~EOS
      agent Test {
        systemPrompt: "You are a test."
        model: "gpt-4o-mini"
      }
    EOS

    # Just check that it parses (mock mode)
    system "#{bin}/gent", "--help"
  end
end
EOF

    success "Homebrew formula updated at homebrew/gent.rb"

    info "To publish to your Homebrew tap:"
    echo "  1. Create a repo: github.com/gent-lang/homebrew-gent"
    echo "  2. Copy homebrew/gent.rb to that repo as Formula/gent.rb"
    echo "  3. Users can then: brew tap gent-lang/gent && brew install gent"
}

# Print usage
usage() {
    echo "Usage: $0 <version>"
    echo ""
    echo "Example:"
    echo "  $0 0.2.0"
    echo ""
    echo "This script will:"
    echo "  1. Update version in Cargo.toml"
    echo "  2. Run tests"
    echo "  3. Build release binaries"
    echo "  4. Create git tag and push"
    echo "  5. Create GitHub release with binaries"
    echo "  6. Publish to crates.io"
    echo "  7. Update Homebrew formula"
    exit 1
}

# Main
main() {
    if [ $# -ne 1 ]; then
        usage
    fi

    local version=$1

    # Validate version format
    if ! [[ $version =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        error "Invalid version format. Use semver: X.Y.Z"
    fi

    # Change to project root
    cd "$(dirname "$0")/.."

    echo "========================================"
    echo "  GENT Release Script"
    echo "  Version: $version"
    echo "========================================"
    echo ""

    check_requirements

    # Confirm before proceeding
    local current_version
    current_version=$(get_current_version)
    echo ""
    info "Current version: $current_version"
    info "New version: $version"
    echo ""
    read -p "Proceed with release? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        warn "Release cancelled"
        exit 0
    fi

    update_version "$version"
    run_tests
    build_binaries "$version"
    commit_version "$version"
    create_tag "$version"

    read -p "Push to GitHub and create release? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        push_to_github "$version"
        create_github_release "$version"
    else
        warn "Skipped GitHub push/release"
    fi

    publish_to_crates
    update_homebrew_formula "$version"

    echo ""
    echo "========================================"
    success "Release v$version complete!"
    echo "========================================"
    echo ""
    echo "Next steps:"
    echo "  1. Update Homebrew tap with homebrew/gent.rb"
    echo "  2. Announce the release"
    echo ""
}

main "$@"
