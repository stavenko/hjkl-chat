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
