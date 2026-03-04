# User Story: View Chat History

## Prerequisites
- New chat functionality implemented (from 07-new-chat)
- User must be authenticated (login flow from 02-login)
- AuthenticatedUser extractor implemented (see @specs/AUTH-MIDDLEWARE.md)

## Flow
```
ChatHistoryView → SelectChat → ContinueChat
```

---

**API:** `chat-list`  
**Endpoint:** `GET /api/chat`

**Response:**
```json
{
  "status": "ok",
  "chats": [
    {
      "id": "uuid",
      "title": "Hello, help me write a poem",
      "preview": "Sure! Here's a poem for you...",
      "updated_at": "ISO8601",
      "message_count": 5
    }
  ]
}
```

---

**API:** `chat-get`  
**Endpoint:** `GET /api/chat/:id`

**Response:**
```json
{
  "status": "ok",
  "chat": {
    "id": "uuid",
    "title": "Hello, help me write a poem",
    "messages": [
      { "id": "uuid", "role": "user", "content": "Hello, help me write a poem", "created_at": "ISO8601" },
      { "id": "uuid", "role": "assistant", "content": "Sure! Here's a poem for you...", "created_at": "ISO8601" }
    ]
  }
}
```
