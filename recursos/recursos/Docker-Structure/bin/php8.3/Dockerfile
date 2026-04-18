# Base
FROM php:8.3-apache

# ====== ENV ======
ENV TZ=UTC
ENV CFLAGS="$CFLAGS -DLDAP_DEPRECATED=1"

# ====== ARG ======
ARG DEBIAN_FRONTEND=noninteractive
ARG XDEBUG_VERSION="xdebug"
ARG XDEBUG_ENABLED="1"
ARG WEBMIN_VERSION="2.630"
ARG WEBMIN_USER
ARG WEBMIN_PASS
ARG WEBMIN_SHA256="0e119b65093442f4ae59a479bf8099fae053000658efea95169710343b4c85c9"

# ====== APT: update/upgrade ======
RUN apt-get update
RUN apt-get upgrade -y --no-install-recommends --fix-missing

# ====== APT: herramientas base (manteniendo tu set) ======
RUN apt-get install -y --no-install-recommends --fix-missing \
    tzdata sed build-essential dialog \
    nano apt-utils cron wget \
    curl git zip openssl \
    gettext-base gnupg libnss3-tools lsb-release \
    perl net-tools iproute2 cups \
    ca-certificates pkg-config

# ====== APT: librerías para extensiones PHP ======
RUN apt-get install -y --no-install-recommends \
    zlib1g-dev libzip-dev libbz2-dev libssl-dev \
    libcurl4-openssl-dev libxml2-dev libxslt1-dev libtidy-dev \
    libsodium-dev libpng-dev libjpeg62-turbo-dev libfreetype6-dev \
    libwebp-dev libavif-dev libicu-dev libgmp-dev \
    libldap2-dev libsasl2-dev libonig-dev

# ====== APT: deps para PECL/phpize (IMPRESCINDIBLE para ftp/xdebug) ======
# $PHPIZE_DEPS lo define la imagen oficial de PHP
RUN apt-get install -y --no-install-recommends $PHPIZE_DEPS

RUN mkdir -p /var/log/cron /var/www/html/log && chmod 755 /var/log/cron /var/www/html/log

# ====== Apache: módulos habituales (puedes ajustar) ======
RUN a2enmod http2 proxy proxy_http rewrite ssl headers proxy_wstunnel

# ====== PHP: configurar extensiones que lo requieren ======
RUN docker-php-ext-configure gd --with-jpeg --with-freetype --with-webp --with-avif
RUN docker-php-ext-configure ldap --with-ldap=/usr --with-ldap-sasl

# ====== PHP: compilar extensiones del core ======
# Nota: NO usar docker-php-ext-enable para estas; ya quedan habilitadas.
RUN docker-php-ext-install -j"$(nproc)" bz2 curl fileinfo gd gettext gmp intl ldap mbstring exif opcache pdo_mysql soap sockets sodium tidy xsl zip

RUN docker-php-ext-install ftp

# ====== PECL: Xdebug ======
RUN if [ "$XDEBUG_ENABLED" = "1" ]; then \
        pecl install ${XDEBUG_VERSION} \
        && docker-php-ext-enable xdebug \
        && printf "%s\n" \
            "zend_extension=xdebug.so" \
            "xdebug.mode=debug,develop" \
            "xdebug.start_with_request=trigger" \
            "xdebug.client_host=host.docker.internal" \
            "xdebug.client_port=9003" \
            "xdebug.discover_client_host=0" \
            "xdebug.idekey=WSDD" \
            > /usr/local/etc/php/conf.d/docker-php-ext-xdebug.ini; \
    fi

RUN echo "ServerName localhost" > /etc/apache2/conf-available/servername.conf && a2enconf servername

# ====== WEBMIN (2.630): repo seguro + instalación ======
# ====== Webmin por .deb verificado (sin repo inseguro) ======
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

# Crear o actualizar usuario/clave Webmin
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
    echo "referers=wm83.wsdd.dock" >> /etc/webmin/config

# ====== Cron: estructura de tareas como en tu original ======
RUN mkdir -p /etc/cron-task
RUN mkdir -p /var/log/cron
# Si tienes archivos reales, cambia el ADD/COPY aquí:
# COPY ./cron/new-task /etc/cron-task/new-task
# COPY ./cron/crontab-ui.sh /etc/cron-task/crontab-ui.sh
# Asignamos permisos defensivos (no falla si no existen aún)
RUN sh -lc 'test -f /etc/cron-task/crontab-ui.sh && chmod +x /etc/cron-task/crontab-ui.sh || true'

# ====== Limpieza APT ======
RUN rm -rf /var/lib/apt/lists/*

# ====== Puertos ======
EXPOSE 10000

# ====== CMD: arranca cron + webmin y deja Apache en foreground ======
CMD sh -lc '\
  if [ -f /etc/cron-task/new-task ]; then \
    cat /etc/cron-task/new-task >> /etc/cron.d/cron-task && chmod 0644 /etc/cron.d/cron-task; \
  fi; \
  touch /var/log/cron/cron.log; \
  service cron start; \
  if [ -x /etc/cron-task/crontab-ui.sh ]; then \
    /etc/cron-task/crontab-ui.sh || true; \
  fi; \
  service webmin start; \
  exec /usr/local/bin/apache2-foreground \
'
