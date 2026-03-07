# Backend

Backend shall have:

1. SMTPProvider – sends user emails.
2. S3Provider – stores all persistent data (user files, compressed archives, keyword files, SQLite database file).
3. LocalFileSystemProvider – local filesystem cache for objects fetched from S3 (avoids repeated downloads).
   - `save(object: Vec<u8>) -> PathBuf`: Saves an object to the filesystem and returns its path.
   - `path(object_id: &str) -> Option<PathBuf>`: Retrieves the stored path for a given object.
   - `delete(object_id: &str) -> Result<()>`: Deletes the stored object.
   - `read(object_id: &str) -> Result<Vec<u8>>`: Returns the contents of the stored object as a byte vector.
4. SQLiteProvider – query engine over a SQLite database whose `.db` file lives in S3 and is cached locally.

## Provider Details

### LocalFileSystemProvider

Used to keep some objects from network in local cache directory.
This directory may be provided by docker or k8s. So it is necessary to write
only within directory `root`, with which it is initialized.

Can keep many files within itself, So it

**Initialization:**

- `new(root: PathBuf) -> Result<Self>`
  - Tries to make emtpy file in the root, then constructs itself and returns.

**Methods:**

- `save(object: Vec<u8>) -> PathBuf`
  - Saves a byte array to the local filesystem with randomized name .
  - Stores under own root.
  - Returns the path to the saved file.

- `delete(PathBuf: &str) -> Result<()>`
  - Removes the stored object from the filesystem.
  - Only can remove files relative to the directory.

- `read(object_id: &str) -> Result<Vec<u8>>`
  - Reads and returns the contents of a stored object as `Vec<u8>`.

- `get_path(filename: &str) -> Result<PathBuf>`
  - Creates directory, where filename must be kept.
  - directory is a directory of filepath `root.join(filename)`

### SQLiteProvider

Query engine over a SQLite database stored in S3 and cached locally.

**Initialization:**

- `new(s3_provider, fs_provider, s3_object_path: &str) -> Self`
  - Downloads the database file from S3 via the S3Provider.
  - Saves it locally using LocalFileSystemProvider.
  - Retrieves the local file path.
  - Opens a `rusqlite` connection to the database file.

In s3_object_path do not exists, then SQLiteProvider creates new database using
LocalFileSystemProvider 'get_file_path(filename)' method. And then uses normal
flow, using `dump_to_s3()' method.

**Methods:**

- `dump_to_s3()`
  - Must be called after every `INSERT` or `UPDATE` query.
  - Internally spawns an async task using `tokio::spawn`.
  - The spawned task:
    - Flushes the database state to disk.
    - Reads the database file via LocalFileSystemProvider.
    - Uploads the file back to S3 using the S3Provider.
  - Returns nothing to the caller (fire-and-forget).

## CLI

All startup logic is implemented in `cli.rs`.

Command-line arguments are parsed using `clap`.

**Commands:**

- `run --config <CONFIG-PATH>`
  - Loads configuration from the specified path.
  - Executes the startup sequence.
  - Starts the Actix HTTP server.

- `download-sqlite --config <CONFIG-PATH>`
  - Loads configuration from the specified path.
  - Downloads the SQLite database file from S3.
  - Saves it to the local filesystem cache.
  - Exits without starting the server.

## Startup Sequence

The `run` command performs the following steps:

1. **S3Provider** – connect to S3-compatible storage, verify bucket exists.
2. **LocalFileSystemProvider** – initialize local cache directory.
3. **SQLiteProvider** – fetch the SQLite database file from S3 via LocalFileSystemProvider, open a `rusqlite` connection.
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

## Config

Config file is a toml file with sections. 

### Main section.

Contains port and addr and does not have its own name:

```toml
addr = "0.0.0.0"
port = 5000
```

### SQLite provider

```toml
addr = "0.0.0.0"
port = 5000

[sqlite]
s3_object_path

```
### S3 provider

Contains all needed data to init bucket

```toml
addr = "0.0.0.0"
port = 5000

[s3]
bucket = ""
region = ""
client_id = ""
client_secret = ""
host = ""

```

### Local FS provider

Contains root path in filesystem

```toml
addr = "0.0.0.0"
port = 5000

[local_fs]
root_path = ""

```

### SMTP provider

Contains settings for SMTP to send

```toml
addr = "0.0.0.0"
port = 5000

[smtp]
host = ""
port = ""
use_tls = ""
username = ""
password = ""

```
Config MUST NOT have a default config. It is never must be constructed in code -
only read from file. Even for testing purpose.



## Error handling

Each endpoint must be very simple:

```
pub async fn endpoint(provider: web::Data<Arc<Provider>>) -> impl Responder {
    let result: ApiResponse<_> = use_case::command((*provider.into_inner()).clone())
        .await
        .into();
    result
}
```

ApiResponse:

```
pub enum ApiResponse<T> {
    Ok(T),
    Err(Error),
}

impl<T> Responder for ApiResponse<T>
where
    T: Serialize + fmt::Debug,
{
    type Body = EitherBody<String>;

    fn respond_to(self, _req: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        let response = match self {
            Self::Ok(r) => serde_json::to_value(&r).map_or_else(
                |_| {
                    tracing::error!("Cannot serialize response: {:?}", r);
                    HttpResponse::InternalServerError()
                        .message_body("Failed to serialize response".to_owned())
                },
                |v| {
                    let response = json!({
                      "status": "ok",
                      "payload": v
                    });

                    HttpResponse::Ok()
                        .content_type(mime::APPLICATION_JSON)
                        .message_body(serde_json::to_string(&response).unwrap())
                },
            ),
            Self::Err(e) => {
                let response = json!({
                  "status": "error",
                  "error": e
                });

                serde_json::to_string(&response)
                    .map_err(|err| err.into())
                    .and_then(|body| {
                        HttpResponse::build(StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS)
                            .content_type(mime::APPLICATION_JSON)
                            .message_body(body)
                    })
            }
        };
        match response {
            Ok(res) => res.map_into_left_body(),
            Err(err) => HttpResponse::from_error(err).map_into_right_body(),
        }
    }
}

impl<T, E> From<Result<T, E>> for ApiResponse<T>
where
    E: Into<Error>,
{
    fn from(value: Result<T, E>) -> Self {
        match value {
            Ok(e) => ApiResponse::Ok(e),
            Err(e) => ApiResponse::Err(e.into()),
        }
    }
}
```

And ApiError

```rust

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Error {
    pub code: String,
    pub message: String,
}
```

This approch allows in use cases to create their own error types:

```
#[derive(thiserror::Error, Debug)]
#[allow(unused)]
pub enum UseCaseError {
    #[error("Internal server error")]
    InternalServerError,
}

impl From<AddTaskError> for Error {
    fn from(value: AddTaskError) -> Self {
        match value {
            AddTaskError::InternalServerError => Error {
                code: "InternalServerError".to_string(),
                message: value.to_string(),
            },
        }
    }
}
```

Remember to keep endpoint small. But usecase must contains all the
busyness-logic.

## Search

Backend computes TF-IDF keywords on file save and stores them in S3 alongside each file. Backend does **not** maintain a search index — the search index lives on the frontend (localStorage/IndexedDB). The frontend rebuilds its index by fetching the TOC and keyword files from the backend when needed (see `SEARCH-ENGINE.md` and `FILE-STORAGE.md`).
