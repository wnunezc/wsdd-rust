# WSDD Help

<!-- Canonical source for the in-app help rendered by src/ui/helps.rs. -->

## Requisitos del sistema

Sistema operativo:
  - Windows 10 (build 2004+) o Windows 11
  - Arquitectura x64 (AMD64)
  - 8 GB RAM minimo — 16 GB recomendado para stacks PHP
  - 20 GB espacio libre en disco

Privilegios:
  - WSDD debe ejecutarse como Administrador (UAC requerido)
  - Necesario para modificar C:\Windows\System32\drivers\etc\hosts

Software instalado automaticamente por WSDD:
  - Chocolatey — gestor de paquetes Windows
  - PowerShell 7.5+ (pwsh.exe) — automatizacion compatible
  - mkcert — generacion de certificados SSL locales

Software que debe estar instalado previamente:
  - Docker Desktop — motor de contenedores principal
  - WSL 2 — subsistema Linux requerido por Docker Desktop

## Instalacion y primer arranque

1. Ejecutar wsdd.exe como Administrador (clic derecho → Ejecutar como administrador).

2. Al primer arranque aparece el Welcome Wizard:
   - Leer y marcar la casilla de requisitos.
   - Hacer clic en 'Siguiente' para iniciar la verificacion del entorno.

3. Antes del primer despliegue del entorno base, WSDD abre un dialogo para
   capturar las credenciales de MySQL y phpMyAdmin.
   - Estas credenciales se guardan en wsdd-secrets.json.
   - Solo se solicitan cuando todavia no existen en la configuracion.

4. El Loader verifica e instala automaticamente:
   - Chocolatey (gestor de paquetes)
   - PowerShell 7.5+
   - mkcert (SSL local)
   - Configuracion de red y certificado raiz

5. Si Docker Desktop no estaba instalado:
   - El Loader se detiene y pide instalarlo manualmente.
   - Tras instalar Docker Desktop, volver a abrir WSDD.

6. Al completar el Loader, aparece el Panel Principal.

Ubicacion de datos de WSDD:
  C:\WSDD-Environment\
  ├── PS-Script\         Scripts de automatizacion PowerShell
  ├── Docker-Structure\  Configuracion de contenedores, proyectos y SSL
  └── wsdd-config.json  Configuracion de la aplicacion

## Panel principal — vision general

El panel principal tiene tres areas:

1. BARRA DE MENU (superior):
   - Archivo → Agregar Proyecto, Salir
   - Docker → Actualizar lista, Recargar Docker Desktop, Limpiar logs
   - Herramientas → Configuracion WSL, Configuracion
   - Ayuda → Ayuda, Acerca de...

