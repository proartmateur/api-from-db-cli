# Estado Actual del Proyecto

## Resumen

Este documento resume el estado actual de `api-from-db-cli` contra el requerimiento inicial descrito en `reqs.md`.

Convención usada:

- `Completado`: existe implementación usable en el proyecto actual.
- `Parcial`: existe implementación, pero no cubre todo el requerimiento o todavía depende de mocks / shortcuts.
- `Pendiente`: todavía no existe implementación funcional.

## Estado General

Situación actual del proyecto:

- Existe una base Rust con separación razonable entre `domain`, `app`, `core`, `adapters` y `ui`.
- Existe una TUI navegable con flujo principal de exploración y generación.
- Existe soporte real para SQL Server y PostgreSQL en conexión, listado de objetos y lectura de esquema de tabla.
- Existe soporte real para ejecución del comando del generador.
- La creación y carga del archivo de configuración ya existen.
- La configuración del generador externo ya puede venir desde `config.json` mediante `gen.cmd` y `gen.flags`.
- Existe un resolver de adapter (`adapters::metadata_adapter_for`) que elimina el hardcodeo de `SqlServerAdapter` en la UI; ahora cualquier motor soportado fluye por la misma tubería.

## Arquitectura y Base Técnica

### Completado

- Estructura modular del proyecto en `src/domain`, `src/core`, `src/app`, `src/adapters`, `src/ui`.
- Modelos principales del dominio:
  - `ConnectionConfig`
  - `GeneratorConfig`
  - `DatabaseObject`
  - `TableSchema`
  - `ColumnSchema`
  - `SoftDeleteConfig`
  - `GeneratedSql`
  - `GeneratedCommand`
  - `ProcessResult`
- Traits base del core:
  - `ConnectionProvider`
  - `MetadataExplorer`
  - `SqlExecutor`
  - `ProcessRunner`
  - `Clipboard`
- Lógica central separada de la UI:
  - `type_mapper`
  - `soft_delete`
  - `ddl_builder`
  - `command_builder`
  - `generation_service`

### Parcial

- La arquitectura está bien encaminada, pero la UI todavía conoce varios detalles del flujo y de los adapters concretos.
- No existe aún una capa de casos de uso más formal para cada historia de usuario; parte de la orquestación sigue en `tui.rs`.
- Ya existe un resolver de adapter por motor (`adapters::metadata_adapter_for`) que centraliza la seleccion de adapter y evita el hardcodeo en la UI, pero todavia faltan casos de uso formales para el resto del flujo.

## Requerimientos Funcionales

### 1. Fuente de conexión

#### Completado

- Opción de `Archivo de configuración` en la TUI.
- Si el archivo no existe, se genera automáticamente una plantilla JSON.
- Se muestra la ruta generada para que el usuario la edite manualmente.
- El archivo ya soporta:
  - conexión SQL Server
  - conexión PostgreSQL
  - `connection_string`
  - `gen.cmd`
  - `gen.flags`
- La TUI puede reintentar la carga del archivo al presionar `Enter`.
- La carga por archivo funciona para SQL Server y PostgreSQL usando el resolver de adapter.
- Opción de `Captura manual` en la TUI con formulario real (host, port, database, username, password) que construye un `ConnectionConfig` válido, prueba la conexión y lista objetos reales para SQL Server y PostgreSQL. La password se oculta en pantalla.

### 2. Motores soportados

#### Completado

- SQL Server:
  - prueba de conexión
  - listado de objetos
  - lectura de esquema de tabla
  - ejecución SQL
- PostgreSQL:
  - prueba de conexión
  - listado de objetos (schemas, tablas y funciones)
  - lectura de esquema de tabla
  - ejecución SQL
- Ambos motores fluyen por el mismo resolver de adapter, sin lógica de motor embebida en la UI.

### 3. Exploración de objetos

#### Completado

- SQL Server:
  - listado real de tablas
  - listado real de stored procedures
  - agrupación por schema a nivel de dato mostrado
- PostgreSQL:
  - listado real de tablas
  - listado real de funciones
  - agrupación por schema a nivel de dato mostrado
- En la TUI se pueden seleccionar objetos y navegar al detalle.

#### Parcial

