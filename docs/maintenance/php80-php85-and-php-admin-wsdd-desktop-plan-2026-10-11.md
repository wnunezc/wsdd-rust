# WSDD Desktop: soporte PHP 8.0, PHP 8.5 y preparacion para WSDD-PHP-Admin

Fecha: 2026-10-11

## Alcance

Este documento describe los cambios requeridos en WSDD Desktop para:

- agregar soporte a PHP 8.0;
- agregar soporte a PHP 8.5;
- dejar preparada la integracion futura de WSDD-PHP-Admin.

No implementa WSDD-PHP-Admin y no abre una release nueva. La integracion real debe quedar para una tarea futura cuando WSDD-PHP-Admin tenga artefacto instalable, contrato de runtime y formato de perfil persistente estables.

Documento relacionado: `docs/maintenance/php-container-admin-requirements-2026-10-11.draft.md`.

## Estado actual observado

WSDD Desktop soporta actualmente estas versiones PHP:

- PHP 5.6
- PHP 7.2
- PHP 7.4
- PHP 8.1
- PHP 8.2
- PHP 8.3
- PHP 8.4

Las versiones viven como variantes explicitas en `src/models/project/php_version.rs` y como recursos por version bajo `recursos/recursos/Docker-Structure/bin/phpX.Y/`.

Cada version PHP tiene:

- directorio `phpX.Y`;
- `Dockerfile`;
- `webserver.phpXX.yml`;
- `options.phpXX.yml`;
- `php.ini`;
- carpeta `vhost/`;
- carpeta `cron/`;
- dominios base `phpXX.wsdd.dock`, `cronXX.wsdd.dock` y `wmXX.wsdd.dock`;
- contenedor compartido `WSDD-Web-Server-PHPX.Y`;
- tag compose `phpXX`;
- credenciales Webmin por version.

## Versiones nuevas

### PHP 8.0

PHP 8.0 debe agregarse como version intermedia entre PHP 7.4 y PHP 8.1.

Convenciones propuestas:

- enum: `Php80`
- directorio: `php8.0`
- compose tag: `php80`
- container tag legacy: `PHP80`
- runtime container tag: `PHP8.0`
- contenedor: `WSDD-Web-Server-PHP8.0`
- compose: `webserver.php80.yml` y `options.php80.yml`
- dominios base: `php80.wsdd.dock`, `cron80.wsdd.dock`, `wm80.wsdd.dock`

### PHP 8.5

PHP 8.5 debe agregarse como version posterior a PHP 8.4.

Convenciones propuestas:

- enum: `Php85`
- directorio: `php8.5`
- compose tag: `php85`
- container tag legacy: `PHP85`
- runtime container tag: `PHP8.5`
- contenedor: `WSDD-Web-Server-PHP8.5`
- compose: `webserver.php85.yml` y `options.php85.yml`
- dominios base: `php85.wsdd.dock`, `cron85.wsdd.dock`, `wm85.wsdd.dock`

## Cambios requeridos en WSDD Desktop para PHP 8.0 y PHP 8.5

### Modelo Rust

Actualizar `src/models/project/php_version.rs`:

- agregar `Php80` y `Php85`;
- incluir ambas versiones en `PhpVersion::all()` en orden cronologico;
- agregar `dir_name()`, `container_tag()`, `runtime_container_tag()`, `compose_tag()`, `display_name()` y `host_suffix()`;
- aceptar `php8.0`, `php80`, `8.0`, `PHP 8.0`, `php8.5`, `php85`, `8.5` y `PHP 8.5` en `FromStr`;
- ampliar tests de parsing, tags, dominios base y deteccion desde nombre de contenedor.

### Recursos Docker

Crear nuevos directorios:

- `recursos/recursos/Docker-Structure/bin/php8.0/`
- `recursos/recursos/Docker-Structure/bin/php8.5/`

Cada directorio debe incluir, como minimo:

- `Dockerfile`;
- `webserver.php80.yml` o `webserver.php85.yml`;
- `options.php80.yml` o `options.php85.yml`;
- `php.ini`;
- `vhost/vhost.conf`;
- `cron/` con la misma estructura funcional que PHP 8.1 a 8.4.

El ZIP embebido `recursos/docker-structure.zip` no debe asumirse sincronizado por modificar `recursos/recursos/`. Cuando se implemente, el pipeline de recursos debe regenerar y validar el ZIP.

### Dockerfile

Para PHP 8.0 y PHP 8.5, el Dockerfile debe alinearse con la familia moderna PHP 8.x de WSDD:

