FROM debian:bullseye-slim
WORKDIR /taka_the_discord_bot_api

RUN apt-get update
RUN apt-get install -y openssl ca-certificates wget 
RUN wget https://dl.google.com/linux/direct/google-chrome-stable_current_amd64.deb && apt-get install -y ./google-chrome-stable_current_amd64.deb
RUN apt-get install libxss1
RUN rm -rf /var/lib/apt/lists/*
RUN update-ca-certificates
COPY --from=taka_the_discord_bot_dependencies /app/build/taka_the_discord_bot_api .
COPY --from=taka_the_discord_bot_dependencies /app/taka_the_discord_bot_api/.env.prod ./.env

CMD ["/taka_the_discord_bot_api/taka_the_discord_bot_api"]