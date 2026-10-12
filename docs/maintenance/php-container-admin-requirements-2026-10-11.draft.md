# Plan maestro: WSDD-PHP-Admin

Fecha: 2026-10-11

## Vision

Crear WSDD-PHP-Admin: una aplicacion web instalable programaticamente dentro de un contenedor PHP para administrar extensiones y configuracion PHP del mismo contenedor, sin comandos manuales del usuario final y sin depender de un panel externo que controle contenedores propios.

WSDD-PHP-Admin debe resolver el problema de drift entre desarrollo, instalacion local y contenedores reconstruidos: toda extension o ajuste aplicado desde la interfaz debe quedar representado en un perfil persistente y debe poder restaurarse automaticamente despues de una reconstruccion o recreacion del contenedor.

Aunque el primer consumidor sera WSDD, el diseno debe ser compatible con cualquier imagen PHP razonablemente estandar, incluyendo imagenes basadas en Apache, FPM o CLI, siempre que el contenedor incluya o permita instalar las herramientas necesarias.

## Objetivo del proyecto

Desarrollar WSDD-PHP-Admin como administrador web de extensiones/configuracion PHP con:

- interfaz web clara y usable;
- instalacion programatica dentro del contenedor PHP;
- deteccion del runtime PHP disponible;
- instalacion, activacion y desactivacion de extensiones;
- perfil persistente montable con Docker bind mount o volumen;
- reconciliacion automatica del estado al iniciar el contenedor;
- auditoria de cambios;
- separacion entre configuracion base del contenedor y configuracion administrada por WSDD-PHP-Admin.

## Alcance inicial

El alcance inicial de WSDD-PHP-Admin es administrar extensiones PHP y configuracion PHP del contenedor donde la aplicacion esta instalada.

El proyecto no debe intentar administrar contenedores externos, crear proyectos web, reemplazar Docker Compose, reemplazar WSDD, reemplazar Webmin ni convertirse en un panel de hosting completo.

## Versiones PHP objetivo para WSDD

WSDD debe poder instalar WSDD-PHP-Admin sobre contenedores PHP de estas versiones:

- PHP 5.6
- PHP 7.2
- PHP 7.4
- PHP 8.0 (nueva aun WSDD no tiebne presente esta version)
- PHP 8.1
- PHP 8.2
- PHP 8.3
- PHP 8.4
- PHP 8.5 (nueva aun WSDD no tiebne presente esta version)

Nota: WSDD actualmente soporta PHP 5.6, 7.2, 7.4, 8.1, 8.2, 8.3 y 8.4. PHP 8.0 y PHP 8.5 quedan definidos como versiones objetivo adicionales para esta linea de trabajo.

## Caracteristicas funcionales

### 1. Aplicacion web

WSDD-PHP-Admin debe ser una aplicacion web, no solo una CLI o scripts sueltos. La UI/UX es critica: el usuario debe poder entender el estado del contenedor, las extensiones disponibles y las acciones posibles sin conocer comandos internos.

Caracteristicas esperadas:

- dashboard de estado PHP;
- listado de extensiones instaladas;
- listado de extensiones disponibles;
- filtros por estado;
- acciones claras para instalar, activar, desactivar y revertir;
- vista de logs de instalacion;
- indicacion visual de operaciones en progreso, exito o error.

### 2. Instalacion programatica dentro del contenedor PHP

WSDD debe poder instalar WSDD-PHP-Admin durante el build del contenedor PHP. La instalacion debe ocurrir despues de que PHP ya esta disponible dentro del contenedor.

La instalacion no debe requerir intervencion manual del usuario final.

### 3. Administracion del mismo contenedor

WSDD-PHP-Admin debe administrar el contenedor donde esta instalado. No debe depender de contenedores anidados ni de controlar Docker desde dentro del contenedor como requisito base.

El modelo correcto es:

- WSDD-PHP-Admin corre dentro del contenedor PHP;
- detecta el PHP local;
- instala o activa extensiones para ese PHP local;
- registra el estado en un perfil persistente.

### 4. Administracion de extensiones PHP

WSDD-PHP-Admin debe permitir:

- listar extensiones cargadas por `php -m`;
- listar extensiones instaladas pero desactivadas;
- listar extensiones disponibles para instalar;
- instalar extensiones faltantes;
- activar extensiones instaladas;
- desactivar extensiones instaladas;
- mostrar errores de instalacion;
- mostrar dependencias requeridas cuando se conozcan;
- verificar que una extension quedo cargada realmente.

### 5. Cero comandos manuales para el usuario final

El usuario final no debe ejecutar manualmente:

- `docker-php-ext-install`;
- `docker-php-ext-enable`;
- `pecl install`;
- `apt install`;
- `apk add`;
- scripts internos del contenedor.

WSDD-PHP-Admin puede usar estas herramientas internamente si existen y si el perfil persistente registra lo que se hizo.

### 6. Perfil persistente