- imagen base `php:8.0-apache` y `php:8.5-apache` cuando exista imagen estable upstream;
- Apache, cron, Xdebug 3 y Webmin desde repositorio APT oficial;
- Composer instalado de forma no interactiva en `/usr/local/bin/composer`;
- canal Composer moderno `--2`;
- `mysqli` habilitado como en la linea local `v2.1.3`;
- soporte de CA mkcert dentro del contenedor;
- `WEBMIN_USER`, `WEBMIN_PASS` y `XDEBUG_ENABLED` como build args;
- `referers=wm80.wsdd.dock` y `referers=wm85.wsdd.dock`;
- ningun prerequisito especifico de WSDD-PHP-Admin hasta que el contrato de instalacion este cerrado.

PHP 8.5 puede requerir una decision de disponibilidad: si `php:8.5-apache` aun no esta estable en Docker Hub al momento de implementar, la version debe quedar bloqueada o marcada como no seleccionable hasta que la imagen exista y pase build.

### Compose

Crear `webserver.php80.yml` y `webserver.php85.yml` con la misma estructura que PHP 8.4:

- `container_name: WSDD-Web-Server-PHP8.0` / `WSDD-Web-Server-PHP8.5`;
- servicio `webserver80` / `webserver85`;
- `build.context: .`;
- `dockerfile: Dockerfile`;
- build args Webmin y Xdebug;
- `expose` para `80`, `9003` y `10000`;
- red externa `wsdd-network`;
- `extra_hosts: host.docker.internal:host-gateway`;
- mount de `../../ssl/wsdd-mkcert-rootCA.crt`;
- labels `wsdd.webserver.description` y `wsdd.webserver.role`.

Crear `options.php80.yml` y `options.php85.yml` con:

- `VIRTUAL_HOST: php80.wsdd.dock,cron80.wsdd.dock,wm80.wsdd.dock`;
- `VIRTUAL_HOST: php85.wsdd.dock,cron85.wsdd.dock,wm85.wsdd.dock`;
- `VIRTUAL_PORT: 80`;
- `CLIENT_MAX_BODY_SIZE: 16m`;
- mounts de `dev`, `vhost`, `php.ini`, `cron`, logs Apache y logs cron.

Las funciones que modifican `VIRTUAL_HOST` deben seguir agregando dominios de proyectos sin duplicar hosts base.

### Vhost base

Crear `vhost/vhost.conf` para PHP 8.0 y PHP 8.5 con los bloques base:

- `php80.wsdd.dock` / `php85.wsdd.dock` hacia `/var/www/html/dev`;
- `cron80.wsdd.dock` / `cron85.wsdd.dock` como proxy a `127.0.0.1:8000`;
- `wm80.wsdd.dock` / `wm85.wsdd.dock` como proxy a `127.0.0.1:10000`;
- marcadores `### DO NOT TOUCH ###` y `### PERSONAL PROJECTS ###` preservados.

### Templates embebidos

Actualizar `src/handlers/docker_deploy/templates.rs`:

- agregar `include_str!` para Dockerfile PHP 8.0 y PHP 8.5;
- agregar `include_str!` para `webserver.php80.yml` y `webserver.php85.yml`;
- extender `dockerfile_template()`;
- extender `webserver_template()`;
- ampliar tests para garantizar que no queden placeholders `__WSDD_`.

### Certificados internos

`src/handlers/mkcert.rs` ya deriva dominios desde `PhpVersion::all()` y `base_container_domains()`. Al agregar `Php80` y `Php85`, debe generar certificados para:

- `php80.wsdd.dock`;
- `cron80.wsdd.dock`;
- `wm80.wsdd.dock`;
- `php85.wsdd.dock`;
- `cron85.wsdd.dock`;
- `wm85.wsdd.dock`.

Actualizar tests de conteo y cobertura de dominios.

### UI y configuracion

Como la UI recorre `PhpVersion::all()`, las nuevas versiones deben aparecer automaticamente en:

- alta de proyectos;
- ajustes de credenciales Webmin por version;
- sincronizacion de recursos Docker/Webmin.

Aun asi, se debe revisar visualmente que la lista no se corte ni desborde en Settings y Add Project.

### Storage, deploy, backup y recovery

Los flujos actuales usan `compose_tag()`, `dir_name()` y `runtime_container_tag()`. Debe validarse que PHP 8.0 y PHP 8.5 funcionen en:

- Classic Bind;
- Native Linux Project;
- Managed Native Linux;
- Fast Sync Workspace si sigue habilitado;
- deploy con rebuild `--build`;
- remove;
- backup/restore;
- runtime storage metadata;
- WSL runtime recovery;
- exclusion por version PHP `php-runtime:PHPx.y`.

### Documentacion de usuario

Cuando el soporte se implemente, actualizar:

- `README.md`;
- `docs/readme/README.es.md`;
- `docs/readme/README.fr.md`;
- `docs/readme/README.hi.md`;
- `docs/readme/README.zh.md`;
- `docs/help/user-guide.*.md`;
- notas de release solo si el usuario autoriza preparar release.

