# Guía de contribución

## Requisitos del entorno

- **Rust stable** (se recomienda instalar con [rustup](https://rustup.rs/))
- **cargo** (incluido con rustup)
- En Linux: dependencias del sistema para el clipboard (`arboard`):
  ```bash
  # Ubuntu / Debian
  sudo apt install xclip
  # o
  sudo apt install xsel
  ```
- Acceso a una base de datos PostgreSQL o SQL Server para pruebas de integración.

## Compilar

```bash
# Debug
cargo build

# Release
cargo build --release
```

## Ejecutar

```bash
# Desde la raíz del proyecto (necesita gen o gen.exe en este directorio)
cargo run
# o
./target/release/api-from-db-cli
```

## Tests

```bash
# Todos los tests unitarios
cargo test

# Test específico
cargo test nombre_del_test

# Con output
cargo test -- --nocapture
```

Los tests unitarios actuales cubren `TypeMapper` y `CommandBuilder`. Están en los mismos archivos fuente (`src/app/type_mapper.rs`, `src/app/command_builder.rs`).

## Linter

```bash
cargo clippy
```

El proyecto no debe tener warnings de clippy en PR.

## Bases de datos de prueba con Docker

### PostgreSQL

```bash
docker run --name pg-test \
  -e POSTGRES_PASSWORD=test123 \
  -e POSTGRES_DB=testdb \
  -p 5433:5432 \
  -d postgres:16
```

Crear tabla de prueba:

```bash
docker exec -i pg-test psql -U postgres -d testdb <<'SQL'
CREATE TABLE users (
  id SERIAL PRIMARY KEY,
  name VARCHAR(100) NOT NULL,
  email VARCHAR(200),
  created_at TIMESTAMP DEFAULT NOW()
);
SQL
```

Configuración para la TUI (captura manual):
- Motor: PostgreSQL
- Host: `localhost`
- Port: `5433`
- Database: `testdb`
- Username: `postgres`
- Password: `test123`

Limpiar:
```bash
docker rm -f pg-test
```

### SQL Server

```bash
docker run --name sqlserver-test \
  -e "ACCEPT_EULA=Y" \
  -e "SA_PASSWORD=Test12345!" \
  -p 1434:1433 \
  -d mcr.microsoft.com/mssql/server:2022-latest
```

Crear base de datos y tabla:

```bash
docker exec -i sqlserver-test /opt/mssql-tools18/bin/sqlcmd \
  -S localhost -U sa -P 'Test12345!' -C \
  -Q "CREATE DATABASE testdb;"

docker exec -i sqlserver-test /opt/mssql-tools18/bin/sqlcmd \
  -S localhost -U sa -P 'Test12345!' -C -d testdb \
  -Q "CREATE TABLE dbo.Users (Id INT IDENTITY(1,1) PRIMARY KEY, Name NVARCHAR(100) NOT NULL, CreatedAt DATETIME2 DEFAULT GETUTCDATE());"
```

Configuración para la TUI (captura manual):
- Motor: SQL Server
- Host: `localhost`
- Port: `1434`
- Database: `testdb`
- Username: `sa`
- Password: `Test12345!`

Limpiar:
```bash
docker rm -f sqlserver-test
```

## Estructura del proyecto

```
src/
├── main.rs                    ← entry point: instancia TuiApp y llama run()
├── lib.rs                     ← expone módulos públicos
├── domain/
│   └── mod.rs                 ← todos los modelos de datos del sistema
├── core/
│   ├── ports.rs               ← traits: ConnectionProvider, MetadataExplorer, SqlExecutor, ProcessRunner, Clipboard
│   └── error.rs               ← AppError
├── app/
│   ├── generation_service.rs  ← orquestador principal: preview_from_table()
│   ├── type_mapper.rs         ← tipos SQL → NormalizedType (con tests)
│   ├── soft_delete.rs         ← resolución de soft delete
│   ├── ddl_builder.rs         ← generación de ALTER TABLE
│   └── command_builder.rs     ← construcción del comando (con tests)
├── adapters/
│   ├── mod.rs                 ← metadata_adapter_for() + trait MetadataAdapter
│   ├── postgres.rs            ← adaptador PostgreSQL
│   ├── sqlserver.rs           ← adaptador SQL Server
│   ├── clipboard.rs           ← adaptador de clipboard (arboard)
│   ├── process.rs             ← adaptador de proceso externo
│   └── config.rs              ← carga/serialización de ConnectionConfig
└── ui/
    ├── tui.rs                 ← coordinador: event loop, dispatch
    ├── state.rs               ← AppState y tipos de estado
    ├── navigation.rs          ← helpers de navegación (índices, selecciones)
    ├── mod.rs
    ├── screens/               ← render de cada pantalla
    ├── handlers/              ← manejo de teclas por pantalla
    ├── use_cases/             ← lógica de flujo con outcomes tipados
    └── mocks/                 ← datos simulados
```

## Convenciones

### Cuándo crear un use case

Crear un archivo en `use_cases/` cuando:
- La lógica no cabe cómodamente en un handler.
- Se reutiliza desde más de una pantalla.
- Necesita hablar con `adapters`, `app` o `domain`.
- Produce outcomes claros del flujo.

No crear un use case cuando:
- Solo se mueve un índice.
- Solo se cambia una selección local.
- Solo se transforma un dato visual para render.

### Outcomes tipados

Los use cases devuelven enums de outcome, no `bool` ni `Option`. Ejemplo:

```rust
pub(crate) enum SoftDeleteOutcome {
    ShowSqlPreview,
    ShowCommandPreview,
    RequestManualField,
    Error(String),
}
```

Esto hace que los handlers sean legibles y exhaustivos.

### Adapters

Siempre usar `adapters::metadata_adapter_for(engine)` desde la UI y los use cases. Nunca instanciar `PostgresAdapter` o `SqlServerAdapter` directamente fuera de `adapters/mod.rs`.

### Tests

Agregar tests unitarios en el mismo archivo fuente bajo `#[cfg(test)]`. Los tests de integración con DB real se hacen manualmente usando los contenedores Docker documentados arriba.
