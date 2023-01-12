FROM python:3

ARG UID=1000

WORKDIR /app

RUN apt-get update && \
	apt-get upgrade -y

ADD requirements.txt /app
ADD server.py /app

ENV FLASK_APP=/app/server.py

RUN mkdir /data && \
	useradd -m lynux -u $UID -g 0 && \
	chown -R lynux:0 /app /data

USER lynux

RUN python -m venv ./venv && \
	/app/venv/bin/python -m pip install -r /app/requirements.txt
