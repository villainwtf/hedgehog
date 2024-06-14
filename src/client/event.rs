use chrono::Utc;

use serde_json::{json, Value};
use tokio::sync::oneshot::channel;
use uuid::Uuid;

use crate::{
    data::{Event, Person},
    error::PosthogError,
};

use super::{
    queue::{PosthogRequest, QueuedRequest},
    PosthogClient,
};

impl PosthogClient {
    pub fn enqueue_event(&self, person: &Person, event: Event) -> Result<(), PosthogError> {
        let event_json = self.get_event_json(person, event);

        self.queue.enqueue_capture_event(QueuedRequest {
            request: PosthogRequest::CaptureEvent { body: event_json },
            ..Default::default()
        });

        Ok(())
    }

    pub async fn capture_event(&self, person: &Person, event: Event) -> Result<(), PosthogError> {
        let event_json = self.get_event_json(person, event);

        let (tx, rx) = channel();

        self.queue
            .dispatch_request(QueuedRequest {
                request: PosthogRequest::CaptureEvent { body: event_json },
                response_tx: Some(tx),
            })
            .await;

        rx.await.map(|_| ()).map_err(|_| PosthogError::QueueError)
    }

    fn get_event_json(&self, person: &Person, event: Event) -> Value {
        json!({
            "api_key": self.api_key,
            "uuid": Uuid::new_v4().to_string(),
            "timestamp": Utc::now(),
            "distinct_id": person.distinct_id,
            "event": event.name,
            "properties": event.build_properties(person),
        })
    }
}
