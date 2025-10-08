#!/usr/bin/env bash

set -e

# Change to project root directory (parent of scripts)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_ROOT"

BUILD_DIR="build"

function show_usage() {
    echo "Usage: $0 [command] [args...]"
    echo ""
    echo "Commands:"
    echo "  test [filter]     Build and run tests (optional gtest filter)"
    echo "  interp [args...]  Build and run interpreter with arguments"
    echo "  compile [args...] Build and run compiler with arguments"
    echo "  build             Build only (no run)"
    echo ""
    echo "Examples:"
    echo "  $0 test                    # Run all tests"
    echo "  $0 test "LexerTest.*"      # Run only lexer tests"
    echo "  $0 interp program.clause   # Run interpreter on file"
    echo "  $0 compile output.o input.clause"
    exit 1
}

function build() {
    echo "Building project..."
    cmake --build "$BUILD_DIR"
    echo ""
}

if [ -z "$1" ]; then
    show_usage
fi

COMMAND="$1"
shift

case "$COMMAND" in
    test)
        build
        if [ -n "$1" ]; then
            echo "Running tests with filter: $1"
            "./$BUILD_DIR/clause_tests" --gtest_filter="$1"
        else
            echo "Running all tests..."
            "./$BUILD_DIR/clause_tests"
        fi
        ;;
    interp)
        build
        echo "Running interpreter..."
        "./$BUILD_DIR/clause" "$@"
        ;;
    compile)
        build
        echo "Running compiler..."
        "./$BUILD_DIR/clausec" "$@"
        ;;
    build)
        build
        echo "Build complete!"
        ;;
    *)
        echo "Error: Unknown command '$COMMAND'"
        echo ""
        show_usage
        ;;
esac
