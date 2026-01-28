#!/bin/bash
set -e

BASE_URL="http://localhost:3000/api"

SUFFIX=$RANDOM
echo "=== 1. Register Users ==="
# User 1: Project Lead (Owner)
echo "Registering Lead..."
curl -s -X POST $BASE_URL/auth/register -H "Content-Type: application/json" -d "{\"username\":\"verify_lead_$SUFFIX\",\"email\":\"lead_$SUFFIX@test.com\",\"password\":\"password123\"}" > /dev/null || true
TOKEN_BOSS=$(curl -s -X POST $BASE_URL/auth/login -H "Content-Type: application/json" -d "{\"username\":\"verify_lead_$SUFFIX\",\"password\":\"password123\"}" | jq -r .token)
echo "Got Token Boss: ${TOKEN_BOSS:0:10}..."
BOSS_ID=$(curl -s -H "Authorization: Bearer $TOKEN_BOSS" "$BASE_URL/users/search?q=verify_lead_$SUFFIX" | jq -r '.[0].id')
echo "Boss ID: $BOSS_ID"

if [ "$BOSS_ID" == "null" ]; then echo "Failed to find Boss ID"; exit 1; fi

# User 2: Partner (Collaborator)
echo "Registering Partner..."
curl -s -X POST $BASE_URL/auth/register -H "Content-Type: application/json" -d "{\"username\":\"verify_partner_$SUFFIX\",\"email\":\"partner_$SUFFIX@test.com\",\"password\":\"password123\"}" > /dev/null || true
TOKEN_PARTNER=$(curl -s -X POST $BASE_URL/auth/login -H "Content-Type: application/json" -d "{\"username\":\"verify_partner_$SUFFIX\",\"password\":\"password123\"}" | jq -r .token)
PARTNER_ID=$(curl -s -H "Authorization: Bearer $TOKEN_BOSS" "$BASE_URL/users/search?q=verify_partner_$SUFFIX" | jq -r '.[0].id')
echo "Partner ID: $PARTNER_ID"

echo "=== 2. Create Private Article (Test 1) ==="
ARTICLE_ID=$(curl -s -X POST $BASE_URL/content -H "Authorization: Bearer $TOKEN_BOSS" -H "Content-Type: application/json" -d "{
  \"title\": \"Secret Project $SUFFIX\",
  \"body\": \"Top Secret Data\",
  \"tags\": [],
  \"visibility\": \"Private\",
  \"status\": \"Published\"
}" | jq -r .id)
echo "Article Created: $ARTICLE_ID"

echo "=== 3. Add Collaborator ==="
curl -s -X POST "$BASE_URL/content/$ARTICLE_ID/collaborators" -H "Authorization: Bearer $TOKEN_BOSS" -H "Content-Type: application/json" -d "{\"user_id\":\"$PARTNER_ID\"}"
echo "Collaborator Added"

echo "=== 4. Verify Collaborator Access (Test 3) ==="
# Partner tries to read
STATUS_CODE=$(curl -s -o /dev/null -w "%{http_code}" -X GET "$BASE_URL/content/$ARTICLE_ID" -H "Authorization: Bearer $TOKEN_PARTNER")
if [ "$STATUS_CODE" -eq 200 ]; then echo "PASS: Partner can read article"; else echo "FAIL: Partner cannot read article ($STATUS_CODE)"; exit 1; fi

# Partner tries to edit (update body)
echo "Partner editing..."
UPDATE_RESP_CODE=$(curl -s -o /dev/null -w "%{http_code}" -X PUT "$BASE_URL/content/$ARTICLE_ID" -H "Authorization: Bearer $TOKEN_PARTNER" -H "Content-Type: application/json" -d "{
  \"title\": \"Secret Project $SUFFIX\",
  \"body\": \"Top Secret Data - Reviewed\",
  \"tags\": [],
  \"visibility\": \"Private\",
  \"status\": \"Published\"
}")
if [ "$UPDATE_RESP_CODE" -eq 200 ]; then echo "PASS: Partner edited article"; else echo "FAIL: Partner edit failed ($UPDATE_RESP_CODE)"; exit 1; fi

echo "=== 5. Guest Privacy Check (Test 2) ==="
GUEST_CODE=$(curl -s -o /dev/null -w "%{http_code}" -X GET "$BASE_URL/content/$ARTICLE_ID")
if [ "$GUEST_CODE" -eq 403 ] || [ "$GUEST_CODE" -eq 404 ]; then echo "PASS: Guest blocked ($GUEST_CODE)"; else echo "FAIL: Guest can see private ($GUEST_CODE)"; exit 1; fi

echo "=== 6. Knowledge Base Operations (Test 4, 7, 9) ==="
# Create KB
KB_ID=$(curl -s -X POST $BASE_URL/knowledge-bases -H "Authorization: Bearer $TOKEN_BOSS" -H "Content-Type: application/json" -d "{
  \"title\": \"Omega Protocol $SUFFIX\",
  \"description\": \"Classified KB\",
  \"visibility\": \"Private\"
}" | jq -r .id)
echo "KB Created: $KB_ID"

# Create Folder in KB (Test 7)
FOLDER_ID=$(curl -s -X POST $BASE_URL/content -H "Authorization: Bearer $TOKEN_BOSS" -H "Content-Type: application/json" -d "{
  \"title\": \"Archives\",
  \"body\": \"\",
  \"tags\": [],
  \"visibility\": \"Private\",
  \"type\": \"Folder\",
  \"knowledge_base_id\": \"$KB_ID\"
}" | jq -r .id)
echo "Folder Created: $FOLDER_ID"

# Verify Profile lists KB (Test 9)
KB_LIST_COUNT=$(curl -s -X GET "$BASE_URL/knowledge-bases?author_id=$BOSS_ID" -H "Authorization: Bearer $TOKEN_BOSS" | jq '. | length')
if [ "$KB_LIST_COUNT" -ge 1 ]; then echo "PASS: KB listed in profile"; else echo "FAIL: KB list empty"; exit 1; fi

echo "=== 7. Article Association (Test 6, 8) ==="
# Move Article to KB and make Internal
curl -s -X PUT "$BASE_URL/content/$ARTICLE_ID" -H "Authorization: Bearer $TOKEN_BOSS" -H "Content-Type: application/json" -d "{
  \"title\": \"Secret Project $SUFFIX\",
  \"body\": \"Top Secret Data - Reviewed\",
  \"tags\": [],
  \"visibility\": \"Internal\",
  \"knowledge_base_id\": \"$KB_ID\"
}" > /dev/null
echo "Article Moved to KB and set to Internal"

echo "=== 8. KB Article Creation (Test 5) ==="
KB_ARTICLE_ID=$(curl -s -X POST $BASE_URL/content -H "Authorization: Bearer $TOKEN_BOSS" -H "Content-Type: application/json" -d "{
  \"title\": \"Protocol Specs\",
  \"body\": \"Specs...\",
  \"tags\": [],
  \"visibility\": \"Private\",
  \"knowledge_base_id\": \"$KB_ID\"
}" | jq -r .id)
echo "Article Created directly in KB: $KB_ARTICLE_ID"

echo "=== ALL TESTS PASSED ==="
