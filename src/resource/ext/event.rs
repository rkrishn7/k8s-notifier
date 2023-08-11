use k8s_openapi::api::core::v1::Event;

/// Helper methods for [`Event`]
pub trait EventExt {
    /// Type of this event
    fn typ(&self) -> Option<&String>;
    /// Reason for this event
    fn reason(&self) -> Option<&String>;
    /// Message for this event
    fn message(&self) -> Option<&String>;
    /// The time at which this event was first recorded
    fn first_timestamp(&self) -> Option<&chrono::DateTime<chrono::Utc>>;
    /// The time at which this event was last recorded
    fn last_timestamp(&self) -> Option<&chrono::DateTime<chrono::Utc>>;
    /// The name of the involved object
    fn involved_object_name(&self) -> Option<&String>;
    /// The kind of the involved object
    fn involved_object_kind(&self) -> Option<&String>;
    /// The number of times this event has occurred.
    fn count(&self) -> Option<i32>;
}

impl EventExt for Event {
    fn typ(&self) -> Option<&String> {
        self.type_.as_ref()
    }

    fn reason(&self) -> Option<&String> {
        self.reason.as_ref()
    }
    fn message(&self) -> Option<&String> {
        self.message.as_ref()
    }

    fn first_timestamp(&self) -> Option<&chrono::DateTime<chrono::Utc>> {
        self.first_timestamp.as_ref().map(|t| &t.0)
    }

    fn last_timestamp(&self) -> Option<&chrono::DateTime<chrono::Utc>> {
        self.last_timestamp.as_ref().map(|t| &t.0)
    }

    fn involved_object_name(&self) -> Option<&String> {
        self.involved_object.name.as_ref()
    }

    fn involved_object_kind(&self) -> Option<&String> {
        self.involved_object.kind.as_ref()
    }

    fn count(&self) -> Option<i32> {
        self.count
    }
}
