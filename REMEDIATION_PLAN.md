# WSDD - Plan de Remediacion Tecnica

## Estado

- Estado del plan: ACTIVO Y OBLIGATORIO
- Fecha de activacion: 2026-04-11
- Vigencia: hasta cerrar todos los paquetes abiertos de este documento
- Baseline publicado de referencia: `HEAD a83ba2c` / `main publicado (1.0.0-rc.11)`
- Validacion updater RC6 -> RC7: OK, cerrada por el usuario
- Decision global del usuario: NO publicar `1.0.0` estable hasta cerrar todos los problemas del plan
- Estrategia de releases actual: continuar con versiones RC mientras el plan siga abierto

## Alcance

Este plan cubre la reparacion de los hallazgos detectados en la evaluacion general:

- seguridad y secretos
- CI/CD y gates de release
- lifecycle real de Docker/WSL/PowerShell
- robustez de deploy/remove
- arquitectura, SRP y orquestacion
- configuracion y paths
- documentacion viva
- testing y validacion
- deuda tecnica menor antes de `1.0.0`

## Regla obligatoria para futuras sesiones

Antes de implementar cualquier cambio relacionado con estos hallazgos, el agente debe:

1. Leer `AGENTS.md`, este archivo y `08-AI-Context/wsdd-rust.md`
2. Identificar el paquete de trabajo afectado
3. Presentar las opciones disponibles si existe mas de una via razonable
4. Esperar aprobacion explicita del usuario antes de editar codigo, scripts o docs del alcance
5. Ejecutar solo el alcance aprobado
6. Reportar validaciones y esperar confirmacion del usuario antes de marcar el paquete como cerrado

Reglas adicionales:

- No mezclar cambios locales no commitados con el estado publicado al analizar o validar
- No cerrar un paquete por inferencia; solo por confirmacion explicita del usuario
- No abrir implementacion paralela de varios paquetes sin aprobacion explicita del usuario

## Gate confirmado para `1.0.0` estable

No publicar `1.0.0` estable hasta cerrar todos los paquetes abiertos de este plan,
salvo nueva instruccion explicita del usuario.

## Estado de paquetes

| ID | Prioridad | Estado actual | Decision del usuario |
|---|---|---|---|
| `WP-01` | Alto | Cerrado por confirmacion explicita del usuario en `rc.11` | Opcion `B` aprobada |
| `WP-02` | Alto | Cerrado por confirmacion explicita del usuario en `rc.12` | Opcion `B` aprobada |
| `WP-03` | Alto | Cerrado por confirmacion explicita del usuario en `rc.13` | Opcion `A` endurecida aprobada |
| `WP-04` | Alto | Cerrado por confirmacion explicita del usuario en `rc.14` | Opcion `B` aprobada |
| `WP-05` | Alto | Cerrado por confirmacion explicita del usuario en `rc.15` | Opcion `B` acotada aprobada |
| `WP-06` | Medio | Pendiente de decision | Pendiente |
| `WP-07` | Medio | Pendiente de decision | Pendiente |
| `WP-08` | Medio | Pendiente de decision | Pendiente |
| `WP-09` | Medio | Pendiente de decision | Pendiente |
| `WP-10` | Bajo | Pendiente de decision | Pendiente |

## Paquetes de trabajo

### `WP-01` Seguridad de secretos y credenciales por defecto

- Prioridad: Alto
- Estado actual:
  - opcion `B` aprobada por el usuario
  - implementadas credenciales bajo demanda para MySQL + phpMyAdmin
  - implementadas credenciales Webmin por version PHP, pedidas una sola vez por version
  - `1.0.0-rc.8` detecto regresion en version legacy de Webmin durante prueba manual
  - `1.0.0-rc.9` quedo invalidado para instalaciones limpias por un `docker-structure.zip` mal reempaquetado
