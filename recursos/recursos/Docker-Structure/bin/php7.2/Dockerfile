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
	curl zip unzip openssl gettext-base libnss3-tools ca-certificates \
    gnupg perl net-tools iproute2 cups

RUN set -eux; \
    EXPECTED_SIGNATURE="$(curl -fsSL https://composer.github.io/installer.sig)"; \
    curl -fsSL -o /tmp/composer-setup.php https://getcomposer.org/installer; \
    ACTUAL_SIGNATURE="$(php -r "echo hash_file('sha384', '/tmp/composer-setup.php');")"; \
    if [ "$EXPECTED_SIGNATURE" != "$ACTUAL_SIGNATURE" ]; then \
        echo 'ERROR: Invalid Composer installer signature' >&2; \
        rm -f /tmp/composer-setup.php; \
        exit 1; \
    fi; \
    if php -r 'exit(PHP_VERSION_ID < 70205 ? 0 : 1);'; then \
        COMPOSER_CHANNEL="--2.2"; \
    else \
        COMPOSER_CHANNEL="--2"; \
    fi; \
    php /tmp/composer-setup.php "$COMPOSER_CHANNEL" --install-dir=/usr/local/bin --filename=composer --quiet; \
    rm -f /tmp/composer-setup.php; \
    composer --version

RUN docker-php-ext-install pdo pdo_mysql mysqli

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

# ====== WSDD-PHP-Admin runtime state ======
RUN mkdir -p /var/lib/wsdd-php-admin/operations /var/log/wsdd-php-admin && \
    chown -R www-data:www-data /var/lib/wsdd-php-admin /var/log/wsdd-php-admin

EXPOSE 10000
EXPOSE 8989

RUN set -eux; \
    rm -f /etc/apt/sources.list.d/webmin.list /etc/apt/sources.list.d/jcameron.list /usr/share/keyrings/webmin.gpg; \
    curl -fsSL -o /tmp/webmin-setup-repo.sh https://raw.githubusercontent.com/webmin/webmin/master/webmin-setup-repo.sh; \
    printf 'y\n' | sh /tmp/webmin-setup-repo.sh; \
    apt-get update; \
    apt-get install -y --install-recommends --fix-missing webmin; \
    rm -f /tmp/webmin-setup-repo.sh

RUN sed -i 's/ssl=1/ssl=0/' /etc/webmin/miniserv.conf && \
    echo "redirect_port=80" >> /etc/webmin/miniserv.conf && \
    USERNAME="$WEBMIN_USER" && PASSWORD="$WEBMIN_PASS" && \
    HASHED_PASSWORD=$(perl -e 'print crypt($ARGV[0], "password")' $PASSWORD) && \
    echo "$USERNAME:$HASHED_PASSWORD:0" >> /etc/webmin/miniserv.users && \
    echo "$USERNAME: *" >> /etc/webmin/webmin.acl && \
    PHP_INI_DIR="/usr/local/etc/php" && \
    echo "php_ini=$PHP_INI_DIR" >> /etc/webmin/phpini/config && \
    echo "referers=wm72.wsdd.dock" >> /etc/webmin/config

# ====== WSDD-PHP-Admin 1.0.0: verified release code only ======
RUN set -eu; \
    for tool in curl sha256sum unzip mktemp find; do command -v "$tool" >/dev/null; done; \
    staging="$(mktemp -d)"; \
    trap 'rm -rf "$staging"' EXIT HUP INT TERM; \
    curl --fail --silent --show-error --location --retry 3 \
        --proto '=https' --proto-redir '=https' --tlsv1.2 \
        --output "$staging/release.zip" \
        'https://github.com/wnunezc/WSDD-PHP-Admin/releases/download/v1.0.0/WSDD-PHP-Admin-1.0.0.zip'; \
    printf '%s  %s\n' \
        'e1b0b153c6a1816a871aa9caab22bbf438673b9706a5292c555ca30dca353054' \
        "$staging/release.zip" | sha256sum --check --strict -; \
    unzip -q "$staging/release.zip" -d "$staging/extracted"; \
    payload="$staging/extracted/WSDD-PHP-Admin-1.0.0"; \
    for required in VERSION bin/wsdd-php-admin config/wsdd-php-admin.php \
        config/extension-catalog.php config/php-compatibility.json \
        public/index.php public/runtime-probe.php public/runtime-health.php \
        src/Support/bootstrap.php src/Runtime/RuntimeStarter.php; do \
        test -s "$payload/$required"; \
    done; \
    test "$(cat "$payload/VERSION")" = '1.0.0'; \
    mkdir -p /opt; \
    test ! -e /opt/wsdd-php-admin; \
    mv "$payload" /opt/wsdd-php-admin; \
    chown -R root:root /opt/wsdd-php-admin; \
    find /opt/wsdd-php-admin -type d -exec chmod 0755 {} +; \
    find /opt/wsdd-php-admin -type f -exec chmod 0644 {} +; \
    chmod 0755 /opt/wsdd-php-admin/bin/wsdd-php-admin; \
    test -x /opt/wsdd-php-admin/bin/wsdd-php-admin; \
    rm -rf "$staging"; \
    trap - EXIT HUP INT TERM

CMD sh -lc '\
  update-ca-certificates >/dev/null 2>&1 || true; \
  if [ -f /etc/cron-task/new-task ]; then \
    cat /etc/cron-task/new-task >> /etc/cron.d/cron-task && chmod 0644 /etc/cron.d/cron-task; \
  fi; \
  touch /var/log/cron/cron.log; \
  mkdir -p /var/lib/wsdd-php-admin/operations /var/log/wsdd-php-admin; \
  chown -R www-data:www-data /var/lib/wsdd-php-admin /var/log/wsdd-php-admin || true; \
  cron; \
  if [ -x /etc/cron-task/crontab-ui.sh ]; then \
    /etc/cron-task/crontab-ui.sh || true; \
  fi; \
  service webmin restart; \
  WSDD_PHP_ADMIN_ROOT=""; \
  if [ -x /var/www/html/phpadmin72.dock/bin/wsdd-php-admin ]; then \
    WSDD_PHP_ADMIN_ROOT="/var/www/html/phpadmin72.dock"; \
  elif [ -x /opt/wsdd-php-admin/bin/wsdd-php-admin ]; then \
    WSDD_PHP_ADMIN_ROOT="/opt/wsdd-php-admin"; \
  fi; \
  if [ -n "$WSDD_PHP_ADMIN_ROOT" ]; then \
    php "$WSDD_PHP_ADMIN_ROOT/bin/wsdd-php-admin" runtime:start >> /var/log/wsdd-php-admin/agent.log 2>&1 & \
  else \
    echo "WSDD-PHP-Admin not available: no executable CLI found at /var/www/html/phpadmin72.dock/bin/wsdd-php-admin or /opt/wsdd-php-admin/bin/wsdd-php-admin" >> /var/log/wsdd-php-admin/agent.log; \
  fi; \
  exec /usr/local/bin/apache2-foreground \
'
