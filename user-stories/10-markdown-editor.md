# User Story: Create/Edit Markdown File

## Prerequisites
- File creation implemented (from 10-create-file)
- User must be authenticated (login flow from 02-login)
- AuthenticatedUser extractor implemented (see @specs/AUTH-MIDDLEWARE.md)

## Flow
```
FileEditor → CreateFile → SaveMarkdown
```

---

**API:** `file-create`  
**Endpoint:** `POST /api/files`

**Request:**
```json
{
  "filename": "notes.md",
  "content": "# My Notes\n\n## Introduction\n\nThis is my markdown document."
}
```

**Response:**
```json
{
  "status": "ok",
  "file": {
    "id": "uuid",
    "filename": "notes.md",
    "path": "/user/notes.md",
    "created_at": "ISO8601"
  }
}
```

---

**API:** `file-update`  
**Endpoint:** `PUT /api/files/:id`

**Request:**
```json
{
  "content": "# My Notes\n\n## Introduction\n\nThis is my markdown document.\n\n## Updated Section\n\nNew content added."
}
```

**Response:**
```json
{
  "status": "ok",
  "file": {
    "id": "uuid",
    "filename": "notes.md",
    "path": "/user/notes.md",
    "updated_at": "ISO8601"
  }
}
```
