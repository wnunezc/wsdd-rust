// WebStack Deployer for Docker
// Copyright (c) 2026 Walter Nunez / Icaros Net S.A
// All Rights Reserved.
//
// This software is provided for development use only.
// Unauthorized commercial use is prohibited.
//
// Redistribution and modification allowed only through
// the official GitHub repository.
//
// This software is provided AS IS, without warranty of any kind.
// The author shall not be liable for any damages.
//
// Contact: wnunez@lh-2.net
//! Pantalla de ayuda y manual de usuario de WSDD.
//! Equivalente a `Forms/Helps.cs` — con manual completo, secciones colapsables y buscador.

use crate::app::WsddApp;
use crate::i18n::{tr, Language};
use crate::ui::ActiveView;

// ── Estructura del manual ──────────────────────────────────────────────────────

struct Section {
    title: &'static str,
    content: &'static str,
}

fn help_sections(language: Language) -> &'static [Section] {
    match language {
        Language::Es => SECTIONS_ES,
        Language::Fr => SECTIONS_FR,
        Language::Hi => SECTIONS_EN, // Fallback to English
        Language::Zh => SECTIONS_ZH,
        _ => SECTIONS_EN,
    }
}

const SECTIONS_ES: &[Section] = &[
    Section {
        title: "Requisitos del sistema",
        content: "\
Sistema operativo:
  • Windows 10 (build 2004+) o Windows 11
  • Arquitectura x64 (AMD64)
  • 8 GB RAM minimo — 16 GB recomendado para stacks PHP
  • 20 GB espacio libre en disco

Privilegios:
  • WSDD debe ejecutarse como Administrador (UAC requerido)
  • Necesario para modificar C:\\Windows\\System32\\drivers\\etc\\hosts

Software instalado automaticamente por WSDD:
  • Chocolatey — gestor de paquetes Windows
  • PowerShell 7.5+ (pwsh.exe) — automatizacion compatible
  • mkcert — generacion de certificados SSL locales

Software que debe estar instalado previamente:
  • Docker Desktop — motor de contenedores principal
  • WSL 2 — subsistema Linux requerido por Docker Desktop
",
    },
    Section {
        title: "Instalacion y primer arranque",
        content: "\
1. Ejecutar wsdd.exe como Administrador (clic derecho → Ejecutar como administrador).

2. Al primer arranque aparece el Welcome Wizard:
   • Leer y marcar la casilla de requisitos.
   • Hacer clic en 'Siguiente' para iniciar la verificacion del entorno.

3. Antes del primer despliegue del entorno base, WSDD abre un dialogo para
   capturar las credenciales de MySQL y phpMyAdmin.
   • Estas credenciales se guardan en wsdd-secrets.json.
   • Solo se solicitan cuando todavia no existen en la configuracion.

4. El Loader verifica e instala automaticamente:
   • Chocolatey (gestor de paquetes)
   • PowerShell 7.5+
   • mkcert (SSL local)
   • Configuracion de red y certificado raiz

5. Si Docker Desktop no estaba instalado:
   • El Loader se detiene y pide instalarlo manualmente.
   • Tras instalar Docker Desktop, volver a abrir WSDD.

6. Al completar el Loader, aparece el Panel Principal.

Ubicacion de datos de WSDD:
  C:\\WSDD-Environment\\
  ├── PS-Script\\         Scripts de automatizacion PowerShell
  ├── Docker-Structure\\  Configuracion de contenedores, proyectos y SSL
  └── wsdd-config.json  Configuracion de la aplicacion
",
    },
    Section {
        title: "Panel principal — vision general",
        content: "\
El panel principal tiene tres areas:

1. BARRA DE MENU (superior):
   • Archivo → Agregar Proyecto, Salir
   • Docker → Actualizar lista, Recargar Docker Desktop, Limpiar logs
   • Herramientas → Configuracion WSL, Configuracion
   • Ayuda → Ayuda, Acerca de...

2. TOOLBAR (debajo del menu):
   • ⬡ phpMyAdmin   — Abre phpMyAdmin en el navegador (http://pma.wsdd.dock)
   • ⚡ Terminal PS  — Abre PowerShell 7 en C:\\WSDD-Environment
   • ⬛ Terminal CMD — Abre CMD en C:\\WSDD-Environment
   • + Agregar      — Formulario de nuevo proyecto
   • ↺ Refrescar    — Actualiza contenedores y proyectos
   • Selector de tema (derecha) — Cambia entre Dark Neutral, Dark Blue, Dark Warm, Light

3. TABS CENTRALES:
   • Contenedores   — Lista de contenedores Docker activos/inactivos
   • Proyectos      — Lista de proyectos WSDD registrados

4. PANEL DE LOG (inferior):
   • Muestra el historial de operaciones con color por nivel.
   • Tiene mas altura util y puede redimensionarse para ver mas contenido.
   • Boton 'Copiar' — copia el log al portapapeles.
   • Boton 'Limpiar' — borra el log visible.

5. BARRA DE ESTADO (inferior):
   • Muestra contenedores detectados, contenedores activos, proyectos y logs.
   • Muestra estado de Docker y consumo visible de CPU/RAM.
",
    },
    Section {
        title: "Gestion de contenedores Docker",
        content: "\
La tab 'Contenedores' muestra todos los contenedores de WSDD con su estado actual.

COLUMNAS:
  • Nombre       — Nombre del contenedor Docker
  • Estado       — Running / Exited / ...
  • Imagen       — Imagen Docker base
  • Toolbox (⚙)  — Acciones avanzadas del contenedor

ACCIONES POR CONTENEDOR (columnas de accion):
  • ▶ Start   — Iniciar el contenedor
  • ■ Stop    — Detener el contenedor
  • ↺ Restart — Reiniciar el contenedor

TOOLBOX DE CONTENEDOR (boton ⚙):
  • Ver logs del contenedor en tiempo real
  • Abrir terminal TTY interactivo dentro del contenedor
  • Ver URLs y puertos expuestos
  • Informacion detallada (imagen, puertos, volumenes)

POLLING AUTOMATICO:
  Los contenedores se actualizan automaticamente cada 3 segundos.
  No es necesario refrescar manualmente (aunque el boton ↺ existe para forzarlo).

PRESENTACION:
  • La vista principal deja mas margen visual alrededor de la cuadricula.
  • El objetivo es mejorar legibilidad y evitar texto pegado al borde.

NOTA: Solo se muestran contenedores del stack WSDD (prefijo 'wsdd-' o relacionados).
Para ver todos los contenedores Docker del sistema, usar Docker Desktop o la CLI.
",
    },
    Section {
        title: "Gestion de proyectos",
        content: "\
La tab 'Proyectos' muestra los proyectos web registrados en WSDD.

COLUMNAS:
  • Nombre       — Nombre del proyecto
  • Dominio      — Dominio local (ej: miapp.wsdd.dock)
  • PHP          — Version de PHP asignada (5.6 — 8.4)
  • Estado       — Deployed / Not Deployed
  • Acciones     — Deploy, Remove, Toolbox

ACCIONES:
  • ⬆ Deploy    — Despliega el proyecto (crea contenedores, SSL, hosts)
  • ⬇ Remove    — Elimina el despliegue (NO borra los archivos de codigo)
  • ⚙ Toolbox   — Acciones avanzadas del proyecto

TOOLBOX DE PROYECTO:
  • Abrir carpeta del proyecto en el Explorador de Windows
  • Abrir el proyecto en el navegador
  • Ver informacion detallada (rutas, dominio, entrypoint)

ELIMINAR PROYECTO:
  • Al hacer clic en Remove, se solicita confirmacion.
  • Se eliminan: contenedores, SSL, entrada en hosts, registro JSON.
  • Los archivos de codigo fuente NO se eliminan.
",
    },
    Section {
        title: "Agregar un proyecto",
        content: "\
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
   • Si esa version de PHP todavia no tiene credenciales de Webmin guardadas
     y el contenedor aun no existe, WSDD abrira un dialogo para capturarlas.
   • Esas credenciales se guardan por version PHP y se reutilizan en futuros
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
   i) Actualiza C:\\Windows\\System32\\drivers\\etc\\hosts
",
    },
    Section {
        title: "Deploy y Remove — flujo detallado",
        content: "\
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
  • Remove NO elimina el codigo fuente del proyecto.
  • Remove SI revierte las entradas de hosts y los SSL gestionados por WSDD.
  • El dominio puede tardar unos segundos en dejar de resolver tras el Remove.
",
    },
    Section {
        title: "Configuracion de WSDD (Settings)",
        content: "\
Acceder via: Herramientas → Configuracion

GENERAL:
  • Ruta de proyectos    — Directorio base para nuevos proyectos (default: C:\\WSDD-Projects)
  • Docker Desktop path  — Ruta al ejecutable de Docker Desktop (opcional, para relanzarlo)
  • WSL Distro           — Distribucion WSL2 activa (ej: Ubuntu-22.04)
  • Max lineas en log    — Limite de lineas conservadas en el panel de log (100-10000)
  • Auto-iniciar contenedores — Inicia los contenedores WSDD al abrir la aplicacion

PHP (contenedores Docker):
  Estos valores se aplican al GENERAR nuevos contenedores.
  No afectan contenedores ya existentes (necesitan redeploy).
  • memory_limit              — Limite de RAM para PHP (ej: 512M)
  • upload_max_filesize       — Tamano maximo de archivos subidos (ej: 256M)
  • Timezone                  — Zona horaria PHP (ej: America/Mexico_City, UTC)

PREREQUISITOS:
  • Credenciales de MySQL/phpMyAdmin — se solicitan antes del primer deploy
    del entorno base si todavia no existen en la configuracion.
  • Se guardan en wsdd-secrets.json y se reutilizan en siguientes arranques.

HERRAMIENTAS:
  • Version de Webmin — Version instalada en los contenedores PHP (ej: 2.630)
  • Credenciales de Webmin por version PHP — se solicitan solo la primera vez
    que se despliega una version cuyo contenedor aun no existe.
  • Cambiarlas despues no rota automaticamente el usuario ya existente dentro
    del contenedor; aplican al siguiente rebuild administrado por WSDD.

Los ajustes se guardan en: C:\\WSDD-Environment\\wsdd-config.json
Los secretos se guardan en: C:\\WSDD-Environment\\wsdd-secrets.json
",
    },
    Section {
        title: "Configuracion de WSL2 (WSL Settings)",
        content: "\
Acceder via: Herramientas → Configuracion WSL

Modifica: %USERPROFILE%\\.wslconfig

RECURSOS DEL SISTEMA:
  • Nucleos de CPU    — Limitar los nucleos asignados a WSL2.
                        'Sin limite' usa todos los disponibles.
                        Recomendado: 50-70% de los nucleos fisicos.

  • RAM maxima        — Limitar la RAM asignada a WSL2.
                        'Sin limite' permite a WSL2 consumir hasta toda la RAM del sistema.
                        Recomendado: 4-8 GB para stacks WSDD tipicos.

  • Swap              — Espacio de intercambio virtual.
                        Con RAM suficiente, 0 (deshabilitado) es lo optimo.

RENDIMIENTO Y MEMORIA:
  • Recuperacion de memoria — Como WSL2 devuelve RAM libre al host Windows:
    - Deshabilitado: WSL2 retiene la RAM indefinidamente (mas rapido pero consume mas)
    - Gradual: libera memoria poco a poco (balance recomendado)
    - Drop Cache: libera agresivamente al terminar procesos (mas RAM libre en host)

  • Aplicaciones GUI (WSLg) — Soporte para apps Linux con interfaz grafica.
                              Desactivar si no se usa mejora el rendimiento.

RED:
  • Localhost forwarding — Acceder a puertos de WSL2 via 127.0.0.1 en Windows.
                          WSDD requiere esto activo para acceder a los servicios.
  • Modo de red:
    - NAT (recomendado) — Red virtual aislada. Maxima compatibilidad.
    - Mirrored          — Comparte la red del host. Experimental. Solo Win11 23H2+.

NOTA IMPORTANTE:
  Los cambios en .wslconfig requieren reiniciar WSL2:
  Abrir PowerShell como Admin y ejecutar: wsl --shutdown
  Luego volver a abrir Docker Desktop.
",
    },
    Section {
        title: "Certificados SSL y HTTPS",
        content: "\
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
  C:\\WSDD-Environment\\Docker-Structure\\ssl\\
  ├── {dominio}.crt  — Certificado
  └── {dominio}.key  — Clave privada
",
    },
    Section {
        title: "Troubleshooting",
        content: "\
PROBLEMA: Los contenedores no aparecen en la lista.
  Solucion:
  • Verificar que Docker Desktop esta corriendo (icono en la bandeja del sistema).
  • Hacer clic en ↺ Refrescar o ir a Docker → Actualizar lista de contenedores.
  • Revisar el panel de log por mensajes de error de conexion.

PROBLEMA: 'Docker no encontrado' al iniciar WSDD.
  Solucion:
  • Instalar Docker Desktop manualmente desde docker.com
  • O dejar que el Loader lo instale (requiere Chocolatey).
  • Tras la instalacion, puede requerirse reiniciar Windows.

PROBLEMA: El dominio .wsdd.dock no resuelve en el navegador.
  Solucion:
  • Verificar que el proyecto esta en estado 'Deployed'.
  • Revisar C:\\Windows\\System32\\drivers\\etc\\hosts — debe tener una linea con el dominio.
  • Si no esta: hacer Remove y Deploy nuevamente.
  • Verificar que WSDD se ejecuto como Administrador.

PROBLEMA: HTTPS muestra error de certificado.
  Solucion:
  • Ejecutar en PowerShell: mkcert -install
  • Reiniciar el navegador completamente.
  • Si persiste: Remove + Deploy del proyecto.

PROBLEMA: El deploy falla con error de Docker.
  Solucion:
  • Revisar el log del panel inferior para el error especifico.
  • Verificar que Docker Desktop esta en estado 'Running' (no 'Starting').
  • Probar Docker → Recargar Docker Desktop desde el menu.
  • Reiniciar Docker Desktop manualmente y esperar que cargue completamente.

PROBLEMA: PowerShell da error de codificacion (bytes invalidos).
  Solucion:
  • WSDD usa -NoProfile -NonInteractive en todos los comandos PS.
  • Si ves errores de encoding en el log, verificar que pwsh.exe es PS7+:
    pwsh --version  (debe ser 7.x)

PROBLEMA: Chocolatey no se instala.
  Solucion:
  • Verificar politica de ejecucion de PowerShell:
    Get-ExecutionPolicy  (debe ser RemoteSigned o Bypass)
  • Ejecutar como Admin: Set-ExecutionPolicy RemoteSigned -Scope LocalMachine
  • Reintentar el Loader (cerrar y abrir WSDD).

PROBLEMA: El log muestra 'Access denied' al modificar hosts.
  Solucion:
  • WSDD debe ejecutarse como Administrador.
  • Cerrar WSDD y ejecutar wsdd.exe con clic derecho → Ejecutar como administrador.
",
    },
    Section {
        title: "Preguntas frecuentes (FAQ)",
        content: "\
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
R: C:\\WSDD-Environment\\Docker-Structure\\projects\\{nombre}.json

P: ¿Como hago backup de mis proyectos WSDD?
R: Respaldar el directorio completo de tu codigo fuente (la ruta de trabajo)
   y el archivo JSON de C:\\WSDD-Environment\\Docker-Structure\\projects\\
   El JSON contiene la configuracion del proyecto (dominio, PHP, SSL, etc.)
",
    },
    Section {
        title: "Referencia de archivos y rutas",
        content: "\
C:\\WSDD-Environment\\
├── wsdd-config.json              Configuracion de la aplicacion WSDD
├── PS-Script\\                   Scripts PowerShell de automatizacion
│   ├── dd-detector.ps1           Deteccion de entorno y dependencias
│   ├── dd-isinstalled.ps1        Verifica si una herramienta esta instalada
│   ├── dd-isrunning.ps1          Verifica si Docker esta activo
│   ├── dd-issettingup.ps1        Verifica estado de configuracion
│   ├── dd-setting.ps1            Lectura/escritura de configuracion
│   ├── dd-start.ps1              Inicio de servicios WSDD
│   ├── dd-stop.ps1               Detencion de servicios WSDD
│   ├── dd-fixmysqlpermission.ps1 Reparacion de permisos MySQL
│   └── wsl-shutdown.ps1          Reinicio de WSL2
├── Docker-Structure\\
│   ├── bin\\
│   │   └── php{X.X}\\
│   │       └── options.php{XX}.yml   Configuracion de contenedores PHP
│   ├── ssl\\
│   │   ├── {dominio}.crt            Certificado SSL del dominio
│   │   └── {dominio}.key            Clave privada SSL
│   └── projects\\
│       └── {nombre}.json            Datos de cada proyecto registrado

%USERPROFILE%\\.wslconfig         Configuracion de recursos WSL2

C:\\Windows\\System32\\drivers\\etc\\hosts
                                  Archivo de hosts (modificado por WSDD al hacer deploy)

Logs de WSDD:
  Los logs son en-memoria durante la sesion.
  Usar el boton 'Copiar' para guardar el log antes de cerrar WSDD.
",
    },
];

