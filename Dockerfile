FROM rust AS builder

WORKDIR /usr/ne-student-api

COPY ./src ./src
COPY ./sql ./sql
COPY ./Cargo.lock .
COPY ./Cargo.toml .

RUN cargo build --release

FROM ubuntu
ARG PORT=5505

RUN apt-get update && apt-get -y install ca-certificates libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/ne-student-api
COPY --from=builder /usr/ne-student-api/target/release/ne-student-api .

EXPOSE 5505

CMD ./ne-student-api