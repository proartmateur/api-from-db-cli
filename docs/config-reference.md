# Referencia del archivo de configuración

El archivo de configuración permite definir los datos de conexión y la configuración del generador externo sin tener que capturarlos manualmente cada vez.

## Ubicación

Coloca el archivo en el directorio desde donde ejecutas la herramienta. La TUI lo busca con el nombre que hayas definido al seleccionar **Archivo de configuración**.

## Estructura

```json
{
  "id": "string",
  "name": "string",
  "engine": "postgresql | sqlserver",
  "host": "string | null",
  "port": "number | null",
  "database": "string | null",
  "username": "string | null",
  "password": "string | null",
  "connection_string": "string | null",
  "gen": {
    "cmd": "string",
    "flags": ["string"]
  }
}
```

## Campos

| Campo | Tipo | Requerido | Descripción |
|-------|------|-----------|-------------|
| `id` | string | Sí | Identificador único de la configuración |
| `name` | string | Sí | Nombre descriptivo que se muestra en la TUI |
| `engine` | string | Sí | Motor de base de datos: `postgresql` o `sqlserver` |
| `host` | string | Condicional | Host o IP del servidor. Requerido si no se usa `connection_string` |
| `port` | number | No | Puerto. Por defecto: `5432` para PostgreSQL, `1433` para SQL Server |
| `database` | string | Condicional | Nombre de la base de datos. Requerido si no se usa `connection_string` |
| `username` | string | Condicional | Usuario. Requerido si no se usa `connection_string` |
| `password` | string | Condicional | Contraseña. Requerido si no se usa `connection_string` |
| `connection_string` | string | No | Cadena de conexión completa. Si se provee, tiene precedencia sobre los campos individuales |
| `gen.cmd` | string | Sí | Ruta o nombre del ejecutable del generador |
| `gen.flags` | array | No | Flags adicionales que se pasan al generador antes del nombre de la tabla |

## Cómo se construye el comando

El comando generado tiene la forma:

```
{gen.cmd} {gen.flags...} {tabla} {campo1:tipo1,campo2:tipo2,...}
```

Por ejemplo, con `cmd: "./gen"` y `flags: ["--mvc"]` para una tabla `users`:

```
./gen --mvc users id:int,name:str,deleted_at:delete_at
```

## Tipos normalizados

Los tipos de la base de datos se normalizan antes de pasarlos al generador:

| Tipos SQL | Token del generador |
|-----------|---------------------|
| `integer`, `int`, `bigint`, `smallint` | `int` |
| `varchar`, `nvarchar`, `text`, `char` | `str` |
| `boolean`, `bit` | `bool` |
| `timestamp`, `datetime`, `datetime2` | `datetime` |
| `date` | `date` |
| `time` | `time` |
| `decimal`, `numeric` | `Decimal` |
| `float`, `real`, `double precision` | `float` |
| `json`, `jsonb` | `dict` |
| Campo de soft delete activo | `delete_at` |
| Tipo no reconocido | `str` (fallback) |

## Ejemplos completos

### PostgreSQL

```json
{
  "id": "PostgreSQL",
  "name": "PostgreSQL 33.12",
  "engine": "postgresql",
  "host": "172.16.33.12",
  "port": 5435,
  "database": "aleph",
  "username": "user",
  "password": "password",
  "connection_string": null,
  "gen": {
    "cmd": "./gen",
    "flags": ["--mvc"]
  }
}
```

### SQL Server

```json
{
  "id": "SqlServer",
  "name": "SQL Server Producción",
  "engine": "sqlserver",
  "host": "192.168.1.10",
  "port": 1433,
  "database": "mi_db",
  "username": "sa",
  "password": "contraseña",
  "connection_string": null,
  "gen": {
    "cmd": "gen.exe",
    "flags": []
  }
}
```

### Con connection string

```json
{
  "id": "SqlServerCS",
  "name": "SQL Server vía connection string",
  "engine": "sqlserver",
  "host": null,
  "port": null,
  "database": null,
  "username": null,
  "password": null,
  "connection_string": "Server=192.168.1.10,1433;Database=mi_db;User=sa;Password=contraseña;",
  "gen": {
    "cmd": "gen.exe",
    "flags": ["--mvc"]
  }
}
```
