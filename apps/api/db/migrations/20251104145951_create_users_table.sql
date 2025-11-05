-- Create users table
-- This table stores basic user information for the attendance management system.
-- Authentication-related fields will be added in a separate table in future migrations.

CREATE TABLE users (
    -- Primary key: UUID generated automatically
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- User's display name
    name VARCHAR(100) NOT NULL,

    -- Email address: used for user identification and contact
    -- Expected to be organization-assigned email address
    -- Supports alias emails for privacy
    -- Unique constraint enforced only for active users (deleted_at IS NULL)
    email VARCHAR(255) NOT NULL,

    -- Profile picture URL (compatible with Auth0/Google profile pictures)
    picture TEXT,

    -- Timestamp when the user record was created
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,

    -- Timestamp when the user record was last updated
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,

    -- Soft delete timestamp: NULL means active, non-NULL means deleted
    -- Allows email address reuse for different users over time
    deleted_at TIMESTAMP WITH TIME ZONE
);

-- Add table comment
COMMENT ON TABLE users IS 'Stores basic user information for the attendance management system';

-- Add column comments
COMMENT ON COLUMN users.id IS 'Unique identifier for the user (UUID)';
COMMENT ON COLUMN users.name IS 'Display name of the user';
COMMENT ON COLUMN users.email IS 'Email address for user identification and contact';
COMMENT ON COLUMN users.picture IS 'URL of the user profile picture';
COMMENT ON COLUMN users.created_at IS 'Timestamp when the user was created';
COMMENT ON COLUMN users.updated_at IS 'Timestamp when the user was last updated';
COMMENT ON COLUMN users.deleted_at IS 'Soft delete timestamp (NULL = active, non-NULL = deleted)';

-- Partial unique index: email must be unique among active users only
-- This allows email address reuse when a user is deleted and a new user is created
CREATE UNIQUE INDEX idx_users_active_email ON users(email) WHERE deleted_at IS NULL;

-- Partial index on email for active users to speed up lookups
CREATE INDEX idx_users_email ON users(email) WHERE deleted_at IS NULL;
