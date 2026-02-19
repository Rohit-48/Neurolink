# Testing NeuroLink Rust Microservice

## Quick Start - Automated Testing

```bash
cd /home/giyu/Dev/Neuroport/rust-service
./test.sh
```

This script will:
1. Build release binaries
2. Start the server
3. Test health endpoint
4. Create a 10MB test file
5. Test chunked transfer via API
6. Test CLI (neuroshare)
7. Verify file integrity
8. Cleanup

## Manual Testing

### 1. Build the Project

```bash
cd /home/giyu/Dev/Neuroport/rust-service
cargo build --release
```

Binaries created:
- `target/release/neurolinkd` - Server
- `target/release/neuroshare` - CLI

### 2. Start the Server

Terminal 1:
```bash
./target/release/neurolinkd
```

Expected output:
```
INFO neurolinkd: Starting NeuroLink Rust Microservice v2.0.0
INFO neurolinkd: Storage path: ./shared
INFO neurolinkd: Listening on port: 3030
INFO neurolinkd: Server starting on http://0.0.0.0:3030
```

### 3. Test Health Endpoint

Terminal 2:
```bash
curl http://localhost:3030/health
```

Expected:
```json
{"success":true,"data":"healthy","error":null}
```

### 4. Test Chunked Transfer API

#### Step 1: Initialize Transfer
```bash
# Create test file
dd if=/dev/urandom of=test.bin bs=1024 count=1024

# Initialize transfer
curl -X POST http://localhost:3030/transfer/init \
  -H "Content-Type: application/json" \
  -d '{
    "filename": "test.bin",
    "total_size": 1048576,
    "chunk_size": 262144
  }'
```

Save the `transfer_id` from response.

#### Step 2: Upload Chunks
```bash
# Split file into 4 chunks (256KB each)
split -b 262144 test.bin chunk_

# Upload chunks (replace TRANSFER_ID with actual ID)
TRANSFER_ID="trans_..."

for i in 0 1 2 3; do
  curl -X POST http://localhost:3030/transfer/chunk \
    -F "transfer_id=$TRANSFER_ID" \
    -F "chunk_index=$i" \
    -F "chunk=@chunk_$(printf '%c' $(printf '%02d' $i | tr '0' 'a'))"
done
```

#### Step 3: Complete Transfer
```bash
curl -X POST http://localhost:3030/transfer/complete \
  -H "Content-Type: application/json" \
  -d '{"transfer_id": "'$TRANSFER_ID'"}'
```

#### Step 4: Check Status
```bash
curl http://localhost:3030/transfer/$TRANSFER_ID/status
```

### 5. Test CLI Tool

#### Send Single File
```bash
# Create test file
dd if=/dev/urandom of=large-file.bin bs=1024 count=10240

# Send with progress bar
./target/release/neuroshare send large-file.bin \
  --host localhost \
  --port 3030
```

#### Send Multiple Files
```bash
./target/release/neuroshare send file1.bin file2.bin file3.bin \
  --host 192.168.1.100 \
  --port 3030
```

#### Custom Chunk Size
```bash
./target/release/neuroshare send huge-file.zip \
  --host localhost \
  --port 3030 \
  --chunk-size 4096  # 4MB chunks
```

### 6. Verify Transfer

```bash
# Check file exists in shared directory
ls -lh shared/

# Verify file integrity (compare hashes)
sha256sum test.bin
sha256sum shared/test.bin
```

### 7. Test Concurrent Uploads

Terminal 1:
```bash
# Start server
./target/release/neurolinkd
```

Terminal 2 & 3:
```bash
# Simultaneous uploads from different terminals
./target/release/neuroshare send file1.bin --host localhost --port 3030
```

```bash
./target/release/neuroshare send file2.bin --host localhost --port 3030
```

### 8. Test Error Handling

#### Invalid Transfer ID
```bash
curl http://localhost:3030/transfer/invalid-id/status
```

Expected: Error response

#### Out of Order Chunks
```bash
# Try uploading chunk 5 before chunk 0
curl -X POST http://localhost:3030/transfer/chunk \
  -F "transfer_id=$TRANSFER_ID" \
  -F "chunk_index=5" \
  -F "chunk=@chunk_aa"
```

### 9. Performance Testing

#### Large File Test
```bash
# Create 100MB file
dd if=/dev/urandom of=big-file.bin bs=1024 count=102400

# Time the transfer
time ./target/release/neuroshare send big-file.bin \
  --host localhost \
  --port 3030
```

#### Compare with Node.js
```bash
# Time Node.js version
time node dist/cli/share.js send big-file.bin --host localhost --port 3000

# Time Rust version
time ./target/release/neuroshare send big-file.bin --host localhost --port 3030
```

### 10. Stress Testing

```bash
# Create many small files
for i in {1..100}; do
  dd if=/dev/urandom of=small-$i.bin bs=1024 count=10
done

# Upload all
./target/release/neuroshare send small-*.bin --host localhost --port 3030
```

## Testing Integration with Node.js

### Full Stack Test

Terminal 1 - Start Node.js server:
```bash
cd /home/giyu/Dev/Neuroport
node dist/cli/main.js --port 3000
```

Terminal 2 - Start Rust service:
```bash
cd /home/giyu/Dev/Neuroport/rust-service
./target/release/neurolinkd
```

Terminal 3 - Send file:
```bash
./rust-service/target/release/neuroshare send file.bin \
  --host localhost \
  --port 3030
```

Check both services work together:
- Node.js web UI: http://localhost:3000
- Rust API: http://localhost:3030/health

## Debugging

### Enable Debug Logging
```bash
RUST_LOG=debug ./target/release/neurolinkd
```

### Check Server Logs
```bash
# In another terminal
curl http://localhost:3030/health
# Watch server output for request logs
```

### Test with Verbose Output
```bash
# Add -v flag if implemented
./target/release/neuroshare send file.bin -v
```

## Expected Results

✅ **Success Indicators:**
- Health check returns `healthy`
- Transfer completes without errors
- File appears in `shared/` directory
- File hash matches original
- Progress bar shows completion
- Server logs show successful requests

❌ **Error Indicators:**
- Connection refused (server not running)
- Transfer ID not found (wrong ID)
- Hash mismatch (corruption)
- Timeout (network issues)

## Cleanup

```bash
# Remove test files
rm -f test.bin chunk_* small-*.bin large-file.bin big-file.bin

# Stop server
Ctrl+C

# Remove shared files
rm -rf shared/*
```

## Troubleshooting

### Port Already in Use
```bash
# Find process
lsof -i :3030

# Kill it
kill <PID>

# Or use different port
./target/release/neurolinkd --port 3031
```

### Permission Denied
```bash
# Make binaries executable
chmod +x target/release/neurolinkd
chmod +x target/release/neuroshare
```

### Build Errors
```bash
# Clean and rebuild
cargo clean
cargo build --release
```

## Next Steps

After successful testing:
1. ✅ Chunked transfers working
2. ✅ CLI tool functional
3. ✅ Progress tracking accurate
4. ✅ Error handling robust
5. 🔄 Ready for: Hash deduplication, compression, bandwidth optimization
