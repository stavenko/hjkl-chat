# Authenticated User Middleware

## Purpose

Actix-web extractor that validates the JWT access token from the `Authorization: Bearer <token>` header, looks up the user in SQLite, and provides the authenticated user to endpoint handlers.

## Extractor

```rust
// Usage in endpoint handler:
async fn some_protected_endpoint(user: AuthenticatedUser, ...) -> impl Responder {
    // user.id, user.email are available
}
```

`AuthenticatedUser` is an Actix extractor (implements `FromRequest`). When placed in a handler signature, it:

1. Reads the `Authorization` header from the request
2. Extracts and validates the Bearer JWT token
3. Extracts `user_id` from the token claims
4. Queries SQLite (from Actix app data) to load the user by id
5. Returns the user or rejects the request with 401 Unauthorized

## Struct

```rust
pub struct AuthenticatedUser {
    pub id: String,
    pub email: String,
}
```

## Error Responses

All errors return HTTP 401 with a JSON body:

```json
{
  "status": "error",
  "message": "..."
}
```

| Condition | Message |
|-----------|---------|
| Missing Authorization header | "Authorization header required" |
| Malformed header (not Bearer) | "Invalid authorization format" |
| Invalid or expired JWT | "Invalid or expired token" |
| User not found in database | "User not found" |

## Location

`src/api/extractors/authenticated_user.rs`

## Dependencies

- SQLiteProvider registered as Actix-web app data
- JWT secret from application config
- User table in SQLite (id, email, created_at)
