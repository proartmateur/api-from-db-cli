# Arquitectura del TUI

## Objetivo

Este documento describe la arquitectura actual del TUI de `api-from-db-cli`, sus responsabilidades principales y la forma en que se reparte la lógica entre módulos.

La intención es que el TUI sea:

- mantenible
- fácil de extender
- entendible para futuros refactors
- alineado con separación de responsabilidades

## Vista General

Actualmente el TUI está organizado en estas capas:

- `src/ui/state.rs`
- `src/ui/screens/`
- `src/ui/handlers/`
- `src/ui/use_cases/`
- `src/ui/navigation.rs`
- `src/ui/mocks/`
- `src/ui/tui.rs`

Idea principal:

- `tui.rs` coordina
- `state.rs` modela el estado
- `screens/` dibuja
- `handlers/` responde a teclas
- `use_cases/` ejecuta lógica de flujo
- `navigation.rs` resuelve navegación simple
- `mocks/` concentra datos simulados

## Responsabilidades por Módulo

### `src/ui/tui.rs`

Es el punto de entrada del TUI.

Responsabilidades:

- inicializar estado
- levantar y restaurar terminal
- correr el event loop
- despachar teclas al handler correcto
- despachar render al screen correcto
- ofrecer algunos helpers compartidos de UI

No debería contener lógica de negocio compleja.

Hoy todavía conserva:

- construcción del footer
- render de listas/utilidades compartidas
- armado visual de salida del proceso
- acceso al `generator_config_for_current_flow`

### `src/ui/state.rs`

Centraliza el modelo de estado del flujo.

Contiene:

- `Screen`
- `ConnectionSource`
- `EngineOption`
- `SoftDeleteStrategy`
- `SqlPreviewAction`
- `CommandPreviewAction`
- `CatalogMode`
- `GeneratorBinaryState`
- `Catalog`
- `AppState`
- estados locales de cada pantalla

Esta capa define “qué sabe el TUI”.

No debe conocer render ni interacción con terminal.

### `src/ui/screens/`

Cada archivo representa una pantalla o vista renderizable.

Responsabilidad:

- transformar estado en widgets de `ratatui`

No deberían:

- mutar estado
- ejecutar casos de uso
- decidir navegación

Pantallas actuales:

- `connection_source.rs`
- `engine_select.rs`
- `object_explorer.rs`
- `object_details.rs`
- `soft_delete_strategy.rs`
- `manual_soft_delete_field.rs`
- `sql_preview.rs`
- `command_preview.rs`
- `process_result.rs`

### `src/ui/handlers/`

Cada archivo representa la reacción a teclado para una pantalla.

Responsabilidad:

- interpretar `KeyCode`
- actualizar estado de navegación
- invocar casos de uso cuando se necesita lógica real
- aplicar el resultado del caso de uso al `AppState`

No deberían:

- renderizar
- contener demasiada lógica de negocio

### `src/ui/use_cases/`

Es la capa semántica del flujo del TUI.

Aquí viven operaciones con intención clara de negocio o de interacción:

- `generator_validation.rs`
- `config_file_connection.rs`
- `table_preview.rs`
- `generator_command.rs`
- `soft_delete_resolution.rs`
- `soft_delete_inspection.rs`

Esta capa responde preguntas como:

- ¿el generador existe y está listo?
- ¿se debe crear el archivo de configuración o cargarlo?
- ¿cómo se obtiene el preview de una tabla?
- ¿qué pasa si el usuario elige cierta estrategia de soft delete?
- ¿qué columnas son candidatas para soft delete?
- ¿realmente hace falta pedir una decisión de soft delete?

Esto ayuda a que los handlers sean pequeños y legibles.

### `src/ui/navigation.rs`

Contiene navegación pequeña y reutilizable.

Ejemplos:

- `next_*`
- `previous_*`
- `select_next`
- `select_previous`

Su responsabilidad es mínima:

- mover índices
- rotar selecciones semánticas

No debe conocer lógica de negocio.

### `src/ui/mocks/`

Contiene todos los datos y salidas simuladas del TUI.

Incluye:

- catálogos mock
- tablas mock
- objetos mock
- resultado mock de ejecución externa

