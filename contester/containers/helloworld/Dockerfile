FROM alpine:latest

# Создаём скрипт, который выводит hello и читает строку из stdin
RUN echo '#!/bin/sh' > /app.sh && \
  echo '# Wait for stdin to be available' >> /app.sh && \
  #echo 'while ! read -t 0; do sleep 0.1; done' >> /app.sh && \
  echo 'echo "READY"' >> /app.sh && \
  echo 'read USER_INPUT' >> /app.sh && \
  echo 'echo User sent: "$USER_INPUT"' >> /app.sh && \
  echo 'read USER_INPUT' >> /app.sh && \
  echo 'echo baunda"$USER_INPUT"' >> /app.sh && \
  echo 'echo a; while true; do echo 2; done' >> /app.sh && \
  chmod +x /app.sh

# Устанавливаем точку входа
#CMD ["/bin/sh", "/app.sh"]

CMD [ "/bin/sh", "-c", "echo READY && cat" ]
