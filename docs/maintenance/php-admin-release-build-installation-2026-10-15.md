# Instalacion de WSDD-PHP-Admin 1.0.0 durante build

Actualizacion posterior: la sonda que este informe identifica como faltante ya
esta implementada en fuente WSDD. Ver php-admin-direct-probe-infrastructure-2026-10-15.md
para el alcance actual y sus verificaciones. El analisis original siguiente es
historico; la validacion manual end-to-end sigue pendiente.

## Alcance y archivos

Cambio de recursos para PHP 5.6, 7.2, 7.4, 8.1, 8.2, 8.3 y 8.4; no agrega versiones ni certifica compatibilidad del runtime.

Dockerfiles modificados, relativos a la raiz WSDD:

| PHP | Archivo |
| --- | --- |
| 5.6 | `recursos/recursos/Docker-Structure/bin/php5.6/Dockerfile` |
| 7.2 | `recursos/recursos/Docker-Structure/bin/php7.2/Dockerfile` |
| 7.4 | `recursos/recursos/Docker-Structure/bin/php7.4/Dockerfile` |
| 8.1 | `recursos/recursos/Docker-Structure/bin/php8.1/Dockerfile` |
| 8.2 | `recursos/recursos/Docker-Structure/bin/php8.2/Dockerfile` |
| 8.3 | `recursos/recursos/Docker-Structure/bin/php8.3/Dockerfile` |
| 8.4 | `recursos/recursos/Docker-Structure/bin/php8.4/Dockerfile` |

Tambien se actualizan las siete entradas de `recursos/docker-structure.zip`, este informe y el mapa documental. La continuidad se registra en los MD canonicos externos.

Webmin se instala mediante RUN: descarga su configurador oficial, prepara APT, instala el paquete y configura sus archivos antes del CMD. PHP 5.6 conserva su variante con snapshot y reintentos. PHP-Admin sigue el mismo reparto build/arranque: RUN independiente despues de la configuracion de Webmin; el CMD existente conserva la seleccion Dev/Release y runtime:start. No se modifican los bloques Webmin, sus repositorios ni sus credenciales.

## Herramientas y bloque comun

Los siete Dockerfiles ya instalan curl y ca-certificates. Ninguno declaraba unzip; se agrega explicitamente a su lista APT base, junto a zip. sha256sum y mktemp pertenecen a coreutils y find a findutils en las bases Debian. El RUN verifica la disponibilidad de sus herramientas antes de descargar. No se ejecutaron comprobaciones dentro de imagenes; su disponibilidad efectiva queda sujeta al build manual.

El bloque siguiente es identico en las siete versiones. No ejecuta PHP, Composer, scripts del paquete, runtime:start, reconciliacion ni servicios.

```dockerfile
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
```

Solo se instala codigo verificado, propiedad root:root; directorios 0755, archivos 0644, CLI 0755. No otorga escritura del codigo a www-data. La instalacion rechaza un destino preexistente en vez de introducir otra carpeta o sobrescribirlo. Los temporales se limpian mediante trap, tambien si falla la descarga, checksum o extraccion.

Los mounts y preparacion existentes de /var/lib/wsdd-php-admin y /var/log/wsdd-php-admin permanecen intactos. No se ejecuta ninguna inicializacion del perfil durante build.

## Revision estatica realizada

- Paquete oficial descargado solamente a TEMP para inspeccion; SHA256 coincide con e1b0b153c6a1816a871aa9caab22bbf438673b9706a5292c555ca30dca353054.
- Estructura real: WSDD-PHP-Admin-1.0.0/; VERSION contiene 1.0.0; existen CLI, public, configuracion y bootstrap. El ZIP no incluye deploy/container/install.sh.
- Se compara SHA256 del texto completo de cada CMD antes/despues: siete coincidencias exactas.
- Los siete bloques de instalacion coinciden; sh -n verifica sintaxis sin ejecutarlos; diff --check sin errores.
- ZIP de recursos: 85 entradas, solo siete Dockerfiles actualizados, hashes iguales a fuente y otras entradas intactas. No se incluye el paquete ni el ZIP de PHP-Admin.
- No se construyeron imagenes, ejecutables ni MSI; no se iniciaron o reconstruyeron contenedores ni se ejecutaron baterias funcionales.

