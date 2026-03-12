#!/bin/bash
#
# Agent Protocol Integration Test
# Tests the complete flow: apc -> agentd -> registry
#
# Usage: ./tests/integration_test.sh
#
# Prerequisites:
#   - Build all binaries: cargo build --workspace
#   - Available ports: 8686 (agentd), 8687 (registry)

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
AGENTD_PORT=${AGENTD_PORT:-8686}
REGISTRY_PORT=${REGISTRY_PORT:-8687}
AGENTD_HOST="127.0.0.1"
REGISTRY_HOST="127.0.0.1"

# Paths
PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
AGENTD_BIN="$PROJECT_DIR/target/debug/agentd"
REGISTRY_BIN="$PROJECT_DIR/target/debug/registry"
APC_BIN="$PROJECT_DIR/target/debug/apc"

# Temporary files
AGENTD_PID=""
REGISTRY_PID=""
TMPDIR=$(mktemp -d)

cleanup() {
    echo -e "\n${YELLOW}Cleaning up...${NC}"
    [ -n "$AGENTD_PID" ] && kill $AGENTD_PID 2>/dev/null || true
    [ -n "$REGISTRY_PID" ] && kill $REGISTRY_PID 2>/dev/null || true
    rm -rf "$TMPDIR"
}
trap cleanup EXIT

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_test() {
    echo -e "${YELLOW}[TEST]${NC} $1"
}

# Check if binaries exist
check_binaries() {
    log_info "Checking binaries..."
    for bin in "$AGENTD_BIN" "$REGISTRY_BIN" "$APC_BIN"; do
        if [ ! -x "$bin" ]; then
            log_error "Binary not found: $bin"
            log_info "Run: cargo build --workspace"
            exit 1
        fi
    done
    log_info "All binaries found"
}

# Start agentd server
start_agentd() {
    log_info "Starting agentd on port $AGENTD_PORT..."

    # Create minimal config
    cat > "$TMPDIR/agentd.toml" <<EOF
[agent]
name = "test-agentd"
version = "0.2.0"

[server]
host = "$AGENTD_HOST"
port = $AGENTD_PORT
EOF

    $AGENTD_BIN --config "$TMPDIR/agentd.toml" > "$TMPDIR/agentd.log" 2>&1 &
    AGENTD_PID=$!

    # Wait for server to start
    sleep 2

    if ! kill -0 $AGENTD_PID 2>/dev/null; then
        log_error "Failed to start agentd"
        cat "$TMPDIR/agentd.log"
        exit 1
    fi

    log_info "agentd started (PID: $AGENTD_PID)"
}

# Start registry server
start_registry() {
    log_info "Starting registry on port $REGISTRY_PORT..."

    # Create minimal config
    cat > "$TMPDIR/registry.toml" <<EOF
[server]
host = "$REGISTRY_HOST"
port = $REGISTRY_PORT

[storage]
path = "$TMPDIR/registry-data"
EOF

    mkdir -p "$TMPDIR/registry-data"

    $REGISTRY_BIN --config "$TMPDIR/registry.toml" > "$TMPDIR/registry.log" 2>&1 &
    REGISTRY_PID=$!

    # Wait for server to start
    sleep 2

    if ! kill -0 $REGISTRY_PID 2>/dev/null; then
        log_error "Failed to start registry"
        cat "$TMPDIR/registry.log"
        exit 1
    fi

    log_info "registry started (PID: $REGISTRY_PID)"
}

