#!/bin/bash
DATE_SUFFIX=$(date +%s)
USERNAME="validtest_${DATE_SUFFIX}"
PASSWORD="password123"

# Register
echo "Registering..."
curl -s -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d "{\"username\": \"$USERNAME\", \"email\": \"${USERNAME}@test.com\", \"password\": \"$PASSWORD\"}"

# Login
echo -e "\nLogging in..."
LOGIN_RESPONSE=$(curl -s -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d "{\"username\": \"$USERNAME\", \"password\": \"$PASSWORD\"}")
TOKEN=$(echo $LOGIN_RESPONSE | python3 -c "import sys, json; print(json.load(sys.stdin).get('token', ''))")

echo "Token: $TOKEN"

TITLE="Unique Title ${DATE_SUFFIX}"

# 1. Publish Article A
echo -e "\n1. Publishing Article A '$TITLE'..."
RES_A=$(curl -s -w "\nHTTP_STATUS:%{http_code}" -X POST http://localhost:3000/api/content \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{
    \"title\": \"$TITLE\",
    \"body\": \"Body A\",
    \"tags\": [],
    \"category\": null,
    \"visibility\": \"Public\",
    \"status\": \"Published\",
    \"snapshot\": true
  }")
echo "$RES_A" | grep "HTTP_STATUS"
ID_A=$(echo "$RES_A" | grep -v "HTTP_STATUS" | python3 -c "import sys, json; print(json.load(sys.stdin)['id'])")

# 2. Try Duplicate Title (New Article)
echo -e "\n2. Assessing Duplicate Title (New Article)..."
RES_DUP=$(curl -s -w "\nHTTP_STATUS:%{http_code}" -X POST http://localhost:3000/api/content \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{
    \"title\": \"$TITLE\",
    \"body\": \"Body B\",
    \"tags\": [],
    \"category\": null,
    \"visibility\": \"Public\",
    \"status\": \"Published\",
    \"snapshot\": true
  }")
echo "$RES_DUP" | grep "HTTP_STATUS" # Expect 409

# 3. No-Op Update
echo -e "\n3. Assessing No-Op Update..."
RES_NOOP=$(curl -s -w "\nHTTP_STATUS:%{http_code}" -X PUT http://localhost:3000/api/content/$ID_A \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{
    \"title\": \"$TITLE\",
    \"body\": \"Body A\",
    \"tags\": [],
    \"category\": null,
    \"visibility\": \"Public\",
    \"status\": \"Published\",
    \"snapshot\": false
  }")
echo "$RES_NOOP" | grep "HTTP_STATUS"
echo "$RES_NOOP" | grep "no-op" # Expect "status": "no-op"

# 4. Valid Update (Change Body)
echo -e "\n4. Assessing Valid Update..."
RES_UPDATE=$(curl -s -w "\nHTTP_STATUS:%{http_code}" -X PUT http://localhost:3000/api/content/$ID_A \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{
    \"title\": \"$TITLE\",
    \"body\": \"Body Modified\",
    \"tags\": [],
    \"category\": null,
    \"visibility\": \"Public\",
    \"status\": \"Published\",
    \"snapshot\": true
  }")
echo "$RES_UPDATE" | grep "HTTP_STATUS"
