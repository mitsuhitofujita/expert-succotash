-- Create main schema in attendance_dev database
CREATE SCHEMA IF NOT EXISTS main;
GRANT ALL ON SCHEMA main TO attendance_user;
ALTER USER attendance_user SET search_path TO main, public;

-- Set default privileges in main schema for attendance_dev
ALTER DEFAULT PRIVILEGES FOR USER attendance_user IN SCHEMA main GRANT ALL ON TABLES TO attendance_user;
ALTER DEFAULT PRIVILEGES FOR USER attendance_user IN SCHEMA main GRANT ALL ON SEQUENCES TO attendance_user;

-- Create test database for automated testing
CREATE DATABASE attendance_test ENCODING 'UTF-8' LC_COLLATE 'C' LC_CTYPE 'C';

-- Grant privileges to test user
GRANT ALL PRIVILEGES ON DATABASE attendance_test TO attendance_user;

-- Connect to test database and setup main schema
\c attendance_test

CREATE SCHEMA IF NOT EXISTS main;
GRANT ALL ON SCHEMA main TO attendance_user;
ALTER USER attendance_user SET search_path TO main, public;

-- Set default privileges in main schema for attendance_user
ALTER DEFAULT PRIVILEGES FOR USER attendance_user IN SCHEMA main GRANT ALL ON TABLES TO attendance_user;
ALTER DEFAULT PRIVILEGES FOR USER attendance_user IN SCHEMA main GRANT ALL ON SEQUENCES TO attendance_user;