- `1.0.0-rc.10` corrigio la estructura de `Docker-Structure`, pero quedo invalidado por `sha256` desactualizado del `.deb` de Webmin
- `1.0.0-rc.11` preparado con `sha256` actualizado de Webmin, deduplicacion de log Docker en deploy de proyectos y reparacion/autocuracion de `Docker-Structure`
- `1.0.0-rc.11` fue recompilado localmente sin cambiar version para corregir el bootstrap del usuario Webmin en `PHP 8.1/8.2/8.3/8.4` y restaurar las URLs base `php/cron/wmXX.wsdd.dock` en `hosts` y toolbox de contenedor
  - cerrado por confirmacion explicita del usuario
- Hallazgos base:
  - `recursos/recursos/Docker-Structure/init.yml`
  - Dockerfiles PHP/Webmin con credenciales por defecto
- Objetivo:
  - eliminar secretos hardcodeados del baseline
  - generar configuracion segura por instalacion
- Opciones a elegir:
  - `A` Generar secretos por instalacion y guardarlos en `.env` locales de Docker
  - `B` Generar secretos por instalacion y guardarlos en `wsdd-config.json`, inyectandolos a templates al desplegar
  - `C` Mezcla de `.env` para contenedores + almacenamiento local seguro para datos sensibles fuera de Docker
- Sugerencia tecnica:
  - `A` o `C`
- Aprobacion requerida:
  - aprobada opcion `B` para este paquete
- Criterio de cierre:
  - no quedan credenciales fijas en recursos publicados
  - instalacion nueva genera credenciales unicas
  - README/help/release notes quedan alineados

### `WP-02` CI/CD y gate de release

- Prioridad: Alto
- Estado actual:
  - opcion `B` aprobada por el usuario
  - workflows `CI` y `Release MSI` implementados en `.github/workflows/`
  - gate remoto validado en GitHub Actions
  - build MSI y artefacto `rc.12` validados dentro del cierre de fase
  - cerrado por confirmacion explicita del usuario
- Hallazgo base:
  - no existe `.github/workflows/`
- Objetivo:
  - automatizar verificaciones minimas antes de merge/release
- Opciones a elegir:
  - `A` Pipeline minima: `fmt`, `check`, `clippy`, tests unitarios
  - `B` Pipeline completa: minima + build MSI + artefactos + smoke del launcher
  - `C` Mantener validacion manual local con scripts versionados, sin GitHub Actions por ahora
- Sugerencia tecnica:
  - `B`
- Aprobacion requerida:
  - aprobada opcion `B` para este paquete
- Criterio de cierre:
  - existe gate reproducible para PR y release
  - el proceso de release deja evidencia verificable

### `WP-03` Lifecycle real de WSL/Docker/PowerShell

- Prioridad: Alto
- Estado actual:
  - opcion `A` endurecida aprobada por el usuario
  - `PowerShell 7.5+` convertido en prerequisito real antes de ejecutar scripts `PS1`
  - `start/stop/configure` y `Reload Docker Desktop` alineados al lifecycle oficial por scripts
  - acciones GUI para `Start/Restart/Shutdown WSL` implementadas y validadas manualmente
  - secretos segregados a `wsdd-secrets.json`
  - `Last poll` removido de la barra de estado
  - dialogos criticos ajustados como modales bloqueantes durante el cierre de `rc.13`
  - cerrado por confirmacion explicita del usuario
- Hallazgos base:
  - `wsl-shutdown.ps1` vacio
  - scripts con polling y tokens magicos
- Objetivo:
  - que start/stop/configure representen acciones reales y verificables
- Opciones a elegir:
  - `A` Completar los scripts PS1 existentes y mantener la logica actual
  - `B` Mover parte del control critico a Rust y dejar PS1 solo como wrapper minimo
  - `C` Reducir alcance funcional y eliminar acciones no confiables hasta rehacerlas
- Sugerencia tecnica:
  - `A` endurecida
- Aprobacion requerida:
  - aprobada opcion `A` endurecida para este paquete