2. TOOLBAR (debajo del menu):
   - ⬡ phpMyAdmin   — Abre phpMyAdmin en el navegador (http://pma.wsdd.dock)
   - ⚡ Terminal PS  — Abre PowerShell 7 en C:\WSDD-Environment
   - ⬛ Terminal CMD — Abre CMD en C:\WSDD-Environment
   - + Agregar      — Formulario de nuevo proyecto
   - ↺ Refrescar    — Actualiza contenedores y proyectos
   - Selector de tema (derecha) — Cambia entre Dark Neutral, Dark Blue, Dark Warm, Light

3. TABS CENTRALES:
   - Contenedores   — Lista de contenedores Docker activos/inactivos
   - Proyectos      — Lista de proyectos WSDD registrados

4. PANEL DE LOG (inferior):
   - Muestra el historial de operaciones con color por nivel.
   - Tiene mas altura util y puede redimensionarse para ver mas contenido.
   - Boton 'Copiar' — copia el log al portapapeles.
   - Boton 'Limpiar' — borra el log visible.

5. BARRA DE ESTADO (inferior):
   - Muestra contenedores detectados, contenedores activos, proyectos y logs.
   - Muestra estado de Docker y consumo visible de CPU/RAM.

## Gestion de contenedores Docker

La tab 'Contenedores' muestra todos los contenedores de WSDD con su estado actual.

COLUMNAS:
  - Nombre       — Nombre del contenedor Docker
  - Estado       — Running / Exited / ...
  - Imagen       — Imagen Docker base
  - Toolbox (⚙)  — Acciones avanzadas del contenedor

ACCIONES POR CONTENEDOR (columnas de accion):
  - ▶ Start   — Iniciar el contenedor
  - ■ Stop    — Detener el contenedor
  - ↺ Restart — Reiniciar el contenedor

TOOLBOX DE CONTENEDOR (boton ⚙):
  - Ver logs del contenedor en tiempo real
  - Abrir terminal TTY interactivo dentro del contenedor
  - Ver URLs y puertos expuestos
  - Informacion detallada (imagen, puertos, volumenes)

POLLING AUTOMATICO:
  Los contenedores se actualizan automaticamente cada 3 segundos.
  No es necesario refrescar manualmente (aunque el boton ↺ existe para forzarlo).

PRESENTACION:
  - La vista principal deja mas margen visual alrededor de la cuadricula.
  - El objetivo es mejorar legibilidad y evitar texto pegado al borde.

NOTA: Solo se muestran contenedores del stack WSDD (prefijo 'wsdd-' o relacionados).
Para ver todos los contenedores Docker del sistema, usar Docker Desktop o la CLI.

## Gestion de proyectos

La tab 'Proyectos' muestra los proyectos web registrados en WSDD.

COLUMNAS:
  - Nombre       — Nombre del proyecto
  - Dominio      — Dominio local (ej: miapp.wsdd.dock)
  - PHP          — Version de PHP asignada (5.6 — 8.4)
  - Estado       — Deployed / Not Deployed
  - Acciones     — Deploy, Remove, Toolbox

ACCIONES:
  - ⬆ Deploy    — Despliega el proyecto (crea contenedores, SSL, hosts)
  - ⬇ Remove    — Elimina el despliegue (NO borra los archivos de codigo)
  - ⚙ Toolbox   — Acciones avanzadas del proyecto

TOOLBOX DE PROYECTO:
  - Abrir carpeta del proyecto en el Explorador de Windows
  - Abrir el proyecto en el navegador
  - Ver informacion detallada (rutas, dominio, entrypoint)

ELIMINAR PROYECTO:
  - Al hacer clic en Remove, se solicita confirmacion.
  - Se eliminan: contenedores, SSL, entrada en hosts, registro JSON.
  - Los archivos de codigo fuente NO se eliminan.

## Agregar un proyecto

Para agregar un nuevo proyecto web:

1. Hacer clic en '+ Agregar' en el toolbar o ir a Archivo → Agregar Proyecto.

2. Completar el formulario:

   Nombre:
     Identificador del proyecto (sin espacios, solo letras/numeros/guiones).
     Ejemplo: mi-proyecto

   Dominio:
     Subdominio local. Se agrega automaticamente el sufijo '.wsdd.dock'.
     Ejemplo: escribir 'miapp' → dominio final: miapp.wsdd.dock

   Version PHP:
     Seleccionar entre PHP 5.6, 7.2, 7.4, 8.1, 8.2, 8.3, 8.4.

   Ruta de trabajo:
     Directorio raiz del proyecto en el disco local.
     Usar el boton 'Seleccionar...' para elegir la carpeta.
     Este directorio se monta como volumen en el contenedor Docker.

   Entry Point:
     Archivo principal de la aplicacion.
     Opciones: index.php, index.html, index.htm, custom (especificar).

   SSL:
     Checkbox para generar certificado SSL local con mkcert.
     Recomendado: activo. Genera HTTPS automaticamente.

3. Hacer clic en 'Deploy' para crear el proyecto.
   - Si esa version de PHP todavia no tiene credenciales de Webmin guardadas
     y el contenedor aun no existe, WSDD abrira un dialogo para capturarlas.
   - Esas credenciales se guardan por version PHP y se reutilizan en futuros
     proyectos de la misma version.

4. El proceso de deploy:
   a) Guarda el proyecto en JSON
   b) Crea el volumen Docker
   c) Configura options.yml para la version PHP
   d) Sincroniza los recursos Docker/Webmin de esa version PHP
   e) Para y elimina el contenedor PHP anterior
   f) Reconstruye el contenedor PHP con el nuevo proyecto
   g) Genera el vhost.conf del servidor web
   h) Genera el certificado SSL (si aplica)
   i) Actualiza C:\Windows\System32\drivers\etc\hosts

