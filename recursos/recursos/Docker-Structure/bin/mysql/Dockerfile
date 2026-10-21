FROM mysql:26.7.0@sha256:625f74ddf93652d9d0f8d4ea7d741b9189554a5e96d27bb770abd0902d89c6e3

COPY primary.cnf replica.cnf /etc/wsdd-mysql/
COPY wsdd-mysql /usr/local/bin/wsdd-mysql
RUN chmod 0755 /usr/local/bin/wsdd-mysql

ENTRYPOINT ["/usr/local/bin/wsdd-mysql"]
CMD ["run"]
