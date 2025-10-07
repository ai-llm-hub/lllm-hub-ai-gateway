#!/bin/bash

# AI Gateway Run Script
# Usage: ./run.sh [command] [options]

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
PORT=${PORT:-3001}
HOST=${HOST:-0.0.0.0}
ENVIRONMENT=${ENVIRONMENT:-development}
MONGODB_URL=${MONGODB_URL:-mongodb://localhost:27017}
MONGODB_DATABASE=${MONGODB_DATABASE:-llm_hub_dev}

# Functions
print_header() {
    echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║        AI Gateway Run Script           ║${NC}"
    echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
    echo ""
}

print_usage() {
    echo -e "${GREEN}Usage:${NC}"
    echo "  ./run.sh [command] [options]"
    echo ""
    echo -e "${GREEN}Commands:${NC}"
    echo "  build          Build the project (debug mode)"
    echo "  build --release Build the project (release mode)"
    echo "  run            Run the server (debug mode)"
    echo "  run --release  Run the server (release mode)"
    echo "  test           Run all tests"
    echo "  check          Check code without building"
    echo "  clippy         Run Clippy linter"
    echo "  fmt            Format code"
    echo "  clean          Clean build artifacts"
    echo "  watch          Watch and rebuild on changes"
    echo "  health         Check health endpoint"
    echo "  help           Show this help message"
    echo ""
    echo -e "${GREEN}Environment Variables:${NC}"
    echo "  PORT                 Server port (default: 3001)"
    echo "  HOST                 Server host (default: 0.0.0.0)"
    echo "  ENVIRONMENT          Environment (default: development)"
    echo "  MONGODB_URL          MongoDB connection URL"
    echo "  MONGODB_DATABASE     MongoDB database name"
    echo ""
}

check_requirements() {
    echo -e "${YELLOW}Checking requirements...${NC}"

    # Check if Rust is installed
    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}Error: Rust is not installed${NC}"
        echo "Please install Rust from https://rustup.rs/"
        exit 1
    fi

    # Check if MongoDB is running (optional check)
    if command -v mongosh &> /dev/null; then
        if mongosh --eval "db.version()" --quiet $MONGODB_URL > /dev/null 2>&1; then
            echo -e "${GREEN}✓ MongoDB is running${NC}"
        else
            echo -e "${YELLOW}⚠ MongoDB connection failed (server may still work with remote DB)${NC}"
        fi
    fi

    echo -e "${GREEN}✓ All requirements met${NC}"
    echo ""
}

build_app() {
    local mode=${1:-debug}
    echo -e "${BLUE}Building AI Gateway (${mode} mode)...${NC}"

    if [ "$mode" == "release" ]; then
        cargo build --release
        echo -e "${GREEN}✓ Release build complete${NC}"
    else
        cargo build
        echo -e "${GREEN}✓ Debug build complete${NC}"
    fi
}

run_app() {
    local mode=${1:-debug}
    echo -e "${BLUE}Starting AI Gateway...${NC}"
    echo -e "${YELLOW}Environment: ${ENVIRONMENT}${NC}"
    echo -e "${YELLOW}Server: http://${HOST}:${PORT}${NC}"
    echo -e "${YELLOW}Swagger UI: http://${HOST}:${PORT}/swagger-ui/${NC}"
    echo ""

    export AI_GATEWAY_SERVER_HOST=$HOST
    export AI_GATEWAY_SERVER_PORT=$PORT
    export AI_GATEWAY_DATABASE_MONGODB_URL=$MONGODB_URL
    export AI_GATEWAY_DATABASE_MONGODB_DATABASE=$MONGODB_DATABASE
    export ENVIRONMENT=$ENVIRONMENT

    if [ "$mode" == "release" ]; then
        cargo run --release
    else
        cargo run
    fi
}

test_app() {
    echo -e "${BLUE}Running tests...${NC}"
    cargo test
    echo -e "${GREEN}✓ All tests passed${NC}"
}

check_code() {
    echo -e "${BLUE}Checking code...${NC}"
    cargo check
    echo -e "${GREEN}✓ Code check complete${NC}"
}

run_clippy() {
    echo -e "${BLUE}Running Clippy...${NC}"
    cargo clippy -- -D warnings
    echo -e "${GREEN}✓ Clippy analysis complete${NC}"
}

format_code() {
    echo -e "${BLUE}Formatting code...${NC}"
    cargo fmt
    echo -e "${GREEN}✓ Code formatted${NC}"
}

clean_build() {
    echo -e "${BLUE}Cleaning build artifacts...${NC}"
    cargo clean
    echo -e "${GREEN}✓ Build artifacts cleaned${NC}"
}

watch_changes() {
    echo -e "${BLUE}Watching for changes...${NC}"

    # Check if cargo-watch is installed
    if ! cargo watch --version &> /dev/null; then
        echo -e "${YELLOW}Installing cargo-watch...${NC}"
        cargo install cargo-watch
    fi

    echo -e "${YELLOW}Watching files, server will restart on changes...${NC}"
    cargo watch -x run
}

check_health() {
    echo -e "${BLUE}Checking health endpoint...${NC}"

    local health_url="http://${HOST}:${PORT}/health"
    echo -e "${YELLOW}Checking: ${health_url}${NC}"

    if command -v curl &> /dev/null; then
        response=$(curl -s $health_url)
        if [ $? -eq 0 ]; then
            echo -e "${GREEN}✓ Server is healthy${NC}"
            echo -e "${GREEN}Response: ${response}${NC}"
        else
            echo -e "${RED}✗ Server is not responding${NC}"
        fi
    else
        echo -e "${YELLOW}curl is not installed, cannot check health${NC}"
    fi
}

# Main script
print_header

# Parse command
case "${1:-help}" in
    build)
        check_requirements
        build_app "${2}"
        ;;
    run)
        check_requirements
        run_app "${2}"
        ;;
    test)
        test_app
        ;;
    check)
        check_code
        ;;
    clippy)
        run_clippy
        ;;
    fmt|format)
        format_code
        ;;
    clean)
        clean_build
        ;;
    watch)
        check_requirements
        watch_changes
        ;;
    health)
        check_health
        ;;
    help|--help|-h)
        print_usage
        ;;
    *)
        echo -e "${RED}Unknown command: ${1}${NC}"
        echo ""
        print_usage
        exit 1
        ;;
esac