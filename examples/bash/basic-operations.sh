#!/bin/bash
# Basic File Operations Example
# This script demonstrates basic CRUD operations with the PMP File API

set -e  # Exit on error

# Configuration
API_BASE="${API_BASE_URL:-http://localhost:3000}"
STORAGE="${STORAGE_NAME:-local-storage}"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== PMP File API - Basic Operations Demo ===${NC}\n"

# Create test file
TEST_FILE="demo-file.txt"
echo "This is a test file created at $(date)" > /tmp/$TEST_FILE

echo -e "${GREEN}1. Uploading file...${NC}"
curl -X PUT "$API_BASE/api/v1/file/$STORAGE" \
  -F "file=@/tmp/$TEST_FILE" \
  -F 'metadata={"description": "Demo file", "category": "test"}' \
  -s | jq '.'

echo -e "\n${GREEN}2. Getting file metadata...${NC}"
curl -s "$API_BASE/api/v1/file/$STORAGE/$TEST_FILE/metadata" | jq '.'

echo -e "\n${GREEN}3. Listing all files...${NC}"
curl -s "$API_BASE/api/v1/file/$STORAGE" | jq '.[] | {name: .file_name, size: .size, created: .created_at}'

echo -e "\n${GREEN}4. Downloading file...${NC}"
curl -s "$API_BASE/api/v1/file/$STORAGE/$TEST_FILE" -o /tmp/downloaded-$TEST_FILE
echo "Downloaded to: /tmp/downloaded-$TEST_FILE"
cat /tmp/downloaded-$TEST_FILE

echo -e "\n${GREEN}5. Updating file tags...${NC}"
curl -X PUT "$API_BASE/api/v1/file/$STORAGE/$TEST_FILE/tags" \
  -H "Content-Type: application/json" \
  -d '{"tags": ["demo", "test", "example"]}' \
  -s | jq '{name: .file_name, tags: .tags}'

echo -e "\n${GREEN}6. Searching for files...${NC}"
curl -X POST "$API_BASE/api/v1/search/$STORAGE" \
  -H "Content-Type: application/json" \
  -d '{"query": "demo", "tags": ["test"]}' \
  -s | jq '.results[] | {name: .file_name, tags: .tags}'

echo -e "\n${GREEN}7. Soft deleting file...${NC}"
curl -X DELETE "$API_BASE/api/v1/file/$STORAGE/$TEST_FILE" -s | jq '.'

echo -e "\n${GREEN}8. Checking trash...${NC}"
curl -s "$API_BASE/api/v1/trash/$STORAGE" | jq '.[] | {name: .file_name, deleted_at: .deleted_at}'

echo -e "\n${GREEN}9. Restoring file from trash...${NC}"
curl -X POST "$API_BASE/api/v1/trash/$STORAGE/$TEST_FILE/restore" -s | jq '{name: .file_name, deleted: .is_deleted}'

echo -e "\n${GREEN}10. Permanently deleting file...${NC}"
curl -X DELETE "$API_BASE/api/v1/file/$STORAGE/$TEST_FILE" -s | jq '.'
curl -X DELETE "$API_BASE/api/v1/trash/$STORAGE" -s | jq '.'

# Cleanup
rm -f /tmp/$TEST_FILE /tmp/downloaded-$TEST_FILE

echo -e "\n${BLUE}=== Demo Complete ===${NC}"
echo -e "${YELLOW}Check DOCUMENTATION.md for more examples!${NC}"
