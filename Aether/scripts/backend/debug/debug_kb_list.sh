#!/bin/bash
DATE_SUFFIX=$(date +%s)
USERNAME="kblist_test_${DATE_SUFFIX}"
EMAIL="kblist_${DATE_SUFFIX}@test.com"
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
  -d '{"title": "KB for List Test", "visibility": "Public"}' | python3 -c "import sys, json; print(json.load(sys.stdin).get('id', ''))")
echo "KB ID: $KB_ID"

echo "Creating Article in KB..."
curl -s -X POST http://localhost:3000/api/content \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"title\": \"Linked Article for List\",
    \"body\": \"This article belongs to a KB.\",
    \"tags\": [],
    \"visibility\": \"Public\",
    \"knowledge_base_id\": \"$KB_ID\",
    \"status\": \"Published\"
  }"

echo "Listing Articles without Filter (Global)..."
curl -s -X GET "http://localhost:3000/api/content?limit=5" \
  -H "Authorization: Bearer $TOKEN" | grep "Linked Article for List" && echo "Found in Global List"

echo "Listing Articles validation with Filter (KB ID)..."
RESP=$(curl -s -X GET "http://localhost:3000/api/content?knowledge_base_id=$KB_ID" \
  -H "Authorization: Bearer $TOKEN")

echo "Filtered Response: $RESP"

if echo "$RESP" | grep -q "Linked Article for List"; then
    echo "SUCCESS: Found article in filtered list."
else
    echo "FAILED: Article not found in filtered list."
    exit 1
fi
