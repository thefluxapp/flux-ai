use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct StreamsSettings {
    pub limit_cnt: i64,
    pub execution: ExecutionSettings,
    pub messaging: MessagingSettings,
    pub kv: KVSettings,
}

#[derive(Deserialize, Clone)]
pub struct ExecutionSettings {
    pub interval_ms: i64,
    pub batch_cnt: usize,
}

#[derive(Deserialize, Clone)]
pub struct MessagingSettings {
    pub message: MessagingMessageSettings,
    pub stream: MessagingStreamSettings,
    // pub name: String,
    // pub consumer: String,
    // pub subjects: MessagingSubjectsSettings,
}

#[derive(Deserialize, Clone)]
pub struct MessagingMessageSettings {
    pub subjects: Vec<String>,
    pub consumer: String,
}

// #[derive(Deserialize, Clone)]
// pub struct MessagingSubjectsSettings {
//     pub request: String,
//     pub response: String,
// }

#[derive(Deserialize, Clone)]
pub struct MessagingStreamSettings {
    pub subject: String,
}

#[derive(Deserialize, Clone)]
pub struct KVSettings {
    pub name: String,
}