Esto evita contaminar `tui.rs` y los handlers con datos hardcodeados.

## Flujo de Ejecución

El flujo general del TUI es:

1. `TuiApp::run()` inicia terminal y event loop.
2. `event_loop()` espera eventos del teclado.
3. `handle_key()` decide qué handler usar según `state.screen`.
4. El handler:
   - actualiza selección simple, o
   - llama un caso de uso, o
   - cambia de pantalla
5. `draw()` decide qué screen renderizar según `state.screen`.
6. El screen lee `AppState` y dibuja la vista actual.

## Ejemplo de Flujo Real

Caso: usuario selecciona estrategia de soft delete.

1. La pantalla visible es `SoftDeleteStrategy`.
2. `handlers/soft_delete_strategy.rs` recibe `Enter`.
3. El handler llama `use_cases/soft_delete_resolution.rs`.
4. El caso de uso devuelve un outcome:
   - `ShowSqlPreview`
   - `ShowCommandPreview`
   - `RequestManualField`
   - `Error`
5. El handler aplica ese outcome al `AppState`.
6. `draw()` renderiza la nueva pantalla correspondiente.

Este patrón hoy es la dirección recomendada para nuevas funcionalidades.

## Relación con Otras Capas

El TUI se apoya en capas del proyecto ya existentes:

- `src/domain/`
- `src/app/`
- `src/core/`
- `src/adapters/`

### `domain`

Modela conceptos del sistema:

- conexiones
- tablas
- columnas
- tipos normalizados
- previews
- resultados de proceso

### `app`

Contiene lógica de negocio reusable:

- `generation_service`
- `type_mapper`
- `soft_delete`
- `ddl_builder`
- `command_builder`

### `core`

Expone puertos y errores base.

Ejemplos:

- `MetadataExplorer`
- `ProcessRunner`

### `adapters`

Conecta con infraestructura real:

- SQL Server
- archivos de configuración
- procesos externos

## Principios Aplicados

La arquitectura actual intenta acercarse a:

- `SRP`: render, manejo de eventos, casos de uso y mocks están separados
- `DRY`: navegación y consultas simples ya no están repetidas
- `KISS`: cada módulo tiene una intención clara
- `CQS`: consultas como `soft_delete_inspection` no mutan estado; casos de uso de resolución sí producen outcomes
- `YAGNI`: todavía no se introdujo una capa extra de abstracción si el caso de uso actual no la necesita

## Estado Actual de la Descomposición

Ya extraído:

- estado del TUI
- screens
- handlers
- navigation
- mocks
- validación del generador
- carga de configuración
- carga de preview de tabla
- ejecución del generador
- resolución de soft delete
- inspección de soft delete

Todavía en `tui.rs`:

- event loop
- dispatch principal
- helpers visuales compartidos
- `generator_config_for_current_flow`
- renderizado de texto del resultado del proceso

## Dirección Recomendada para Próximos Cambios

Cuando se agregue una nueva funcionalidad al TUI, la secuencia recomendada es:

1. agregar o ajustar estado en `state.rs`
2. crear o extender un `use_case` si hay lógica real
3. conectar la acción desde un `handler`
4. renderizar el resultado en un `screen`
5. dejar `tui.rs` solo como coordinador

## Criterios para Nuevos Casos de Uso

Conviene crear un archivo en `use_cases/` cuando:

- la lógica ya no cabe cómodamente en un handler
- se reutiliza desde más de una pantalla
- necesita hablar con `adapters`, `app` o `domain`
- produce outcomes claros del flujo
- mejora la lectura del handler

No conviene crear un caso de uso cuando:

- solo se mueve un índice
- solo se cambia una selección local
- solo se transforma directamente un dato visual para render

## Resumen

La arquitectura actual del TUI ya no depende de un único archivo monolítico.

Hoy el diseño favorece:

- coordinación central en `tui.rs`
- estado explícito en `state.rs`
- render aislado en `screens/`
- interacción aislada en `handlers/`
- lógica del flujo en `use_cases/`
- utilidades pequeñas en `navigation.rs`
- datos simulados en `mocks/`

Esta estructura debe servir como base para seguir creciendo el proyecto sin volver a concentrar demasiada responsabilidad en una sola unidad.
