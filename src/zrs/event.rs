impl super::Event {
    pub fn from(e: &fsr::Event) -> super::Event {
        super::Event {
            event_id: e.event_id(),
            priority: e.priority(),
            owner: e.owner().to_string(),
            subclass_name: e.subclass_name().to_string(),
            key: e.key(),
            flags: e.flags(),
            headers: e.headers().clone(),
            body: e.body().to_string(),
        }
    }
}
