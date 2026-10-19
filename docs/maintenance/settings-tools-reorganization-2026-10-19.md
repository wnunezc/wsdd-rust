# Settings y Herramientas: reorganización de la interfaz

## Alcance y contrato de compatibilidad

Settings ahora edita únicamente el idioma. Guardar escribe `wsdd-config.json`, conserva
las demás claves y no reescribe `wsdd-secrets.json`, archivos Compose ni `.wslconfig`.
Cancelar descarta el borrador. El cambio de idioma se aplica después de guardar.
No hay migración destructiva del JSON ni cambio de versión del esquema.

Los modelos, valores anteriores y rutas de consumo de las opciones ocultadas se
mantienen. Ocultar un control no equivale a eliminar una función ni a declarar que
funcione de extremo a extremo. En particular, Xdebug conserva su valor de
configuración y el generador PHP lo sigue leyendo; no hay un nuevo control por
contenedor ni verificación de la ruta IDE/proxy. Las credenciales MySQL y Webmin
ya guardadas siguen disponibles para los flujos que las consumen.

## Inventario de controles anteriores

| Campo anterior de Settings | Destino y efecto actual |
| --- | --- |
| Idioma | Permanece en Settings; Guardar sólo cambia `language`. |
| Ruta de proyectos, distro WSL, almacenamiento Linux administrado, estado de tarea | Pasan a Herramientas → Configuración WSL → Almacenamiento y recuperación WSDD. Tienen Guardar independiente de `.wslconfig`. La ruta es un alias/base para proyectos nuevos; no mueve proyectos existentes. Activar almacenamiento administrado puede aprovisionar WSDD-Storage y conciliar la tarea de recuperación. |
| Redis: activación, puerto y autoarranque | Herramientas → Redis. Guardar persiste parámetros sin desplegar; Deploy activa y ejecuta Compose; Detener para el contenedor sin borrar su volumen. |
| Memcached: activación, puerto, memoria y autoarranque | Herramientas → Memcached, mismas acciones. |
| Mailpit: activación, puertos SMTP/UI, host virtual y autoarranque | Herramientas → Mailpit, mismas acciones. |
| ngrok: activación, autoarranque, Agent API, inspección, token y dashboard | Herramientas → ngrok. Guardar persiste token/opciones sin desplegar; Deploy activa, recrea/aplica Compose y espera respuesta de Agent API; Comprobar Agent API consulta estado sin cambiar Docker; Detener para el contenedor. |
| Credenciales MySQL de prerequisitos | Ya no se editan en Settings. El diálogo de primer despliegue sigue solicitándolas cuando faltan. Cambiarlas en JSON no rota cuentas de una base existente. |
| Matriz de credenciales Webmin por versión | Ya no se edita en Settings. El despliegue PHP sigue pidiendo el par de la versión si falta y lo guarda; valores históricos siguen utilizables para builds. Cambiar el JSON no rota usuarios del contenedor. |
| Xdebug global | Se oculta; `xdebug_enabled` se conserva en configuración y generación. No se promete activación por contenedor ni paso por el proxy. |
| Memoria PHP, límite de subida, zona horaria | Se ocultan; sus valores y uso por plantillas existentes se conservan. No hay flujo UI que los aplique a un runtime existente. |
| Ruta decorativa de Docker Desktop, máximo de líneas de log y autoarranque global | Se ocultan; las claves se conservan para compatibilidad. No se anuncia funcionalidad desde Settings. |

## Comportamiento de botones y fallos

- Cada herramienta guarda sólo su parte de `AppSettings`. Redis, Memcached y
  Mailpit guardan sólo el JSON público; ngrok también guarda el secreto del token.
- Deploy guarda la configuración seleccionada, marca ese servicio como habilitado,
  regenera las plantillas de servicios opcionales y ejecuta `docker compose up -d
  --no-deps` del servicio elegido. Un fallo queda visible y se puede reintentar;
  puede quedar la configuración habilitada aunque Docker no lo haya iniciado.
- Detener ejecuta `docker stop` del servicio elegido sin borrar contenedor o
  volumen y sin deshabilitar su configuración guardada. Un futuro Deploy lo
  vuelve a iniciar. Cancelar descarta cambios del formulario, sin detener un
  trabajo Docker que ya comenzó.