## Deploy y Remove — flujo detallado

FLUJO DE DEPLOY:
  1. handlers::project::save()    → guarda {nombre}.json en Docker-Structure/projects/
  2. docker volume create         → crea volumen con el codigo del proyecto
  3. handlers::yml::add_project_to_options_yml()
                                  → registra el proyecto en options.php{XX}.yml
  4. Sincroniza Dockerfile + webserver.yml + credenciales de Webmin para esa version PHP
  5. docker stop + docker rm      → detiene y elimina el contenedor PHP anterior
  6. docker create --build + up -d → reconstruye e inicia el contenedor PHP
  7. Genera vhost.conf            → configura el virtual host del servidor web
  8. mkcert + SSL                 → genera certificado si SSL esta activo
  9. Proxy restart                → reinicia el proxy para aplicar vhost
  10. handlers::hosts::update_host() → agrega el dominio a /etc/hosts (Windows hosts)

FLUJO DE REMOVE:
  1. handlers::yml::remove_project_from_options_yml()
                                  → elimina el proyecto del options.yml
  2. Elimina el bloque vhost.conf del proyecto
  3. Reconstruye el contenedor PHP sin el proyecto
  4. handlers::hosts::remove_domains() → elimina el dominio del archivo hosts
  5. Elimina los archivos SSL del proyecto en Docker-Structure/ssl
  6. docker volume rm             → elimina el volumen del proyecto
  7. handlers::project::delete()  → borra el JSON del proyecto

IMPORTANTE:
  - Remove NO elimina el codigo fuente del proyecto.
  - Remove SI revierte las entradas de hosts y los SSL gestionados por WSDD.
  - El dominio puede tardar unos segundos en dejar de resolver tras el Remove.

## Configuracion de WSDD (Settings)

Acceder via: Herramientas → Configuracion

GENERAL:
  - Ruta de proyectos    — Directorio base para nuevos proyectos (default: C:\WSDD-Projects)
  - Docker Desktop path  — Ruta al ejecutable de Docker Desktop (opcional, para relanzarlo)
  - WSL Distro           — Distribucion WSL2 activa (ej: Ubuntu-22.04)
  - Max lineas en log    — Limite de lineas conservadas en el panel de log (100-10000)
  - Auto-iniciar contenedores — Inicia los contenedores WSDD al abrir la aplicacion

PHP (contenedores Docker):
  Estos valores se aplican al GENERAR nuevos contenedores.
  No afectan contenedores ya existentes (necesitan redeploy).
  - memory_limit              — Limite de RAM para PHP (ej: 512M)
  - upload_max_filesize       — Tamano maximo de archivos subidos (ej: 256M)
  - Timezone                  — Zona horaria PHP (ej: America/Mexico_City, UTC)
  - Xdebug                    — Activo por defecto en contenedores PHP nuevos
    o reconstruidos. PHP 8.x usa Xdebug 3 con modo debug,develop,
    host.docker.internal, puerto 9003 e inicio por trigger. PHP 5.6/7.x usa el
    equivalente de trigger de Xdebug 2 en el mismo host/puerto.

Debug con IDE / agentes:
  - Configurar VS Code, PHPStorm u otro listener DBGp en el puerto 9003.
  - Mapear la ruta Windows del proyecto a /var/www/html/{dominio-del-proyecto}.
  - Las IA/agentes tambien pueden escuchar si ejecutan un listener compatible
    con DBGp/Xdebug en el host Windows; WSDD solo configura el contenedor PHP
    para conectarse de vuelta.

