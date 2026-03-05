# User Story: Logout

## Prerequisites
- S3Provider, LocalFileSystemProvider, SQLiteProvider, SMTPProvider implemented (see @specs/BACKEND.md)
- All providers wired into application startup and registered as Actix-web app data
- User model and session/token handling implemented (from registration)
- Login endpoint implemented (user must be logged in to log out)
- AuthenticatedUser extractor implemented (see @specs/AUTH-MIDDLEWARE.md)

## Flow
```
LogoutForm
```

---

**Form:** `LogoutForm`  
**API:** `logout`  
**Endpoint:** `POST /api/auth/logout`

**Request:**
```json
{ "refresh_token": "jwt-token" }
```

**Response:**
```json
{
  "status": "ok",
  "message": "Logged out successfully"
}
```

---

## Acceptance Criteria

### Backend
- [ ] `POST /api/auth/logout` — accepts `refresh_token`, invalidates the session in SQLite, returns success
- [ ] Request requires valid `Authorization: Bearer <access_token>` header (uses `AuthenticatedUser` extractor)
- [ ] Integration tests cover: successful logout, logout with invalid token, logout with already-invalidated session
- [ ] `cargo test` — all tests pass, zero failures
- [ ] Backend starts with config file, serves HTTP on configured port
- [ ] `docker-compose.yml` includes backend, frontend, and required dependencies

### Frontend
- [ ] Logout action callable from authenticated UI (button or menu item)
- [ ] Calls `auth_service::logout` with the stored refresh token
- [ ] On success, `AuthState` is cleared, tokens are removed from `localStorage`
- [ ] User is navigated to `/login` after logout
- [ ] `auth_service` module implements `logout` async function
- [ ] Frontend unit tests pass — logout clears state, navigation occurs after logout