const SECTIONS_EN: &[Section] = &[
    Section {
        title: "System requirements",
        content: "\
Operating system:
  • Windows 10 (build 2004+) or Windows 11
  • x64 architecture (AMD64)
  • 8 GB RAM minimum — 16 GB recommended for PHP stacks
  • 20 GB free disk space

Privileges:
  • WSDD must run as Administrator (UAC required)
  • Needed to modify C:\\Windows\\System32\\drivers\\etc\\hosts

Software installed automatically by WSDD:
  • Chocolatey — Windows package manager
  • PowerShell 7.5+ (pwsh.exe) — compatible automation runtime
  • mkcert — local SSL certificate generation

Software that must be pre-installed:
  • Docker Desktop — main container engine
  • WSL 2 — Linux subsystem required by Docker Desktop
",
    },
    Section {
        title: "Installation and first launch",
        content: "\
1. Run wsdd.exe as Administrator (right-click → Run as administrator).

2. On first launch the Welcome Wizard appears:
   • Read and check the requirements checkbox.
   • Click 'Next' to start environment verification.

3. Before the base environment is deployed for the first time, WSDD opens a
   dialog to capture MySQL and phpMyAdmin credentials.
   • These credentials are stored in wsdd-secrets.json.
   • They are requested only when missing from configuration.

4. The Loader verifies and automatically installs:
   • Chocolatey (package manager)
   • PowerShell 7.5+
   • mkcert (local SSL)
   • Network configuration and root certificate

5. If Docker Desktop was not installed:
   • The Loader stops and asks you to install it manually.
   • After installing Docker Desktop, open WSDD again.

6. When the Loader completes, the Main Panel appears.

WSDD data location:
  C:\\WSDD-Environment\\
  ├── PS-Script\\         PowerShell automation scripts
  ├── Docker-Structure\\  Container, project, and SSL configuration
  └── wsdd-config.json  Application settings
",
    },
    Section {
        title: "Main panel — overview",
        content: "\
The main panel has three areas:

1. MENU BAR (top):
   • File → Add Project, Exit
   • Docker → Refresh list, Reload Docker Desktop, Clear logs
   • Tools → WSL Settings, Settings
   • Help → Help, About...

2. TOOLBAR (below menu):
   • ⬡ phpMyAdmin   — Open phpMyAdmin in browser (http://pma.wsdd.dock)
   • ⚡ PS Terminal  — Open PowerShell 7 in C:\\WSDD-Environment
   • ⬛ CMD Terminal — Open CMD in C:\\WSDD-Environment
   • + Add          — New project form
   • ↺ Refresh      — Update containers and projects
   • Theme selector (right) — Switch between Dark Neutral, Dark Blue, Dark Warm, Light

3. CENTER TABS:
   • Containers   — List of active/inactive Docker containers
   • Projects     — List of registered WSDD projects

4. LOG PANEL (bottom):
   • Shows operation history with color-coded levels.
   • Has more usable height and can be resized for longer output.
   • 'Copy' button — copies log to clipboard.
   • 'Clear' button — clears visible log.

5. STATUS BAR (bottom):
   • Shows detected containers, running containers, projects and logs.
   • Shows Docker state and visible CPU/RAM usage.
",
    },
    Section {
        title: "Docker container management",
        content: "\
The 'Containers' tab shows all WSDD containers with their current state.

COLUMNS:
  • Name       — Docker container name
  • Status     — Running / Exited / ...
  • Image      — Base Docker image
  • Toolbox (⚙)  — Advanced container actions

ACTIONS PER CONTAINER (action columns):
  • ▶ Start   — Start the container
  • ■ Stop    — Stop the container
  • ↺ Restart — Restart the container

CONTAINER TOOLBOX (⚙ button):
  • View real-time container logs
  • Open interactive TTY terminal inside container
  • View exposed URLs and ports
  • Detailed info (image, ports, volumes)

AUTO POLLING:
  Containers update automatically every 3 seconds.
  Manual refresh not required (though ↺ button exists to force it).

PRESENTATION:
  • The main view leaves more breathing room around the grid.
  • The goal is to improve readability and avoid text touching the edge.

NOTE: Only WSDD stack containers are shown (prefix 'wsdd-' or related).
To see all Docker containers, use Docker Desktop or CLI.
",
    },
    Section {
        title: "Project management",
        content: "\
The 'Projects' tab shows web projects registered in WSDD.

COLUMNS:
  • Name       — Project name
  • Domain     — Local domain (e.g., myapp.wsdd.dock)
  • PHP        — Assigned PHP version (5.6 — 8.4)
  • Status     — Deployed / Not Deployed
  • Actions    — Deploy, Remove, Toolbox

ACTIONS:
  • ⬆ Deploy    — Deploy the project (creates containers, SSL, hosts)
  • ⬇ Remove    — Remove deployment (does NOT delete source files)
  • ⚙ Toolbox   — Advanced project actions

PROJECT TOOLBOX:
  • Open project folder in Windows Explorer
  • Open project in browser
  • View detailed info (paths, domain, entrypoint)

DELETE PROJECT:
  • Clicking Remove prompts for confirmation.
  • Removed: containers, SSL, hosts entry, JSON record.
  • Source code files are NOT deleted.
",
    },
    Section {
        title: "Adding a project",
        content: "\
To add a new web project:

1. Click '+ Add' in toolbar or go to File → Add Project.

2. Fill in the form:

   Name:
     Project identifier (no spaces, only letters/numbers/hyphens).
     Example: my-project

   Domain:
     Local subdomain. The '.wsdd.dock' suffix is added automatically.
     Example: type 'myapp' → final domain: myapp.wsdd.dock

   PHP Version:
     Select from PHP 5.6, 7.2, 7.4, 8.1, 8.2, 8.3, 8.4.

   Working Directory:
     Project root directory on local disk.
     Use 'Browse...' button to select folder.
     This directory is mounted as a volume in the Docker container.

   Entry Point:
     Main application file.
     Options: index.php, index.html, index.htm, custom (specify).

   SSL:
     Checkbox to generate local SSL certificate with mkcert.
     Recommended: enabled. Generates HTTPS automatically.

3. Click 'Deploy' to create the project.
   • If that PHP version does not yet have saved Webmin credentials and its
     container does not exist, WSDD opens a dialog to capture them.
   • Those credentials are stored per PHP version and reused for future
     projects on the same version.

4. Deploy process:
   a) Saves project to JSON
   b) Creates Docker volume
   c) Configures options.yml for PHP version
   d) Syncs Docker/Webmin resources for that PHP version
   e) Stops and removes previous PHP container
   f) Rebuilds PHP container with new project
   g) Generates vhost.conf for web server
   h) Generates SSL certificate (if enabled)
   i) Updates C:\\Windows\\System32\\drivers\\etc\\hosts
