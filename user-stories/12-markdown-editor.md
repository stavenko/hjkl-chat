# User Story: Create/Edit Markdown File

## Prerequisites
- File creation implemented (from 11-create-file)
- Files browser implemented (from 07-files-browser)
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

---

## Acceptance Criteria

### Backend
- [ ] `POST /api/files` — creates a new markdown file with `filename` and `content`, stores in S3, returns file metadata
- [ ] `PUT /api/files/:id` — updates existing file content in S3, returns updated file metadata with new `updated_at`
- [ ] Both endpoints require valid `Authorization: Bearer <access_token>` header
- [ ] User can only update their own files
- [ ] Integration tests cover: create markdown file, update existing file, content persisted in S3, unauthorized access rejected, update non-existent file returns error, cross-user update rejected
- [ ] `cargo test` — all tests pass, zero failures
- [ ] Backend starts with config file, serves HTTP on configured port
- [ ] `docker/local/docker-compose.yml` includes backend, frontend, MinIO, and MailHog services

### Frontend
- [ ] `FileEditorPage` — markdown editor view opened when clicking a file in the files browser or creating a new file
- [ ] Markdown editor component with text editing area for raw markdown input
- [ ] Markdown preview component rendering the markdown content as formatted HTML
- [ ] Save button calls `files_service::create_file` (new file) or `files_service::update_file` (existing file)
- [ ] `files_service` module implements `update_file` async function with `Authorization: Bearer` header
- [ ] Frontend unit tests pass — editor rendering, preview rendering, save flow for new and existing files, service function mocking
