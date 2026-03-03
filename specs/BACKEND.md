# Backend

Backend shall have:

1. SMTPProvider – sends user emails.
2. S3Provider – stores all persistent data (user files, compressed archives, keyword files, SQLite database file).
3. CacheProvider – local filesystem cache for objects fetched from S3 (avoids repeated downloads).
4. SQLiteProvider – query engine over a SQLite database whose `.db` file lives in S3 and is cached locally.

## Startup Sequence

1. **S3Provider** – connect to S3-compatible storage, verify bucket exists.
2. **CacheProvider** – initialize local cache directory.
3. **SQLiteProvider** – fetch the SQLite database file from S3 via CacheProvider, open a `rusqlite` connection.
4. **SMTPProvider** – connect to the mail server.
5. **Actix data** – register all providers as application data.
6. **Serve** – bind and start the HTTP server.

## What SQLite Stores

- Users (id, email, created_at).
- Password hashes (user_id, hash, algorithm).
- Sessions: access tokens and refresh tokens (user_id, token, expires_at).

SQLite is the only query engine; its database file is periodically flushed back to S3 so it survives restarts.

## What S3 Stores

- The SQLite database file itself.
- All user files (chats, documents, anything created by a user), compressed as `.tgz`.
- TF-IDF keyword files (`.kw.tgz`) alongside each user file.
- TOC (table of contents) structures per user.

## Search

Backend computes TF-IDF keywords on file save and stores them in S3 alongside each file. Backend does **not** maintain a search index — the search index lives on the frontend (localStorage/IndexedDB). The frontend rebuilds its index by fetching the TOC and keyword files from the backend when needed (see `SEARCH-ENGINE.md` and `FILE-STORAGE.md`).
