CREATE TABLE IF NOT EXISTS registration_sessions (
    id UUID PRIMARY KEY,
    email TEXT UNIQUE NOT NULL,
    verification_code TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    resend_available_at TIMESTAMP NOT NULL
);