- El diálogo indica estado observado del contenedor y de la operación. El
  inventario de contenedores depende del sondeo de Docker y puede actualizarse
  después del resultado del trabajo. ngrok requiere además respuesta HTTP
  satisfactoria de Agent API antes de declarar Deploy completado.
- El Deploy de las cuatro herramientas registra una operación con etapas
  medidas: preparación, descarga/inicio Compose, verificación del contenedor y,
  para ngrok, verificación de Agent API. El diálogo muestra las etapas y permite
  abrir o copiar su log completo, redactado. Las imágenes son publicadas; no se
  declara una construcción BuildKit cuando sólo ocurre un pull de Compose.
- Redis, Memcached y Mailpit tienen modelos, plantillas y comandos, pero **no
  hay validación integral en una instalación Docker real**. La UI muestra esa
  limitación; que Compose termine y el contenedor esté `running` no prueba una
  conexión funcional desde una aplicación PHP.
- En WSL, Guardar configuración global sólo escribe `.wslconfig` si hubo
  cambios y la lectura inicial fue correcta. Guardar almacenamiento WSDD
  persiste sólo configuración WSDD; no escribe `.wslconfig`. Al activar
  almacenamiento administrado puede ejecutar aprovisionamiento, por decisión
  explícita del usuario en ese botón. Si falla la conciliación de la tarea al
  desactivar almacenamiento, el error queda visible y otro clic en Guardar
  dentro del diálogo reintenta esa conciliación.

## Verificación y límites

Se agregaron pruebas de serialización de configuración con idioma, aislamiento
de herramientas y del guardado WSL, y paridad/completitud
de claves de los cinco catálogos de idioma. `cargo fmt --all -- --check`,
`cargo check --workspace`, `cargo clippy --workspace -- -D warnings` y
`cargo test --workspace --quiet` pasaron: 222 pruebas (205 + 11 + 6).
No se ha desplegado ni detenido ningún
contenedor del usuario ni instalado MSI en esta tarea. El ejercicio
pendiente de aceptación manual consiste en probar Guardar/Deploy/Detener en
Docker, la Agent API de ngrok, y la separación de ambos botones WSL.

## MSI local para aceptación

Por petición posterior del usuario se ejecutó `cargo wix-msi --nocapture`
con exit code 0. El primer MSI de Settings/Herramientas quedó preservado en
`D:\OpsZone\DevWorkspace\Engines\Rust-Resource\cargo-targets\WSDD\wix\wsdd-2.1.3-settings-tools-pre-trace-2026-10-19-x86_64.msi`:
27 041 792 bytes, SHA-256
`3D2369FC3055E6041843C97174E5D7E0C80183B1F252BB49EECD3E4FB7930532`.
La propiedad MSI `ProductVersion` es `2.1.3` y
su archivo `.sha256.txt` coincide con el archivo. Es un instalador
**local**, no el asset público v2.1.3. La compilación local anterior se
conservó como `wsdd-2.1.3-pre-settings-2026-10-19-x86_64.msi` con su checksum
original `35FD5EAC959EF13A2327FBA3D52D74883FCE1119DACA1405B7AE1390ACEF02F1`.
La plantilla WiX permite actualización de la misma ProductVersion y las
compilaciones verificadas tienen ProductCode distinto con el mismo UpgradeCode;
no se probó la instalación.
Antes del build se verificó por SHA-256 cada entrada de los ZIP embebidos
contra su fuente: 92/92 Docker y 13/13 PowerShell. No se instalaron los MSI
ni se modificó `C:\WSDD-Environment`.

Tras detectar que Deploy de herramientas carecía de etapas, se ejecutó de nuevo
`cargo wix-msi --nocapture` con exit code 0. El MSI local **actual** que incluye
la trazabilidad de ngrok y las demás herramientas está en
`D:\OpsZone\DevWorkspace\Engines\Rust-Resource\cargo-targets\WSDD\wix\wsdd-2.1.3-x86_64.msi`:
27 058 176 bytes, SHA-256
`346A2BF679D581C1EF9DC13210C37854FCD6640E5A29DDA7B45FB5D6331586BC`.
El `.sha256.txt` adjunto fue cotejado y el ProductVersion leído del MSI sigue
siendo `2.1.3`. Formato, check, Clippy y 222 pruebas del workspace pasaron
antes de este segundo empaquetado. El comportamiento en Docker continúa
pendiente de aceptación manual.

Relacionado: [issue #16](https://github.com/wnunezc/wsdd-rust/issues/16).
