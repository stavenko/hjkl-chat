# User Story: Files Browser

## Prerequisites
- User must be authenticated (login flow from 02-login)
- Design system components implemented (see Design System section below)

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
