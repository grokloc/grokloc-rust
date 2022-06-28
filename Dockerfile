FROM rust:slim-bullseye
WORKDIR /grokloc
RUN apt update
RUN apt install -y \
    sqlite3 \
    libsqlite3-0 \
    libsqlite3-dev \
    pkg-config \
    libssl-dev \
    make

CMD ["tail", "-f", "/dev/null"]