SERVICIOS OPCIONALES:
  Redis, Memcached y Mailpit estan desactivados por defecto y no se despliegan con el
  stack base. Activa el servicio en Settings, revisa puertos/autoarranque y
  guarda para desplegarlo.
  - Redis: host de contenedor redis / WSDD-Redis-Server, puerto interno 6379,
    puerto host por defecto 6379, volumen persistente wsdd-redis-data.
  - Memcached: host de contenedor memcached / WSDD-Memcached-Server, puerto
    interno 11211, puerto host por defecto 11211, cache volatil.
  - Mailpit: host SMTP mailpit / WSDD-Mailpit-Server, puerto SMTP interno 1025,
    UI en puerto 8025, UI local por defecto http://mailpit.wsdd.dock.
  - Ejemplos para frameworks:
    Redis: REDIS_HOST=redis, REDIS_PORT=6379.
    Memcached: MEMCACHED_HOST=memcached, MEMCACHED_PORT=11211.
    Mailpit: MAIL_HOST=mailpit, MAIL_PORT=1025, MAIL_MAILER=smtp.

PREREQUISITOS:
  - Credenciales de MySQL/phpMyAdmin — se solicitan antes del primer deploy
    del entorno base si todavia no existen en la configuracion.
  - Se guardan en wsdd-secrets.json y se reutilizan en siguientes arranques.

HERRAMIENTAS:
  - Version de Webmin — Version instalada en los contenedores PHP (ej: 2.630)
  - Credenciales de Webmin por version PHP — se solicitan solo la primera vez
    que se despliega una version cuyo contenedor aun no existe.
  - Cambiarlas despues no rota automaticamente el usuario ya existente dentro
    del contenedor; aplican al siguiente rebuild administrado por WSDD.

Los ajustes se guardan en: C:\WSDD-Environment\wsdd-config.json
Los secretos se guardan en: C:\WSDD-Environment\wsdd-secrets.json

## Configuracion de WSL2 (WSL Settings)

Acceder via: Herramientas → Configuracion WSL

Modifica: %USERPROFILE%\.wslconfig

RECURSOS DEL SISTEMA:
  - Nucleos de CPU    — Limitar los nucleos asignados a WSL2.
                        'Sin limite' usa todos los disponibles.
                        Recomendado: 50-70% de los nucleos fisicos.

  - RAM maxima        — Limitar la RAM asignada a WSL2.
                        'Sin limite' permite a WSL2 consumir hasta toda la RAM del sistema.
                        Recomendado: 4-8 GB para stacks WSDD tipicos.

  - Swap              — Espacio de intercambio virtual.
                        Con RAM suficiente, 0 (deshabilitado) es lo optimo.

RENDIMIENTO Y MEMORIA:
  - Recuperacion de memoria — Como WSL2 devuelve RAM libre al host Windows:
    - Deshabilitado: WSL2 retiene la RAM indefinidamente (mas rapido pero consume mas)
    - Gradual: libera memoria poco a poco (balance recomendado)
    - Drop Cache: libera agresivamente al terminar procesos (mas RAM libre en host)

  - Aplicaciones GUI (WSLg) — Soporte para apps Linux con interfaz grafica.
                              Desactivar si no se usa mejora el rendimiento.

RED:
  - Localhost forwarding — Acceder a puertos de WSL2 via 127.0.0.1 en Windows.
                          WSDD requiere esto activo para acceder a los servicios.
  - Modo de red:
    - NAT (recomendado) — Red virtual aislada. Maxima compatibilidad.
    - Mirrored          — Comparte la red del host. Experimental. Solo Win11 23H2+.

NOTA IMPORTANTE:
  Los cambios en .wslconfig requieren reiniciar WSL2:
  Abrir PowerShell como Admin y ejecutar: wsl --shutdown
  Luego volver a abrir Docker Desktop.

## Certificados SSL y HTTPS

WSDD usa mkcert para generar certificados SSL de confianza local.

