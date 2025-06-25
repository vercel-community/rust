#!/usr/bin/env bash

set -euo pipefail

# Default values
DRY_RUN=true
VERSION_TYPE=""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to show usage
usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Release vercel rust crates in dependency order.

OPTIONS:
    --run-for-real    Actually run cargo release (default: dry run)
    --version TYPE    Version bump type: patch|minor|stable
    -h, --help        Show this help message

EXAMPLES:
    $0 --version patch                    # Dry run patch release
    $0 --run-for-real --version minor     # Actually release with minor bump
    $0 --run-for-real --version stable    # Actually release with stable bump

EOF
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --run-for-real)
            DRY_RUN=false
            shift
            ;;
        --version)
            VERSION_TYPE="$2"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Validate required arguments
if [[ -z "$VERSION_TYPE" ]]; then
    print_error "Version type is required. Use --version patch|minor|stable"
    usage
    exit 1
fi

if [[ "$VERSION_TYPE" != "patch" && "$VERSION_TYPE" != "minor" && "$VERSION_TYPE" != "stable" ]]; then
    print_error "Invalid version type: $VERSION_TYPE. Must be patch, minor, or stable"
    exit 1
fi

# Check if cargo-release is installed
if ! command -v cargo-release &> /dev/null; then
    print_error "cargo-release is not installed. Install it with: cargo install cargo-release"
    exit 1
fi

# Crates in dependency order
CRATES=(
    "vercel_runtime_router"
    "vercel_runtime_macro" 
    "vercel_runtime"
    "vercel_axum"
)

# Function to get current version of a crate
get_crate_version() {
    local crate_name=$1
    local cargo_toml="crates/$crate_name/Cargo.toml"
    
    if [[ ! -f "$cargo_toml" ]]; then
        print_error "Cargo.toml not found for $crate_name at $cargo_toml"
        return 1
    fi
    
    # Extract version from Cargo.toml
    grep '^version = ' "$cargo_toml" | sed 's/version = "\(.*\)"/\1/' | tr -d '"'
}

# Function to update dependency version in a crate's Cargo.toml
update_dependency_version() {
    local dependent_crate=$1
    local dependency_crate=$2
    local new_version=$3
    local cargo_toml="crates/$dependent_crate/Cargo.toml"
    
    if [[ ! -f "$cargo_toml" ]]; then
        print_warning "Cargo.toml not found for $dependent_crate"
        return 0
    fi
    
    # Check if the dependency exists in the file
    if grep -q "^$dependency_crate = " "$cargo_toml"; then
        print_status "Updating $dependency_crate dependency in $dependent_crate to version $new_version"
        
        if [[ "$DRY_RUN" == "false" ]]; then
            # Update the dependency version
            sed -i.bak "s/^$dependency_crate = \"[^\"]*\"/$dependency_crate = \"$new_version\"/" "$cargo_toml"
            rm -f "$cargo_toml.bak"
        else
            print_status "DRY RUN: Would update $dependency_crate version to $new_version in $dependent_crate"
        fi
    fi
}

# Function to release a single crate
release_crate() {
    local crate_name=$1
    local crate_path="crates/$crate_name"
    
    print_status "Processing crate: $crate_name"
    
    if [[ ! -d "$crate_path" ]]; then
        print_error "Crate directory not found: $crate_path"
        return 1
    fi
    
    # Get current version
    local current_version
    current_version=$(get_crate_version "$crate_name")
    print_status "Current version of $crate_name: $current_version"
    
    # Change to crate directory
    pushd "$crate_path" > /dev/null
    
    # Prepare release command
    local release_cmd="cargo release"
    
    # Add version type
    case "$VERSION_TYPE" in
        "patch")
            release_cmd="$release_cmd patch"
            ;;
        "minor")
            release_cmd="$release_cmd minor"
            ;;
        "stable")
            release_cmd="$release_cmd release"
            ;;
    esac
    
    # Add dry-run flag if needed
    if [[ "$DRY_RUN" == "true" ]]; then
        release_cmd="$release_cmd --dry-run"
        print_status "DRY RUN: $release_cmd"
    else
        print_status "EXECUTING: $release_cmd"
    fi
    
    # Execute the release command
    if $release_cmd; then
        print_success "Successfully processed release for $crate_name"
        
        # Get the new version after release (for real releases)
        if [[ "$DRY_RUN" == "false" ]]; then
            local new_version
            new_version=$(get_crate_version "$crate_name")
            print_success "New version of $crate_name: $new_version"
            
            # Return to root directory
            popd > /dev/null
            
            # Update dependencies in other crates
            update_dependencies_after_release "$crate_name" "$new_version"
            
            # Run cargo update to refresh Cargo.lock
            print_status "Running cargo update to refresh Cargo.lock"
            cargo update
            
        else
            popd > /dev/null
        fi
    else
        print_error "Failed to release $crate_name"
        popd > /dev/null
        return 1
    fi
}

# Function to update dependencies after a crate is released
update_dependencies_after_release() {
    local released_crate=$1
    local new_version=$2
    
    print_status "Updating dependencies for released crate $released_crate (version $new_version)"
    
    # Define which crates depend on which
    case "$released_crate" in
        "vercel_runtime_router")
            update_dependency_version "vercel_runtime_macro" "vercel_runtime_router" "$new_version"
            update_dependency_version "vercel_runtime" "vercel_runtime_router" "$new_version"
            ;;
        "vercel_runtime_macro")
            update_dependency_version "vercel_runtime" "vercel_runtime_macro" "$new_version"
            ;;
        "vercel_runtime")
            update_dependency_version "vercel_axum" "vercel_runtime" "$new_version"
            ;;
    esac
}

# Main execution
main() {
    print_status "Starting cargo release process"
    print_status "Mode: $(if [[ "$DRY_RUN" == "true" ]]; then echo "DRY RUN"; else echo "LIVE RELEASE"; fi)"
    print_status "Version type: $VERSION_TYPE"
    print_status "Release order: ${CRATES[*]}"
    
    if [[ "$DRY_RUN" == "false" ]]; then
        print_warning "This will actually release crates to crates.io!"
        read -p "Are you sure you want to continue? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            print_status "Release cancelled by user"
            exit 0
        fi
    fi
    
    # Release each crate in order
    for crate in "${CRATES[@]}"; do
        print_status "============================================"
        release_crate "$crate"
        
        # Add a small delay between releases to avoid rate limiting
        if [[ "$DRY_RUN" == "false" ]]; then
            print_status "Waiting 10 seconds before next release..."
            sleep 10
        fi
    done
    
    print_success "============================================"
    print_success "All crates processed successfully!"
    
    if [[ "$DRY_RUN" == "true" ]]; then
        print_status "This was a dry run. Use --run-for-real to actually release."
    else
        print_success "All crates have been released!"
        print_status "Don't forget to commit and push any Cargo.toml changes."
    fi
}

# Run main function
main "$@" 