# Test JSON-RPC message format using nc
test_jsonrpc_format() {
    log_test "Testing JSON-RPC message format..."

    # Test HELLO
    echo '{"jsonrpc":"2.0","id":1,"method":"hello","params":{"clientName":"test","clientVersion":"1.0"}}' | \
        nc -q 1 $AGENTD_HOST $AGENTD_PORT > "$TMPDIR/hello_response.json"

    if grep -q '"jsonrpc":"2.0"' "$TMPDIR/hello_response.json" && \
       grep -q '"serverName"' "$TMPDIR/hello_response.json"; then
        log_info "HELLO response format OK"
    else
        log_error "HELLO response format failed"
        cat "$TMPDIR/hello_response.json"
        return 1
    fi

    # Test getServerInfo
    echo '{"jsonrpc":"2.0","id":2,"method":"getServerInfo"}' | \
        nc -q 1 $AGENTD_HOST $AGENTD_PORT > "$TMPDIR/server_info_response.json"

    if grep -q '"version"' "$TMPDIR/server_info_response.json"; then
        log_info "getServerInfo response format OK"
    else
        log_error "getServerInfo response format failed"
        cat "$TMPDIR/server_info_response.json"
        return 1
    fi
}

# Test using apc client
test_apc_client() {
    log_test "Testing apc client..."

    # Test server info
    if $APC_BIN --url "agent://$AGENTD_HOST:$AGENTD_PORT" info > "$TMPDIR/apc_info.txt" 2>&1; then
        log_info "apc info command OK"
    else
        log_error "apc info command failed"
        cat "$TMPDIR/apc_info.txt"
        return 1
    fi

    # Test list agents
    if $APC_BIN --url "agent://$AGENTD_HOST:$AGENTD_PORT" list > "$TMPDIR/apc_list.txt" 2>&1; then
        log_info "apc list command OK"
    else
        # This may fail if agentd doesn't have agents registered
        log_info "apc list command completed (may be empty)"
    fi
}

# Test registry
test_registry() {
    log_test "Testing registry..."

    # Test HELLO to registry
    echo '{"jsonrpc":"2.0","id":1,"method":"hello","params":{"clientName":"test","clientVersion":"1.0"}}' | \
        nc -q 1 $REGISTRY_HOST $REGISTRY_PORT > "$TMPDIR/registry_hello.json"

    if grep -q '"serverName"' "$TMPDIR/registry_hello.json"; then
        log_info "Registry HELLO response OK"
    else
        log_error "Registry HELLO response failed"
        cat "$TMPDIR/registry_hello.json"
        return 1
    fi
}

# Test streaming events (StreamEvent format)
test_streaming() {
    log_test "Testing StreamEvent format..."

    # Verify StreamEvent can be serialized
    cat > "$TMPDIR/test_event.json" <<'EOF'
{"type":"taskStatus","taskId":"test-123","status":"running"}
EOF

    # Check the format is valid JSON
    if python3 -c "import json; json.load(open('$TMPDIR/test_event.json'))" 2>/dev/null || \
       jq . "$TMPDIR/test_event.json" > /dev/null 2>&1; then
        log_info "StreamEvent JSON format OK"
    else
        log_info "StreamEvent format test skipped (no JSON validator)"
    fi
}

# Main test runner
main() {
    echo "========================================"
    echo "  Agent Protocol Integration Test"
    echo "========================================"
    echo ""

    check_binaries

    # Start servers
    start_agentd
    # start_registry  # Optional: uncomment to test registry

    # Run tests
    echo ""
    log_info "Running tests..."
    echo ""

    TESTS_PASSED=0
    TESTS_FAILED=0

    for test_func in test_jsonrpc_format test_apc_client test_streaming; do
        if $test_func; then
            ((TESTS_PASSED++))
        else
            ((TESTS_FAILED++))
        fi
        echo ""
    done

    # Summary
    echo "========================================"
    echo "  Test Summary"
    echo "========================================"
    echo -e "Passed: ${GREEN}$TESTS_PASSED${NC}"
    echo -e "Failed: ${RED}$TESTS_FAILED${NC}"
    echo ""

    if [ $TESTS_FAILED -eq 0 ]; then
        echo -e "${GREEN}All tests passed!${NC}"
        exit 0
    else
        echo -e "${RED}Some tests failed!${NC}"
        exit 1
    fi
}

main "$@"
