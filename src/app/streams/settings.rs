use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct StreamsSettings {
    pub execution: ExecutionSettings,
    pub messaging: MessagingSettings,
    pub kv: KVSettings,
}

#[derive(Deserialize, Clone)]
pub struct ExecutionSettings {
    pub period_ms: i64,
    pub batch_size: usize,
}

#[derive(Deserialize, Clone)]
pub struct MessagingSettings {
    pub name: String,
    pub consumer: String,
    pub subjects: MessagingSubjectsSettings,
}

#[derive(Deserialize, Clone)]
pub struct MessagingSubjectsSettings {
    pub request: String,
    pub response: String,
}

#[derive(Deserialize, Clone)]
pub struct KVSettings {
    pub name: String,
}