",
    },
    Section {
        title: "Deploy and Remove — detailed flow",
        content: "\
DEPLOY FLOW:
  1. handlers::project::save()    → saves {name}.json in Docker-Structure/projects/
  2. docker volume create         → creates volume with project code
  3. handlers::yml::add_project_to_options_yml()
                                  → registers project in options.php{XX}.yml
  4. Syncs Dockerfile + webserver.yml + Webmin credentials for that PHP version
  5. docker stop + docker rm      → stops and removes previous PHP container
  6. docker create --build + up -d → rebuilds and starts PHP container
  7. Generate vhost.conf          → configures web server virtual host
  8. mkcert + SSL                 → generates certificate if SSL enabled
  9. Proxy restart                → restarts proxy to apply vhost
  10. handlers::hosts::update_host() → adds domain to /etc/hosts (Windows hosts)

REMOVE FLOW:
  1. handlers::yml::remove_project_from_options_yml()
                                  → removes project from options.yml
  2. Removes project vhost.conf block
  3. Rebuilds PHP container without the project
  4. handlers::hosts::remove_domains() → removes project domain from hosts
  5. Removes project SSL files from Docker-Structure/ssl
  6. docker volume rm             → removes project volume
  7. handlers::project::delete()  → deletes project JSON

IMPORTANT:
  • Remove does NOT delete project source code.
  • Remove reverts hosts entries and WSDD-managed SSL files for that domain.
  • Domain may take a few seconds to stop resolving after Remove.
",
    },
    Section {
        title: "WSDD Settings",
        content: "\
Access via: Tools → Settings

GENERAL:
  • Projects path    — Base directory for new projects (default: C:\\WSDD-Projects)
  • Docker Desktop path  — Path to Docker Desktop executable (optional, for relaunching)
  • WSL Distro           — Active WSL2 distribution (e.g., Ubuntu-22.04)
  • Max log lines    — Limit of lines kept in log panel (100-10000)
  • Auto-start containers — Start WSDD containers on app launch

PHP (Docker containers):
  These values apply when GENERATING new containers.
  They do not affect existing containers (require redeploy).
  • memory_limit              — PHP RAM limit (e.g., 512M)
  • upload_max_filesize       — Max upload file size (e.g., 256M)
  • Timezone                  — PHP timezone (e.g., America/Mexico_City, UTC)

PREREQUISITES:
  • MySQL/phpMyAdmin credentials — requested before the first base-environment
    deploy when they are still missing from configuration.
  • They are stored in wsdd-secrets.json and reused on later launches.

TOOLS:
  • Webmin version — Version installed in PHP containers (e.g., 2.630)
  • Webmin credentials by PHP version — requested only the first time a
    version is deployed and its container does not already exist.
  • Changing them later does not automatically rotate the existing user inside
    the container; they apply on the next WSDD-managed rebuild.

Settings are saved to: C:\\WSDD-Environment\\wsdd-config.json
Secrets are saved to: C:\\WSDD-Environment\\wsdd-secrets.json
",
    },
    Section {
        title: "WSL2 Settings",
        content: "\
Access via: Tools → WSL Settings

Modifies: %USERPROFILE%\\.wslconfig

SYSTEM RESOURCES:
  • CPU Cores    — Limit cores assigned to WSL2.
                   'No limit' uses all available.
                   Recommended: 50-70% of physical cores.

  • Max RAM        — Limit RAM assigned to WSL2.
                     'No limit' allows WSL2 to consume all system RAM.
                     Recommended: 4-8 GB for typical WSDD stacks.

  • Swap              — Virtual swap space.
                        With sufficient RAM, 0 (disabled) is optimal.

PERFORMANCE AND MEMORY:
  • Memory reclaim — How WSL2 returns free RAM to Windows host:
    - Disabled: WSL2 retains RAM indefinitely (faster but uses more)
    - Gradual: releases memory gradually (recommended balance)
    - Drop Cache: aggressively releases on process termination (more free RAM on host)

  • GUI applications (WSLg) — Support for Linux apps with graphical interface.
                              Disable if not used to improve performance.

NETWORK:
  • Localhost forwarding — Access WSL2 ports via 127.0.0.1 on Windows.
                          WSDD requires this to be active.
  • Network mode:
    - NAT (recommended) — Isolated virtual network. Maximum compatibility.
    - Mirrored          — Shares host network. Experimental. Only Win11 23H2+.

IMPORTANT NOTE:
  Changes to .wslconfig require restarting WSL2:
  Open PowerShell as Admin and run: wsl --shutdown
  Then reopen Docker Desktop.
",
    },
    Section {
        title: "SSL certificates and HTTPS",
        content: "\
WSDD uses mkcert to generate locally-trusted SSL certificates.

HOW IT WORKS:
  1. mkcert creates a local Certificate Authority (CA) on your system.
  2. The CA is installed as trusted in Windows Certificate Store.
  3. For each project with SSL enabled, a certificate signed by the CA is generated.
  4. The reverse proxy uses certificates to serve HTTPS.

DOMAINS:
  All projects use the .wsdd.dock suffix
  Example: myapp.wsdd.dock → https://myapp.wsdd.dock

RENEW A CERTIFICATE:
  Remove the project and Deploy again.
  Certificate regenerates automatically.

BROWSER TRUST:
  If browser doesn't trust certificates:
  1. Verify mkcert is installed: mkcert -version (in PowerShell)
  2. Reinstall CA: mkcert -install
  3. Restart browser.

CERTIFICATE LOCATION:
  C:\\WSDD-Environment\\Docker-Structure\\ssl\\
  ├── {domain}.crt  — Certificate
  └── {domain}.key  — Private key
",
    },
    Section {
        title: "Troubleshooting",
        content: "\
PROBLEM: Containers don't appear in the list.
  Solution:
  • Verify Docker Desktop is running (icon in system tray).
  • Click ↺ Refresh or go to Docker → Refresh container list.
  • Check log panel for connection error messages.

PROBLEM: 'Docker not found' when starting WSDD.
  Solution:
  • Install Docker Desktop manually from docker.com
  • Or let the Loader install it (requires Chocolatey).
  • After installation, a Windows restart may be required.

PROBLEM: The .wsdd.dock domain doesn't resolve in browser.
  Solution:
  • Verify project is in 'Deployed' state.
  • Check C:\\Windows\\System32\\drivers\\etc\\hosts — should have a line with the domain.
  • If not: Remove and Deploy again.
  • Verify WSDD ran as Administrator.

PROBLEM: HTTPS shows certificate error.
  Solution:
  • Run in PowerShell: mkcert -install
  • Completely restart browser.
  • If persists: Remove + Deploy the project.

PROBLEM: Deploy fails with Docker error.
  Solution:
  • Check bottom log panel for specific error.
  • Verify Docker Desktop is in 'Running' state (not 'Starting').
  • Try Docker → Reload Docker Desktop from menu.
  • Manually restart Docker Desktop and wait for full load.

PROBLEM: PowerShell gives encoding error (invalid bytes).
  Solution:
  • WSDD uses -NoProfile -NonInteractive in all PS commands.
  • If you see encoding errors in log, verify pwsh.exe is PS7+:
    pwsh --version  (should be 7.x)

PROBLEM: Chocolatey doesn't install.
  Solution:
  • Check PowerShell execution policy:
    Get-ExecutionPolicy  (should be RemoteSigned or Bypass)
  • Run as Admin: Set-ExecutionPolicy RemoteSigned -Scope LocalMachine
  • Retry Loader (close and reopen WSDD).

PROBLEM: Log shows 'Access denied' when modifying hosts.
  Solution:
  • WSDD must run as Administrator.
  • Close WSDD and run wsdd.exe with right-click → Run as administrator.
",
    },
    Section {
        title: "Frequently Asked Questions (FAQ)",
        content: "\
Q: Can I have multiple PHP versions at the same time?
A: Yes. Each project has its own PHP version and container.
   You can have PHP 5.6, 8.1, and 8.4 simultaneously in different projects.

Q: Does WSDD modify my source code files?
A: No. WSDD only mounts your directory as a Docker volume (read-write).
   Files are not copied or modified internally.

Q: What happens if I delete a project from Windows Explorer?
A: The Docker container and WSDD registry still exist.
   Do 'Remove' from WSDD first to clean up properly,
   then delete the directory.

Q: Can I use WSDD with Laravel / Symfony / WordPress projects?
A: Yes, with any PHP framework. Configure Entry Point correctly:
   - Laravel/Symfony: public/index.php
   - WordPress: index.php
   - Custom: specify relative path to working directory.

Q: How do I update automation scripts?
A: PS1 scripts are embedded in the WSDD binary.
   To update, recompilation from source code is required.

Q: Does WSDD work with WSL 1?
A: No. Docker Desktop requires WSL 2. WSDD assumes WSL 2.

Q: Can I change an existing project's domain?
A: Currently no in-place editing. Do Remove and add the project
   again with the new domain.

Q: Where is project data saved?
A: C:\\WSDD-Environment\\Docker-Structure\\projects\\{name}.json

Q: How do I backup my WSDD projects?
A: Back up the complete directory of your source code (working path)
   and the JSON file from C:\\WSDD-Environment\\Docker-Structure\\projects\\
   The JSON contains project configuration (domain, PHP, SSL, etc.)
",
    },
    Section {
        title: "File and path reference",
        content: "\
C:\\WSDD-Environment\\
├── wsdd-config.json              WSDD application settings
├── PS-Script\\                   PowerShell automation scripts
│   ├── dd-detector.ps1           Environment and dependency detection
│   ├── dd-isinstalled.ps1        Checks if a tool is installed
│   ├── dd-isrunning.ps1          Checks if Docker is active
│   ├── dd-issettingup.ps1        Checks configuration state
│   ├── dd-setting.ps1            Read/write configuration
│   ├── dd-start.ps1              WSDD services start
│   ├── dd-stop.ps1               WSDD services stop
│   ├── dd-fixmysqlpermission.ps1 MySQL permissions repair
│   └── wsl-shutdown.ps1          WSL2 restart
├── Docker-Structure\\
│   ├── bin\\
│   │   └── php{X.X}\\
│   │       └── options.php{XX}.yml   PHP container configuration
│   ├── ssl\\
│   │   ├── {domain}.crt            Domain SSL certificate
│   │   └── {domain}.key            SSL private key
│   └── projects\\
│       └── {name}.json            Data for each registered project

%USERPROFILE%\\.wslconfig         WSL2 resource configuration

C:\\Windows\\System32\\drivers\\etc\\hosts
                                  Hosts file (modified by WSDD on deploy)

WSDD Logs:
  Logs are in-memory during the session.
  Use 'Copy' button to save log before closing WSDD.
",
    },
];

