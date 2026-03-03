# User Story: Start New Chat

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
