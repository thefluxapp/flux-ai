use axum::{extract::State, response::Html, routing::get, Router};
use std::{env, net::SocketAddr, str::FromStr, sync::Arc};
use tokio::sync::RwLock;

use super::candle::Candle;

const PROMPT: &str = "Наташа: Привет, у тебя есть номер Максима? Ксюша: Извини, не могу найти Ксюша: Спроси Диму Ксюша: Он звонил ей когда они последний раз были в парке Наташа: Я его не очень хорошо знаю Ксюша: Не бойся, он очень добрый и отзывчивый Наташа: Может быть лучше ты напишешь ему? Ксюша: Просто напиши 🙂 Наташа: Ох.. Ну ладно Наташа: Пока Ксюша: Пока, пока";

pub async fn run() {
    let candle = Candle::new(env::var("HUGGINGFACE_MODEL").unwrap());

    println!("CANDLE IS READY");

    // let text = candle.call(PROMPT.to_string()).unwrap();
    // println!("{}", text);

    let app = Router::new()
        .route("/summarize", get(handler))
        .with_state(Arc::new(RwLock::new(AppState { candle })));

    let addr = SocketAddr::from_str("0.0.0.0:3000").unwrap();

    println!("SERVER IS READY TO ACCEPT");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

struct AppState {
    candle: Candle,
}

async fn handler(State(state): State<Arc<RwLock<AppState>>>) -> Html<String> {
    let candle = &mut state.write().await.candle;
    let text = candle.call(PROMPT.to_string()).unwrap();

    Html(text)
}
