FROM python:3.11-alpine

ARG UID=1000

ENV BADGE_DIV=6891 \
    BADGE_NUMBER=18147 \
    BADGE_NAME="Guillerot Lucas" \
    PYTHONUNBUFFERED=1 \
    PATH="/home/lynux/.local/bin:$PATH"

WORKDIR /app

RUN mkdir /data && \
    adduser -u $UID -G root -D lynux && \
    chown -R lynux:root /app /data

USER lynux

COPY --chown=lynux:root requirements.txt .
RUN pip install --no-cache-dir --user -r requirements.txt

COPY --chown=lynux:root update_balance_history.py .

CMD ["python", "update_balance_history.py"]