## Decision de subdominio para WSDD-PHP-Admin

Subdominio elegido por version:

```text
phpadmin{suffix}.wsdd.dock
```

Ejemplos:

- `phpadmin80.wsdd.dock`
- `phpadmin84.wsdd.dock`
- `phpadmin85.wsdd.dock`

Motivos:

- deja claro que es administracion PHP, no phpMyAdmin;
- evita colision con `pma.wsdd.dock`;
- mantiene el mismo sufijo numerico que `phpXX`, `cronXX` y `wmXX`;
- permite que cada contenedor PHP tenga su propio panel y perfil;
- no consume un dominio global que pueda confundirse con el estado de una unica version.

No se recomienda `admin.wsdd.dock` porque WSDD-PHP-Admin opera dentro de un contenedor PHP concreto. Tampoco se recomienda `pmaXX.wsdd.dock`, porque se parece demasiado a phpMyAdmin.

## Preparacion futura para WSDD-PHP-Admin

Esta seccion es solo planificacion. No debe implementarse en WSDD Desktop hasta que WSDD-PHP-Admin este listo.

### Dockerfile futuro

Cuando exista artefacto instalable, cada Dockerfile PHP compatible debera:

- instalar prerequisitos minimos de WSDD-PHP-Admin;
- copiar o descargar el artefacto versionado;
- instalarlo en una ruta propia, por ejemplo `/opt/wsdd-php-admin`;
- registrar un proceso de arranque o supervisor si la app escucha en puerto propio;
- no hardcodear extensiones administradas por el usuario;
- dejar la configuracion deseada al perfil persistente.

La instalacion debe ser reproducible durante build. El estado mutable no debe quedar dentro de la imagen.

### Puerto 8989

Puerto interno reservado:

```text
8989
```

Modelo recomendado:

- WSDD-PHP-Admin escucha dentro del contenedor PHP en `127.0.0.1:8989`;
- Apache expone el panel mediante un vhost interno;
- nginx-proxy sigue entrando por `VIRTUAL_PORT: 80`;
- no se publica `8989` directamente al host;
- opcionalmente se agrega `8989` a `expose` solo como documentacion de red interna.

Este modelo mantiene el mismo patron que Webmin y cron: el reverse proxy entra al Apache del contenedor y Apache enruta al servicio local.

### Compose futuro

Cuando se active WSDD-PHP-Admin, los `webserver.phpXX.yml` podrian agregar:

```yaml
expose:
  - "8989" # WSDD-PHP-Admin internal app
```

Los `options.phpXX.yml` deberan agregar:

```yaml
environment:
  VIRTUAL_HOST: phpXX.wsdd.dock,cronXX.wsdd.dock,wmXX.wsdd.dock,phpadminXX.wsdd.dock
  VIRTUAL_PORT: 80
volumes:
  - ../../state/php-admin/phpXX:/var/lib/wsdd-php-admin
```

La ruta host es conceptual. La implementacion debe elegir una ubicacion persistente que WSDD no sobrescriba durante reconciliacion de recursos, reextraccion o actualizacion.

### Vhost futuro

Cada `vhost/vhost.conf` debera agregar un bloque base dentro de la zona `DO NOT TOUCH`:

```apache
<VirtualHost *:80>
    ServerName phpadminXX.wsdd.dock
    Protocols h2 h2c http/1.1
    ProxyPass / http://127.0.0.1:8989/
    ProxyPassReverse / http://127.0.0.1:8989/
    ProxyPreserveHost On
    ErrorLog /var/www/html/log/phpadminXX.wsdd.dock_error.log
    CustomLog /var/www/html/log/phpadminXX.wsdd.dock_access.log combined
</VirtualHost>
```

No debe vivir en la zona `PERSONAL PROJECTS`, porque es un endpoint interno administrado por WSDD.

### VIRTUAL_HOST futuro

Agregar `phpadminXX.wsdd.dock` a `VIRTUAL_HOST` base de cada version con WSDD-PHP-Admin habilitado.

Las funciones de `src/handlers/yml.rs` que agregan/remueven dominios de proyectos deben preservar este host base y no tratarlo como dominio de usuario.

### Certificados y hosts futuro

Al agregar `phpadminXX.wsdd.dock`, WSDD debe:

- incluirlo en `PhpVersion::base_container_domains()` o en una funcion nueva de dominios internos por version;
- generar certificado mkcert interno;
- reconocerlo como endpoint interno en `mkcert::is_internal_endpoint_domain()`;
- asegurar entrada en hosts si el flujo de hosts internos se amplia mas alla de `pma`/`mysql`.

Si se decide no modificar `base_container_domains()` porque hoy representa exactamente tres dominios, crear una funcion separada como `admin_container_domain()` y actualizar tests de conteo de certificados.

