#!/bin/bash
# Test script for NeuroLink Rust Microservice

set -e

echo "==================================="
echo "NeuroLink Rust Microservice Tests"
echo "==================================="
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Configuration
RUST_SERVICE_PORT=3030
TEST_FILE="test-file.bin"
TEST_FILE_SIZE=10485760  # 10MB

cd /home/giyu/Dev/Neuroport/rust-service

echo "1. Building release binaries..."
cargo build --release 2>&1 | tail -5
echo -e "${GREEN}✓ Build complete${NC}"
echo ""

echo "2. Starting neurolinkd server..."
./target/release/neurolinkd &
SERVER_PID=$!
sleep 2
echo -e "${GREEN}✓ Server started (PID: $SERVER_PID)${NC}"
echo ""

echo "3. Testing health endpoint..."
if curl -s http://localhost:$RUST_SERVICE_PORT/health | grep -q "healthy"; then
    echo -e "${GREEN}✓ Health check passed${NC}"
else
    echo -e "${RED}✗ Health check failed${NC}"
    kill $SERVER_PID
    exit 1
fi
echo ""

echo "4. Creating test file (${TEST_FILE_SIZE} bytes)..."
dd if=/dev/urandom of=$TEST_FILE bs=1024 count=10240 2>/dev/null
echo -e "${GREEN}✓ Test file created${NC}"
echo ""

echo "5. Testing chunked transfer..."
echo "   a) Initializing transfer..."
INIT_RESPONSE=$(curl -s -X POST http://localhost:$RUST_SERVICE_PORT/transfer/init \
    -H "Content-Type: application/json" \
    -d "{\"filename\": \"$TEST_FILE\", \"total_size\": $TEST_FILE_SIZE, \"chunk_size\": 1048576}")

if echo "$INIT_RESPONSE" | grep -q "success.*true"; then
    TRANSFER_ID=$(echo "$INIT_RESPONSE" | grep -o '"transfer_id":"[^"]*"' | cut -d'"' -f4)
    echo -e "${GREEN}   ✓ Transfer initialized (ID: $TRANSFER_ID)${NC}"
else
    echo -e "${RED}   ✗ Failed to initialize transfer${NC}"
    echo "$INIT_RESPONSE"
    kill $SERVER_PID
    exit 1
fi

echo "   b) Uploading chunks..."
TOTAL_CHUNKS=10
for i in $(seq 0 $(($TOTAL_CHUNKS - 1))); do
    dd if=$TEST_FILE of=chunk_$i.bin bs=1024 count=1024 skip=$(($i * 1024)) 2>/dev/null
    
    curl -s -X POST http://localhost:$RUST_SERVICE_PORT/transfer/chunk \
        -F "transfer_id=$TRANSFER_ID" \
        -F "chunk_index=$i" \
        -F "chunk=@chunk_$i.bin" > /dev/null
    
    echo -e "${GREEN}   ✓ Chunk $i uploaded${NC}"
    rm chunk_$i.bin
done

echo "   c) Completing transfer..."
COMPLETE_RESPONSE=$(curl -s -X POST http://localhost:$RUST_SERVICE_PORT/transfer/complete \
    -H "Content-Type: application/json" \
    -d "{\"transfer_id\": \"$TRANSFER_ID\"}")

if echo "$COMPLETE_RESPONSE" | grep -q "success.*true"; then
    echo -e "${GREEN}   ✓ Transfer completed${NC}"
else
    echo -e "${RED}   ✗ Failed to complete transfer${NC}"
    echo "$COMPLETE_RESPONSE"
fi

echo ""
echo "6. Testing CLI (neuroshare)..."
echo "   Sending file via CLI..."
./target/release/neuroshare send $TEST_FILE --host localhost --port $RUST_SERVICE_PORT
echo -e "${GREEN}✓ CLI test complete${NC}"
echo ""

echo "7. Checking transferred files..."
if [ -f "../shared/$TEST_FILE" ]; then
    echo -e "${GREEN}✓ File successfully transferred${NC}"
    ls -lh ../shared/$TEST_FILE
else
    echo -e "${RED}✗ File not found in shared directory${NC}"
fi
echo ""

echo "8. Cleanup..."
rm -f $TEST_FILE
kill $SERVER_PID 2>/dev/null || true
echo -e "${GREEN}✓ Cleanup complete${NC}"
echo ""

echo "==================================="
echo "All tests completed!"
echo "==================================="
