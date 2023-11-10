import os
from fastapi import Request, FastAPI
from transformers import pipeline

tf_model = os.getenv("TF_MODEL")

summarizer = pipeline("summarization", model=tf_model)
app = FastAPI()

@app.post("/summarizer")
async def root(request: Request):
  json = await request.json()
  text = "summarize: " + json["text"]
  result = summarizer(text, truncation = True, num_beams = 3, max_new_tokens = 200, repetition_penalty = 10.0)

  return { "text": result[0]["summary_text"] }