- La TUI muestra metadata básica para funciones / stored procedures, pero la exploración detallada de funciones PostgreSQL todavía no está.

### 4. Lectura de estructura de tabla

#### Completado

- SQL Server:
  - lectura real de columnas
  - tipo SQL original
  - nulabilidad
  - posición ordinal
  - detección de primary key
- PostgreSQL:
  - lectura real de columnas
  - tipo PostgreSQL original (con tamaño/precisión cuando aplica)
  - nulabilidad
  - posición ordinal
  - detección de primary key
- La TUI ya consume esa metadata real cuando la tabla viene de cualquiera de los dos motores.

### 5. Normalización de tipos

#### Completado

- Existe `TypeMapper` para PostgreSQL y SQL Server.
- Ya hay mapeo orientado a tipos útiles para Python / SQLAlchemy.
- Casos ya contemplados:
  - `varchar(...)` / `nvarchar(...)` -> `str`
  - enteros -> `int`
  - booleanos / `bit` -> `bool`
  - timestamps / datetimes -> `datetime`
  - `json/jsonb` -> `dict`
  - `decimal/numeric` -> `Decimal`
- Hay tests unitarios de este módulo.

#### Parcial

- El catálogo de tipos todavía no está completo para todos los casos posibles de ambos motores.
- No existe aún una política interactiva para tipos desconocidos.

### 6. Soft delete

#### Completado

- Detección automática de `deleted_at` compatible.
- Selección manual de una columna datetime existente.
- Opción de continuar sin endpoint delete.
- Opción de crear `deleted_at`.
- Generación de SQL de `ALTER TABLE` para:
  - PostgreSQL
  - SQL Server
- Vista previa del SQL antes de continuar.
- Opción de copiar SQL y:
  - continuar al comando
  - quedarse en la pantalla

#### Parcial

- El `ALTER TABLE` real todavía no está conectado desde la TUI; hoy el flujo de confirmación sigue siendo principalmente de preview.
- No existe aún un paso real de “refrescar metadata después de ejecutar manualmente el ALTER TABLE”.

### 7. Generación del comando

#### Completado

- Construcción real del comando con:
  - `cmd`
  - `flags`
  - entidad
  - `campos:tipos`
- La configuración ya puede venir desde `config.json`.
- El preview del comando muestra:
  - `cmd`
  - `flags`
  - comando final
- Si una columna participa como soft delete, se envía con tipo `delete_at`.

#### Parcial

- No existe todavía validación de rutas del ejecutable configurado en `gen.cmd` más allá de la validación global de `gen` / `gen.exe` en la raíz.

### 8. Ejecución del generador externo

#### Completado

- `Ejecutar comando` ya corre el proceso real.
- Se captura:
  - `stdout`
  - `stderr`
  - `exit code`
  - `success`
- La pantalla de resultado ya soporta scroll con:
  - `↑/↓`
  - `PgUp/PgDn`
  - `Home/End`
- Sigue existiendo la opción de copiar el comando y marcar ejecución externa.

#### Parcial

- La TUI todavía no interpreta semánticamente la salida del generador; solo la muestra.
- No existe todavía una verificación posterior para confirmar que realmente se generó código en disco.

## Validaciones y UX en la TUI

### Completado

- Validación al abrir la TUI para buscar `gen` o `gen.exe` en la raíz del proyecto.
- Si no existe, el flujo no deja avanzar y muestra un mensaje claro.
- Los mensajes críticos del panel derecho se resaltan con estilo de alerta.
- El preview de resultados ya muestra logs completos navegables.

### Parcial

- El foco visual está mejorado, pero todavía puede crecer bastante la UX:
  - confirmaciones más explícitas
  - mejores estados intermedios
  - indicadores de conexión real vs mock más visibles

## Seguridad y Reglas de Negocio

### Completado

- El comando se ejecuta con `std::process::Command` y argumentos separados.
- La configuración del generador se normaliza desde archivo.
- Existe separación entre SQL preview y ejecución.
- Hay soporte para copiar SQL / comando sin forzar ejecución inmediata.

### Parcial

- Las credenciales siguen modeladas como `String`; todavía no se migran a un wrapper más seguro.
- No existe aún una estrategia formal de logs / redacción de secretos.
- La ejecución real de operaciones DDL desde la TUI todavía no está cerrada de punta a punta con confirmación persistente.

