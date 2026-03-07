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

---

## Acceptance Criteria

### Backend
- [ ] `GET /api/chat` — returns list of user's chats with `id`, `title`, `preview`, `updated_at`, `message_count`
- [ ] `GET /api/chat/:id` — returns full chat with all messages (role, content, created_at)
- [ ] Both endpoints require valid `Authorization: Bearer <access_token>` header
- [ ] User can only see their own chats (not other users' chats)
- [ ] Integration tests cover: list chats (empty and populated), get single chat with messages, unauthorized access rejected, non-existent chat returns error, cross-user access rejected
- [ ] `cargo test` — all tests pass, zero failures
- [ ] Backend starts with config file, serves HTTP on configured port
- [ ] `docker/local/docker-compose.yml` includes backend, frontend, MinIO, and MailHog services

### Frontend
- [ ] `ChatHistoryPage` — displays list of user's chats with title, preview, and timestamp
- [ ] Selecting a chat loads full message history via `chat_service::get_chat` and displays it
- [ ] User can continue a conversation from the loaded chat
- [ ] `chat_service` module implements `list_chats` and `get_chat` async functions with `Authorization: Bearer` header
- [ ] Frontend unit tests pass — chat list rendering, chat loading, message display, service function mocking
