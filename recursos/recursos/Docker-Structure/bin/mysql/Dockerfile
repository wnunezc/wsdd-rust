FROM mysql:latest

RUN mkdir -p /var/lib/mysql /var/log/mysql \
    && chown -R mysql:mysql /var/lib/mysql /var/log/mysql \
    && chmod -R 0755 /var/lib/mysql /var/log/mysql /etc/mysql

RUN echo '[client]' >> /etc/mysql/my.cnf
RUN echo 'local_infile=1' >> /etc/mysql/my.cnf

RUN echo '[mysqld]' >> /etc/mysql/my.cnf
RUN echo 'authentication_policy=caching_sha2_password' >> /etc/mysql/my.cnf
RUN echo 'local_infile=1' >> /etc/mysql/my.cnf
RUN echo 'sql_mode="STRICT_TRANS_TABLES,ERROR_FOR_DIVISION_BY_ZERO,ALLOW_INVALID_DATES"' >> /etc/mysql/my.cnf

RUN echo 'innodb_change_buffering=all' >> /etc/mysql/my.cnf
RUN echo 'bulk_insert_buffer_size=4G' >> /etc/mysql/my.cnf
RUN echo 'innodb_log_buffer_size=256M' >> /etc/mysql/my.cnf
RUN echo 'innodb_buffer_pool_size=4G' >> /etc/mysql/my.cnf
RUN echo 'innodb_change_buffer_max_size=25' >> /etc/mysql/my.cnf
RUN echo 'innodb_flush_log_at_trx_commit=1' >> /etc/mysql/my.cnf
RUN echo 'innodb_doublewrite=off' >> /etc/mysql/my.cnf

RUN echo 'information_schema_stats_expiry=0' >> /etc/mysql/my.cnf

RUN cd /var/log/mysql
RUN ln -s /var/log/mysql/mysql-log-slow-queries.log
RUN ln -s /var/log/mysql/mysql-error.log