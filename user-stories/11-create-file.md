# User Story: Create File Manually

## Prerequisites
- Files browser implemented (from 07-files-browser)
- S3Provider and LocalFileSystemProvider implemented (see @specs/BACKEND.md)
- User must be authenticated (login flow from 02-login)
- AuthenticatedUser extractor implemented (see @specs/AUTH-MIDDLEWARE.md)

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

---

## Acceptance Criteria

### Backend
- [ ] `POST /api/files` — accepts `path` and `content`, stores file in S3 under `{user_id}/{path}`, saves metadata in SQLite, returns file `id`, `path`, `link`
- [ ] `GET /api/files/suggestions?path=/ab` — returns path prefix suggestions from existing files for the authenticated user
- [ ] Both endpoints require valid `Authorization: Bearer <access_token>` header
- [ ] Integration tests cover: create file, file stored in S3, metadata in SQLite, path suggestions return correct results, duplicate path handling, unauthorized access rejected
- [ ] `cargo test` — all tests pass, zero failures
- [ ] Backend starts with config file, serves HTTP on configured port
- [ ] `docker-compose.yml` includes backend, frontend, MinIO, and required dependencies

### Frontend
- [ ] `CreateFilePage` (or file editor view) accessible from files browser via `AddFileButton`
- [ ] Path input field with autocomplete — calls `files_service::path_suggestions` as user types, displays dropdown of matching paths
- [ ] Content editor for file body
- [ ] Submit creates file via `files_service::create_file`, navigates back to files browser on success
- [ ] `files_service` module implements `create_file` and `path_suggestions` async functions with `Authorization: Bearer` header
- [ ] Frontend unit tests pass — path autocomplete behavior, file creation flow, service function mocking
