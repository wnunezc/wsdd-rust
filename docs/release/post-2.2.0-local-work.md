# Trabajo local posterior a v2.2.0

Este registro conserva la preparación local posterior a `v2.2.0`. El código y los recursos se incorporan a `v2.3.0`; las notas públicas están en `2.3.0-draft.md`. La conversión del datadir y su backup son operaciones locales y no se incluyen en el MSI.

## MySQL Test

- Ensayo aislado de un contenedor con principal Linux efímero y réplica NTFS persistente; controles manuales Deploy, Validate y Recreate, etapas y log.
- Correcciones posteriores a las pruebas del usuario: longitud del secreto de replicación, lectura de `SHOW REPLICA STATUS`, relay logs y comparación del estado SQL restaurado. Detalle: `../maintenance/mysql-test-single-container-replication-2026-10-20.md`.

## MySQL Core de producción

- Conversión lógica local de los datos actuales de `lower_case_table_names=2` a `1`, con informe y backup aislados fuera del repositorio. La ruta persistente original contiene el datadir convertido.
- Servicio `wsdd-core/database` adaptado a dos instancias MySQL 26.7.0 en un contenedor, con principal efímero y réplica en el bind NTFS original. Se mantienen el nombre de contenedor, puerto 3306, red, phpMyAdmin y datos de usuario.
- Controles manuales Docker → MySQL Core, salud de ambas instancias, testigo y hash SQL para la recreación; sincronización de recursos y ZIP embebido; guardas que rechazan iniciar un contenedor legado sobre el datadir convertido.
- Parada con espera de réplica y marcador solo tras apagado limpio. Backup completo permitido únicamente con MySQL detenido limpiamente; restauración completa bloqueada para este layout hasta disponer de un procedimiento atómico verificado.
- El usuario instaló el MSI local y el Core arrancó `healthy`. Validaciones SQL antes y después de una carga de 51,2 MB pasaron; se verificaron ambas copias y se retiró la base temporal de medición. El usuario ejecutó `Recreate` desde WSDD: restauración y hash SQL confirmados, réplica al día. Siguen pendientes parada/arranque ordinarios y prueba de una aplicación real. Cifras y límites: `../maintenance/mysql-core-replicated-datadir-cutover-2026-10-20.md`.
