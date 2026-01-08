#!/bin/bash
# debug_kb_move.sh
# Verifies moving an article into a Knowledge Base

BASE_URL="http://localhost:3000/api"

echo "Registering User..."
USER_ID=$(curl -s -X POST "$BASE_URL/auth/register" \
  -H "Content-Type: application/json" \
  -d '{"username": "kbmover", "password": "password123", "email": "kbmover@example.com"}' | jq -r '.id // empty')

if [ -z "$USER_ID" ] || [ "$USER_ID" == "null" ]; then
    echo "Login..."
    TOKEN=$(curl -s -X POST "$BASE_URL/auth/login" \
      -H "Content-Type: application/json" \
      -d '{"username": "kbmover", "password": "password123"}' | jq -r '.token')
else
    TOKEN=$(curl -s -X POST "$BASE_URL/auth/login" \
      -H "Content-Type: application/json" \
      -d '{"username": "kbmover", "password": "password123"}' | jq -r '.token')
fi

echo "Token: $TOKEN"

echo "Creating Knowledge Base..."
KB_ID=$(curl -s -X POST "$BASE_URL/knowledge-bases" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"title": "Destination KB", "visibility": "Public"}' | jq -r '.id')
echo "KB ID: $KB_ID"

echo "Creating Standalone Article..."
ARTICLE_ID=$(curl -s -X POST "$BASE_URL/content" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"title": "Standalone Article", "body": "Content", "tags": [], "visibility": "Public"}' | jq -r '.id')
echo "Article ID: $ARTICLE_ID"

echo "Verifying Article has no KB..."
KB_VAL=$(curl -s -X GET "$BASE_URL/content/$ARTICLE_ID" \
  -H "Authorization: Bearer $TOKEN" | jq -r '.node.knowledge_base_id')
echo "Initial KB: $KB_VAL"

if [ "$KB_VAL" != "null" ]; then
    echo "ERROR: Article initially has KB ID!"
    exit 1
fi

echo "Moving Article to KB..."
curl -s -X PUT "$BASE_URL/content/$ARTICLE_ID" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"title\": \"Standalone Article\", \"body\": \"Content\", \"tags\": [], \"visibility\": \"Public\", \"knowledge_base_id\": \"$KB_ID\", \"type\": \"Article\"}" > /dev/null

echo "Verifying Article has KB..."
NEW_KB_VAL=$(curl -s -X GET "$BASE_URL/content/$ARTICLE_ID" \
  -H "Authorization: Bearer $TOKEN" | jq -r '.node.knowledge_base_id')
echo "New KB: $NEW_KB_VAL"

if [ "$NEW_KB_VAL" == "$KB_ID" ]; then
    echo "SUCCESS: Article moved to KB."
else
    echo "FAILURE: KB ID mismatch. Expected $KB_ID, got $NEW_KB_VAL"
    exit 1
fi