### Persistencia futuro

El perfil persistente de WSDD-PHP-Admin debe quedar fuera del filesystem efimero del contenedor.

Recomendacion inicial:

```text
C:\WSDD-Environment\Docker-Structure\state\php-admin\phpXX\
```

Montado dentro del contenedor como:

```text
/var/lib/wsdd-php-admin
```

Contenido esperado:

- perfil declarativo;
- logs de auditoria;
- cache o metadata de reconciliacion;
- ultimo estado funcional conocido;
- archivos `.ini` administrados por WSDD-PHP-Admin si aplica.

Esta ruta no debe ser eliminada por:

- sync de Dockerfile;
- render de compose;
- reextraccion parcial de recursos;
- deploy/remove de proyectos;
- rebuild de contenedor;
- actualizacion de WSDD Desktop.

### Configuracion en WSDD Desktop futuro

WSDD Desktop debera decidir si WSDD-PHP-Admin queda:

- habilitado globalmente para todas las versiones PHP nuevas;
- habilitado por version PHP;
- habilitado solo para PHP 8.x moderno inicialmente.

Para reducir riesgo, la primera integracion deberia ser opt-in y por version, empezando por PHP 8.4.

### Seguridad futuro

WSDD-PHP-Admin tendra capacidad de modificar extensiones y configuracion PHP. WSDD Desktop debe exigir que la integracion incluya:

- autenticacion propia;
- secretos persistentes fuera de la imagen;
- bloqueo de comandos arbitrarios;
- CSRF;
- logs de auditoria;
- acceso solo por dominios `*.wsdd.dock`;
- no publicar puerto host directo para `8989`.

## Orden recomendado de implementacion

1. Agregar PHP 8.0 como version completa sin WSDD-PHP-Admin.
2. Agregar PHP 8.5 solo cuando la imagen upstream estable exista y compile.
3. Validar deploy/remove/rebuild/storage/recovery para PHP 8.0 y PHP 8.5.
4. Actualizar documentacion de usuario de versiones PHP soportadas.
5. Esperar artefacto estable de WSDD-PHP-Admin.
6. Integrar WSDD-PHP-Admin primero en PHP 8.4 con subdominio `phpadmin84.wsdd.dock`.
7. Extender a PHP 8.0, 8.1, 8.2, 8.3 y 8.5 cuando el contrato funcione.
8. Evaluar PHP 5.6, 7.2 y 7.4 como compatibilidad legacy separada.

## Criterios de aceptacion para PHP 8.0 y PHP 8.5

- La UI permite seleccionar PHP 8.0 y PHP 8.5.
- WSDD genera certificados internos para sus dominios base.
- WSDD crea contenedores `WSDD-Web-Server-PHP8.0` y `WSDD-Web-Server-PHP8.5`.
- `phpXX.wsdd.dock`, `cronXX.wsdd.dock` y `wmXX.wsdd.dock` responden.
- Webmin solicita/usa credenciales por version.
- Composer esta disponible en `PATH`.
- `mysqli` esta cargado.
- Xdebug respeta el setting global.
- Deploy reconstruye la imagen con `--build`.
- Managed Native Linux mantiene permisos `www-data`.
- Remove limpia vhost, volumen y `VIRTUAL_HOST` sin tocar otros hosts base.
- Tests Rust de versionado, templates, yml y mkcert pasan.

## Criterios de aceptacion para la integracion futura de WSDD-PHP-Admin

- `phpadmin84.wsdd.dock` responde sobre HTTPS local cuando este habilitado.
- El panel entra por Apache/nginx-proxy, no por puerto host directo.
- El puerto interno `8989` no se publica al host.
- El perfil persiste tras rebuild y recreate.
- El perfil no se borra por sync/reconciliacion de recursos WSDD.
- `VIRTUAL_HOST` conserva `phpadminXX.wsdd.dock` al agregar/remover proyectos.
- El vhost de PHP-Admin no se mezcla con `PERSONAL PROJECTS`.
- Los logs y auditoria quedan fuera del filesystem efimero.
- La integracion puede apagarse sin romper `phpXX`, `cronXX` ni `wmXX`.

## Notas abiertas

- Confirmar disponibilidad real y estabilidad de `php:8.5-apache` al momento de implementacion.
- Definir si PHP 8.0 se considera soporte activo o compatibilidad legacy.
- Definir nombre final del artefacto de WSDD-PHP-Admin.
- Definir formato final del perfil persistente.
- Definir si WSDD-PHP-Admin se instala en todas las versiones o solo en versiones seleccionadas.
- Definir estrategia para PHP 5.6, 7.2 y 7.4, porque sus toolchains pueden requerir tratamiento legacy.
