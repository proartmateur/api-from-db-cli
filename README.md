# api-from-db-cli

Herramienta de terminal (TUI) que se conecta a una base de datos, explora su estructura y construye automáticamente el comando para generar una API con un generador externo (GEN CLI 2.1).

## ¿Qué hace?

1. Se conecta a PostgreSQL o SQL Server (manualmente o desde un archivo de configuración).
2. Lista tablas, funciones y stored procedures disponibles.
3. Lee el esquema de la tabla seleccionada (columnas, tipos, primary keys).
4. Resuelve la configuración de soft delete (`deleted_at`).
5. Construye y ejecuta el comando de tu generador externo con los tipos normalizados.

## Requisitos previos

- El binario `gen` (Linux/macOS) o `gen.exe` (Windows) debe existir en el directorio desde donde se ejecuta la herramienta.
- Acceso a una base de datos PostgreSQL o SQL Server.

## Instalación

```bash
cargo build --release
```

El binario queda en `target/release/api-from-db-cli`.

## Inicio rápido

```bash
./target/release/api-from-db-cli
```

Al iniciar, la herramienta verifica que el binario del generador exista. Si no lo encuentra, muestra un mensaje y no permite avanzar.

## Flujo de uso

```
1. Fuente de conexión    → Elegir: Archivo de configuración | Captura manual
2. Explorador de objetos → Navegar tablas, funciones y stored procedures
3. Detalle de tabla      → Ver columnas, tipos y primary keys
4. Soft delete           → Configurar o ignorar el campo deleted_at
5. Preview SQL           → Revisar el ALTER TABLE (si aplica)
6. Preview del comando   → Revisar el comando antes de ejecutarlo
7. Resultado             → Ver stdout, stderr y código de salida
```

## Opciones de conexión

### Archivo de configuración

Coloca un archivo `.config.json` en el directorio de trabajo y selecciona **Archivo de configuración** en la TUI. Si el archivo no existe, la herramienta genera una plantilla automáticamente.

Ejemplo para PostgreSQL:

```json
{
  "id": "mi-proyecto",
  "name": "Mi Base de Datos",
  "engine": "postgresql",
  "host": "localhost",
  "port": 5432,
  "database": "mi_db",
  "username": "usuario",
  "password": "contraseña",
  "connection_string": null,
  "gen": {
    "cmd": "./gen",
    "flags": ["--mvc"]
  }
}
```

Ver la [referencia completa del archivo de configuración](docs/config-reference.md).

### Captura manual

Ingresa host, puerto, base de datos, usuario y contraseña directamente en la TUI. La contraseña se oculta en pantalla.

## Soft delete

La herramienta detecta automáticamente si la tabla tiene un campo compatible con soft delete (tipo datetime nullable llamado `deleted_at` o similar). Si no existe, ofrece tres opciones:

| Opción | Descripción |
|--------|-------------|
| Crear `deleted_at` | Genera y muestra un `ALTER TABLE` para que lo revises antes de ejecutarlo |
| Seleccionar campo existente | Elige una columna datetime existente para usarla como campo de borrado lógico |
| Continuar sin delete | Genera el comando sin endpoint de delete |

## Teclas de navegación

| Tecla | Acción |
|-------|--------|
| `↑` / `↓` | Navegar lista |
| `Enter` | Confirmar selección |
| `Esc` | Volver a la pantalla anterior |
| `q` | Salir de la aplicación |
| `PgUp` / `PgDn` | Desplazar texto largo |
| `Home` / `End` | Ir al inicio / final del texto |

## Motores soportados

| Motor | Versión mínima recomendada |
|-------|---------------------------|
| PostgreSQL | 13+ |
| SQL Server | 2017+ |
