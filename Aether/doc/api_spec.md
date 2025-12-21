# Aether API Specification

Base URL: `/api`
Content-Type: `application/json`

## Authentication

### Login
**POST** `/auth/login`

**Request:**
```json
{
  "username": "commander",
  "password": "secret_password"
}
```

**Response (200 OK):**
```json
{
  "token": "eyJhbGciOi...",
  "user": {
    "id": "uuid-string",
    "perms": 1
  }
}
```

### Register
**POST** `/auth/register`

**Request:**
```json
{
  "username": "new_user",
  "email": "user@example.com",
  "password": "secret_password"
}
```

**Response (201 Created):**
```json
{
  "message": "User created"
}
```

---

## Content (Planned)

### List Posts
**GET** `/content?limit=10&offset=0`

**Response (200 OK):**
```json
[
  {
    "id": "uuid",
    "title": "System Update",
    "type": "Markdown",
    "data": { "content": "..." },
    "tags": ["system"],
    "created_at": "2023-10-27T10:00:00Z"
  }
]
```

### Create Post
**POST** `/content`
*(Requires Auth)*

**Request:**
```json
{
  "title": "New Protocol",
  "slug": "new-protocol",
  "type": "Markdown",
  "data": { "content": "# Hello" },
  "tags": ["protocol"]
}
```

