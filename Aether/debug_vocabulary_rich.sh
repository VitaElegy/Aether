#!/bin/bash

# 1. Login to get token
echo "Logging in as admin..."
LOGIN_RESP=$(curl -s -X POST http://localhost:3000/api/auth/login      -H "Content-Type: application/json"      -d '{"username": "admin", "password": "admin"}')
TOKEN=$(echo $LOGIN_RESP | grep -o '"token":"[^"]*' | cut -d'"' -f4)

if [ -z "$TOKEN" ]; then
    echo "Login failed"
    echo $LOGIN_RESP
    exit 1
fi

echo "Got Token: $TOKEN"

# 2. Create Rich Vocabulary
echo "Creating vocabulary 'Magnificent' with roots and examples..."
curl -s -X POST http://localhost:3000/api/vocabulary \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "word": "Magnificent",
    "definition": "Extremely beautiful, elaborate, or impressive.",
    "translation": "宏伟的",
    "phonetic": "/magˈnɪfɪs(ə)nt/",
    "root": "magni",
    "language": "en",
    "examples": [
      {
        "sentence": "A dramatic landscape of magnificent mountains.",
        "translation": "壮丽群山的戏剧性景观。",
        "note": "Great for describing nature.",
        "image_url": "http://example.com/mountain.jpg"
      },
      {
        "sentence": "She looked magnificent in her wedding dress.",
        "translation": "她穿着婚纱看起来美极了。",
        "note": "Complimenting appearance."
      }
    ]
  }'

echo -e "\n\nChecking List..."
# 3. List
curl -s -X GET "http://localhost:3000/api/vocabulary?limit=1" \
  -H "Authorization: Bearer $TOKEN" | grep "Magnificent"

echo -e "\n\nDone."
