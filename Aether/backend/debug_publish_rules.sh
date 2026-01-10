#!/bin/bash
# debug_publish_rules.sh

BASE_URL="http://localhost:3000/api"

echo "1. Login..."
TOKEN=$(curl -s -X POST "$BASE_URL/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"username": "testuser", "password": "password123"}' | jq -r '.token')

if [ "$TOKEN" == "null" ]; then
    echo "Login failed. Creating user..."
    curl -s -X POST "$BASE_URL/auth/register" \
      -H "Content-Type: application/json" \
      -d '{"username": "testuser", "password": "password123", "email": "test@example.com"}'
    TOKEN=$(curl -s -X POST "$BASE_URL/auth/login" \
      -H "Content-Type: application/json" \
      -d '{"username": "testuser", "password": "password123"}' | jq -r '.token')
fi

# 2. Duplicate Title Check
echo "2. Testing Duplicate Title..."
TITLE="Unique Title"
echo "Token: $TOKEN"

# Create first
uuid1=$(uuidgen)
RESP1=$(curl -s -X POST "$BASE_URL/content" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"title\": \"$TITLE\", \"slug\": \"slug-$uuid1\", \"status\": \"Draft\", \"body\": \"Content 1\", \"tags\": [], \"visibility\": \"Public\"}")
echo "Create 1 Resp: $RESP1"

# Create second (Should Fail)
uuid2=$(uuidgen)
RESP=$(curl -s -X POST "$BASE_URL/content" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"title\": \"$TITLE\", \"slug\": \"slug-$uuid2\", \"status\": \"Draft\", \"body\": \"Content 2\", \"tags\": [], \"visibility\": \"Public\"}")

echo "Create 2 Resp: $RESP"

if echo "$RESP" | grep -q "already exists"; then
    echo "PASSED: Duplicate title rejected."
else
    echo "FAILED: Duplicate title allowed. Resp: $RESP"
    # exit 1 
fi

# 3. No-Op Versioning Check
echo "3. Testing No-Op Versioning..."
# Get ID of first article
LIST=$(curl -s -G "$BASE_URL/content" -H "Authorization: Bearer $TOKEN")
ID=$(echo "$LIST" | jq -r '.[0].id')
echo "Target ID: $ID"

# Get initial version count
H1=$(curl -s "$BASE_URL/content/$ID/history" -H "Authorization: Bearer $TOKEN" | jq '. | length')
echo "Initial Versions: $H1"

# Update with SAME content, SAME title (No Reason)
curl -s -X PUT "$BASE_URL/content/$ID" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"title\": \"$TITLE\", \"slug\": \"slug-$uuid1\", \"status\": \"Draft\", \"body\": \"Content 1\", \"tags\": [], \"visibility\": \"Public\"}" > /dev/null

H2=$(curl -s "$BASE_URL/content/$ID/history" -H "Authorization: Bearer $TOKEN" | jq '. | length')
echo "Post No-Op Versions: $H2"

if [ "$H1" == "$H2" ]; then
    echo "PASSED: No-Op update did not create new version."
else
    echo "FAILED: No-Op created version."
fi

# 4. Metadata Only Update
echo "4. Testing Metadata Only Update..."
# Update Title (Metadata) -> Should NOT create version (based on hash check of BODY)
NEW_TITLE="Unique Title Updated"
curl -s -X PUT "$BASE_URL/content/$ID" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"title\": \"$NEW_TITLE\", \"slug\": \"slug-$uuid1\", \"status\": \"Published\", \"body\": \"Content 1\", \"tags\": [], \"visibility\": \"Public\"}" > /dev/null

H3=$(curl -s "$BASE_URL/content/$ID/history" -H "Authorization: Bearer $TOKEN" | jq '. | length')
echo "Post Metadata Update Versions: $H3"

if [ "$H2" == "$H3" ]; then
    echo "PASSED: Metadata update did not create new version."
else
    echo "FAILED: Metadata update created version."
fi

# 5. Content Update
echo "5. Testing Content Update..."
curl -s -X PUT "$BASE_URL/content/$ID" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"title\": \"$NEW_TITLE\", \"slug\": \"slug-$uuid1\", \"status\": \"Published\", \"body\": \"Content Changed\", \"tags\": [], \"visibility\": \"Public\", \"_reason\": \"Valid Update\"}" > /dev/null

H4=$(curl -s "$BASE_URL/content/$ID/history" -H "Authorization: Bearer $TOKEN" | jq '. | length')
echo "Post Content Update Versions: $H4"

if [ "$H4" -gt "$H3" ]; then
     echo "PASSED: Content update created new version."
else
     echo "FAILED: Content update missed version."
fi
