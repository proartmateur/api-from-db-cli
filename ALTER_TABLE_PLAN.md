# Plan: Ejecución real de ALTER TABLE para `deleted_at`

## Contexto

Hoy el flujo de soft delete en la TUI genera el `ALTER TABLE ... ADD deleted_at ...` correctamente (`app/ddl_builder.rs`), lo muestra en la pantalla `SqlPreview`, pero **nunca lo ejecuta**: `handlers/sql_preview.rs:23` solo transita de pantalla a `CommandPreview`. El `SqlExecutor` existe en ambos adapters (`SqlServerAdapter`, `PostgresAdapter`) pero no está cableado al flujo de la TUI. Tampoco hay confirmación explícita ni refresco de metadata post-DDL.

Este documento describe cómo cerrar el ciclo DDL de punta a punta cumpliendo RN-009, RN-012, RN-013, RN-014, RN-015, RNF-008.

## Requisitos cubiertos

- **RN-009**: ejecutar SQL solo si el usuario confirma, refrescar estructura, verificar que `deleted_at` existe, continuar solo si el campo fue verificado.
- **RN-012**: confirmación explícita escribiendo "ALTERAR". Enter/silencio/timeout no son confirmación.
- **RN-013**: copiar SQL al portapapeles si el entorno lo permite; si no, mostrarlo en pantalla; permitir ejecución manual + refresco posterior.
- **RN-014**: cancelación deja la DB intacta; permite regresar a selección de campo de soft delete.
- **RN-015**: en fallo mostrar mensaje, SQL ejecutado, motor, schema, tabla, detalle del error. No continuar como si el campo existiera.
- **RNF-008**: toda operación DDL requiere confirmación explícita.

## Paso a paso

### 1. Crear use case `ddl_execution` (`src/ui/use_cases/ddl_execution.rs`)

Input:
- `engine: DatabaseEngine`
- `connection_config: &ConnectionConfig`
- `generated_sql: &GeneratedSql`
- `selected: &DatabaseObject` (para schema + table)

Comportamiento:
1. Resolver adapter via `metadata_adapter_for(engine)`.
2. Llamar `adapter.execute_sql(connection_config, generated_sql.sql)`.
3. Si ok → reler tabla con `adapter.get_table_schema(connection_config, schema, table)`.
4. Verificar que existe una columna `deleted_at` con tipo datetime compatible (reusar `TypeMapper::analyze_table` + chequeo de `is_deleted_at_candidate`).
5. Devolver outcome.

Outcome:
```rust
pub(crate) enum DdlExecutionOutcome {
    Executed {
        refreshed_schema: TableSchema,
        message: String,
    },
    ExecutionError {
        sql: String,
        engine: DatabaseEngine,
        schema: String,
        table: String,
        error: String,
    },
    VerificationError {
        message: String, // ALTER ok pero deleted_at no aparecio
    },
}
```

Cumple RN-009 (ejecutar, refrescar, verificar) y RN-015 (datos del error).

### 2. Añadir estado de confirmación explícita en `state.rs`

`SqlPreviewScreenState` gana:
- `confirming: bool`
- `confirm_text: String`

Cuando `confirming == true`, el handler espera a que el usuario escriba "ALTERAR" (RN-012). Si escribe otra cosa, no ejecuta. `Esc` cancela la confirmación y vuelve al menu de acciones.

Cumple RN-012: Enter/silencio/timeout no son confirmacion.

### 3. Modificar `handlers/sql_preview.rs`

- `ExecuteAndContinue`: en lugar de transitar directo, pone `confirming = true`.
- Cuando `confirming == true`:
  - Teclas de texto editan `confirm_text`.
  - `Enter` compara con "ALTERAR":
    - Si coincide → llama al use case `ddl_execution`.
      - Si `Executed` → reconstruir preview con schema refrescado (paso 5), transitar a `CommandPreview`.
      - Si `ExecutionError` o `VerificationError` → mostrar error (RN-015), limpiar confirmacion, quedarse en `SqlPreview`.
    - Si no coincide → mensaje "Texto incorrecto. Escribe ALTERAR para confirmar.", limpiar `confirm_text`.
  - `Esc` sale de confirmacion sin ejecutar.
- `CopyAndContinue` y `CopyAndStay`: invoca `Clipboard::copy` (adapter real, ver seccion clipboard). Transita o se queda segun corresponda.
- `Cancel`: ya funciona, sin cambios.

### 4. Modificar `screens/sql_preview.rs`

Cuando `confirming == true`:
- Muestra el SQL.
- Muestra advertencia RN-012: "Esta operacion modificara la estructura de la tabla seleccionada. Revisa cuidadosamente el SQL antes de continuar."
- Muestra input de texto con lo que el usuario va escribiendo.
- Pista: "Escribe ALTERAR y presiona Enter para confirmar. Esc para cancelar."

Cuando `confirming == false`: render actual sin cambios.

### 5. Refrescar el preview con el schema verificado

Tras `ALTER TABLE` exitoso + verificacion:
1. Reconstruir `preview` con `GenerationService::preview_from_table` usando:
   - `refreshed_schema` (que ahora tiene `deleted_at` como columna real detectada).
   - `SoftDeletePreference::PreferDeleteEndpoint` (para que el resolver marque el campo).
   - `manually_selected_field = None` (ya esta detectado automaticamente).
