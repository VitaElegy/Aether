#!/bin/bash
DATE_SUFFIX=$(date +%s)
USERNAME="kbtest_${DATE_SUFFIX}"
EMAIL="kb_${DATE_SUFFIX}@test.com"
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

# Create KB
echo "Creating Knowledge Base..."
CREATE_RESPONSE=$(curl -v -X POST http://localhost:3000/api/knowledge-bases \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "title": "My Knowledge Base",
    "description": "A test KB",
    "tags": ["test", "api"],
    "visibility": "Public",
    "cover_image": "https://example.com/image.jpg"
  }')

echo "Create Response: $CREATE_RESPONSE"

# List KBs
echo "Listing Knowledge Bases..."
curl -v -s -X GET http://localhost:3000/api/knowledge-bases \
  -H "Authorization: Bearer $TOKEN"
