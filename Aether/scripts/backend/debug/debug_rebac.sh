#!/bin/bash

# Base URL
API_URL="http://localhost:3000/api"

echo "Checking Health..."
curl -v "http://localhost:3000/"
echo -e "\n"

echo "=== 1. Creating Test Data ==="
# User IDs (Mock UUIDs since we don't have a user creation API handy without auth, 
# but we can try to use existing admin or just use any UUID for the graph since the graph is checking existence)
# Note: In real Zanzibar, IDs must exist in User table if FK enforced, but our Relationships table doesn't strictly enforce FK on "subject_id" to allow flexible subjects.
# Let's check schema: Relationships has no FK constraints on entity_id/subject_id in the create table SQL.
USER_A="00000000-0000-0000-0000-000000000001"
GROUP_G="00000000-0000-0000-0000-000000000002"
NODE_X="00000000-0000-0000-0000-000000000003"

echo "User A: $USER_A"
echo "Group G: $GROUP_G"
echo "Node X: $NODE_X"

echo -e "\n=== 2. Setting up Relationships ==="
# User A is MEMBER of Group G
echo "Adding: User A -> member -> Group G"
curl -v -X POST "$API_URL/permissions/tuple" \
  -H "Content-Type: application/json" \
  -d '{
    "entity_type": "group",
    "entity_id": "'$GROUP_G'",
    "relation": "member",
    "subject_type": "user",
    "subject_id": "'$USER_A'"
  }'

# Group G is VIEWER of Node X
echo -e "\nAdding: Group G -> viewer -> Node X"
curl -v -X POST "$API_URL/permissions/tuple" \
  -H "Content-Type: application/json" \
  -d '{
    "entity_type": "node",
    "entity_id": "'$NODE_X'",
    "relation": "viewer",
    "subject_type": "group",
    "subject_id": "'$GROUP_G'"
  }'

echo -e "\n\n=== 3. verifying Permissions ==="

# Check 1: Can User A read Node X? (Should be TRUE via Group G)
echo "Check: Can User A READ Node X? (Expected: true)"
# Removed -s to see errors
RESPONSE=$(curl -v "$API_URL/permissions/check?user_id=$USER_A&entity_id=$NODE_X&action=read")
echo "Response: $RESPONSE"

if [[ $RESPONSE == *'"allowed":true'* ]]; then
    echo "✅ PASS: Recursive check worked."
else
    echo "❌ FAIL: Recursive check failed."
fi

# Check 2: Can User A write Node X? (Should be FALSE)
echo -e "\nCheck: Can User A WRITE Node X? (Expected: false)"
RESPONSE=$(curl -v "$API_URL/permissions/check?user_id=$USER_A&entity_id=$NODE_X&action=write")
echo "Response: $RESPONSE"

if [[ $RESPONSE == *'"allowed":false'* ]]; then
    echo "✅ PASS: Negative check worked."
else
    echo "❌ FAIL: Negative check failed."
fi

echo -e "\n=== 4. Cleaning Up ==="
# Delete tuples
curl -X DELETE "$API_URL/permissions/tuple" -H "Content-Type: application/json" -d '{ "entity_type": "group", "entity_id": "'$GROUP_G'", "relation": "member", "subject_type": "user", "subject_id": "'$USER_A'" }' > /dev/null
curl -X DELETE "$API_URL/permissions/tuple" -H "Content-Type: application/json" -d '{ "entity_type": "node", "entity_id": "'$NODE_X'", "relation": "viewer", "subject_type": "group", "subject_id": "'$GROUP_G'" }' > /dev/null
echo "Done."
