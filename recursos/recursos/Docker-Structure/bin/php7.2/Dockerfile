FROM php:7.2-apache

ARG DEBIAN_FRONTEND=noninteractive
ARG XDEBUG_VERSION="xdebug-2.9.8"
ARG XDEBUG_ENABLED="1"
ARG WEBMIN_USER
ARG WEBMIN_PASS

RUN sed -i 's|http://deb.debian.org/debian|http://archive.debian.org/debian|g' /etc/apt/sources.list && \
    sed -i '/security.debian.org/d' /etc/apt/sources.list && \
    sed -i '/buster-updates/d' /etc/apt/sources.list && \
    echo 'Acquire::Check-Valid-Until "false";' > /etc/apt/apt.conf.d/99no-check-valid-until && \
    echo 'Acquire::AllowInsecureRepositories "true";' >> /etc/apt/apt.conf.d/99no-check-valid-until && \
    echo 'APT::Get::AllowUnauthenticated "true";' >> /etc/apt/apt.conf.d/99no-check-valid-until

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

RUN if [ "$XDEBUG_ENABLED" = "1" ]; then \
        pecl channel-update pecl.php.net \
        && pecl install ${XDEBUG_VERSION} \
        && docker-php-ext-enable xdebug \
        && printf "%s\n" \
            "zend_extension=xdebug.so" \
            "xdebug.remote_enable=1" \
            "xdebug.remote_autostart=0" \
            "xdebug.remote_host=host.docker.internal" \
            "xdebug.remote_port=9003" \
            "xdebug.remote_connect_back=0" \
            "xdebug.default_enable=1" \
            "xdebug.idekey=WSDD" \
            > /usr/local/etc/php/conf.d/docker-php-ext-xdebug.ini; \
    fi

RUN service apache2 restart

RUN mkdir -p /var/log/cron && \
    chmod 755 /var/log/cron

RUN mkdir -p /var/www/html/log && \
    chmod 755 /var/www/html/log

EXPOSE 10000

RUN wget --no-check-certificate -qO - http://www.webmin.com/jcameron-key.asc | apt-key add - && \
    sh -c 'echo "deb http://download.webmin.com/download/repository sarge contrib" > /etc/apt/sources.list.d/webmin.list' && \
    apt-get update && \
    apt-get install -y webmin --no-install-recommends --fix-missing

RUN sed -i 's/ssl=1/ssl=0/' /etc/webmin/miniserv.conf && \
    echo "redirect_port=80" >> /etc/webmin/miniserv.conf && \
    USERNAME="$WEBMIN_USER" && PASSWORD="$WEBMIN_PASS" && \
    HASHED_PASSWORD=$(perl -e 'print crypt($ARGV[0], "password")' $PASSWORD) && \
    echo "$USERNAME:$HASHED_PASSWORD:0" >> /etc/webmin/miniserv.users && \
    echo "$USERNAME: *" >> /etc/webmin/webmin.acl && \
    PHP_INI_DIR="/usr/local/etc/php" && \
    echo "php_ini=$PHP_INI_DIR" >> /etc/webmin/phpini/config && \
    echo "referers=wm72.wsdd.dock" >> /etc/webmin/config

CMD cat /etc/cron-task/new-task >> /etc/cron.d/cron-task && \
    chmod 0644 /etc/cron.d/cron-task && \
    touch /var/log/cron/cron.log && \
    cron && \
    chmod +x /etc/cron-task/crontab-ui.sh && \
    /etc/cron-task/crontab-ui.sh && \
    service webmin restart && \
    /usr/local/bin/apache2-foreground && \
    tail -f /var/log/cron/cron.log
