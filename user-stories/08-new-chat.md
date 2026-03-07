# User Story: Start New Chat

## Prerequisites
- S3Provider, LocalFileSystemProvider, SQLiteProvider implemented (see @specs/BACKEND.md)
- All providers wired into application startup and registered as Actix-web app data
- User model and session/token handling implemented (from registration)
- Login endpoint implemented (user must be authenticated)
- AuthenticatedUser extractor implemented (see @specs/AUTH-MIDDLEWARE.md)

## Flow
```
ChatInterface → NewChat → AssistantResponse
```

---

**Form:** `NewChatForm`  
**API:** `chat-create`  
**Endpoint:** `POST /api/chat`

**Request:**
```json
{
  "message": "Hello, help me write a poem"
}
```

**Response:**
```json
{
  "status": "ok",
  "chat_id": "uuid",
  "message": {
    "id": "uuid",
    "role": "assistant",
    "content": "Sure! Here's a poem for you..."
  }
}
```

---

## Acceptance Criteria

### Backend
- [ ] `POST /api/chat` — accepts `message`, creates a new chat, sends message to LLM, returns `chat_id` and assistant response
- [ ] Request requires valid `Authorization: Bearer <access_token>` header
- [ ] Chat and messages are persisted in SQLite
- [ ] Integration tests cover: create chat with valid message, unauthorized request rejected, empty message rejected
- [ ] `cargo test` — all tests pass, zero failures
- [ ] Backend starts with config file, serves HTTP on configured port
- [ ] `docker/local/docker-compose.yml` includes backend, frontend, MinIO, and MailHog services

### Frontend
- [ ] `ChatPage` exists at a route for new/existing chats, requires authenticated user
- [ ] `NewChatForm` — message input field and submit button, calls `chat_service::create_chat`
- [ ] Assistant response is displayed in the chat view after submission
- [ ] User message and assistant response are rendered with distinct styling (user vs assistant)
- [ ] `chat_service` module implements `create_chat` async function with `Authorization: Bearer` header
- [ ] Frontend unit tests pass — message submission, response rendering, service function mocking