- Criterio de cierre:
  - start/stop/configure tienen comportamiento real
  - existe validacion post-accion y logs claros

### `WP-04` Estrategia de jobs y no-bloqueo de UI

- Prioridad: Alto
- Estado actual:
  - opcion `B` aprobada por el usuario
  - implementado en `rc.14` y cerrado por confirmacion explicita del usuario
  - coordinacion central de jobs largos desde `app.rs`
  - migrados al coordinador: loader requirements, polling, lifecycle Docker/WSL, start/stop/restart de contenedores, deploy/remove, backup/restore, toolbox de contenedor
  - reemplazados sleeps fijos del deploy base por polling de readiness real
  - notas no bloqueantes documentadas para futuro:
    - no existe estado `queued` formal
    - quedan hilos utilitarios puntuales fuera del coordinador (`folder picker`, bridges de streaming y helpers internos)
- Hallazgos base:
  - `std::thread::spawn` disperso
  - sleeps fijos en `docker_deploy.rs`
  - uso parcial de `tokio` sin orquestacion central
- Objetivo:
  - centralizar ejecucion larga y estado de tareas
- Opciones a elegir:
  - `A` Mantener threads pero introducir un `TaskRunner` central con canales
  - `B` Estandarizar en `tokio` + `spawn_blocking` para IO pesado
  - `C` Diseñar cola de comandos/estado mas formal antes de tocar handlers
- Sugerencia tecnica:
  - `B`
- Aprobacion requerida:
  - aprobada opcion `B` para este paquete
- Criterio de cierre:
  - la UI no dispara jobs ad hoc sin control
  - cada tarea larga tiene estado, logs y finalizacion trazable

### `WP-05` Robustez de deploy/remove e idempotencia

- Prioridad: Alto
- Estado actual:
  - opcion `B` acotada aprobada por el usuario
  - implementado en `rc.15`
  - validado manualmente por el usuario
  - cerrado por confirmacion explicita del usuario
- Hallazgos base:
  - deploy secuencial con rollback limitado
  - remove con limitacion conocida en `hosts`
  - rutas SSL desalineadas entre README/help, backup/restore y deploy real
- Objetivo:
  - reducir estados parciales y dejar operaciones repetibles
- Opciones a elegir:
  - `A` Agregar rollback/compensacion minima a los pasos actuales
  - `B` Rehacer deploy/remove como pipeline explicita con estados y compensaciones
  - `C` Mantener flujo actual y solo ampliar validaciones/logs
- Sugerencia tecnica:
  - `B`
- Aprobacion requerida:
  - aprobada opcion `B` acotada para este paquete
- Criterio de cierre:
  - deploy/remove soportan reintentos
  - fallos dejan diagnostico y recuperacion clara
  - ruta SSL canonica queda en `C:\WSDD-Environment\Docker-Structure\ssl`
  - documentacion publica y ayuda interna quedan alineadas al comportamiento real

### `WP-06` Configuracion, paths y supuestos de entorno

- Prioridad: Medio
- Hallazgos base:
  - paths Windows y ubicaciones de trabajo hardcodeadas
- Objetivo:
  - centralizar rutas y supuestos ambientales
- Opciones a elegir:
  - `A` Introducir `PathConfig` central y reemplazar literales gradualmente
  - `B` Introducir `PathConfig` + `EnvConfig` desde el inicio
  - `C` Mantener hardcodes y documentarlos mejor
- Sugerencia tecnica:
  - `B`
- Aprobacion requerida:
  - elegir opcion `A`, `B` o `C`
- Criterio de cierre:
  - no hay rutas criticas duplicadas por todo el codigo
  - los tests pueden aislar paths relevantes

### `WP-07` Refactor SRP y modulos grandes

- Prioridad: Medio
- Hallazgos base:
  - archivos grandes y con mas de una responsabilidad
- Objetivo:
  - dividir modulos por dominio y reducir acoplamiento
