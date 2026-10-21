FROM mysql:8.4.11

COPY primary.cnf replica.cnf /etc/wsdd-mysql-test/
COPY wsdd-mysql-test /usr/local/bin/wsdd-mysql-test
RUN chmod 0755 /usr/local/bin/wsdd-mysql-test

ENTRYPOINT ["/usr/local/bin/wsdd-mysql-test"]
CMD ["run"]
