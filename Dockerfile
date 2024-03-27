FROM debian:bookworm-slim AS runtime
WORKDIR /usr/src/app
COPY target/release/game-leaderboard-backend .

RUN apt-get update && apt-get install -y libssl3 && rm -rf /var/lib/apt/lists/*

EXPOSE 80
CMD ["./game-leaderboard-backend"]
