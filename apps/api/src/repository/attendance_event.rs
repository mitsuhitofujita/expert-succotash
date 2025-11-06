use crate::error::Result;
use crate::models::{AttendanceEvent, CreateAttendanceEvent};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

/// Attendance event repository for database operations
/// Handles creation and retrieval of immutable attendance events
/// Note: Events are immutable, so no update or delete operations are provided
#[derive(Clone)]
pub struct AttendanceEventRepository {
    pool: PgPool,
}

impl AttendanceEventRepository {
    /// Create a new `AttendanceEventRepository` instance
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Find an attendance event by ID
    ///
    /// # Arguments
    /// * `id` - The UUID of the attendance event
    ///
    /// # Returns
    /// * `Ok(Some(AttendanceEvent))` - Event found
    /// * `Ok(None)` - Event not found
    ///
    /// # Errors
    /// Returns `AppError` if database query fails
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<AttendanceEvent>> {
        let event = sqlx::query_as!(
            AttendanceEvent,
            r#"
            SELECT id, user_id, event_type, event_time, recorded_at, created_at
            FROM attendance_events
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(event)
    }

    /// Find all attendance events for a specific user
    /// Returns events ordered by `event_time` in descending order (most recent first)
    ///
    /// # Arguments
    /// * `user_id` - The UUID of the user
    ///
    /// # Returns
    /// * `Ok(Vec<AttendanceEvent>)` - List of events (may be empty)
    ///
    /// # Errors
    /// Returns `AppError` if database query fails
    pub async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<AttendanceEvent>> {
        let events = sqlx::query_as!(
            AttendanceEvent,
            r#"
            SELECT id, user_id, event_type, event_time, recorded_at, created_at
            FROM attendance_events
            WHERE user_id = $1
            ORDER BY event_time DESC
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(events)
    }

    /// Create a new attendance event
    /// The `recorded_at` timestamp is set to the current server time automatically
    ///
    /// # Arguments
    /// * `event` - The attendance event creation request data
    ///
    /// # Returns
    /// * `Ok(AttendanceEvent)` - The created event with generated ID and timestamps
    ///
    /// # Errors
    /// Returns `AppError` if database query fails
    pub async fn create(&self, event: CreateAttendanceEvent) -> Result<AttendanceEvent> {
        let recorded_at = Utc::now();

        let created_event = sqlx::query_as!(
            AttendanceEvent,
            r#"
            INSERT INTO attendance_events (user_id, event_type, event_time, recorded_at)
            VALUES ($1, $2, $3, $4)
            RETURNING id, user_id, event_type, event_time, recorded_at, created_at
            "#,
            event.user_id,
            event.event_type,
            event.event_time,
            recorded_at
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(created_event)
    }
}
