#!/bin/bash
set -x

# Base URL
API_URL="http://localhost:3000/api"

echo "Checking Health..."
curl -v "http://localhost:3000/"
echo -e "\n"

# 1. Register User (Owner)
echo "=== 1. Registering User ==="
USERNAME="rebac_user_$(date +%s)"
curl -s -X POST "$API_URL/auth/register" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "'$USERNAME'",
    "email": "'$USERNAME'@test.com",
    "password": "password123"
  }'

# 2. Login
echo -e "\n=== 2. Logging In ==="
TOKEN=$(curl -s -X POST "$API_URL/auth/login" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "'$USERNAME'",
    "password": "password123"
  }' | jq -r '.token')

if [ "$TOKEN" == "null" ] || [ -z "$TOKEN" ]; then
    echo "❌ Login Failed"
    exit 1
fi
echo "Token Acquired"

# 3. Create Public Article
echo -e "\n=== 3. Creating Public Article ==="
UUID=$(uuidgen | tr '[:upper:]' '[:lower:]')
echo "Article ID: $UUID"

JSON_DATA='{
    "title": "Public ReBAC Test '$USERNAME'",
    "body": "This is a public article",
    "tags": [],
    "category": "Test",
    "status": "Published",
    "visibility": "Public"
}'
echo "Sending JSON: $JSON_DATA"

RESPONSE=$(curl -s -X POST "$API_URL/content" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "$JSON_DATA")

echo "Create Response: $RESPONSE"

# Extract ID from Response (Handling both {"id":"..."} and {"error":"..."})
CREATED_ID=$(echo $RESPONSE | jq -r '.id')
echo "Created ID from API: $CREATED_ID"

if [ "$CREATED_ID" == "null" ]; then
    echo "❌ Creation Failed"
    # Use UUID anyway for checking (will fail)
    CHECK_UUID=$UUID
else
    CHECK_UUID=$CREATED_ID
fi

# 4. Verify Explicit Permission (Owner Access)
# Extract User ID (Needed for Manual Tuple & Checks)
echo -e "\n=== 4. Fetching User ID ==="
USER_ID=$(curl -s -X POST "$API_URL/auth/login" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "'$USERNAME'",
    "password": "password123"
  }' | jq -r '.user.id')

echo "User ID: $USER_ID"

# Debug: Manually Add Tuple (to verify if logic works at all)
echo -e "\n=== Debug: Manually Adding Tuple ==="
MANUAL_TUPLE='{
    "entity_type": "node",
    "entity_id": "'$CHECK_UUID'",
    "relation": "owner",
    "subject_type": "user",
    "subject_id": "'$USER_ID'"
}'
curl -s -X POST "$API_URL/permissions/tuple" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "$MANUAL_TUPLE"

echo -e "\n=== 5. Verifying Owner Access ==="
CHECK_OWNER=$(curl -s "$API_URL/permissions/check?user_id=$USER_ID&entity_id=$CHECK_UUID&action=write")
echo "Can Owner Write? $CHECK_OWNER"
if [[ $CHECK_OWNER == *'"allowed":true'* ]]; then
    echo "✅ PASS: Owner Write Access"
else
    echo "❌ FAIL: Owner Write Access"
fi

# 5. Verify Public Access (Guest)
echo -e "\n=== 5. Verifying Public Access (Guest) ==="
GUEST_ID="00000000-0000-0000-0000-000000000000" # Nil UUID simulates Guest
CHECK_PUBLIC=$(curl -s "$API_URL/permissions/check?user_id=$GUEST_ID&entity_id=$CHECK_UUID&action=read")
echo "Can Guest Read? $CHECK_PUBLIC"
if [[ $CHECK_PUBLIC == *'"allowed":true'* ]]; then
    echo "✅ PASS: Public Read Access"
else
    echo "❌ FAIL: Public Read Access"
fi

echo -e "\n=== Done ==="
