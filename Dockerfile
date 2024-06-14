FROM rust:1.78.0-alpine3.20 AS rust-builder
WORKDIR /transfery
COPY ./ ./
RUN apk add --update --no-cache build-base pkgconfig libressl-dev &&\
    cargo build --release

FROM node:22.3.0-alpine3.20 AS node-builder
COPY ./front-end.sh ./
RUN apk add --update --no-cache curl git &&\
    sh front-end.sh &&\
    cd transfery-vue &&\
    npm install &&\
    npm run build

FROM alpine:3.20 as certs

FROM scratch
COPY --from=rust-builder /transfery/target/release/transfery /
COPY --from=node-builder /transfery-vue/dist /
COPY --from=certs /etc/ssl/cert.pem /etc/ssl/
ENTRYPOINT [ "/transfery", "--init" ]
