-- Create attendance_events table
-- This table records all raw clock-in/out events from users as immutable audit trail.
-- It stores three timestamps (event_time, recorded_at, created_at) for comprehensive auditing:
-- - event_time: The time specified by the client (user's intended time)
-- - recorded_at: Server request reception time (for detecting delays/tampering)
-- - created_at: Database record creation time (DB-level audit trail)

CREATE TABLE attendance_events (
    -- Primary key: UUID generated automatically
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Foreign key to users table
    -- ON DELETE CASCADE ensures events are deleted when the user is deleted
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    -- Event type: 'clock_in', 'clock_out', 'break_start', 'break_end'
    -- Represents the type of attendance event
    event_type VARCHAR(20) NOT NULL,

    -- Client-specified time: The time the user intended for this event
    -- This is provided by the client and can be set retroactively for corrections
    -- Microsecond precision (TIMESTAMP(6)) for distinguishing near-simultaneous events
    event_time TIMESTAMP(6) WITH TIME ZONE NOT NULL,

    -- Server reception time: When the API request was received by the server
    -- Set by the application layer (current timestamp when processing the request)
    -- Used for detecting network delays and potential tampering
    -- Microsecond precision (TIMESTAMP(6)) for high-resolution audit trail
    recorded_at TIMESTAMP(6) WITH TIME ZONE NOT NULL,

    -- Database creation time: When the record was written to the database
    -- Automatically set by the database (DEFAULT CURRENT_TIMESTAMP)
    -- Provides database-level audit trail
    -- Microsecond precision (TIMESTAMP(6)) for complete audit accuracy
    created_at TIMESTAMP(6) WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Add table comment
COMMENT ON TABLE attendance_events IS 'Immutable audit trail of all attendance clock-in/out events';

-- Add column comments
COMMENT ON COLUMN attendance_events.id IS 'Unique identifier for the attendance event (UUID)';
COMMENT ON COLUMN attendance_events.user_id IS 'Reference to the user who created this event';
COMMENT ON COLUMN attendance_events.event_type IS 'Type of event: clock_in, clock_out, break_start, break_end';
COMMENT ON COLUMN attendance_events.event_time IS 'Client-specified event time (user intended time, can be retroactive)';
COMMENT ON COLUMN attendance_events.recorded_at IS 'Server reception time (for detecting delays/tampering)';
COMMENT ON COLUMN attendance_events.created_at IS 'Database record creation time (DB-level audit trail)';

-- Index for efficient querying by user and event time (descending order for recent-first queries)
CREATE INDEX idx_attendance_events_user_event_time ON attendance_events(user_id, event_time DESC);

-- Index for efficient daily attendance queries in Asia/Tokyo timezone
-- This index supports queries filtering by user and date, then ordering by time
CREATE INDEX idx_attendance_events_user_date_time ON attendance_events(
    user_id,
    DATE(event_time AT TIME ZONE 'Asia/Tokyo'),
    event_time
);
