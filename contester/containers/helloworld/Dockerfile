FROM alpine:latest

# Создаём скрипт, который выводит hello и читает строку из stdin
RUN echo '#!/bin/sh' > /app.sh && \
  echo 'echo "hello "' >> /app.sh && \
  echo 'read input' >> /app.sh && \
  echo 'echo "$input"' >> /app.sh && \
  echo 'while true; do echo 1; done' >> /app.sh && \
  chmod +x /app.sh

# Устанавливаем точку входа
ENTRYPOINT ["/app.sh"]