const SECTIONS_FR: &[Section] = &[
    Section {
        title: "Configuration requise",
        content: "\
Systeme d'exploitation:
  • Windows 10 (build 2004+) ou Windows 11
  • Architecture x64 (AMD64)
  • 8 Go RAM minimum — 16 Go recommandes pour les stacks PHP
  • 20 Go d'espace disque libre

Privileges:
  • WSDD doit s'executer en tant qu'Administrateur (UAC requis)
  • Necessaire pour modifier C:\\Windows\\System32\\drivers\\etc\\hosts

Logiciels installes automatiquement par WSDD:
  • Chocolatey — gestionnaire de paquets Windows
  • Docker Desktop — moteur de conteneurs
  • mkcert — generation de certificats SSL locaux
  • WSL 2 — sous-systeme Linux (requis par Docker Desktop)

Logiciels a installer au prealable:
  • PowerShell 7 (pwsh.exe) — obligatoire pour l'automatisation
",
    },
    Section {
        title: "Installation et premier lancement",
        content: "\
1. Executer wsdd.exe en tant qu'Administrateur (clic droit → Executer en tant qu'administrateur).

2. Au premier lancement, l'Assistant de bienvenue apparait:
   • Lire et cocher la case des prerequis.
   • Cliquer sur 'Suivant' pour demarrer la verification de l'environnement.

3. Avant le premier deploy de l'environnement de base, WSDD ouvre un dialogue
   pour capturer les identifiants MySQL et phpMyAdmin.
   • Ces identifiants sont enregistres dans wsdd-config.json.
   • Ils ne sont demandes que s'ils manquent dans la configuration.

4. Le Loader verifie et installe automatiquement:
   • Chocolatey (gestionnaire de paquets)
   • Docker Desktop
   • mkcert (SSL local)
   • Configuration reseau et certificat racine

4. Si Docker Desktop n'etait pas installe:
   • WSDD l'installe via Chocolatey.
   • Un redemarrage du systeme peut etre necessaire.
   • Apres le redemarrage, relancer WSDD.

5. Une fois le Loader termine, le Panneau Principal apparait.

Emplacement des donnees WSDD:
  C:\\WSDD-Environment\\
  ├── PS-Script\\         Scripts d'automatisation PowerShell
  ├── Docker-Structure\\  Configuration des conteneurs, projets et SSL
  └── wsdd-config.json  Parametres de l'application
",
    },
    Section {
        title: "Panneau principal — apercu",
        content: "\
Le panneau principal comporte trois zones:

1. BARRE DE MENU (haut):
   • Fichier → Ajouter Projet, Quitter
   • Docker → Actualiser liste, Recharger Docker Desktop, Effacer logs
   • Outils → Parametres WSL, Parametres
   • Aide → Aide, A propos...

2. BARRE D'OUTILS (sous le menu):
   • ⬡ phpMyAdmin   — Ouvrir phpMyAdmin dans le navigateur
   • ⚡ Terminal PS  — Ouvrir PowerShell 7 dans C:\\WSDD-Environment
   • ⬛ Terminal CMD — Ouvrir CMD dans C:\\WSDD-Environment
   • + Ajouter      — Formulaire nouveau projet
   • ↺ Actualiser   — Mettre a jour conteneurs et projets
   • Selecteur de theme (droite) — Basculer entre themes

3. ONGLETS CENTRAUX:
   • Conteneurs   — Liste des conteneurs Docker actifs/inactifs
   • Projets      — Liste des projets WSDD enregistres

4. PANNEAU DE LOG (bas):
   • Affiche l'historique des operations avec code couleur.
   • Bouton 'Copier' — copie le log dans le presse-papiers.
   • Bouton 'Effacer' — efface le log visible.
",
    },
    Section {
        title: "Gestion des conteneurs Docker",
        content: "\
L'onglet 'Conteneurs' affiche tous les conteneurs WSDD avec leur etat actuel.

COLONNES:
  • Nom       — Nom du conteneur Docker
  • Statut    — Running / Exited / ...
  • Image     — Image Docker de base
  • Toolbox (⚙)  — Actions avancees du conteneur

ACTIONS PAR CONTENEUR:
  • ▶ Demarrer   — Demarrer le conteneur
  • ■ Arreter    — Arreter le conteneur
  • ↺ Redemarrer — Redemarrer le conteneur

TOOLBOX CONTENEUR (bouton ⚙):
  • Voir les logs du conteneur en temps reel
  • Ouvrir un terminal TTY interactif dans le conteneur
  • Voir les URLs et ports exposes
  • Informations detaillees (image, ports, volumes)

POLLING AUTOMATIQUE:
  Les conteneurs se mettent a jour automatiquement toutes les 3 secondes.
  L'actualisation manuelle n'est pas necessaire.
",
    },
    Section {
        title: "Gestion des projets",
        content: "\
L'onglet 'Projets' affiche les projets web enregistres dans WSDD.

COLONNES:
  • Nom       — Nom du projet
  • Domaine   — Domaine local (ex: monapp.wsdd.dock)
  • PHP       — Version PHP assignee (5.6 — 8.4)
  • Statut    — Deployed / Not Deployed
  • Actions   — Deploy, Remove, Toolbox

ACTIONS:
  • ⬆ Deploy    — Deployer le projet (cree conteneurs, SSL, hosts)
  • ⬇ Remove    — Supprimer le deploiement (NE supprime PAS les fichiers sources)
  • ⚙ Toolbox   — Actions avancees du projet

TOOLBOX PROJET:
  • Ouvrir le dossier du projet dans l'Explorateur Windows
  • Ouvrir le projet dans le navigateur
  • Voir les informations detaillees (chemins, domaine, entrypoint)

SUPPRIMER PROJET:
  • Cliquer sur Remove demande une confirmation.
  • Sont supprimes: conteneurs, SSL, entree hosts, enregistrement JSON.
  • Les fichiers de code source NE sont PAS supprimes.
",
    },
    Section {
        title: "Ajouter un projet",
        content: "\
Pour ajouter un nouveau projet web:

1. Cliquer sur '+ Ajouter' dans la barre d'outils ou aller dans Fichier → Ajouter Projet.

2. Remplir le formulaire:

   Nom:
     Identifiant du projet (sans espaces, uniquement lettres/chiffres/tirets).
     Exemple: mon-projet

   Domaine:
     Sous-domaine local. Le suffixe '.wsdd.dock' est ajoute automatiquement.
     Exemple: taper 'monapp' → domaine final: monapp.wsdd.dock

   Version PHP:
     Selectionner parmi PHP 5.6, 7.2, 7.4, 8.1, 8.2, 8.3, 8.4.

   Repertoire de travail:
     Repertoire racine du projet sur le disque local.
     Utiliser le bouton 'Parcourir...' pour selectionner le dossier.

   Point d'entree:
     Fichier principal de l'application.
     Options: index.php, index.html, index.htm, personnalise.

   SSL:
     Case a cocher pour generer un certificat SSL local avec mkcert.
     Recommande: active. Genere HTTPS automatiquement.

3. Cliquer sur 'Deploy' pour creer le projet.
",
    },
    Section {
        title: "Deploy et Remove — flux detaille",
        content: "\
FLUX DEPLOY:
  1. handlers::project::save()    → sauvegarde {nom}.json
  2. docker volume create         → cree le volume avec le code du projet
  3. Enregistre le projet dans options.php{XX}.yml
  4. docker stop + docker rm      → arrete et supprime le conteneur PHP precedent
  5. docker create --build + up -d → reconstruit et demarre le conteneur PHP
  6. Genere vhost.conf            → configure le virtual host
  7. mkcert + SSL                 → genere le certificat si SSL actif
  8. Proxy restart                → redemarre le proxy
  9. Met a jour le fichier hosts

FLUX REMOVE:
  1. Supprime le projet du options.yml
  2. Supprime le bloc vhost.conf du projet
  3. Reconstruit le conteneur PHP sans le projet
  4. Retire le domaine du fichier hosts
  5. Supprime les fichiers SSL du projet dans Docker-Structure/ssl
  6. docker volume rm             → supprime le volume du projet
  7. Supprime le JSON du projet

IMPORTANT:
  • Remove NE supprime PAS le code source du projet.
  • Remove revert aussi les entrees hosts et les SSL geres par WSDD.
  • Le domaine peut mettre quelques secondes a ne plus resoudre.
",
    },
    Section {
        title: "Parametres WSDD",
        content: "\
Acces via: Outils → Parametres

GENERAL:
  • Chemin des projets    — Repertoire de base pour nouveaux projets
  • Chemin Docker Desktop — Chemin vers l'executable Docker Desktop
  • Distro WSL            — Distribution WSL2 active
  • Max lignes de log     — Limite de lignes conservees dans le panneau de log
  • Demarrage auto        — Demarre les conteneurs WSDD au lancement

PHP (conteneurs Docker):
  Ces valeurs s'appliquent lors de la GENERATION de nouveaux conteneurs.
  • memory_limit              — Limite RAM pour PHP
  • upload_max_filesize       — Taille max des fichiers uploades
  • Timezone                  — Fuseau horaire PHP

PREREQUIS:
  • Identifiants MySQL/phpMyAdmin — demandes avant le premier deploy
    de l'environnement de base s'ils n'existent pas encore dans la configuration.
  • Ils sont enregistres dans wsdd-config.json et reutilises ensuite.

OUTILS:
  • Version Webmin — Version installee dans les conteneurs PHP
  • Identifiants Webmin par version PHP — demandes une seule fois lors du
    premier deploy d'une version dont le conteneur n'existe pas encore.

Les modifications sont enregistrees dans: C:\\WSDD-Environment\\wsdd-config.json
",
    },
    Section {
        title: "Parametres WSL2",
        content: "\
Acces via: Outils → Parametres WSL

Modifie: %USERPROFILE%\\.wslconfig

RESSOURCES SYSTEME:
  • Coeurs CPU    — Limiter les coeurs assignes a WSL2.
  • RAM max       — Limiter la RAM assignee a WSL2.
  • Swap          — Espace d'echange virtuel.

PERFORMANCE ET MEMOIRE:
  • Recuperation memoire — Comment WSL2 libere la RAM vers l'hote Windows.
  • Applications GUI (WSLg) — Support des apps Linux avec interface graphique.

RESEAU:
  • Localhost forwarding — Acceder aux ports WSL2 via 127.0.0.1.
  • Mode reseau:
    - NAT (recommande) — Reseau virtuel isole.
    - Mirrored — Partage le reseau de l'hote. Experimental.

NOTE IMPORTANTE:
  Les modifications de .wslconfig necessitent un redemarrage de WSL2:
  Ouvrir PowerShell en Admin et executer: wsl --shutdown
",
    },
    Section {
        title: "Certificats SSL et HTTPS",
        content: "\
WSDD utilise mkcert pour generer des certificats SSL de confiance locale.

FONCTIONNEMENT:
  1. mkcert cree une Autorite de Certification (CA) locale sur votre systeme.
  2. La CA est installee comme approuvee dans le Windows Certificate Store.
  3. Pour chaque projet avec SSL actif, un certificat signe par la CA est genere.
  4. Le proxy inverse utilise les certificats pour servir HTTPS.

DOMAINES:
  Tous les projets utilisent le suffixe .wsdd.dock
  Exemple: monapp.wsdd.dock → https://monapp.wsdd.dock

RENOUVELER UN CERTIFICAT:
  Faire Remove du projet puis Deploy a nouveau.
  Le certificat est regenere automatiquement.

EMPLACEMENT DES CERTIFICATS:
  C:\\WSDD-Environment\\Docker-Structure\\ssl\\
  ├── {domaine}.crt  — Certificat
  └── {domaine}.key  — Cle privee
",
    },
    Section {
        title: "Depannage",
        content: "\
PROBLEME: Les conteneurs n'apparaissent pas dans la liste.
  Solution:
  • Verifier que Docker Desktop est en cours d'execution.
  • Cliquer sur ↺ Actualiser.
  • Verifier le panneau de log pour les messages d'erreur.

PROBLEME: 'Docker non trouve' au demarrage de WSDD.
  Solution:
  • Installer Docker Desktop manuellement depuis docker.com
  • Ou laisser le Loader l'installer.
  • Apres l'installation, un redemarrage peut etre necessaire.

PROBLEME: Le domaine .wsdd.dock ne resout pas dans le navigateur.
  Solution:
  • Verifier que le projet est en etat 'Deployed'.
  • Verifier le fichier hosts.
  • Si absent: faire Remove et Deploy a nouveau.
  • Verifier que WSDD s'est execute en tant qu'Administrateur.

PROBLEME: HTTPS affiche une erreur de certificat.
  Solution:
  • Executer dans PowerShell: mkcert -install
  • Redemarrer completement le navigateur.
  • Si persiste: Remove + Deploy du projet.

PROBLEME: Le deploy echoue avec une erreur Docker.
  Solution:
  • Verifier le panneau de log pour l'erreur specifique.
  • Verifier que Docker Desktop est en etat 'Running'.
  • Essayer Docker → Recharger Docker Desktop depuis le menu.
",
    },
    Section {
        title: "Questions frequentes (FAQ)",
        content: "\
Q: Puis-je avoir plusieurs versions de PHP en meme temps?
R: Oui. Chaque projet a sa propre version PHP et son propre conteneur.

Q: WSDD modifie-t-il mes fichiers de code source?
R: Non. WSDD monte uniquement votre repertoire comme volume Docker.
   Les fichiers ne sont ni copies ni modifies en interne.

Q: Que se passe-t-il si je supprime un projet depuis l'Explorateur Windows?
R: Le conteneur Docker et l'enregistrement WSDD existent toujours.
   Faire 'Remove' depuis WSDD d'abord pour nettoyer correctement.

Q: Puis-je utiliser WSDD avec des projets Laravel / Symfony / WordPress?
R: Oui, avec n'importe quel framework PHP. Configurer le Point d'entree:
   - Laravel/Symfony: public/index.php
   - WordPress: index.php

Q: Comment mettre a jour les scripts d'automatisation?
R: Les scripts PS1 sont embarques dans le binaire WSDD.
   Pour mettre a jour, une recompilation depuis le code source est necessaire.

Q: WSDD fonctionne-t-il avec WSL 1?
R: Non. Docker Desktop necessite WSL 2. WSDD suppose WSL 2.
",
    },
    Section {
        title: "Reference des fichiers et chemins",
        content: "\
C:\\WSDD-Environment\\
├── wsdd-config.json              Parametres de l'application WSDD
├── PS-Script\\                   Scripts d'automatisation PowerShell
├── Docker-Structure\\
│   ├── bin\\
│   │   └── php{X.X}\\
│   │       └── options.php{XX}.yml   Configuration des conteneurs PHP
│   ├── ssl\\
│   │   ├── {domaine}.crt            Certificat SSL du domaine
│   │   └── {domaine}.key            Cle privee SSL
│   └── projects\\
│       └── {nom}.json            Donnees de chaque projet enregistre

%USERPROFILE%\\.wslconfig         Configuration des ressources WSL2

C:\\Windows\\System32\\drivers\\etc\\hosts
                                  Fichier hosts (modifie par WSDD lors du deploy)

Logs WSDD:
  Les logs sont en memoire pendant la session.
  Utiliser le bouton 'Copier' pour sauvegarder le log avant de fermer WSDD.
",
    },
];