## Integracion HTTP: impedimento Release pendiente

Estado observado en los siete directorios PHP:

1. Panel: bin/phpX.Y/vhost/vhost.conf contiene phpadminXX.wsdd.dock con ProxyPass y ProxyPassReverse hacia http://127.0.0.1:8989/. Existe tambien la generacion del mismo bloque en src/handlers/yml.rs::ensure_php_admin_vhost_conf.
2. Sonda Release: no existe un vhost directo o alias que sirva /opt/wsdd-php-admin/public/runtime-probe.php con Apache mod_php. El vhost del panel envia toda ruta a 8989, por lo que no sustituye esa sonda.
3. Entorno: ni los compose webserver/options ni los recursos de WSDD establecen WSDD_PHP_ADMIN_APACHE_PROBE_HOST.

El codigo publicado src/Runtime/ApacheRuntimeProbe.php toma esa variable o basename de la raiz. En Release el basename es wsdd-php-admin, sin punto, y no supera la validacion del Host de la sonda. La sonda solicita http://127.0.0.1/runtime-probe.php y verifica que app_root corresponda a la raiz elegida y sapi sea apache2handler.

Falta configurar, por version, un vhost directo protegido para loopback con DocumentRoot /opt/wsdd-php-admin/public, y su Host en el entorno del contenedor. El paquete incluye deploy/apache/wsdd-php-admin.probe.example.conf con Host phpadmin-runtime.internal como ejemplo. Ese Host se usa en el header de una peticion a loopback; no requiere publicarlo en VIRTUAL_HOST ni agregarlo a hosts de Windows.

La ruta directa NO debe pasar por el proxy 8989. La sonda debe corresponder a la misma raiz que selecciona el CMD: un vhost fijo de Release no debe presentarse como solucion Dev cuando el CMD selecciona un checkout montado.

En este alcance solo se informa el impedimento: no se modifican vhost, compose, entorno ni CMD. Instalar el ZIP no demuestra que runtime:start o Release funcionen.

## Distribucion y prueba manual sin bind Dev

1. La app usa include_str! de los Dockerfiles en src/handlers/docker_deploy/templates.rs; sync_php_version_resources_sync los escribe en el entorno instalado. El ZIP inicial usa src/resources/mod.rs. No hace falta un nuevo instalador de recursos en Rust.
2. El MSI anterior no contiene estos nuevos RUN. Para probar mediante Deploy de WSDD se necesita un ejecutable/MSI generado despues de estos cambios. No se genero en esta tarea de revision estatica. Alternativamente, usar los Dockerfiles actualizados en un entorno de prueba aislado, evitando que un WSDD anterior los sobrescriba.
3. En ese entorno, usar un proyecto normal de la version elegida sin montar su checkout Dev: /var/www/html/phpadmin.dock para PHP 8.3; /var/www/html/phpadminXX.dock para las demas. Conservar los mounts persistentes; no borrar datos existentes para probar.
4. Reconstruir la imagen. Esperado: descarga HTTPS oficial, checksum OK antes de unzip, presencia de /opt/wsdd-php-admin/bin/wsdd-php-admin ejecutable y VERSION 1.0.0, sin /opt/wsdd-php-admin/WSDD-PHP-Admin-1.0.0/. Si cualquier paso falla, el build debe fallar.
5. Inspeccionar el codigo y permisos en la imagen de prueba: propietario root, lectura/recorrido para www-data, CLI ejecutable. No debe haber perfiles, credenciales, claves, operaciones ni logs Dev dentro de la raiz de codigo.
6. Al arrancar sin Dev, el CMD debe elegir /opt/wsdd-php-admin y llamar runtime:start como antes. Inspeccionar agent.log y el panel; no aceptar la existencia del codigo como readiness.
7. La prueba HTTP Release queda bloqueada por la integracion de sonda descrita arriba. Una vez implementada, verificar solicitud directa de loopback con Host, clave y nonce validos: app_root=/opt/wsdd-php-admin y sapi=apache2handler. Verificar aparte el panel mediante 8989 y la reutilizacion/inicializacion del perfil por la aplicacion.

Release sigue pendiente de prueba manual del usuario.

