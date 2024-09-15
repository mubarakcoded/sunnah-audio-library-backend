-- Enable the UUID extension if not already enabled
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create the users table
CREATE TABLE "users" (
    user_id UUID NOT NULL PRIMARY KEY DEFAULT uuid_generate_v4 (),
    username VARCHAR(255) NOT NULL UNIQUE,
    password VARCHAR(255) NOT NULL,
    role VARCHAR(50) NOT NULL,
    status BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP(0) NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP(0) NOT NULL DEFAULT NOW()
);


-- Insert a test user with a hashed password
INSERT INTO
    "users" (username, password, role)
VALUES (
        'testuser',
        '$argon2id$v=19$m=19456,t=2,p=1$RoU+NtOLyH+KomnjUfQP7w$G2TGsv4t6kHk9H53ui2CMZ68hZ3JuQdGTPr3jrirOHk',
        'service_admin'
    );