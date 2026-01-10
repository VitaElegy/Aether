#!/bin/bash

# Configuration
BASE_URL="http://localhost:3000/api"
USERNAME="testuser_edit_pub_$(date +%s)"
PASSWORD="password123"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

echo "1. Registering user..."
curl -s -X POST "$BASE_URL/auth/register" \
  -H "Content-Type: application/json" \
  -d "{\"username\": \"$USERNAME\", \"password\": \"$PASSWORD\", \"email\": \"$USERNAME@example.com\"}" > /dev/null

echo "2. Logging in..."
TOKEN=$(curl -s -X POST "$BASE_URL/auth/login" \
  -H "Content-Type: application/json" \
  -d "{\"username\": \"$USERNAME\", \"password\": \"$PASSWORD\"}" | jq -r '.token')

TITLE="Published Article $(date +%s)"

echo "3. Creating Published Article..."
RESP=$(curl -s -X POST "$BASE_URL/content" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"title\": \"$TITLE\", \"body\": \"Initial Content\", \"tags\": [], \"visibility\": \"Public\", \"status\": \"Published\"}")

ID=$(echo "$RESP" | jq -r '.id')
echo "   ID Created: $ID"

# Fetch to verify status
GET_RESP=$(curl -s -X GET "$BASE_URL/content/$ID" -H "Authorization: Bearer $TOKEN")
STATUS=$(echo "$GET_RESP" | jq -r '.status')

if [ "$STATUS" == "Published" ]; then
    echo -e "${GREEN}Article Created as Published.${NC}"
else
    echo -e "${RED}Failed to create Published article. Status: $STATUS${NC}"
    exit 1
fi

echo "4. Simulating Auto-Save (Editing Content, Keeping Published)..."
# This simulates the new frontend behavior
curl -s -X PUT "$BASE_URL/content/$ID" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"title\": \"$TITLE\", \"body\": \"Updated Content\", \"tags\": [], \"visibility\": \"Public\", \"status\": \"Published\"}" > /dev/null

# Fetch again
GET_RESP_2=$(curl -s -X GET "$BASE_URL/content/$ID" -H "Authorization: Bearer $TOKEN")
NEW_STATUS=$(echo "$GET_RESP_2" | jq -r '.status')

if [ "$NEW_STATUS" == "Published" ]; then
    echo -e "${GREEN}PASSED: Status remained Published after update.${NC}"
else
    echo -e "${RED}FAILED: Status changed to $NEW_STATUS${NC}"
    exit 1
fi

echo "5. Simulating Explicit Unpublish (Draft)..."
curl -s -X PUT "$BASE_URL/content/$ID" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"title\": \"$TITLE\", \"body\": \"Updated Content\", \"tags\": [], \"visibility\": \"Public\", \"status\": \"Draft\"}" > /dev/null

GET_RESP_3=$(curl -s -X GET "$BASE_URL/content/$ID" -H "Authorization: Bearer $TOKEN")
NEW_STATUS_2=$(echo "$GET_RESP_3" | jq -r '.status')

if [ "$NEW_STATUS_2" == "Draft" ]; then
    echo -e "${GREEN}PASSED: Status changed to Draft explicitly.${NC}"
else
    echo -e "${RED}FAILED: Status failed to change to Draft. Got: $NEW_STATUS_2${NC}"
    exit 1
fi
