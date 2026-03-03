# User Story: Create File Manually

## Flow
```
FileEditor → PathInput → PathSuggestions → SaveFile
```

---

**Form:** `CreateFileForm`  
**API:** `file-create`  
**Endpoint:** `POST /api/files`

**Request:**
```json
{
  "path": "/project/notes.md",
  "content": "# My Notes\n\nThis is my content."
}
```

**Response:**
```json
{
  "status": "ok",
  "file": {
    "id": "uuid",
    "path": "/project/notes.md",
    "link": "file://uuid",
    "created_at": "ISO8601"
  }
}
```

---

## Path Autocomplete

**API:** `path-suggestions`  
**Endpoint:** `GET /api/files/suggestions?path=/ab`

**Response:**
```json
{
  "status": "ok",
  "suggestions": [
    "/abc/",
    "/abf/"
  ]
}
```

---

## S3 Storage

Files are stored in S3 with path as the key:
```
s3://bucket/{user_id}/project/notes.md
```