## Requerimientos No Funcionales

### RNF-001 / RNF-002: separación de lógica y reutilización del core

- `Parcial alto`
- La base va en muy buena dirección.
- La lógica principal ya no vive toda dentro de la UI.
- Aun así, la TUI todavía orquesta parte importante del flujo.

### RNF-003: seguridad de credenciales

- `Parcial bajo`
- Se evita imprimir contraseñas directamente en el flujo principal, pero todavía no hay una política robusta de manejo seguro de secretos.

### RNF-004: extensibilidad

- `Parcial alto`
- La estructura actual facilita agregar motores, pero falta completar PostgreSQL para validar que la abstracción realmente aguanta bien.

### RNF-005: configuración del generador externo

- `Completado`
- Ya existe soporte de `gen.cmd` y `gen.flags` desde archivo.

### RNF-006: observabilidad básica

- `Pendiente`
- Hoy no existe un sistema formal de logging / tracing de eventos relevantes.

### RNF-007: portabilidad

- `Parcial`
- La ejecución del proceso se hace con argumentos separados, lo cual ayuda.
- Falta validar de verdad diferencias entre Windows, Linux y macOS.

### RNF-008: confirmación de operaciones sensibles

- `Parcial`
- La TUI ya muestra preview y decisiones para SQL sensible.
- Falta completar el ciclo real de ejecución DDL con confirmación explícita y refresco.

### RNF-009: mensajes claros

- `Parcial alto`
- La TUI ya muestra mensajes bastante claros y visibles.
- Todavía se puede mejorar la consistencia de errores y siguientes acciones sugeridas.

## Estado por Épicas / Historias de Usuario

### Épica 1: Gestión de conexión

- `Completado`
- Hecho:
  - archivo de configuración
  - creación automática de plantilla
  - carga y validación
  - conexión real SQL Server
  - conexión real PostgreSQL
  - captura manual real (formulario TUI con validación)

### Épica 2: Exploración de objetos

- `Completado`
- Hecho:
  - SQL Server real
  - PostgreSQL real
  - TUI navegable

### Épica 3: Selección de objeto

- `Parcial alto`
- Hecho:
  - seleccionar tabla
  - seleccionar stored procedure / función y mostrar que no están soportados
- Falta:
  - metadata real para funciones PostgreSQL

### Épica 4: Lectura de estructura de tabla

- `Completado`
- Hecho:
  - SQL Server real
  - PostgreSQL real
  - type mapping

### Épica 5: Gestión de soft delete

- `Parcial alto`
- Hecho:
  - detección automática
  - selección manual
  - decisión de continuar sin delete
  - preview de SQL
- Falta:
  - ciclo real completo de ALTER TABLE + refresh

### Épica 6: Generación de comando

- `Completado`
- Ya existe construcción del comando, preview y configuración externa del generador.

### Épica 7: Ejecución del generador externo

- `Completado` para la ejecución base
- Ya corre el proceso real y muestra resultados.
- `Parcial` en validación post-generación.

### Épica 8: Manejo de errores

- `Parcial`
- Hay varios mensajes claros y se corrigieron algunos fallos importantes de integración SQL Server.
- Falta endurecer más validaciones y casos borde.

### Épica 9: Seguridad en operaciones estructurales

- `Parcial`
- Hay UX de preview y control.
- Falta la parte operacional completa del DDL real.

## Lo Más Importante que Falta

- Ejecutar `ALTER TABLE` real desde la TUI con confirmación completa.
- Refrescar metadata real después de crear `deleted_at`.
- Confirmar de forma más fuerte la generación efectiva de código en disco.
- Agregar observabilidad / logging real.
- Revisar seguridad de credenciales.

## Conclusión

El proyecto ya no está en etapa de solo prototipo. Actualmente tiene:

- una base arquitectónica usable,
- una TUI funcional,
- integración real con SQL Server y PostgreSQL,
- conexión por archivo y por captura manual real,
- generación real del comando,
- ejecución real del generador,
- y un flujo bastante sólido de preview y navegación.

El mayor hueco funcional hoy está en:

- cierre total del flujo DDL real para `deleted_at`.
