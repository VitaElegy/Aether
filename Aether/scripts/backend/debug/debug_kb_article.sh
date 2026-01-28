#!/bin/bash
DATE_SUFFIX=$(date +%s)
USERNAME="kba_test_${DATE_SUFFIX}"
EMAIL="kba_${DATE_SUFFIX}@test.com"
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
  -d '{"title": "KB for Article Test", "visibility": "Public"}' | python3 -c "import sys, json; print(json.load(sys.stdin).get('id', ''))")
echo "KB ID: $KB_ID"

echo "Creating Article in KB..."
RESP=$(curl -s -X POST http://localhost:3000/api/content \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"title\": \"Linked Article\",
    \"body\": \"This article belongs to a KB.\",
    \"tags\": [],
    \"visibility\": \"Public\",
    \"knowledge_base_id\": \"$KB_ID\",
    \"status\": \"Published\"
  }")
echo "Raw Article Response: $RESP"

ARTICLE_ID=$(echo $RESP | python3 -c "import sys, json; print(json.load(sys.stdin).get('id', ''))")
echo "Article ID: $ARTICLE_ID"

if [ -z "$ARTICLE_ID" ]; then
    echo "FAILED: Article creation returned no ID."
    exit 1
fi

echo "Fetching Article Details..."
curl -s -X GET http://localhost:3000/api/content/$ARTICLE_ID \
  -H "Authorization: Bearer $TOKEN"
