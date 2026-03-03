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
