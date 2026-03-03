# User Story: Logout

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