WSDD-PHP-Admin debe mantener un perfil persistente en un directorio montado por volumen Docker o bind mount.

El perfil debe ser la fuente de verdad del estado deseado, no solo un log historico.

Debe registrar como minimo:

- version PHP;
- extension;
- estado deseado: activa, desactivada o removida;
- estado observado;
- metodo de instalacion;
- version instalada si aplica;
- dependencias del sistema requeridas;
- origen de la extension: core PHP, PECL, paquete externo, binario, fuente;
- fecha de instalacion;
- fecha de ultima modificacion;
- resultado de ultima reconciliacion.

### 7. Supervivencia a reconstrucciones

Si el contenedor se reconstruye al agregar proyectos, actualizar WSDD o recrear servicios, el perfil debe permanecer en el host y volver a montarse en el nuevo contenedor.

WSDD-PHP-Admin no debe depender del filesystem efimero del contenedor para recordar cambios.

### 8. Reconciliacion automatica

Al iniciar el contenedor, WSDD-PHP-Admin debe leer el perfil persistente y comparar:

- estado deseado;
- estado real del PHP local;
- extensiones instaladas;
- archivos `.ini`;
- dependencias disponibles.

Si falta algo que el perfil declara como requerido, debe reinstalarlo o reactivarlo automaticamente, dejando log de la operacion.

### 9. Compatibilidad con multiples imagenes PHP

El diseno debe intentar funcionar con cualquier imagen PHP compatible, no solo con `php:<version>-apache`.

WSDD-PHP-Admin debe detectar:

- ruta del binario PHP;
- version PHP;
- ruta de `php.ini`;
- `scan dir` de archivos `.ini`;
- si existen herramientas oficiales como `docker-php-ext-install` y `docker-php-ext-enable`;
- si existe PECL;
- sistema operativo base;
- gestor de paquetes disponible: `apt`, `apk`, `dnf`, `yum` u otro;
- tipo de runtime: Apache module, FPM o CLI.

### 10. Integracion con WSDD sin romper arquitectura

WSDD-PHP-Admin debe integrarse con WSDD sin exigir contenedores anidados ni migracion inmediata a PHP-FPM.

Debe funcionar con el modelo actual de WSDD basado en contenedores PHP Apache, por ejemplo `php:8.4-apache`, y dejar abierta la posibilidad de soportar FPM si WSDD lo adopta en el futuro.

### 11. Separacion de configuracion base y configuracion administrada

WSDD-PHP-Admin no debe sobrescribir archivos que WSDD modifica normalmente al agregar proyectos, vhosts o dominios.

Debe operar en zonas propias, por ejemplo:

- directorio persistente propio;
- archivos `.ini` administrados por WSDD-PHP-Admin;
- logs propios;
- metadata propia;
- cache propia.

### 12. Prevencion de drift manual

Todo cambio realizado desde la UI debe quedar registrado en el perfil persistente.

El estado del contenedor no debe depender de cambios manuales invisibles hechos dentro de una sesion interactiva.

### 13. Mantenibilidad

No se debe resolver el problema principalmente agregando extensiones una por una hardcodeadas en el Dockerfile.

El Dockerfile puede instalar prerequisitos base de WSDD-PHP-Admin, pero el set final de extensiones debe provenir del perfil persistente.

### 14. Seguridad

WSDD-PHP-Admin tendra permisos sensibles porque podra instalar extensiones y modificar configuracion PHP.

Debe incluir:

- autenticacion;
- control de sesion;
- proteccion CSRF;
- restriccion de acceso por red cuando aplique;
- logs de auditoria;
- validacion estricta de nombres de extensiones;
- bloqueo de comandos arbitrarios desde la UI;
- separacion entre acciones permitidas y comandos internos.

### 15. Rollback y recuperacion

WSDD-PHP-Admin debe permitir recuperar el contenedor ante una extension o configuracion problematica.

Capacidades deseadas:

- desactivar una extension;
- restaurar archivo `.ini` anterior;
- marcar extension como no deseada;
- registrar ultimo estado funcional conocido;
- permitir reconciliacion en modo seguro.

### 16. Auditoria

Debe registrar:

- usuario o actor;
- accion realizada;
- extension/configuracion afectada;
- fecha;
- resultado;
- salida relevante del instalador;
- estado antes y despues.

### 17. Extensibilidad

El diseno debe permitir agregar:

- nuevas versiones PHP;
- nuevos metodos de instalacion;
- nuevas familias de sistema operativo;
- nuevos tipos de runtime;
- nuevas fuentes de extensiones;
- integracion con WSDD;
- integracion futura con otros orquestadores.

## Arquitectura propuesta

### Componentes principales

1. **Web UI**

   Interfaz de usuario para consultar estado, ejecutar acciones y revisar logs.

2. **API interna**

   Endpoints usados por la UI para listar, instalar, activar, desactivar, reconciliar y consultar logs.

