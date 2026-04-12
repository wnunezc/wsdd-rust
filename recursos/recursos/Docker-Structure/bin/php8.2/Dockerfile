FROM php:8.2-apache

ARG DEBIAN_FRONTEND=noninteractive
ARG XDEBUG_VERSION="xdebug"
ARG WEBMIN_VERSION="2.630"
ARG WEBMIN_USER
ARG WEBMIN_PASS
ARG WEBMIN_SHA256="0e119b65093442f4ae59a479bf8099fae053000658efea95169710343b4c85c9"

RUN apt-get update &&  \
    apt-get upgrade -y --no-install-recommends --fix-missing

RUN apt-get install -y --no-install-recommends --fix-missing \
    tzdata sed build-essential dialog \
	nano apt-utils cron wget git \
	curl zip openssl gettext-base libnss3-tools \
    gnupg perl net-tools iproute2 cups

RUN docker-php-ext-install pdo pdo_mysql

RUN apt-get -y autoremove && \
    apt-get clean

RUN a2enmod http2
RUN a2enmod proxy
RUN a2enmod proxy_http
RUN a2enmod rewrite 
RUN a2enmod ssl 
RUN a2enmod headers 
RUN a2enmod proxy_wstunnel

RUN pecl channel-update pecl.php.net \
    && pecl install ${XDEBUG_VERSION} \
    && docker-php-ext-enable xdebug

RUN echo "zend_extension=xdebug.so" > /usr/local/etc/php/conf.d/docker-php-ext-xdebug.ini \
    && echo "xdebug.mode=debug" >> /usr/local/etc/php/conf.d/docker-php-ext-xdebug.ini \
    && echo "xdebug.start_with_request=yes" >> /usr/local/etc/php/conf.d/docker-php-ext-xdebug.ini \
    && echo "xdebug.client_host=host.docker.internal" >> /usr/local/etc/php/conf.d/docker-php-ext-xdebug.ini \
    && echo "xdebug.client_port=9003" >> /usr/local/etc/php/conf.d/docker-php-ext-xdebug.ini

RUN service apache2 restart

RUN mkdir -p /var/log/cron && \
    chmod 755 /var/log/cron

RUN mkdir -p /var/www/html/log && \
    chmod 755 /var/www/html/log

EXPOSE 10000

RUN rm -f /etc/apt/sources.list.d/webmin.list
RUN rm -f /usr/share/keyrings/webmin.gpg
RUN apt-get update
RUN curl -fsSL -o /tmp/webmin_${WEBMIN_VERSION}_all.deb \
    https://www.webmin.com/download/deb/webmin_${WEBMIN_VERSION}_all.deb
RUN echo "${WEBMIN_SHA256}  /tmp/webmin_${WEBMIN_VERSION}_all.deb" > /tmp/webmin.sha256
RUN sha256sum -c /tmp/webmin.sha256
RUN apt-get update
RUN apt-get install -y --no-install-recommends /tmp/webmin_${WEBMIN_VERSION}_all.deb
RUN rm -f /tmp/webmin_${WEBMIN_VERSION}_all.deb /tmp/webmin.sha256

RUN sed -i 's/ssl=1/ssl=0/' /etc/webmin/miniserv.conf && \
    if ! grep -q '^redirect_port=' /etc/webmin/miniserv.conf; then echo "redirect_port=80" >> /etc/webmin/miniserv.conf; fi && \
    USERNAME="$WEBMIN_USER" && PASSWORD="$WEBMIN_PASS" && \
    if grep -q "^${USERNAME}:" /etc/webmin/miniserv.users; then \
        /usr/share/webmin/changepass.pl /etc/webmin "$USERNAME" "$PASSWORD"; \
    else \
        HASHED_PASSWORD=$(perl -e 'print crypt($ARGV[0], "password")' "$PASSWORD") && \
        echo "$USERNAME:$HASHED_PASSWORD:0" >> /etc/webmin/miniserv.users && \
        echo "$USERNAME: *" >> /etc/webmin/webmin.acl; \
    fi && \
    PHP_INI_DIR="/usr/local/etc/php" && \
    echo "php_ini=$PHP_INI_DIR" > /etc/webmin/phpini/config && \
    echo "referers=wm82.wsdd.dock" >> /etc/webmin/config
    
CMD cat /etc/cron-task/new-task >> /etc/cron.d/cron-task && \
    chmod 0644 /etc/cron.d/cron-task && \
    touch /var/log/cron/cron.log && \
    cron && \
    chmod +x /etc/cron-task/crontab-ui.sh && \
    /etc/cron-task/crontab-ui.sh && \
    service webmin restart && \
    /usr/local/bin/apache2-foreground && \
    tail -f /var/log/cron/cron.log