- Opciones a elegir:
  - `A` Refactor incremental por archivo mas riesgoso
  - `B` Refactor por vertical funcional completa
  - `C` Congelar features y hacer una reestructuracion amplia de una vez
- Sugerencia tecnica:
  - `A`
- Aprobacion requerida:
  - elegir opcion `A`, `B` o `C`
- Criterio de cierre:
  - los modulos criticos quedan divididos en responsabilidades claras
  - baja el costo de prueba y mantenimiento

### `WP-08` Documentacion viva y ayuda en app

- Prioridad: Medio
- Hallazgos base:
  - `src/ui/helps.rs` duplica y desalinea documentacion
- Objetivo:
  - dejar una sola fuente razonable para ayuda funcional
- Opciones a elegir:
  - `A` Mover ayuda a markdown embebido y renderizarla en la UI
  - `B` Generar ayuda desde fragmentos de README/docs
  - `C` Mantener ayuda en Rust pero con proceso formal de sincronizacion
- Sugerencia tecnica:
  - `A`
- Aprobacion requerida:
  - elegir opcion `A`, `B` o `C`
- Criterio de cierre:
  - ayuda UI y docs publicas no se contradicen

### `WP-09` Estrategia de testing y validacion de release

- Prioridad: Medio
- Hallazgos base:
  - `cargo test` compila pero no ejecuta por elevacion en este entorno
  - faltan pruebas de integracion/release mas formales
- Objetivo:
  - separar claramente pruebas unitarias, integracion y validaciones elevadas
- Opciones a elegir:
  - `A` Unit tests + checklist manual elevada
  - `B` Unit + integracion aislada + checklist manual elevada + smoke release
  - `C` Mock-heavy sin pruebas reales del entorno
- Sugerencia tecnica:
  - `B`
- Aprobacion requerida:
  - elegir opcion `A`, `B` o `C`
- Criterio de cierre:
  - existe una estrategia reproducible para validar release

### `WP-10` Limpieza de deuda tecnica menor

- Prioridad: Bajo
- Hallazgos base:
  - `display_selector.rs` pendiente
  - `LoaderOutcome::NeedsReboot` sin uso real
  - flags de `rustfmt` incompatibles con stable
  - `expect()` en produccion en `ps_script.rs`
- Objetivo:
  - cerrar deuda visible de baja escala antes de `1.0.0`
- Opciones a elegir:
  - `A` Corregir estos puntos al final como paquete unico
  - `B` Corregir cada punto dentro del paquete principal que lo toque
  - `C` Posponer para `1.0.1`
- Sugerencia tecnica:
  - `B`
- Aprobacion requerida:
  - elegir opcion `A`, `B` o `C`
- Criterio de cierre:
  - no quedan placeholders/dead branches evidentes en el baseline release

## Orden recomendado de ejecucion

1. Definir si `1.0.0` estable queda gated por este plan completo o por un subconjunto aprobado
2. Cerrar `WP-01`
3. Cerrar `WP-03`
4. Cerrar `WP-02`
5. Cerrar `WP-04`
6. Cerrar `WP-05`
7. Cerrar `WP-06`
8. Cerrar `WP-07`
9. Cerrar `WP-08`
10. Cerrar `WP-09`
11. Cerrar `WP-10`

Decision ya tomada:

- `1.0.0` estable queda gated por el plan completo
- Se continua con releases RC hasta completar la remediacion

## Protocolo de sesion mientras el plan siga activo

Al inicio:

1. Resumir que paquetes siguen abiertos
2. Aclarar si el analisis usa estado publicado o local no commitado
3. Pedir decision del usuario sobre el siguiente paquete/opcion si aun no existe

Durante la implementacion:

1. Tocar solo el paquete aprobado
2. No mezclar reparaciones no aprobadas
3. Validar inmediatamente el alcance editado

Al cierre:

1. Informar cambios, validaciones y riesgos residuales
2. Esperar confirmacion del usuario para cambiar el estado del paquete
3. Actualizar este plan y el resumen de sesion