const SECTIONS_ZH: &[Section] = &[
    Section {
        title: "系统要求",
        content: "\
操作系统:
  • Windows 10 (build 2004+) 或 Windows 11
  • x64 架构 (AMD64)
  • 最低 8 GB RAM — PHP 堆栈推荐 16 GB
  • 20 GB 可用磁盘空间

权限:
  • WSDD 必须以管理员身份运行 (需要 UAC)
  • 需要修改 C:\\Windows\\System32\\drivers\\etc\\hosts

WSDD 自动安装的软件:
  • Chocolatey — Windows 包管理器
  • Docker Desktop — 容器引擎
  • mkcert — 本地 SSL 证书生成
  • WSL 2 — Linux 子系统 (Docker Desktop 需要)

必须预先安装的软件:
  • PowerShell 7 (pwsh.exe) — 自动化必需
",
    },
    Section {
        title: "安装和首次启动",
        content: "\
1. 以管理员身份运行 wsdd.exe (右键 → 以管理员身份运行)。

2. 首次启动时会出现欢迎向导:
   • 阅读并勾选要求复选框。
   • 点击「下一步」开始环境验证。

3. 在首次部署基础环境之前，WSDD 会打开一个对话框来获取
   MySQL 和 phpMyAdmin 凭据。
   • 这些凭据会保存在 wsdd-config.json 中。
   • 只有在配置中缺失时才会请求。

4. 加载器自动验证和安装:
   • Chocolatey (包管理器)
   • Docker Desktop
   • mkcert (本地 SSL)
   • 网络配置和根证书

4. 如果 Docker Desktop 未安装:
   • WSDD 通过 Chocolatey 安装它。
   • 可能需要重启系统。
   • 重启后再次运行 WSDD。

5. 加载器完成后，显示主面板。

WSDD 数据位置:
  C:\\WSDD-Environment\\
  ├── PS-Script\\         PowerShell 自动化脚本
  ├── Docker-Structure\\  容器、项目和 SSL 配置
  └── wsdd-config.json  应用程序设置
",
    },
    Section {
        title: "主面板 — 概览",
        content: "\
主面板有三个区域:

1. 菜单栏 (顶部):
   • 文件 → 添加项目, 退出
   • Docker → 刷新列表, 重新加载 Docker Desktop, 清除日志
   • 工具 → WSL 设置, 设置
   • 帮助 → 帮助, 关于...

2. 工具栏 (菜单下方):
   • ⬡ phpMyAdmin   — 在浏览器中打开 phpMyAdmin
   • ⚡ PS 终端      — 在 C:\\WSDD-Environment 中打开 PowerShell 7
   • ⬛ CMD 终端    — 在 C:\\WSDD-Environment 中打开 CMD
   • + 添加         — 新项目表单
   • ↺ 刷新         — 更新容器和项目
   • 主题选择器 (右侧) — 切换主题

3. 中央选项卡:
   • 容器   — 活动/非活动 Docker 容器列表
   • 项目   — 已注册的 WSDD 项目列表

4. 日志面板 (底部):
   • 显示操作历史，带颜色编码。
   • 「复制」按钮 — 复制日志到剪贴板。
   • 「清除」按钮 — 清除可见日志。
",
    },
    Section {
        title: "Docker 容器管理",
        content: "\
「容器」选项卡显示所有 WSDD 容器及其当前状态。

列:
  • 名称       — Docker 容器名称
  • 状态       — Running / Exited / ...
  • 镜像       — 基础 Docker 镜像
  • 工具箱 (⚙)  — 高级容器操作

每个容器的操作:
  • ▶ 启动   — 启动容器
  • ■ 停止   — 停止容器
  • ↺ 重启   — 重启容器

容器工具箱 (⚙ 按钮):
  • 实时查看容器日志
  • 在容器内打开交互式 TTY 终端
  • 查看暴露的 URL 和端口
  • 详细信息 (镜像, 端口, 卷)

自动轮询:
  容器每 3 秒自动更新。
  无需手动刷新。
",
    },
    Section {
        title: "项目管理",
        content: "\
「项目」选项卡显示 WSDD 中注册的 Web 项目。

列:
  • 名称       — 项目名称
  • 域名       — 本地域名 (例如: myapp.wsdd.dock)
  • PHP        — 分配的 PHP 版本 (5.6 — 8.4)
  • 状态       — Deployed / Not Deployed
  • 操作       — Deploy, Remove, Toolbox

操作:
  • ⬆ Deploy    — 部署项目 (创建容器, SSL, hosts)
  • ⬇ Remove    — 移除部署 (不删除源文件)
  • ⚙ Toolbox   — 高级项目操作

项目工具箱:
  • 在 Windows 资源管理器中打开项目文件夹
  • 在浏览器中打开项目
  • 查看详细信息 (路径, 域名, 入口点)

删除项目:
  • 点击 Remove 会提示确认。
  • 删除内容: 容器, SSL, hosts 条目, JSON 记录。
  • 源代码文件不会被删除。
",
    },
    Section {
        title: "添加项目",
        content: "\
添加新 Web 项目:

1. 点击工具栏中的「+ 添加」或转到 文件 → 添加项目。

2. 填写表单:

   名称:
     项目标识符 (无空格, 仅字母/数字/连字符)。
     示例: my-project

   域名:
     本地子域名。自动添加 '.wsdd.dock' 后缀。
     示例: 输入 'myapp' → 最终域名: myapp.wsdd.dock

   PHP 版本:
     从 PHP 5.6, 7.2, 7.4, 8.1, 8.2, 8.3, 8.4 中选择。

   工作目录:
     本地磁盘上的项目根目录。
     使用「浏览...」按钮选择文件夹。

   入口点:
     应用程序主文件。
     选项: index.php, index.html, index.htm, 自定义。

   SSL:
     使用 mkcert 生成本地 SSL 证书的复选框。
     推荐: 启用。自动生成 HTTPS。

3. 点击「Deploy」创建项目。
",
    },
    Section {
        title: "Deploy 和 Remove — 详细流程",
        content: "\
DEPLOY 流程:
  1. 保存项目到 JSON
  2. 创建包含项目代码的 Docker 卷
  3. 在 options.php{XX}.yml 中注册项目
  4. 停止并删除之前的 PHP 容器
  5. 重建并启动 PHP 容器
  6. 生成 vhost.conf
  7. 生成 SSL 证书 (如果启用)
  8. 重启代理
  9. 更新 hosts 文件

REMOVE 流程:
  1. 从 options.yml 中移除项目
  2. 不含项目重建 PHP 容器
  3. 删除项目卷
  4. 移除项目 vhost.conf 块
  5. 删除项目 JSON

重要:
  • Remove 不会删除项目源代码。
  • 域名可能需要几秒钟才能停止解析。
",
    },
    Section {
        title: "WSDD 设置",
        content: "\
访问方式: 工具 → 设置

常规:
  • 项目路径    — 新项目的基础目录
  • Docker Desktop 路径  — Docker Desktop 可执行文件路径
  • WSL 发行版           — 活动的 WSL2 发行版
  • 最大日志行数    — 日志面板保留的行数限制
  • 自动启动容器 — 应用启动时启动 WSDD 容器

PHP (Docker 容器):
  这些值在生成新容器时应用。
  • memory_limit              — PHP RAM 限制
  • upload_max_filesize       — 最大上传文件大小
  • Timezone                  — PHP 时区

先决条件:
  • MySQL/phpMyAdmin 凭据 — 当配置中还不存在时，会在首次部署基础环境前请求。
  • 这些凭据会保存在 wsdd-config.json 中，并在后续启动时重复使用。

工具:
  • Webmin 版本 — PHP 容器中安装的版本
  • 按 PHP 版本保存的 Webmin 凭据 — 仅在该版本首次部署且容器尚不存在时请求一次。

更改保存到: C:\\WSDD-Environment\\wsdd-config.json
",
    },
    Section {
        title: "WSL2 设置",
        content: "\
访问方式: 工具 → WSL 设置

修改: %USERPROFILE%\\.wslconfig

系统资源:
  • CPU 核心    — 限制分配给 WSL2 的核心数。
  • 最大 RAM    — 限制分配给 WSL2 的 RAM。
  • Swap        — 虚拟交换空间。

性能和内存:
  • 内存回收 — WSL2 如何将空闲 RAM 返回给 Windows 主机。
  • GUI 应用 (WSLg) — 支持带图形界面的 Linux 应用。

网络:
  • Localhost 转发 — 通过 127.0.0.1 访问 WSL2 端口。
  • 网络模式:
    - NAT (推荐) — 隔离的虚拟网络。
    - Mirrored — 共享主机网络。实验性。

重要说明:
  .wslconfig 的更改需要重启 WSL2:
  以管理员身份打开 PowerShell 并运行: wsl --shutdown
",
    },
    Section {
        title: "SSL 证书和 HTTPS",
        content: "\
WSDD 使用 mkcert 生成本地受信任的 SSL 证书。

工作原理:
  1. mkcert 在您的系统上创建本地证书颁发机构 (CA)。
  2. CA 作为受信任的安装在 Windows 证书存储中。
  3. 对于每个启用 SSL 的项目，生成由 CA 签名的证书。
  4. 反向代理使用证书提供 HTTPS。

域名:
  所有项目使用 .wsdd.dock 后缀
  示例: myapp.wsdd.dock → https://myapp.wsdd.dock

更新证书:
  Remove 项目然后重新 Deploy。
  证书自动重新生成。

证书位置:
  C:\\WSDD-Environment\\Docker-Structure\\ssl\\
  ├── {domain}.crt  — 证书
  └── {domain}.key  — 私钥
",
    },
    Section {
        title: "故障排除",
        content: "\
问题: 容器未出现在列表中。
  解决方案:
  • 验证 Docker Desktop 正在运行。
  • 点击 ↺ 刷新。
  • 检查日志面板的错误消息。

问题: 启动 WSDD 时显示「Docker 未找到」。
  解决方案:
  • 从 docker.com 手动安装 Docker Desktop
  • 或让加载器安装它。
  • 安装后可能需要重启 Windows。

问题: .wsdd.dock 域名在浏览器中无法解析。
  解决方案:
  • 验证项目处于「Deployed」状态。
  • 检查 hosts 文件。
  • 如果没有: Remove 然后重新 Deploy。
  • 验证 WSDD 以管理员身份运行。

问题: HTTPS 显示证书错误。
  解决方案:
  • 在 PowerShell 中运行: mkcert -install
  • 完全重启浏览器。
  • 如果持续: Remove + Deploy 项目。

问题: Deploy 失败并显示 Docker 错误。
  解决方案:
  • 检查日志面板的具体错误。
  • 验证 Docker Desktop 处于「Running」状态。
  • 尝试菜单中的 Docker → 重新加载 Docker Desktop。
",
    },
    Section {
        title: "常见问题 (FAQ)",
        content: "\
问: 我可以同时拥有多个 PHP 版本吗?
答: 是的。每个项目都有自己的 PHP 版本和容器。

问: WSDD 会修改我的源代码文件吗?
答: 不会。WSDD 只是将您的目录作为 Docker 卷挂载。
   文件不会在内部被复制或修改。

问: 如果我从 Windows 资源管理器删除项目会怎样?
答: Docker 容器和 WSDD 注册仍然存在。
   首先从 WSDD 执行「Remove」以正确清理。

问: 我可以将 WSDD 用于 Laravel / Symfony / WordPress 项目吗?
答: 是的，适用于任何 PHP 框架。正确配置入口点:
   - Laravel/Symfony: public/index.php
   - WordPress: index.php

问: 如何更新自动化脚本?
答: PS1 脚本嵌入在 WSDD 二进制文件中。
   要更新，需要从源代码重新编译。

问: WSDD 可以与 WSL 1 一起使用吗?
答: 不可以。Docker Desktop 需要 WSL 2。WSDD 假设使用 WSL 2。
",
    },
    Section {
        title: "文件和路径参考",
        content: "\
C:\\WSDD-Environment\\
├── wsdd-config.json              WSDD 应用程序设置
├── PS-Script\\                   PowerShell 自动化脚本
├── Docker-Structure\\
│   ├── bin\\
│   │   └── php{X.X}\\
│   │       └── options.php{XX}.yml   PHP 容器配置
│   └── projects\\
│       └── {name}.json            每个注册项目的数据
│   ├── ssl\\
│   │   ├── {domain}.crt          域名 SSL 证书
│   │   └── {domain}.key          SSL 私钥

%USERPROFILE%\\.wslconfig         WSL2 资源配置

C:\\Windows\\System32\\drivers\\etc\\hosts
                                  hosts 文件 (WSDD 在部署时修改)

WSDD 日志:
  日志在会话期间保存在内存中。
  关闭 WSDD 前使用「复制」按钮保存日志。
",
    },
];

