# draftwright's web app: the command line built from this repository, behind web/app.py.
#   docker build -t draftwright-web .  &&  docker run -p 8000:8000 -e APP_PASSWORD=... draftwright-web

FROM rust:1.99-slim-bookworm AS build
RUN apt-get update && apt-get install -y --no-install-recommends git ca-certificates pkg-config \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /src
COPY . .
RUN cargo build --release --locked --bin draftwright

FROM python:3.12-slim-bookworm
RUN apt-get update && apt-get install -y --no-install-recommends poppler-utils \
    && rm -rf /var/lib/apt/lists/*
COPY --from=build /src/target/release/draftwright /usr/local/bin/draftwright
WORKDIR /app
COPY web/requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt
COPY web/ .
COPY docs/asme-gov-spec.json .
ENV DRAFTWRIGHT_BIN=/usr/local/bin/draftwright \
    DRAFTWRIGHT_CACHE=/tmp/draftwright-cache \
    PORT=8000
EXPOSE 8000
CMD ["sh", "-c", "uvicorn app:app --host 0.0.0.0 --port ${PORT}"]
