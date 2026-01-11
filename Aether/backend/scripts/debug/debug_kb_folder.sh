#!/bin/bash
DATE_SUFFIX=$(date +%s)
USERNAME="kbfolder_test_${DATE_SUFFIX}"
EMAIL="kbfolder_${DATE_SUFFIX}@test.com"
PASSWORD="password123"

echo "Registering..."
curl -s -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d "{\"username\": \"$USERNAME\", \"email\": \"$EMAIL\", \"password\": \"$PASSWORD\"}"

echo -e "\nLogging in..."
TOKEN=$(curl -s -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d "{\"username\": \"$USERNAME\", \"password\": \"$PASSWORD\"}" | python3 -c "import sys, json; print(json.load(sys.stdin).get('token', ''))")
echo "Token: $TOKEN"

echo "Creating KB..."
KB_ID=$(curl -s -X POST http://localhost:3000/api/knowledge-bases \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"title": "KB for Folder Test", "visibility": "Public"}' | python3 -c "import sys, json; print(json.load(sys.stdin).get('id', ''))")
echo "KB ID: $KB_ID"

echo "Creating Folder in KB..."
FOLDER_RESP=$(curl -s -X POST http://localhost:3000/api/content \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"title\": \"My Folder\",
    \"body\": \"\",
    \"tags\": [],
    \"visibility\": \"Public\",
    \"knowledge_base_id\": \"$KB_ID\",
    \"type\": \"Folder\"
  }")

echo "Folder Response: $FOLDER_RESP"

if echo "$FOLDER_RESP" | grep -q "id"; then
    echo "SUCCESS: Folder created."
else
    echo "FAILED: Folder creation failed."
    exit 1
fi

FOLDER_ID=$(echo "$FOLDER_RESP" | python3 -c "import sys, json; print(json.load(sys.stdin).get('id', ''))")

echo "Fetching Folder by ID ($FOLDER_ID)..."
GET_RESP=$(curl -s -X GET "http://localhost:3000/api/content/$FOLDER_ID" \
  -H "Authorization: Bearer $TOKEN")
echo "Get Response: $GET_RESP"

echo "Verifying Folder in List..."
LIST_RESP=$(curl -s -X GET "http://localhost:3000/api/content?knowledge_base_id=$KB_ID" \
  -H "Authorization: Bearer $TOKEN")

echo "List Response: $LIST_RESP"

if echo "$LIST_RESP" | grep -q "My Folder"; then
    echo "SUCCESS: Folder found in list."
else
    echo "FAILED: Folder not found in list."
    exit 1
fi