// ── Render ────────────────────────────────────────────────────────────────────

pub fn render(ctx: &egui::Context, app: &mut WsddApp) {
    let close_label = format!("  {}  ", tr("btn_close"));
    let search_label = tr("help_search");
    let search_hint = tr("help_search_hint");
    let sections_found = tr("help_sections_found");
    let no_results = tr("help_no_results");
    let sections = help_sections(app.settings.language);

    egui::CentralPanel::default().show(ctx, |ui| {
        // ── Cabecera ──────────────────────────────────────────────────────
        ui.horizontal(|ui| {
            ui.heading(format!("{} — WSDD", tr("help_title")));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(&close_label).clicked() {
                    app.ui.active = ActiveView::Main;
                    app.ui.helps_search.clear();
                }
            });
        });

        // ── Buscador ──────────────────────────────────────────────────────
        ui.horizontal(|ui| {
            ui.label(&search_label);
            let search_field = ui.add(
                egui::TextEdit::singleline(&mut app.ui.helps_search)
                    .desired_width(280.0)
                    .hint_text(&search_hint),
            );
            if ui.button("✗").clicked() {
                app.ui.helps_search.clear();
            }
            // Auto-focus al abrir la pantalla
            if search_field.gained_focus() {
                app.ui.helps_search.clear();
            }
        });

        ui.separator();
        ui.add_space(4.0);

        let query = app.ui.helps_search.to_lowercase();
        let is_filtering = !query.is_empty();

        // Contar secciones que hacen match para mostrar contador
        if is_filtering {
            let matches = sections
                .iter()
                .filter(|s| section_matches(s, &query))
                .count();
            let color = if matches == 0 {
                egui::Color32::from_rgb(200, 80, 80)
            } else {
                ui.visuals().weak_text_color()
            };
            ui.label(
                egui::RichText::new(format!("{matches} {sections_found}"))
                    .size(11.0)
                    .color(color),
            );
            ui.add_space(2.0);
        }

        // ── Secciones ─────────────────────────────────────────────────────
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for section in sections {
                    let matches = !is_filtering || section_matches(section, &query);
                    if !matches {
                        continue;
                    }

                    let header =
                        egui::CollapsingHeader::new(egui::RichText::new(section.title).strong())
                            .default_open(is_filtering); // auto-expandir si hay busqueda activa

                    header.show(ui, |ui| {
                        render_section_content(ui, section.content, &query, is_filtering);
                    });

                    ui.add_space(2.0);
                }

                // Mensaje si no hay resultados
                if is_filtering && !sections.iter().any(|s| section_matches(s, &query)) {
                    ui.add_space(20.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new(&no_results).color(ui.visuals().weak_text_color()),
                        );
                    });
                }

                ui.add_space(16.0);
            });
    });
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Retorna true si el titulo o el contenido de la seccion contienen el query.
fn section_matches(section: &Section, query: &str) -> bool {
    section.title.to_lowercase().contains(query) || section.content.to_lowercase().contains(query)
}

/// Renderiza el contenido de una seccion, resaltando los terminos buscados.
fn render_section_content(ui: &mut egui::Ui, content: &str, query: &str, highlight: bool) {
    for line in content.lines() {
        if highlight && !query.is_empty() && line.to_lowercase().contains(query) {
            // Resaltar lineas que contienen el termino buscado
            egui::Frame::none()
                .fill(egui::Color32::from_rgba_unmultiplied(255, 220, 50, 30))
                .inner_margin(egui::Margin::symmetric(4.0, 0.0))
                .show(ui, |ui| {
                    ui.label(egui::RichText::new(line).monospace().size(12.0));
                });
        } else {
            ui.label(egui::RichText::new(line).monospace().size(12.0));
        }
    }
}
