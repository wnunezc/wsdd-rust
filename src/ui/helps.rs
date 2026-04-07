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
use crate::ui::ActiveView;

// ── Estructura del manual ──────────────────────────────────────────────────────

struct Section {
    title: &'static str,
    content: &'static str,
}

const SECTIONS: &[Section] = &[
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
  • Docker Desktop — motor de contenedores
  • mkcert — generacion de certificados SSL locales
  • WSL 2 — subsistema Linux (requerido por Docker Desktop)

Software que debe estar instalado previamente:
  • PowerShell 7 (pwsh.exe) — obligatorio para la automatizacion
",
    },
    Section {
        title: "Instalacion y primer arranque",
        content: "\
1. Ejecutar wsdd.exe como Administrador (clic derecho → Ejecutar como administrador).

2. Al primer arranque aparece el Welcome Wizard:
   • Leer y marcar la casilla de requisitos.
   • Hacer clic en 'Siguiente' para iniciar la verificacion del entorno.

3. El Loader verifica e instala automaticamente:
   • Chocolatey (gestor de paquetes)
   • Docker Desktop
   • mkcert (SSL local)
   • Configuracion de red y certificado raiz

4. Si Docker Desktop no estaba instalado:
   • WSDD lo instala via Chocolatey.
   • Es posible que se requiera reiniciar el sistema.
   • Tras reiniciar, ejecutar WSDD nuevamente.

5. Al completar el Loader, aparece el Panel Principal.

Ubicacion de datos de WSDD:
  C:\\WSDD-Environment\\
  ├── PS-Script\\         Scripts de automatizacion PowerShell
  ├── Docker-Structure\\  Configuracion de contenedores y proyectos
  ├── certs\\            Certificados SSL por dominio
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
   • Boton 'Copiar' — copia el log al portapapeles.
   • Boton 'Limpiar' — borra el log visible.
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
     Seleccionar entre PHP 5.6, 7.0, 7.1, 7.4, 8.0, 8.1, 8.2, 8.3, 8.4.

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

4. El proceso de deploy:
   a) Guarda el proyecto en JSON
   b) Crea el volumen Docker
   c) Configura options.yml para la version PHP
   d) Para y elimina el contenedor PHP anterior
   e) Reconstruye el contenedor PHP con el nuevo proyecto
   f) Genera el vhost.conf del servidor web
   g) Genera el certificado SSL (si aplica)
   h) Actualiza C:\\Windows\\System32\\drivers\\etc\\hosts
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
  4. docker stop + docker rm      → detiene y elimina el contenedor PHP anterior
  5. docker create --build + up -d → reconstruye e inicia el contenedor PHP
  6. Genera vhost.conf            → configura el virtual host del servidor web
  7. mkcert + SSL                 → genera certificado si SSL esta activo
  8. Proxy restart                → reinicia el proxy para aplicar vhost
  9. handlers::hosts::update_host() → agrega el dominio a /etc/hosts (Windows hosts)

FLUJO DE REMOVE:
  1. handlers::yml::remove_project_from_options_yml()
                                  → elimina el proyecto del options.yml
  2. Reconstruye el contenedor PHP sin el proyecto
  3. docker volume rm             → elimina el volumen del proyecto
  4. Elimina el bloque vhost.conf del proyecto
  5. handlers::project::delete()  → borra el JSON del proyecto

IMPORTANTE:
  • Remove NO elimina el codigo fuente del proyecto.
  • Remove NO revierte las entradas de hosts (Docker/mkcert las gestiona).
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

HERRAMIENTAS:
  • Version de Webmin — Version instalada en los contenedores PHP (ej: 2.021)

Los cambios se guardan en: C:\\WSDD-Environment\\wsdd-config.json
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
  C:\\WSDD-Environment\\certs\\{dominio}\\
  ├── cert.pem  — Certificado
  └── key.pem   — Clave privada
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
│   └── projects\\
│       └── {nombre}.json            Datos de cada proyecto registrado
└── certs\\
    └── {dominio}\\
        ├── cert.pem              Certificado SSL del dominio
        └── key.pem               Clave privada SSL

%USERPROFILE%\\.wslconfig         Configuracion de recursos WSL2

C:\\Windows\\System32\\drivers\\etc\\hosts
                                  Archivo de hosts (modificado por WSDD al hacer deploy)

Logs de WSDD:
  Los logs son en-memoria durante la sesion.
  Usar el boton 'Copiar' para guardar el log antes de cerrar WSDD.
",
    },
];

// ── Render ────────────────────────────────────────────────────────────────────

pub fn render(ctx: &egui::Context, app: &mut WsddApp) {
    egui::CentralPanel::default().show(ctx, |ui| {
        // ── Cabecera ──────────────────────────────────────────────────────
        ui.horizontal(|ui| {
            ui.heading("Manual de usuario — WSDD");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("  Cerrar  ").clicked() {
                    app.ui.active = ActiveView::Main;
                    app.ui.helps_search.clear();
                }
            });
        });

        // ── Buscador ──────────────────────────────────────────────────────
        ui.horizontal(|ui| {
            ui.label("Buscar:");
            let search_field = ui.add(
                egui::TextEdit::singleline(&mut app.ui.helps_search)
                    .desired_width(280.0)
                    .hint_text("Escribe para filtrar secciones..."),
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
            let matches = SECTIONS
                .iter()
                .filter(|s| section_matches(s, &query))
                .count();
            let color = if matches == 0 {
                egui::Color32::from_rgb(200, 80, 80)
            } else {
                ui.visuals().weak_text_color()
            };
            ui.label(
                egui::RichText::new(format!("{matches} secciones encontradas"))
                    .size(11.0)
                    .color(color),
            );
            ui.add_space(2.0);
        }

        // ── Secciones ─────────────────────────────────────────────────────
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for section in SECTIONS {
                    let matches = !is_filtering || section_matches(section, &query);
                    if !matches {
                        continue;
                    }

                    let header = egui::CollapsingHeader::new(
                        egui::RichText::new(section.title).strong(),
                    )
                    .default_open(is_filtering); // auto-expandir si hay busqueda activa

                    header.show(ui, |ui| {
                        render_section_content(ui, section.content, &query, is_filtering);
                    });

                    ui.add_space(2.0);
                }

                // Mensaje si no hay resultados
                if is_filtering
                    && !SECTIONS.iter().any(|s| section_matches(s, &query))
                {
                    ui.add_space(20.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new("Sin resultados para esta busqueda.")
                                .color(ui.visuals().weak_text_color()),
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
    section.title.to_lowercase().contains(query)
        || section.content.to_lowercase().contains(query)
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
