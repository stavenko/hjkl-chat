# User Story: Files Browser

## Prerequisites
- User must be authenticated (login flow from 02-login)
- Design system components implemented (see Design System section below)

## Frontend Prerequisites
- Frontend project bootstrapped with Leptos CSR and trunk (see @specs/FRONTEND.md)
- Frontend routing implemented in app.rs (see @specs/GENERIC-FRONTEND.md)
- AuthState implemented — user must be logged in to access this page (see @specs/GENERIC-FRONTEND.md)
- Reusable components implemented: TextInput, Button (see @specs/GENERIC-FRONTEND.md)
- files_service module created with API base URL configuration (see @specs/GENERIC-FRONTEND.md)

## Flow
```
FilesBrowser → (ToggleProject) → (AddProject | AddFile | Filter | OpenFile)
```

---

## UI Layout

Single page with:
- Projects list (accordion style)
- Files list within each expanded project
- Filter field at the bottom

---

## Components

### 1. Project Accordion

- Projects displayed as expandable/collapsible accordion items
- Tapping on project header toggles files view
- Toggle state persisted in `localStorage` (key: `files-browser-expanded-projects`)
- Storage format: `["project-id-1", "project-id-2"]`

**Design System Tokens:**
- `project-active` - style for expanded project
- `project-inactive` - style for collapsed project

---

### 2. Create Project Button

- Positioned at the top, before first project
- Opens modal with input field
- On submit: creates new project, closes modal, refreshes project list

**Design System Component:**
- `AddProjectButton`

---

### 3. Add File Button

- Positioned within each project, before first file
- Only visible when project is expanded
- Opens view described in "create-file" story (11-create-file)

**Design System Component:**
- `AddFileButton`

---

### 4. File List

- Files listed with alternating row colors
- Long filenames wrap to maximum 2 lines
- Clicking on file opens it for viewing/editing

**Design System Tokens:**
- `file-odd` - style for odd-numbered rows
- `file-even` - style for even-numbered rows

---

### 5. Filter Field

- Positioned at the bottom of the page
- When empty: shows all projects and files
- When has ≥1 character: filters projects and files in expanded projects
- Filter matches against:
  - Project name
  - File name (in expanded projects only)

**Design System Component:**
- `FilterInput`

---

## API Endpoints

**API:** `projects-list`  
**Endpoint:** `GET /api/projects`

**Response:**
```json
{
  "status": "ok",
  "projects": [
    {
      "id": "uuid",
      "name": "My Project",
      "description": "Project description",
      "file_count": 12,
      "updated_at": "ISO8601"
    }
  ]
}
```

---

**API:** `files-list-by-project`  
**Endpoint:** `GET /api/files/project/:project_id`

**Response:**
```json
{
  "status": "ok",
  "files": [
    {
      "id": "uuid",
      "name": "document.md",
      "path": "/document.md",
      "size": 1024,
      "created_at": "ISO8601",
      "updated_at": "ISO8601"
    }
  ]
}
```

---

**API:** `projects-create`  
**Endpoint:** `POST /api/projects`

**Request:**
```json
{
  "name": "New Project",
  "description": "Optional description"
}
```

**Response:**
```json
{
  "status": "ok",
  "project": {
    "id": "uuid",
    "name": "New Project",
    "description": "Optional description",
    "created_at": "ISO8601"
  }
}
```

---

## Design System Requirements

The following components/tokens must be implemented:

| Component/Token | Purpose |
|----------------|---------|
| `project-active` | Style for expanded project accordion |
| `project-inactive` | Style for collapsed project accordion |
| `AddProjectButton` | Button component to create new project |
| `AddFileButton` | Button component to add file to project |
| `file-odd` | Style for odd-numbered file rows |
| `file-even` | Style for even-numbered file rows |
| `FilterInput` | Filter input field component |

---

## State Management

**LocalStorage:**
- Key: `files-browser-expanded-projects`
- Value: Array of project IDs that should be expanded
- Updated on each project toggle

**Component State:**
- `filterQuery`: string (current filter text)
- `projects`: array (fetched from API)
- `expandedProjects`: array (from localStorage)
- `filesByProject`: map (project_id → files array)

---

## Acceptance Criteria

### Backend
- [ ] `GET /api/projects` — returns list of projects for the authenticated user with `id`, `name`, `description`, `file_count`, `updated_at`
- [ ] `GET /api/files/project/:project_id` — returns list of files in the project with `id`, `name`, `path`, `size`, `created_at`, `updated_at`
- [ ] `POST /api/projects` — creates a new project with `name` and optional `description`, returns the created project
- [ ] All three endpoints require valid `Authorization: Bearer <access_token>` header
- [ ] Integration tests cover: list projects (empty and populated), list files by project, create project, unauthorized access rejected, non-existent project returns error
- [ ] `cargo test` — all tests pass, zero failures
- [ ] Backend starts with config file, serves HTTP on configured port
- [ ] `docker/local/docker-compose.yml` includes backend, frontend, MinIO, and MailHog services

### Frontend
- [ ] `FilesBrowserPage` exists at route `/files`, requires authenticated user (redirects to `/login` if not authenticated)
- [ ] `ProjectAccordion` component — displays projects as expandable/collapsible items, toggle state persisted in `localStorage` under key `files-browser-expanded-projects`
- [ ] `AddProjectButton` component — opens modal with project name input, on submit calls `files_service::create_project`, refreshes project list
- [ ] `AddFileButton` component — visible within expanded projects, navigates to create-file view
- [ ] `FileList` component — displays files with alternating row styles (`file-odd`/`file-even`), long filenames wrap to max 2 lines, clicking a file opens it
- [ ] `FilterInput` component — positioned at bottom, filters projects and files in expanded projects by name when >=1 character entered
- [ ] `files_service` module implements `list_projects`, `list_files_by_project`, `create_project` async functions with `Authorization: Bearer` header
- [ ] Frontend unit tests pass — accordion toggle, filter logic, project creation flow, service function mocking
