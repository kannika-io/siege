use siege_api_spec::SseEvent;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct Broadcaster {
    tx: broadcast::Sender<SseEvent>,
}

impl Broadcaster {
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self { tx }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<SseEvent> {
        self.tx.subscribe()
    }

    pub fn send(&self, event: SseEvent) {
        let _ = self.tx.send(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn broadcaster_delivers_events() {
        let bc = Broadcaster::new(16);
        let mut rx = bc.subscribe();

        let event = SseEvent::TopicDeleted {
            name: "gone".into(),
        };
        bc.send(event.clone());

        let received = rx.recv().await.unwrap();
        assert!(matches!(received, SseEvent::TopicDeleted { name } if name == "gone"));
    }

    #[test]
    fn send_without_subscribers_does_not_panic() {
        let bc = Broadcaster::new(16);
        bc.send(SseEvent::TopicsSnapshot { topics: vec![] });
    }
}