COMO FUNCIONA:
  1. mkcert crea una Autoridad Certificadora (CA) local en tu sistema.
  2. La CA se instala como confiable en Windows Certificate Store.
  3. Por cada proyecto con SSL activo, se genera un certificado firmado por la CA.
  4. El proxy reverso usa los certificados para servir HTTPS.

DOMINIOS:
  Todos los proyectos usan el sufijo .wsdd.dock
  Ejemplo: miapp.wsdd.dock → https://miapp.wsdd.dock

RENOVAR UN CERTIFICADO:
  Hacer Remove del proyecto y luego Deploy nuevamente.
  El certificado se regenera automaticamente.

CONFIANZA EN EL NAVEGADOR:
  Si el navegador no confia en los certificados:
  1. Verificar que mkcert esta instalado: mkcert -version (en PowerShell)
  2. Reinstalar la CA: mkcert -install
  3. Reiniciar el navegador.

UBICACION DE CERTIFICADOS:
  C:\WSDD-Environment\Docker-Structure\ssl\
  ├── {dominio}.crt  — Certificado
  └── {dominio}.key  — Clave privada

phpMyAdmin y SSL MySQL:
  HTTPS hacia phpMyAdmin protege el trafico navegador → phpMyAdmin. No significa
  que la conexion interna phpMyAdmin → MySQL use TLS de MySQL.
  WSDD no fuerza TLS MySQL por defecto porque frameworks y ORMs existentes
  pueden requerir rutas de CA, ssl-mode y certificados. Tratar TLS MySQL como
  hardening opcional por proyecto, no como default del stack local.

## Troubleshooting

PROBLEMA: Los contenedores no aparecen en la lista.
  Solucion:
  - Verificar que Docker Desktop esta corriendo (icono en la bandeja del sistema).
  - Hacer clic en ↺ Refrescar o ir a Docker → Actualizar lista de contenedores.
  - Revisar el panel de log por mensajes de error de conexion.

PROBLEMA: 'Docker no encontrado' al iniciar WSDD.
  Solucion:
  - Instalar Docker Desktop manualmente desde docker.com
  - O dejar que el Loader lo instale (requiere Chocolatey).
  - Tras la instalacion, puede requerirse reiniciar Windows.

PROBLEMA: El dominio .wsdd.dock no resuelve en el navegador.
  Solucion:
  - Verificar que el proyecto esta en estado 'Deployed'.
  - Revisar C:\Windows\System32\drivers\etc\hosts — debe tener una linea con el dominio.
  - Si no esta: hacer Remove y Deploy nuevamente.
  - Verificar que WSDD se ejecuto como Administrador.

PROBLEMA: HTTPS muestra error de certificado.
  Solucion:
  - Ejecutar en PowerShell: mkcert -install
  - Reiniciar el navegador completamente.
  - Si persiste: Remove + Deploy del proyecto.

PROBLEMA: El deploy falla con error de Docker.
  Solucion:
  - Revisar el log del panel inferior para el error especifico.
  - Verificar que Docker Desktop esta en estado 'Running' (no 'Starting').
  - Probar Docker → Recargar Docker Desktop desde el menu.
  - Reiniciar Docker Desktop manualmente y esperar que cargue completamente.

PROBLEMA: PowerShell da error de codificacion (bytes invalidos).
  Solucion:
  - WSDD usa -NoProfile -NonInteractive en todos los comandos PS.
  - Si ves errores de encoding en el log, verificar que pwsh.exe es PS7+:
    pwsh --version  (debe ser 7.x)

PROBLEMA: Chocolatey no se instala.
  Solucion:
  - Verificar politica de ejecucion de PowerShell:
    Get-ExecutionPolicy  (debe ser RemoteSigned o Bypass)
  - Ejecutar como Admin: Set-ExecutionPolicy RemoteSigned -Scope LocalMachine
  - Reintentar el Loader (cerrar y abrir WSDD).

