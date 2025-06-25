#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
DRY_RUN=true
VERSION_TYPE=""
NO_CONFIRM=false

# Function to print colored output
print_info() {
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

# Function to show help
show_help() {
    cat << EOF
Usage: $0 --version <VERSION_TYPE> [--run-for-real] [--no-confirm]

Arguments:
  --version <VERSION_TYPE>  Version bump type (patch, minor, stable)
  --run-for-real           Run actual release (default: dry run)
  --no-confirm             Skip confirmation prompts between steps
  --help                   Show this help message

Example:
  $0 --version patch --run-for-real
  $0 --version minor
  $0 --version patch --run-for-real --no-confirm

EOF
}

# Function to ask for user confirmation
confirm_step() {
    local step_name="$1"
    if [[ "$NO_CONFIRM" == "true" ]]; then
        return 0
    fi
    
    echo
    print_warning "About to proceed with: $step_name"
    read -p "Do you want to continue? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_info "Operation cancelled by user"
        exit 0
    fi
    echo
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
        --no-confirm)
            NO_CONFIRM=true
            shift
            ;;
        --help)
            show_help
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Validate required arguments
if [[ -z "$VERSION_TYPE" ]]; then
    print_error "Version type is required"
    show_help
    exit 1
fi

# Validate version type
case $VERSION_TYPE in
    patch|minor|stable)
        ;;
    *)
        print_error "Invalid version type: $VERSION_TYPE. Must be one of: patch, minor, stable"
        exit 1
        ;;
esac

# Check if cargo-release is installed
if ! command -v cargo-release &> /dev/null; then
    print_error "cargo-release is not installed. Install it with: cargo install cargo-release"
    exit 1
fi

# Function to get current version from Cargo.toml
get_current_version() {
    local crate_path="$1"
    grep '^version = ' "$crate_path/Cargo.toml" | sed 's/version = "\(.*\)"/\1/'
}

# Function to calculate next version
calculate_next_version() {
    local current_version="$1"
    local version_type="$2"
    
    # Parse current version
    IFS='.' read -ra VERSION_PARTS <<< "$current_version"
    local major="${VERSION_PARTS[0]}"
    local minor="${VERSION_PARTS[1]}"
    local patch="${VERSION_PARTS[2]}"
    
    case $version_type in
        patch)
            echo "$major.$minor.$((patch + 1))"
            ;;
        minor)
            echo "$major.$((minor + 1)).0"
            ;;
        stable)
            echo "$((major + 1)).0.0"
            ;;
    esac
}

# Function to update dependency version in Cargo.toml
update_dependency_version() {
    local cargo_toml="$1"
    local dep_name="$2"
    local new_version="$3"
    
    # Update the dependency version
    sed -i.bak "s/^${dep_name} = \"[^\"]*\"/${dep_name} = \"${new_version}\"/" "$cargo_toml"
    
    # Clean up backup file
    rm -f "${cargo_toml}.bak"
}

# Function to run cargo release
run_cargo_release() {
    local crate_path="$1"
    local version_type="$2"
    local dry_run="$3"
    
    print_info "Running cargo release for $crate_path"
    
    cd "$crate_path"
    
    if [[ "$dry_run" == "true" ]]; then
        print_warning "DRY RUN: cargo release $version_type --no-confirm"
        cargo release "$version_type" --dry-run --no-confirm
    else
        print_info "REAL RUN: cargo release $version_type --no-confirm"
        cargo release "$version_type" --no-confirm
    fi
    
    cd - > /dev/null
}

# Function to get the new version after release
get_new_version_after_release() {
    local current_version="$1"
    local version_type="$2"
    calculate_next_version "$current_version" "$version_type"
}

print_info "Starting cargo release process"
print_info "Version type: $VERSION_TYPE"
print_info "Dry run: $DRY_RUN"
if [[ "$NO_CONFIRM" == "true" ]]; then
    print_info "Confirmation prompts: DISABLED"
else
    print_info "Confirmation prompts: ENABLED (use --no-confirm to disable)"
fi
echo

# Define crate paths and their order
declare -a CRATES=("crates/vercel_runtime_router" "crates/vercel_runtime_macro" "crates/vercel_runtime" "crates/vercel_axum")

# Store original directory
ORIGINAL_DIR=$(pwd)

# Step 1: Release vercel_runtime_router
print_info "=== Step 1: Releasing vercel_runtime_router ==="
ROUTER_CURRENT_VERSION=$(get_current_version "crates/vercel_runtime_router")
ROUTER_NEW_VERSION=$(calculate_next_version "$ROUTER_CURRENT_VERSION" "$VERSION_TYPE")
print_info "Current version: $ROUTER_CURRENT_VERSION -> New version: $ROUTER_NEW_VERSION"

