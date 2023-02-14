import os
from fastapi import Request, FastAPI
from transformers import pipeline

tf_model = os.getenv("TF_MODEL")

summarizer = pipeline("summarization", model=tf_model)
app = FastAPI()

@app.post("/summarizer")
async def root(request: Request):
  json = await request.json()

  return summarizer(json["text"], truncation=True)
