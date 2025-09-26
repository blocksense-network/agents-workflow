//! Server-Sent Events (SSE) streaming support

use aw_rest_api_contract::SessionEvent;
use futures::stream::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::mpsc;

use crate::error::{RestClientError, RestClientResult};
use crate::auth::AuthConfig;

/// SSE event stream for session events
pub struct SessionEventStream {
    receiver: mpsc::Receiver<Result<SessionEvent, RestClientError>>,
    _handle: tokio::task::JoinHandle<()>,
}

impl SessionEventStream {
    /// Create a new SSE stream for session events
    pub async fn connect(
        _base_url: &url::Url,
        _session_id: &str,
        _auth: &AuthConfig,
    ) -> RestClientResult<Self> {
        // TODO: Implement proper SSE streaming with eventsource-client
        // For now, return a placeholder that never yields events
        let (_tx, rx) = mpsc::channel(32);

        let handle = tokio::spawn(async {
            // Placeholder - keep the task alive
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        });

        Ok(SessionEventStream {
            receiver: rx,
            _handle: handle,
        })
    }
}

impl Stream for SessionEventStream {
    type Item = Result<SessionEvent, RestClientError>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        self.receiver.poll_recv(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aw_rest_api_contract::{EventType, SessionStatus};

    #[test]
    fn test_session_event_parsing() {
        let event_json = r#"{
            "type": "status",
            "status": "running",
            "ts": "2025-01-01T12:00:00Z"
        }"#;

        let event: aw_rest_api_contract::SessionEvent = serde_json::from_str(event_json).unwrap();
        assert_eq!(event.event_type, EventType::Status);
        assert_eq!(event.status, Some(SessionStatus::Running));
    }
}
