# Credenciales De Desarrollo Y Relaciones Del Stack WSDD

## Contrato Y Responsabilidades

WSDD es una arquitectura LAMP local para desarrollo, entregada AS-IS. Permite
credenciales genericas y configuracion manual por el desarrollador. No es una
promesa de aislamiento, cifrado, rotacion automatica o seguridad de produccion.
Preferir datos sinteticos/no sensibles; cambiar credenciales genericas o
reutilizadas al usar datos confidenciales, compartir acceso o publicar servicios.
Revisar tambien la exposicion de puertos y permisos: cambiar una clave no basta
para garantizar que los datos no puedan ser comprometidos.

WSDD debe evitar publicar las credenciales que conoce en sus diagnosticos. Esta
responsabilidad no cambia como autentican MySQL, Webmin, phpMyAdmin o ngrok.

## Relaciones Y Fuentes De Configuracion

| Herramienta | Relacion / responsabilidad | Credenciales y alcance |
|---|---|---|
| nginx-proxy | HTTP/HTTPS del host hacia Apache/PHP y herramientas por vhost | No tiene una contrasena propia declarada por WSDD; los certificados mkcert no son passwords de aplicaciones |
| Apache/PHP | Un runtime compartido por los proyectos de la misma version PHP | Cada aplicacion configura sus conexiones; WSDD no administra los usuarios internos de esas aplicaciones |
| MySQL | Servicio `database` en `wsdd-network` | `MYSQL_USER` / `MYSQL_PASSWORD` y `MYSQL_ROOT_PASSWORD`, globales al servicio, no por proyecto |
| phpMyAdmin | Cliente de MySQL, conecta a `database:3306` | `PMA_USER=root`; `PMA_PASSWORD` reutiliza `mysql_root_password`; tambien recibe variables MySQL. No hay una clave independiente de phpMyAdmin en el modelo actual |
| Webmin | Administracion dentro de cada contenedor PHP; backend puerto 10000, URL `wmXX.wsdd.dock` | `WEBMIN_USER` / `WEBMIN_PASS`, un par por version PHP. Fallback legacy `admin` / `admin`, no un valor universal impuesto a todos |
| WSDD-PHP-Admin | Panel/agent del runtime PHP correspondiente; proxy a 8989 y estado/log persistentes por version | WSDD prepara infraestructura y llama `runtime:start`. No define otro usuario/password del panel en AppSettings; no asumir que las credenciales Webmin autentican este panel |
| ngrok | Exposicion opcional de servicios; configura su propia cuenta/tuneles | Un `NGROK_AUTHTOKEN` configurado por el desarrollador, no una clave generica generada por WSDD |
| Redis / Memcached / Mailpit | Servicios opcionales de desarrollo | Sus plantillas actuales no declaran otro par de credenciales administrado por WSDD; no asumir proteccion para acceso externo |

Configuracion de instalacion: `C:\WSDD-Environment\wsdd-config.json` y
`wsdd-secrets.json`. Este ultimo almacena valores en JSON, **no es un vault cifrado**.
WSDD renderiza las credenciales en `Docker-Structure\init.yml`, los compose PHP y
`services\ngrok.yml`. Esos archivos son configuracion operativa, no logs redactados.
Cambios manuales en archivos administrados pueden ser reemplazados por Settings,
Deploy o sincronizacion al arrancar. Mantener los valores de WSDD coherentes con
los del servicio; no tratar una edicion del compose generado como una migracion.

## Escenario 1: Primera Configuracion / Base De Datos Vacia

1. Introducir las credenciales MySQL en la configuracion inicial de WSDD; elegir
   datos de prueba y valores apropiados al uso local. No hay una contrasena MySQL
   universal que esta guia deba dar por supuesta.
2. Configurar Webmin por version PHP en Settings o al solicitarlo el dialogo
   Deploy PHP. Si se conservan valores genericos, conocer su alcance y limitaciones.
3. Desplegar mediante las acciones de WSDD y comprobar acceso al servicio real.
4. Configurar cada aplicacion para conectar a MySQL por `database:3306` dentro de
   la red Docker; desde Windows el compose actual publica `127.0.0.1:3306`.
   `localhost` dentro de un contenedor PHP no es el contenedor MySQL.

La inicializacion de la imagen oficial MySQL utiliza las variables del entorno
para una base nueva. No confundirla con cambiar cuentas de una base ya existente.

## Escenario 2: Cambiar Usuario/Clave MySQL Con Datos Existentes

1. Preparar un backup verificable y una sesion administrativa valida antes de
   modificar cuentas. No borrar `bin\mysql\data`, volumenes ni el entorno.
2. Identificar las cuentas reales y sus componentes usuario/host desde MySQL.
   El desarrollador aplica los cambios con las herramientas SQL de su version
   del servidor y sus permisos. No asumir un host de cuenta ni renombrar una
   cuenta sin revisar grants y clientes. El Dockerfile usa `mysql:latest`; verificar
   la version efectiva, no inferirla del nombre del contenedor.
3. Actualizar los valores correspondientes en Settings de WSDD. Guardar regenera
   configuracion, **no ejecuta ALTER USER ni cambia cuentas en los datos persistidos**.
