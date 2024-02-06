FROM python:3.8.16-alpine3.17
WORKDIR /transfery
COPY ./ ./
RUN apk update &&\
    apk add --no-cache build-base &&\
    pip install --no-cache-dir pipenv &&\
    pipenv sync &&\
    pipenv --clear &&\
    apk del build-base
ENTRYPOINT ["./entrypoint.sh"]
