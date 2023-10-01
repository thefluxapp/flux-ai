FROM rust:1-slim-bullseye

WORKDIR /app

RUN apt update
RUN apt install -y bash gcc g++ cmake build-essential openssl libssl-dev pkg-config python3 python3-dev python3-pip

RUN pip3 install --upgrade pip

RUN pip3 install fastapi uvicorn
RUN pip3 install torch --extra-index-url https://download.pytorch.org/whl/cpu
RUN pip3 install transformers