PROBLEMA: El log muestra 'Access denied' al modificar hosts.
  Solucion:
  - WSDD debe ejecutarse como Administrador.
  - Cerrar WSDD y ejecutar wsdd.exe con clic derecho → Ejecutar como administrador.

## Preguntas frecuentes (FAQ)

P: ¿Puedo tener multiples versiones de PHP al mismo tiempo?
R: Si. Cada proyecto tiene su propia version de PHP y su propio contenedor.
   Puedes tener simultaneamente PHP 5.6, 8.1 y 8.4 en proyectos distintos.

P: ¿WSDD modifica mis archivos de codigo fuente?
R: No. WSDD solo monta tu directorio como volumen en Docker (read-write).
   Los archivos no se copian ni se modifican internamente.

P: ¿Que pasa si borro un proyecto desde el Explorador de Windows?
R: El contenedor Docker y el registro de WSDD siguen existiendo.
   Hacer 'Remove' desde WSDD primero para limpiar correctamente,
   luego borrar el directorio.

P: ¿Puedo usar WSDD con proyectos Laravel / Symfony / WordPress?
R: Si, con cualquier framework PHP. Configurar el Entry Point correctamente:
   - Laravel/Symfony: public/index.php
   - WordPress: index.php
   - Custom: especificar la ruta relativa al directorio de trabajo.

P: ¿Como actualizo los scripts de automatizacion?
R: Los scripts PS1 estan embebidos en el binario de WSDD.
   Para actualizar, se requiere recompilar desde el codigo fuente.

P: ¿WSDD funciona con WSL 1?
R: No. Docker Desktop requiere WSL 2. WSDD asume WSL 2.

P: ¿Puedo cambiar el dominio de un proyecto existente?
R: Actualmente no hay edicion in-place. Hacer Remove y agregar el proyecto
   nuevamente con el nuevo dominio.

P: ¿Donde se guardan los datos de los proyectos?
R: C:\WSDD-Environment\Docker-Structure\projects\{nombre}.json

P: ¿Como hago backup de mis proyectos WSDD?
R: Respaldar el directorio completo de tu codigo fuente (la ruta de trabajo)
   y el archivo JSON de C:\WSDD-Environment\Docker-Structure\projects\
   El JSON contiene la configuracion del proyecto (dominio, PHP, SSL, etc.)

## Referencia de archivos y rutas

```text
C:\WSDD-Environment\
├── wsdd-config.json              Configuracion de la aplicacion WSDD
├── PS-Script\                   Scripts PowerShell de automatizacion
│   ├── dd-detector.ps1           Deteccion de entorno y dependencias
│   ├── dd-isinstalled.ps1        Verifica si una herramienta esta instalada
│   ├── dd-isrunning.ps1          Verifica si Docker esta activo
│   ├── dd-issettingup.ps1        Verifica estado de configuracion
│   ├── dd-setting.ps1            Lectura/escritura de configuracion
│   ├── dd-start.ps1              Inicio de servicios WSDD
│   ├── dd-stop.ps1               Detencion de servicios WSDD
│   ├── dd-fixmysqlpermission.ps1 Reparacion de permisos MySQL
│   └── wsl-shutdown.ps1          Reinicio de WSL2
├── Docker-Structure\
│   ├── bin\
│   │   └── php{X.X}\
│   │       └── options.php{XX}.yml   Configuracion de contenedores PHP
│   ├── ssl\
│   │   ├── {dominio}.crt            Certificado SSL del dominio
│   │   └── {dominio}.key            Clave privada SSL
│   └── projects\
│       └── {nombre}.json            Datos de cada proyecto registrado
```

Otras rutas gestionadas:

- `%USERPROFILE%\.wslconfig` — Configuracion de recursos WSL2.
- `C:\Windows\System32\drivers\etc\hosts` — Archivo de hosts modificado por WSDD al hacer deploy.

Logs de WSDD:

Los logs viven en memoria durante la sesion. Usa el boton 'Copiar' para guardarlos antes de cerrar WSDD.



