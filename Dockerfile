FROM python:3

ARG UID=1000

WORKDIR /app

RUN apt update && \
	apt upgrade -y && \
	python -m pip install --upgrade pip

ADD requirements.txt /app
ADD update_balance_history.py /app
ADD server.py /app

ENV FLASK_APP=/app/server.py

RUN mkdir /data && \
	useradd -m lynux -u $UID -g 0 && \
	chown -R lynux:0 /app /data

USER lynux

RUN python -m venv ./venv && \
	/app/venv/bin/python -m pip install -r /app/requirements.txt

CMD ["/app/venv/bin/flask", "run", "--host", "0.0.0.0"]
