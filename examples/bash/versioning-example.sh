#!/bin/bash
# File Versioning Example
# Demonstrates version control for files

set -e

# Configuration
API_BASE="${API_BASE_URL:-http://localhost:3000}"
STORAGE="${STORAGE_NAME:-local-storage}"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}=== File Versioning Demo ===${NC}\n"

# Create test document with versions
DOC_NAME="contract.txt"

echo -e "${GREEN}1. Creating initial version (v1)...${NC}"
echo "Contract Version 1.0 - Initial Draft" > /tmp/$DOC_NAME
echo "Created: $(date)" >> /tmp/$DOC_NAME
echo "Terms: Basic terms here" >> /tmp/$DOC_NAME

UPLOAD_RESPONSE=$(curl -X PUT "$API_BASE/api/v1/file/$STORAGE" \
  -F "file=@/tmp/$DOC_NAME" \
  -F 'metadata={"version_label": "1.0", "status": "draft"}' \
  -s)

echo "$UPLOAD_RESPONSE" | jq '{name: .file_name, version: .version, version_id: .version_id}'
V1_ID=$(echo "$UPLOAD_RESPONSE" | jq -r '.version_id')

sleep 1

echo -e "\n${GREEN}2. Creating version 2 with revisions...${NC}"
echo "Contract Version 2.0 - Revised" > /tmp/$DOC_NAME
echo "Created: $(date)" >> /tmp/$DOC_NAME
echo "Terms: Updated terms with client feedback" >> /tmp/$DOC_NAME

V2_RESPONSE=$(curl -X POST "$API_BASE/api/v1/file/$STORAGE/$DOC_NAME/versions" \
  -F "file=@/tmp/$DOC_NAME" \
  -s)

echo "$V2_RESPONSE" | jq '{name: .file_name, version: .version, version_id: .version_id, parent: .parent_version_id}'
V2_ID=$(echo "$V2_RESPONSE" | jq -r '.version_id')

sleep 1

echo -e "\n${GREEN}3. Creating version 3 (final)...${NC}"
echo "Contract Version 3.0 - FINAL" > /tmp/$DOC_NAME
echo "Created: $(date)" >> /tmp/$DOC_NAME
echo "Terms: Final approved terms" >> /tmp/$DOC_NAME
echo "Signatures: [Placeholder]" >> /tmp/$DOC_NAME

V3_RESPONSE=$(curl -X POST "$API_BASE/api/v1/file/$STORAGE/$DOC_NAME/versions" \
  -F "file=@/tmp/$DOC_NAME" \
  -s)

echo "$V3_RESPONSE" | jq '{name: .file_name, version: .version, version_id: .version_id, parent: .parent_version_id}'
V3_ID=$(echo "$V3_RESPONSE" | jq -r '.version_id')

echo -e "\n${GREEN}4. Listing all versions...${NC}"
curl -s "$API_BASE/api/v1/file/$STORAGE/$DOC_NAME/versions" | \
  jq '.[] | {version: .version, version_id: .version_id, created: .created_at, parent: .parent_version_id}'

echo -e "\n${GREEN}5. Downloading specific version (v2)...${NC}"
curl -s "$API_BASE/api/v1/file/$STORAGE/$DOC_NAME/versions/$V2_ID" \
  -o /tmp/contract-v2.txt
echo "Content of version 2:"
cat /tmp/contract-v2.txt

echo -e "\n${GREEN}6. Simulating rollback - restoring version 2...${NC}"
RESTORE_RESPONSE=$(curl -X POST "$API_BASE/api/v1/file/$STORAGE/$DOC_NAME/versions/$V2_ID/restore" -s)
echo "$RESTORE_RESPONSE" | jq '{version: .version, version_id: .version_id, message: "Restored from v2"}'

echo -e "\n${GREEN}7. Final version history...${NC}"
curl -s "$API_BASE/api/v1/file/$STORAGE/$DOC_NAME/versions" | \
  jq -r '.[] | "Version \(.version) - \(.created_at) - ID: \(.version_id)"'

# Cleanup
echo -e "\n${GREEN}8. Cleanup...${NC}"
curl -X DELETE "$API_BASE/api/v1/file/$STORAGE/$DOC_NAME" -s | jq '.'
curl -X DELETE "$API_BASE/api/v1/trash/$STORAGE" -s | jq '.'
rm -f /tmp/$DOC_NAME /tmp/contract-v2.txt

echo -e "\n${BLUE}=== Versioning Demo Complete ===${NC}"
echo -e "${YELLOW}Use Cases:${NC}"
echo "  - Document revision tracking"
echo "  - Software release management"
echo "  - Configuration history"
echo "  - Compliance and audit trails"
