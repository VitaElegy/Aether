#!/bin/bash
DATE_SUFFIX=$(date +%s)
USERNAME="curltest_${DATE_SUFFIX}"
EMAIL="curl_${DATE_SUFFIX}@test.com"
PASSWORD="password123"

# Register
echo "Registering..."
curl -s -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d "{\"username\": \"$USERNAME\", \"email\": \"$EMAIL\", \"password\": \"$PASSWORD\"}"

# Login
echo -e "\nLogging in..."
LOGIN_RESPONSE=$(curl -s -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d "{\"username\": \"$USERNAME\", \"password\": \"$PASSWORD\"}")

echo "Login Response: $LOGIN_RESPONSE"
TOKEN=$(echo $LOGIN_RESPONSE | python3 -c "import sys, json; print(json.load(sys.stdin).get('token', ''))")

echo "Token: $TOKEN"

# Try to publish
echo "Publishing..."
PUBLISH_RESPONSE=$(curl -v -X POST http://localhost:3000/api/content \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "title": "Curl Test",
    "body": "Body content",
    "tags": [],
    "category": null,
    "visibility": "Public",
    "status": "Published",
    "reason": "Test commit",
    "snapshot": true
  }')

echo "Publish Response: $PUBLISH_RESPONSE"
ARTICLE_ID=$(echo $PUBLISH_RESPONSE | python3 -c "import sys, json; print(json.load(sys.stdin)['id'])")
echo "Article ID: $ARTICLE_ID"


# Post Comment
echo "Posting Comment..."
curl -v -X POST "http://localhost:3000/api/comments/article/$ARTICLE_ID" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{
    \"text\": \"Test Comment verify\"
  }"