2. El `generated_command` se regenera con `deleted_at:delete_at` como columna real (no como campo a crear).
3. `generated_sql` del nuevo preview sera `None` porque `field_was_created = false`.

### 6. Opcion: refresco manual despues de copiar SQL (RN-013)

Cuando el usuario elige `CopyAndStay` y despues ejecuta el SQL manualmente fuera de la TUI, necesita poder refrescar. Anadir una accion `RefreshMetadata` al menu de `SqlPreview`:
- Vuelve a leer la tabla via `MetadataExplorer::get_table_schema`.
- Verifica si `deleted_at` ya existe.
- Si existe → reconstruir preview y transitar a `CommandPreview`.
- Si no existe → advertencia "El campo deleted_at aun no existe en la tabla."

`SqlPreviewAction` gana una variante `RefreshMetadata` entre `CopyAndStay` y `Cancel`.

### 7. Tests

- Test del use case `ddl_execution` con mocks (no puedo probar contra DB real, pero puedo testear el flujo del outcome y el manejo de errores).
- Test de validacion de texto de confirmacion ("ALTERAR" vs otras entradas).
- `cargo build` + `cargo test` + `cargo clippy`.

## Pruebas con Docker (DB no productiva)

### PostgreSQL

Levantar contenedor:
```bash
docker run --name pg-test-ddl \
  -e POSTGRES_PASSWORD=test123 \
  -e POSTGRES_DB=testdb \
  -p 5433:5432 \
  -d postgres:16
```

Crear tabla de prueba sin `deleted_at`:
```bash
docker exec -i pg-test-ddl psql -U postgres -d testdb <<'SQL'
CREATE TABLE users (
  id SERIAL PRIMARY KEY,
  name VARCHAR(100) NOT NULL,
  email VARCHAR(200),
  created_at TIMESTAMP DEFAULT NOW()
);
CREATE TABLE products (
  id SERIAL PRIMARY KEY,
  name TEXT NOT NULL,
  price NUMERIC(10,2)
);
SQL
```

Configuracion manual en la TUI:
- Motor: PostgreSQL
- Host: localhost
- Port: 5433 (mapeado del contenedor)
- Database: testdb
- Username: postgres
- Password: test123

Flujo a probar:
1. Conectar manualmente → listar objetos → ver `users` y `products`.
2. Seleccionar `users` → ver columnas (id, name, email, created_at).
3. Elegir "Crear campo deleted_at" → ver SQL generado:
   ```sql
   ALTER TABLE "public"."users" ADD COLUMN "deleted_at" TIMESTAMP NULL;
   ```
4. Confirmar escribiendo "ALTERAR" → ejecucion real.
5. Verificar que `deleted_at` aparece en la tabla refrescada.
6. Verificar que el comando generado ahora incluye `deleted_at:delete_at`.
7. Repetir con `products` (sin columnas datetime previas) para verificar que no se ofrece seleccion manual de campo cuando no hay candidatos.
8. Probar cancelacion (Esc) → la DB queda intacta.
9. Probar confirmacion con texto incorrecto → no se ejecuta.
10. Probar copiar SQL (clipboard) → pegar en otra ventana y verificar contenido.
11. Probar refresco manual tras ejecucion manual fuera de la TUI.

Limpieza:
```bash
docker rm -f pg-test-ddl
```

### SQL Server

Levantar contenedor:
```bash
docker run --name sqlserver-test-ddl \
  -e "ACCEPT_EULA=Y" \
  -e "SA_PASSWORD=Test12345!" \
  -p 1434:1433 \
  -d mcr.microsoft.com/mssql/server:2022-latest
```

Crear tabla de prueba:
```bash
docker exec -i sqlserver-test-ddl /opt/mssql-tools18/bin/sqlcmd \
  -S localhost -U sa -P 'Test12345!' -C \
  -Q "CREATE DATABASE testdb;"
docker exec -i sqlserver-test-ddl /opt/mssql-tools18/bin/sqlcmd \
  -S localhost -U sa -P 'Test12345!' -C -d testdb \
  -Q "CREATE TABLE dbo.Users (Id INT IDENTITY(1,1) PRIMARY KEY, Name NVARCHAR(100) NOT NULL, Email NVARCHAR(200), CreatedAt DATETIME2 DEFAULT GETUTCDATE());"
```

Configuracion manual en la TUI:
- Motor: SQL Server
- Host: localhost
- Port: 1434
- Database: testdb
- Username: sa
- Password: Test12345!

Mismo flujo de pruebas que PostgreSQL, verificando que el SQL generado usa sintaxis SQL Server:
```sql
ALTER TABLE [dbo].[Users] ADD [deleted_at] DATETIME2 NULL;
```

Limpieza:
```bash
docker rm -f sqlserver-test-ddl
```

## Lo que NO se hara en este paso

- No se cambiara el event loop a async (la ejecucion DDL bloqueara temporalmente, igual que `process_runner` ya lo hace).
- No se anadira observabilidad/logging formal (deuda separada).
- No se migrara el clipboard a una libreria (ver seccion clipboard aparte).

## Orden de implementacion sugerido

1. Clipboard real (prerrequisito para probar el comando copiado en DB real).
2. Use case `ddl_execution`.
3. Estado de confirmacion + handler + screen.
4. Refresco manual (RN-013).
5. Tests unitarios.
6. Pruebas con Docker (PostgreSQL y SQL Server).