3. **Detector de entorno**

   Modulo que detecta version PHP, rutas de configuracion, runtime, sistema operativo, gestor de paquetes y herramientas disponibles.

4. **Catalogo de extensiones**

   Fuente de datos local/remota con extensiones soportadas, metodos de instalacion y dependencias conocidas por version PHP y sistema operativo.

5. **Motor de instalacion**

   Ejecuta operaciones internas de instalacion usando el metodo apropiado para la imagen detectada.

6. **Motor de reconciliacion**

   Compara perfil persistente contra estado real y aplica cambios pendientes al iniciar o bajo demanda.

7. **Perfil persistente**

   Archivo o conjunto de archivos versionados que describen el estado deseado y el historial de cambios.

8. **Auditoria y logs**

   Registro de acciones y salida de procesos de instalacion/reconciliacion.

## Perfil persistente esperado

El formato exacto debe definirse durante el diseno tecnico, pero debe ser legible, versionado y facil de respaldar.

Ejemplo conceptual:

```json
{
  "schema_version": 1,
  "php": {
    "version": "8.4",
    "runtime": "apache",
    "ini_scan_dir": "/usr/local/etc/php/conf.d"
  },
  "extensions": {
    "bcmath": {
      "desired_state": "enabled",
      "install_method": "docker-php-ext-install",
      "installed_version": null,
      "last_reconciled_at": "2026-10-11T00:00:00Z"
    }
  }
}
```

## Fases de desarrollo

### Fase 1: investigacion tecnica

- Definir imagenes PHP objetivo.
- Comparar Apache, FPM y CLI.
- Identificar herramientas disponibles por imagen.
- Definir formato inicial del perfil persistente.
- Definir modelo de seguridad minimo.

### Fase 2: prototipo local

- Crear aplicacion minima.
- Detectar PHP local.
- Leer `php -m`.
- Detectar `php.ini` y `conf.d`.
- Mostrar extensiones instaladas desde UI.
- Guardar perfil persistente.

### Fase 3: instalacion y reconciliacion

- Instalar extensiones desde UI.
- Activar/desactivar extensiones.
- Registrar cambios en perfil.
- Ejecutar reconciliacion al inicio.
- Validar reconstruccion de contenedor sin perdida de estado.

### Fase 4: integracion WSDD

- Agregar instalacion programatica en templates PHP de WSDD.
- Montar directorio persistente.
- Exponer ruta/dominio interno de administracion.
- Evitar conflicto con vhosts, Webmin, cron y configuracion existente.

### Fase 5: endurecimiento

- Autenticacion.
- CSRF.
- Logs de auditoria.
- Validacion de entradas.
- Modo seguro.
- Rollback basico.

### Fase 6: soporte multi-version

- Validar PHP 5.6, 7.2, 7.4, 8.0, 8.1, 8.2, 8.3, 8.4 y 8.5.
- Documentar diferencias por version.
- Ajustar catalogo de extensiones por version.

## Criterios de aceptacion iniciales

1. WSDD-PHP-Admin se instala programaticamente dentro de un contenedor PHP.
2. La UI muestra version PHP, runtime y extensiones cargadas.
3. La UI permite instalar al menos una extension faltante.
4. La UI permite activar/desactivar una extension instalada.
5. Cada cambio queda guardado en perfil persistente.
6. Al reconstruir el contenedor, el perfil se conserva.
7. Al iniciar el contenedor reconstruido, WSDD-PHP-Admin restaura automaticamente las extensiones declaradas.
8. WSDD puede agregar un proyecto nuevo sin borrar el perfil ni romper la configuracion administrada.
9. No se requiere ejecutar comandos manuales dentro del contenedor.
10. WSDD-PHP-Admin funciona inicialmente sobre PHP 8.4 Apache y queda preparado para extenderse a otras versiones.

## Riesgos tecnicos

- Algunas extensiones requieren dependencias del sistema diferentes segun distribucion base.
- PHP 5.6 y 7.x pueden requerir rutas, repositorios o versiones de dependencias antiguas.
- No todas las imagenes PHP incluyen las mismas herramientas auxiliares.
- Instalar extensiones en runtime puede aumentar tiempo de arranque si hay reconciliacion pendiente.
- Algunas extensiones pueden requerir recompilar o reiniciar servicios.
- WSDD-PHP-Admin tendra permisos sensibles dentro del contenedor.
- El soporte universal para "cualquier imagen PHP" debe manejarse por niveles de compatibilidad, no como promesa absoluta.

## Decision de producto

WSDD-PHP-Admin debe tratarse como un proyecto propio, no como una integracion directa de OpenPanel, Webmin, PIE o `docker-php-extension-installer`.

Esos proyectos pueden servir como referencia o backend tecnico puntual, pero ninguno cumple por si solo el modelo requerido:

- aplicacion web dentro del contenedor PHP;
- administracion del mismo contenedor;
- perfil persistente declarativo;
- reconciliacion automatica tras rebuild;
- compatibilidad con la arquitectura actual de WSDD.

