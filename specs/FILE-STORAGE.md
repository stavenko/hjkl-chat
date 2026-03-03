# File Storage

All user files are stored in S3. Each file is compressed, accompanied by a TF-IDF keyword file, and tracked in a per-user table of contents (TOC).

## S3 Path Layout

```
/users/<user-id>/
    <path>.tgz              # compressed file content
    <path>.kw.tgz            # TF-IDF keywords for the file
    toc/                     # new TOC entries (one empty marker per file)
        <path>               # empty object, key = file path
    toc.tgz                  # compacted TOC archive
```

## File Save

**Endpoint:** `POST /api/files`

**Input:**
```json
{
  "path": "/hjkl-chat-bot/chats/storage-and-authentication.yaml",
  "content": "<file content>"
}
```

**Steps:**
1. Compress `content` and store as `/users/<user-id>/<path>.tgz` in S3.
2. Extract TF-IDF keywords from `content` (see `SEARCH-ENGINE.md`), compress, and store as `/users/<user-id>/<path>.kw.tgz` in S3.
3. Create an empty marker object at `/users/<user-id>/toc/<path>` in S3.

**Response:** `201 Created`

## Keyword File Download

**Endpoint:** `GET /api/files/keywords`

**Query parameters:**
- `path` (required) – the file path (e.g., `/hjkl-chat-bot/chats/storage-and-authentication.yaml`).

**Steps:**
1. Fetch `/users/<user-id>/<path>.kw.tgz` from S3.
2. Return the decompressed keyword list.

**Response:** `200 OK` with plain text body (one keyword per line).

The frontend uses this endpoint to rebuild its local search index when it has no cached index (see `SEARCH-ENGINE.md`).

## Table of Contents (TOC)

The TOC lists all files belonging to a user. It is split into two parts:
- **Compacted TOC** (`/users/<user-id>/toc.tgz`) – a compressed archive containing the full list of file paths accumulated up to the last compaction.
- **New entries** (`/users/<user-id>/toc/*`) – individual empty marker objects created since the last compaction.

### TOC Read

**Endpoint:** `GET /api/toc`

**Query parameters:**
- `next` (optional) – base64-encoded cursor from a previous response.

**Steps:**
1. Check the in-memory index keyed by `<user-id>-toc`.
2. If no index exists (first call):
   a. Download `/users/<user-id>/toc.tgz` from S3, decompress to get the list of compacted paths.
   b. List all objects under `/users/<user-id>/toc/` in S3 to get new entries.
   c. Merge both lists into a single sorted index, store in memory keyed by `<user-id>-toc`.
3. If `next` is provided, decode the base64 cursor to get `from_item` and `items_per_page`.
4. If `next` is not provided, start from the beginning with a default `items_per_page`.
5. Slice the index from `from_item` for `items_per_page` entries.
6. Encode the next cursor as base64 of `from_item + items_per_page` and `items_per_page`.

**Response:**
```json
{
  "items": [
    "/hjkl-chat-bot/chats/storage-and-authentication.yaml",
    "/hjkl-chat-bot/chats/search-design.yaml"
  ],
  "next": "<base64-cursor>"
}
```

`next` is `null` when there are no more items.

### TOC Compaction

**Endpoint:** `POST /api/compact-toc`

**Steps:**
1. Read `/users/<user-id>/toc.tgz` from S3 (may not exist yet).
2. List all objects under `/users/<user-id>/toc/` in S3.
3. Merge the existing compacted list with the new entries, deduplicate, sort.
4. Compress the merged list and write it back as `/users/<user-id>/toc.tgz`.
5. Delete all individual objects under `/users/<user-id>/toc/`.
6. Invalidate the in-memory index for `<user-id>-toc`.

**Response:** `200 OK`