confirm_step "Step 1: Release vercel_runtime_router ($ROUTER_CURRENT_VERSION -> $ROUTER_NEW_VERSION)"

run_cargo_release "crates/vercel_runtime_router" "$VERSION_TYPE" "$DRY_RUN"
print_success "vercel_runtime_router release completed"
echo

# Step 2: Update and release vercel_runtime_macro
print_info "=== Step 2: Updating and releasing vercel_runtime_macro ==="
MACRO_CURRENT_VERSION=$(get_current_version "crates/vercel_runtime_macro")
MACRO_NEW_VERSION=$(calculate_next_version "$MACRO_CURRENT_VERSION" "$VERSION_TYPE")
print_info "Current version: $MACRO_CURRENT_VERSION -> New version: $MACRO_NEW_VERSION"
print_info "Will update vercel_runtime_router dependency to version $ROUTER_NEW_VERSION"

confirm_step "Step 2: Update dependencies and release vercel_runtime_macro ($MACRO_CURRENT_VERSION -> $MACRO_NEW_VERSION)"

# Update vercel_runtime_router dependency
print_info "Updating vercel_runtime_router dependency to version $ROUTER_NEW_VERSION"
update_dependency_version "crates/vercel_runtime_macro/Cargo.toml" "vercel_runtime_router" "$ROUTER_NEW_VERSION"

run_cargo_release "crates/vercel_runtime_macro" "$VERSION_TYPE" "$DRY_RUN"
print_success "vercel_runtime_macro release completed"
echo

# Step 3: Update and release vercel_runtime
print_info "=== Step 3: Updating and releasing vercel_runtime ==="
RUNTIME_CURRENT_VERSION=$(get_current_version "crates/vercel_runtime")
RUNTIME_NEW_VERSION=$(calculate_next_version "$RUNTIME_CURRENT_VERSION" "$VERSION_TYPE")
print_info "Current version: $RUNTIME_CURRENT_VERSION -> New version: $RUNTIME_NEW_VERSION"
print_info "Will update vercel_runtime_router dependency to version $ROUTER_NEW_VERSION"
print_info "Will update vercel_runtime_macro dependency to version $MACRO_NEW_VERSION"

confirm_step "Step 3: Update dependencies and release vercel_runtime ($RUNTIME_CURRENT_VERSION -> $RUNTIME_NEW_VERSION)"

# Update dependencies
print_info "Updating vercel_runtime_router dependency to version $ROUTER_NEW_VERSION"
update_dependency_version "crates/vercel_runtime/Cargo.toml" "vercel_runtime_router" "$ROUTER_NEW_VERSION"

print_info "Updating vercel_runtime_macro dependency to version $MACRO_NEW_VERSION"
update_dependency_version "crates/vercel_runtime/Cargo.toml" "vercel_runtime_macro" "$MACRO_NEW_VERSION"

run_cargo_release "crates/vercel_runtime" "$VERSION_TYPE" "$DRY_RUN"
print_success "vercel_runtime release completed"
echo

# Step 4: Update and release vercel_axum
print_info "=== Step 4: Updating and releasing vercel_axum ==="
AXUM_CURRENT_VERSION=$(get_current_version "crates/vercel_axum")
AXUM_NEW_VERSION=$(calculate_next_version "$AXUM_CURRENT_VERSION" "$VERSION_TYPE")
print_info "Current version: $AXUM_CURRENT_VERSION -> New version: $AXUM_NEW_VERSION"
print_info "Will update vercel_runtime dependency to version $RUNTIME_NEW_VERSION"

confirm_step "Step 4: Update dependencies and release vercel_axum ($AXUM_CURRENT_VERSION -> $AXUM_NEW_VERSION)"

# Update vercel_runtime dependency
print_info "Updating vercel_runtime dependency to version $RUNTIME_NEW_VERSION"
update_dependency_version "crates/vercel_axum/Cargo.toml" "vercel_runtime" "$RUNTIME_NEW_VERSION"

run_cargo_release "crates/vercel_axum" "$VERSION_TYPE" "$DRY_RUN"
print_success "vercel_axum release completed"
echo

# Final summary
print_success "=== Release Summary ==="
print_success "vercel_runtime_router: $ROUTER_CURRENT_VERSION -> $ROUTER_NEW_VERSION"
print_success "vercel_runtime_macro: $MACRO_CURRENT_VERSION -> $MACRO_NEW_VERSION"
print_success "vercel_runtime: $RUNTIME_CURRENT_VERSION -> $RUNTIME_NEW_VERSION"
print_success "vercel_axum: $AXUM_CURRENT_VERSION -> $AXUM_NEW_VERSION"

if [[ "$DRY_RUN" == "true" ]]; then
    print_warning "This was a DRY RUN. No actual releases were made."
    print_info "To perform the actual release, run with --run-for-real flag"
else
    print_success "All releases completed successfully!"
fi 