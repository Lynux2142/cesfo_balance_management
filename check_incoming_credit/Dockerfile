FROM python:3

ARG UID=1000
ENV BADGE_DIV=6891
ENV BADGE_NUMBER=18147
ENV BADGE_NAME="Guillerot Lucas"

WORKDIR /app

RUN apt-get update && \
	apt-get upgrade -y

ADD requirements.txt /app
ADD check_incoming_credit.py /app

RUN mkdir /data && \
	useradd -m lynux -u $UID -g 0 && \
	chown -R lynux:0 /app /data

USER lynux

RUN python -m venv ./venv && \
	/app/venv/bin/python -m pip install -r /app/requirements.txt

CMD ["/app/venv/bin/python", "check_incoming_credit.py"]
