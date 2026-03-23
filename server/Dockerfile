FROM python:3.11-alpine

ARG UID=1000
WORKDIR /app

ENV FLASK_APP=server.py \
    PATH="/home/lynux/.local/bin:$PATH" \
    PYTHONUNBUFFERED=1

RUN mkdir /data && \
    adduser -u $UID -G root -D lynux && \
    chown -R lynux:root /app /data

USER lynux

COPY --chown=lynux:root requirements.txt .

RUN pip install --no-cache-dir --user -r requirements.txt

COPY --chown=lynux:root server.py .

EXPOSE 5000

CMD ["python", "-m", "flask", "run", "--host=0.0.0.0"]