4. Aplicar de forma controlada la configuracion de los clientes afectados. Si cambia
   root, phpMyAdmin tambien necesita el nuevo valor; si cambia el usuario de la
   aplicacion, actualizar su `.env`/configuracion y refrescar su cache de configuracion
   segun su framework. No imprimir esas credenciales en comandos compartidos o logs.
5. Comprobar conexiones desde phpMyAdmin y desde las aplicaciones; preservar una
   via de recuperacion hasta confirmar. Un reinicio no incorpora variables nuevas
   a un contenedor existente; si cambia su environment requiere recreacion controlada.

No existe aqui una nueva mecanica automatica de cambio de claves. El desarrollador
coordina el estado del servidor, sus clientes y los valores que WSDD regenera.

## Escenario 3: phpMyAdmin Tras Cambiar MySQL

No buscar una contrasena independiente del panel en WSDD: la plantilla conecta
como root con la clave MySQL guardada. Alinear el password real de esa cuenta,
`mysql_root_password` y el environment aplicado a phpMyAdmin. Guardar Settings o
reiniciar un contenedor antiguo no garantiza que su environment haya cambiado.
La recuperacion base conserva contenedores existentes; no fuerza su recreacion.
Una autenticacion fallida debe comprobarse en esos tres puntos, no resolverse
eliminando la carpeta de datos MySQL.

## Escenario 4: Webmin Por Version PHP

Cambiar el par de la version afectada en Settings de WSDD y aplicar una
reconstruccion/recreacion controlada del runtime PHP, por ejemplo con Deploy de
un proyecto de esa version. El boton Deploy PHP crea runtimes ausentes: no sustituye
uno existente. Guardar Settings o simplemente Restart no ejecuta el RUN del build.

Actualmente las credenciales Webmin se pasan como build args y se aplican durante
el build. Esta tarea no cambia esa mecanica ni introduce BuildKit secrets.
Si el usuario modifica cuentas directamente en Webmin, debe alinear la configuracion
de WSDD para futuros builds. No asumir que cambiar el nombre elimina cuentas antiguas:
los Dockerfiles no implementan una limpieza/migracion integral de cuentas Webmin.
Reconstruir afecta al runtime compartido por todos los proyectos de esa version;
planificar la interrupcion y comprobar el acceso antes de continuar.

## Escenario 5: Aplicaciones Y WSDD-PHP-Admin

Las claves de login de Moodle, Laravel u otra aplicacion son responsabilidad de
la aplicacion. No se cambian al modificar Webmin ni son necesariamente las del
usuario MySQL administrado por WSDD. Configurar por separado autenticacion de la
aplicacion y conexion a base de datos.

WSDD-PHP-Admin mantiene su contrato runtime/agent dentro del contenedor. No
introducir ni deducir una relacion de claves con Webmin. Para configuracion de
su propio protocolo o autenticacion, seguir la documentacion de ese proyecto.

## Escenario 6: ngrok / Acceso Compartido

Configurar el token propio en Settings; al cambiar el token de un servicio ngrok
habilitado, el codigo solicita aplicar el cambio al servicio. Comprobar la cuenta
y los tuneles efectivos. Si un token se divulgo, el desarrollador lo revoca/reemplaza
en su proveedor y actualiza WSDD. No publicar paneles administrativos o datos
confidenciales por considerar que una URL de desarrollo es privada.

## Diagnosticos: Que Se Suprime Y Que No

El codigo fuente posterior a 2.1.3 suprime campos reconocidos de credenciales y
valores conocidos de MySQL, Webmin y ngrok antes de publicarlos en los canales de
log, archivo de deploy, tracing y logs de contenedores que muestra WSDD. Paneles
principal/loader/contenedores y Copy aplican la misma supresion. Se retienen los
valores anteriores en memoria durante la sesion para cubrir output tardio y copia
de buffers existentes. Valores genericos coincidentes pueden ocultar texto no
secreto; esto no altera datos internos ni la ejecucion de comandos.

Los scripts `dd-setting.ps1` y `wsdd-runtime-recovery.ps1` tambien filtran antes
de escribir sus logs propios. Cargan los valores conocidos de la configuracion
local al iniciar el script; si no pueden interpretarla, suprimen el diagnostico
en lugar de publicar su texto original. El helper se distribuye en el ZIP de
scripts y la reparacion existente de PS-Script lo instala al ejecutar WSDD nuevo.

Los resultados internos de procesos siguen disponibles para interpretar estados
y respuestas. La supresion no modifica contrasenas, SQL, compose operativo ni
argumentos de los procesos. Si el filtro no esta disponible, no publica el texto
original como alternativa. Los fallos de escritura del log se advierten sin
anunciar un archivo completo que no se pudo guardar.

No se reescriben logs antiguos ni logs/metadatos conservados directamente por Docker,
shells, aplicaciones o herramientas externas. No se garantiza detectar secretos
arbitrarios de aplicaciones que WSDD no conoce, representaciones no soportadas ni
datos sensibles que no sean credenciales. Mantener coherentes los valores guardados
en WSDD al cambiar claves manualmente. Revisar antes de compartir diagnosticos y
no adjuntar configuracion operativa, secretos o backups completos como si fueran logs.
Los MSI ya publicados no adquieren esta correccion automaticamente.

La vista operativa por etapas del issue #15 sigue fuera de esta implementacion;
esta guia y la supresion no cambian Deploy/rebuild, rollback ni dependencias.
