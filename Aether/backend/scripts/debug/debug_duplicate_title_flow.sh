#!/bin/bash

# Configuration
BASE_URL="http://localhost:3000/api"
USERNAME="testuser_dup_flow_$(date +%s)"
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

if [ -z "$TOKEN" ] || [ "$TOKEN" == "null" ]; then
    echo -e "${RED}Login failed.${NC}"
    exit 1
fi

TITLE="My Unique Title $(date +%s)"

echo "3. Creating Article 1 (Original)..."
RESP=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/content" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"title\": \"$TITLE\", \"body\": \"Original Content\", \"tags\": [], \"visibility\": \"Public\", \"status\": \"Draft\"}")

HTTP_CODE=$(echo "$RESP" | tail -n1)
BODY=$(echo "$RESP" | head -n-1)

if [ "$HTTP_CODE" == "201" ]; then
    echo -e "${GREEN}Article 1 Created.${NC}"
else
    echo -e "${RED}Failed to create Article 1. Code: $HTTP_CODE${NC}"
    echo "Resp: $BODY"
    exit 1
fi

echo "4. Creating Article 2 (Duplicate Title) - Simulating Editor Auto-Save..."
RESP=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/content" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"title\": \"$TITLE\", \"body\": \"Draft Content Conflict\", \"tags\": [], \"visibility\": \"Public\", \"status\": \"Draft\"}")

HTTP_CODE=$(echo "$RESP" | tail -n1)
BODY=$(echo "$RESP" | head -n-1)

echo "HTTP Code: $HTTP_CODE"
echo "Response Body: $BODY"

if [ "$HTTP_CODE" == "409" ]; then
    echo -e "${GREEN}PASSED: Backend correctly returns 409 Conflict.${NC}"
    
    # Verify Error Message Structure
    ERROR_MSG=$(echo "$BODY" | jq -r '.error')
    if [[ "$ERROR_MSG" == *"already exists"* ]]; then
         echo -e "${GREEN}PASSED: Error message contains 'already exists'.${NC}"
    else
         echo -e "${RED}FAILED: Error message unexpected: $ERROR_MSG${NC}"
    fi
else
    echo -e "${RED}FAILED: Expected 409, got $HTTP_CODE${NC}"
    exit 1
fi
