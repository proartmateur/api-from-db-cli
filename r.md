# REQUIREMENTS.md

# Explorador de base de datos y lanzador de generador de APIs

## 1. Resumen del proyecto

El objetivo del proyecto es construir una herramienta que permita conectarse a bases de datos PostgreSQL o SQL Server, explorar sus objetos principales, seleccionar una tabla, leer su estructura y construir un comando para invocar un generador externo de APIs.

La herramienta deberá ayudar al usuario a generar APIs a partir de tablas existentes, detectando campos, tipos de datos y configuración necesaria para soft delete mediante un campo `deleted_at`.

La herramienta podrá implementarse inicialmente como CLI o TUI, pero su núcleo deberá estar desacoplado para permitir una futura interfaz gráfica.

---

## 2. Objetivo principal

Construir un software que permita:

1. Capturar datos de conexión o leerlos desde un archivo de configuración.
2. Conectarse a PostgreSQL o SQL Server.
3. Mostrar objetos disponibles en la base de datos.
4. Permitir seleccionar una tabla.
5. Leer columnas y tipos de datos.
6. Resolver la configuración de soft delete mediante `deleted_at`.
7. Generar un comando compatible con un generador externo de APIs.
8. Ejecutar el comando generado previa confirmación.
9. Mostrar el resultado de la generación.

---

## 3. Alcance funcional

La primera versión debe soportar:

* PostgreSQL.
* SQL Server.
* Captura manual de conexión.
* Carga de conexión desde archivo.
* Listado de tablas.
* Listado de schemas.
* Listado de funciones en PostgreSQL.
* Listado de stored procedures en SQL Server.
* Selección de tabla.
* Lectura de columnas.
* Lectura de tipos de datos.
* Detección automática de `deleted_at`.
* Selección manual de campo para soft delete.
* Creación segura de campo `deleted_at` mediante SQL revisable.
* Generación de comando para `gen.exe`.
* Ejecución del comando.
* Captura de salida estándar, salida de error y código de salida.

---

## 4. Fuera de alcance inicial

La primera versión no requiere:

* Editor SQL interactivo.
* Autocompletado SQL.
* Consola SQL avanzada.
* Diagramas entidad-relación.
* Comparación entre bases de datos.
* Migraciones complejas.
* Administración de usuarios, roles o permisos.
* Edición visual avanzada de templates.
* Generación completa desde stored procedures.
* Generación completa desde funciones.
* Clon visual de DataGrip.
* Persistencia avanzada de credenciales.
* Múltiples ventanas o interfaz gráfica compleja.

---

## 5. Actores

## 5.1 Usuario desarrollador

Persona que usa la herramienta para conectarse a una base de datos, seleccionar objetos, revisar metadata y generar APIs mediante un generador externo.

## 5.2 Sistema

Software encargado de gestionar conexiones, leer metadata, generar SQL auxiliar, construir comandos y ejecutar procesos externos.

## 5.3 Generador externo de APIs

Programa externo invocado por el sistema.

Formato base esperado:

```txt
gen.exe Tabla campo:tipo,campo2:tipo
```

Ejemplo:

```txt
gen.exe Usuario id:int,nombre:string,email:string,deleted_at:datetime
```

---

## 6. Motores soportados

## 6.1 PostgreSQL

El sistema deberá soportar exploración de:

* Schemas.
* Tablas.
* Funciones.

## 6.2 SQL Server

El sistema deberá soportar exploración de:

* Schemas.
* Tablas.
* Stored procedures.

---

## 7. Flujo principal

```txt
1. El usuario inicia el software.
2. El sistema pregunta si desea ingresar datos de conexión o usar archivo de configuración.
3. El usuario proporciona la información requerida.
4. El sistema intenta conectarse a la base de datos.
5. Si la conexión falla, se muestra un error y se permite reintentar.
6. Si la conexión es exitosa, el sistema lista los objetos disponibles.
7. El usuario selecciona una tabla, función o stored procedure.
8. Si selecciona una tabla:
   8.1 El sistema lee columnas y tipos.
   8.2 El sistema detecta si existe deleted_at compatible.
   8.3 Si no existe, pregunta cómo desea proceder.
   8.4 Si el usuario desea endpoint delete y no existe deleted_at, ofrece crear el campo.
   8.5 Si el usuario acepta crear el campo, muestra SQL ALTER TABLE antes de ejecutarlo.
   8.6 El usuario puede ejecutar, copiar o cancelar el SQL.
   8.7 Si se ejecuta el SQL, el sistema refresca la tabla y verifica deleted_at.
   8.8 El sistema genera el comando para gen.exe.
   8.9 El sistema muestra el comando antes de ejecutarlo.
   8.10 El usuario confirma o cancela.
   8.11 Si confirma, el sistema ejecuta el comando.
   8.12 El sistema espera a que termine.
   8.13 El sistema muestra salida, errores y resultado final.
9. Si selecciona función o stored procedure:
   9.1 El sistema muestra metadata básica.
   9.2 Si la generación desde ese objeto no está soportada, lo indica claramente.
```

---

## 8. Reglas de negocio

## RN-001: Fuente de conexión

El usuario podrá proporcionar datos de conexión mediante:

1. Captura manual.
2. Archivo de configuración.

---

## RN-002: Motores soportados

El sistema deberá soportar:

1. PostgreSQL.
2. SQL Server.

---

## RN-003: Exploración por motor

Para PostgreSQL, el sistema deberá mostrar:

* Schemas.
* Tablas.
* Funciones.

Para SQL Server, el sistema deberá mostrar:

* Schemas.
* Tablas.
* Stored procedures.

---

## RN-004: Lectura de tabla

Cuando el usuario seleccione una tabla, el sistema deberá leer:

* Nombre del schema.
* Nombre de la tabla.
* Columnas.
* Tipos de datos.
* Nulabilidad.
* Posición ordinal.
* Llaves primarias, si están disponibles.
* Posibles campos candidatos para soft delete.

---

## RN-005: Detección automática de `deleted_at`

El sistema deberá buscar una columna llamada:

```txt
deleted_at
```

El campo será considerado compatible si tiene un tipo de fecha/hora soportado por el motor.

Tipos compatibles esperados:

### PostgreSQL

* `timestamp`
* `timestamp without time zone`
* `timestamp with time zone`
* `timestamptz`

### SQL Server

* `datetime`
* `datetime2`
* `smalldatetime`
* `datetimeoffset`

---

## RN-006: Selección manual de campo para soft delete

Si no existe un campo `deleted_at` compatible, el sistema deberá preguntar si el usuario desea seleccionar manualmente una columna existente para soft delete.

El sistema solo deberá permitir seleccionar columnas con tipos compatibles de fecha/hora.

---

## RN-007: Decisión sobre endpoint delete

Si no hay campo compatible y el usuario no selecciona uno manualmente, el sistema deberá preguntar si desea que el API generado tenga endpoint `delete`.

Si el usuario responde que no, el sistema continuará sin soft delete.

Si el usuario responde que sí, el sistema deberá indicar que el endpoint `delete` requiere un campo `deleted_at`.

---

## RN-008: Campo requerido para soft delete

Para generar un endpoint `delete` basado en soft delete, debe existir un campo compatible.

El campo puede provenir de:

1. Detección automática.
2. Selección manual.
3. Creación de nuevo campo `deleted_at`.

---

## RN-009: Creación segura del campo `deleted_at`

Si el usuario desea endpoint `delete` y no existe campo compatible, el sistema deberá preguntar si desea crear el campo `deleted_at`.

Si el usuario acepta, el sistema deberá:

1. Generar el SQL necesario para alterar la tabla.
2. Mostrar el SQL antes de ejecutarlo.
3. Permitir copiar el SQL.
4. Permitir cancelar la operación.
5. Solicitar confirmación explícita antes de ejecutar.
6. Ejecutar el SQL solo si el usuario confirma.
7. Refrescar la estructura de la tabla después de ejecutar.
8. Verificar que el campo `deleted_at` existe.
9. Continuar con la generación solo si el campo fue verificado.

El sistema no debe ejecutar automáticamente un `ALTER TABLE`.

---

## RN-010: SQL generado para PostgreSQL

Para PostgreSQL, el sistema deberá generar un SQL equivalente a:

```sql
ALTER TABLE "schema"."tabla"
ADD COLUMN "deleted_at" TIMESTAMP NULL;
```

Ejemplo:

```sql
ALTER TABLE "public"."users"
ADD COLUMN "deleted_at" TIMESTAMP NULL;
```

Si se configura el uso de zona horaria, podrá usar:

```sql
ALTER TABLE "public"."users"
ADD COLUMN "deleted_at" TIMESTAMPTZ NULL;
```

---

## RN-011: SQL generado para SQL Server

Para SQL Server, el sistema deberá generar un SQL equivalente a:

```sql
ALTER TABLE [schema].[tabla]
ADD [deleted_at] DATETIME2 NULL;
```

Ejemplo:

```sql
ALTER TABLE [dbo].[Users]
ADD [deleted_at] DATETIME2 NULL;
```

---

## RN-012: Confirmación explícita para operaciones DDL

Antes de ejecutar cualquier operación estructural, el sistema deberá mostrar una advertencia.

Mensaje sugerido:

```txt
Esta operación modificará la estructura de la tabla seleccionada.
Revisa cuidadosamente el SQL antes de continuar.
```

Opciones requeridas:

```txt
[1] Ejecutar ALTER TABLE
[2] Copiar SQL
[3] Cancelar
```

El sistema no deberá interpretar Enter, silencio o timeout como confirmación.

---

## RN-013: Copia del SQL

Si el usuario selecciona copiar SQL:

1. El sistema deberá copiar el SQL al portapapeles si el entorno lo permite.
2. Si no es posible copiarlo, deberá mostrarlo en pantalla.
3. El sistema deberá permitir que el usuario lo ejecute manualmente.
4. Después, el usuario podrá solicitar refrescar la estructura de la tabla.
5. El sistema deberá verificar si `deleted_at` ya existe.

---

## RN-014: Cancelación de ALTER TABLE

Si el usuario cancela la operación:

1. No se ejecutará ningún SQL.
2. La base de datos permanecerá intacta.
3. El sistema deberá preguntar si desea continuar sin endpoint `delete`.
4. El sistema deberá permitir regresar a la selección de campo para soft delete.

---

## RN-015: Error al alterar la tabla

Si falla el `ALTER TABLE`, el sistema deberá mostrar:

* Mensaje de error.
* SQL ejecutado.
* Motor de base de datos.
* Schema.
* Tabla afectada.
* Código o detalle del error, si está disponible.

El sistema no deberá continuar como si el campo existiera.

---

## RN-016: Generación del comando

El sistema deberá construir un comando con el formato:

```txt
gen.exe Tabla campo:tipo,campo2:tipo
```

Ejemplo:

```txt
gen.exe Usuario id:int,nombre:string,email:string
```

Ejemplo con `deleted_at`:

```txt
gen.exe Usuario id:int,nombre:string,email:string,deleted_at:datetime
```

---

## RN-017: Vista previa del comando

Antes de ejecutar el comando, el sistema deberá mostrarlo completo al usuario y solicitar confirmación.

Opciones requeridas:

```txt
[1] Ejecutar comando
[2] Copiar comando
[3] Cancelar
```

---

## RN-018: Ejecución del generador externo

Si el usuario confirma, el sistema deberá:

1. Ejecutar el comando.
2. Esperar a que termine.
3. Capturar código de salida.
4. Capturar salida estándar.
5. Capturar salida de error.
6. Mostrar resultado final.

---

## RN-019: Comando posterior del generador

El generador externo puede ejecutar otro comando al finalizar.

El sistema deberá esperar el resultado final del proceso y mostrar cualquier mensaje relevante emitido por el generador.

---

## RN-020: No exponer credenciales

El sistema no deberá imprimir contraseñas en:

* Logs.
* Errores.
* Consola.
* Archivos de salida.
* Vista previa de comandos.

---

## 9. Modelo conceptual

## 9.1 ConnectionConfig

```ts
interface ConnectionConfig {
  id: string
  name: string
  engine: 'postgresql' | 'sqlserver'
  host?: string
  port?: number
  database?: string
  username?: string
  password?: string
  connectionString?: string
  configFilePath?: string
}
```

---

## 9.2 DatabaseObject

```ts
interface DatabaseObject {
  name: string
  schema?: string
  type: 'table' | 'stored_procedure' | 'function'
  engine: 'postgresql' | 'sqlserver'
}
```

---

## 9.3 TableSchema

```ts
interface TableSchema {
  schema: string
  name: string
  columns: ColumnSchema[]
  primaryKeys: string[]
}
```

---

## 9.4 ColumnSchema

```ts
interface ColumnSchema {
  name: string
  dbType: string
  normalizedType: string
  nullable: boolean
  ordinalPosition: number
  isPrimaryKey: boolean
  isDeletedAtCandidate: boolean
}
```

---

## 9.5 SoftDeleteConfig

```ts
interface SoftDeleteConfig {
  enabled: boolean
  field?: string
  fieldWasDetected: boolean
  fieldWasSelectedManually: boolean
  fieldWasCreated: boolean
}
```

---

## 9.6 GeneratedSql

```ts
interface GeneratedSql {
  engine: 'postgresql' | 'sqlserver'
  schema: string
  table: string
  sql: string
  operation: 'alter_table_add_deleted_at'
}
```

---

## 9.7 GeneratedCommand

```ts
interface GeneratedCommand {
  executable: string
  tableName: string
  arguments: string[]
  rawCommand: string
}
```

---

## 9.8 ProcessResult

```ts
interface ProcessResult {
  exitCode: number
  stdout: string
  stderr: string
  success: boolean
}
```

---

## 10. Contrato interno sugerido

Antes de generar el comando final, el sistema debería construir un objeto intermedio.

Ejemplo:

```json
{
  "engine": "postgresql",
  "schema": "public",
  "table": "users",
  "columns": [
    {
      "name": "id",
      "dbType": "integer",
      "normalizedType": "int",
      "nullable": false,
      "isPrimaryKey": true
    },
    {
      "name": "name",
      "dbType": "text",
      "normalizedType": "string",
      "nullable": false,
      "isPrimaryKey": false
    },
    {
      "name": "deleted_at",
      "dbType": "timestamp",
      "normalizedType": "datetime",
      "nullable": true,
      "isPrimaryKey": false
    }
  ],
  "softDelete": {
    "enabled": true,
    "field": "deleted_at",
    "fieldWasDetected": true,
    "fieldWasSelectedManually": false,
    "fieldWasCreated": false
  }
}
```

---

## 11. Normalización de tipos

El sistema debe convertir tipos específicos de cada motor a tipos esperados por el generador.

La tabla exacta de conversión deberá ser configurable o centralizada en un módulo `type_mapper`.

Ejemplo inicial:

| Motor      | Tipo DB     | Tipo normalizado |
| ---------- | ----------- | ---------------- |
| PostgreSQL | integer     | int              |
| PostgreSQL | bigint      | long             |
| PostgreSQL | text        | string           |
| PostgreSQL | varchar     | string           |
| PostgreSQL | boolean     | bool             |
| PostgreSQL | timestamp   | datetime         |
| PostgreSQL | timestamptz | datetime         |
| PostgreSQL | numeric     | decimal          |
| SQL Server | int         | int              |
| SQL Server | bigint      | long             |
| SQL Server | varchar     | string           |
| SQL Server | nvarchar    | string           |
| SQL Server | bit         | bool             |
| SQL Server | datetime    | datetime         |
| SQL Server | datetime2   | datetime         |
| SQL Server | decimal     | decimal          |
| SQL Server | numeric     | decimal          |

Si aparece un tipo desconocido, el sistema deberá:

1. Usar un tipo por defecto configurable.
2. Advertir al usuario.
3. Permitir continuar si el usuario acepta.

---

## 12. Épicas y escenarios Gherkin

# Épica 1: Gestión de conexión

Como usuario desarrollador,
quiero ingresar datos de conexión o indicar un archivo de configuración,
para que el sistema pueda conectarse a una base de datos soportada.

## Feature 1.1: Captura manual de conexión

```gherkin
Feature: Captura manual de datos de conexión
  Como usuario desarrollador
  Quiero ingresar manualmente los datos de conexión
  Para conectarme a una base de datos soportada

  Scenario: Conexión exitosa a PostgreSQL
    Given que el usuario selecciona el motor "PostgreSQL"
    And ingresa host, puerto, base de datos, usuario y contraseña
    When el usuario solicita probar la conexión
    Then el sistema debe intentar conectarse a la base de datos
    And debe mostrar un mensaje de conexión exitosa

  Scenario: Conexión exitosa a SQL Server
    Given que el usuario selecciona el motor "SQL Server"
    And ingresa host, puerto, base de datos, usuario y contraseña
    When el usuario solicita probar la conexión
    Then el sistema debe intentar conectarse a la base de datos
    And debe mostrar un mensaje de conexión exitosa

  Scenario: Error de conexión
    Given que el usuario ingresa datos de conexión inválidos
    When el usuario solicita probar la conexión
    Then el sistema debe mostrar un mensaje de error
    And no debe intentar listar objetos de base de datos
```

---

## Feature 1.2: Carga de conexión desde archivo

```gherkin
Feature: Carga de conexión desde archivo de configuración
  Como usuario desarrollador
  Quiero indicar la ruta de un archivo de configuración
  Para reutilizar conexiones sin capturarlas manualmente

  Scenario: Archivo de configuración válido
    Given que el usuario proporciona una ruta válida a un archivo de configuración
    And el archivo contiene los datos requeridos de conexión
    When el sistema lee el archivo
    Then debe cargar la configuración de conexión
    And debe permitir probar la conexión

  Scenario: Archivo inexistente
    Given que el usuario proporciona una ruta a un archivo inexistente
    When el sistema intenta leer el archivo
    Then debe mostrar un mensaje indicando que el archivo no existe

  Scenario: Archivo inválido
    Given que el usuario proporciona una ruta a un archivo de configuración
    And el archivo no contiene los campos requeridos
    When el sistema intenta cargar la configuración
    Then debe mostrar un mensaje indicando qué campos faltan o son inválidos
```

---

# Épica 2: Exploración de objetos

Como usuario desarrollador,
quiero visualizar tablas, schemas, funciones y stored procedures,
para seleccionar el objeto desde el cual se generará código.

## Feature 2.1: Listado de objetos en PostgreSQL

```gherkin
Feature: Listado de objetos en PostgreSQL
  Como usuario desarrollador
  Quiero ver schemas, tablas y funciones
  Para explorar la estructura de una base PostgreSQL

  Scenario: Listar schemas y tablas
    Given que el sistema está conectado a una base PostgreSQL
    When el sistema solicita los objetos disponibles
    Then debe mostrar un árbol de schemas
    And cada schema debe mostrar sus tablas
    And cada schema debe mostrar sus funciones cuando existan

  Scenario: Base PostgreSQL sin tablas de usuario
    Given que el sistema está conectado a una base PostgreSQL
    And no existen tablas de usuario
    When el sistema solicita los objetos disponibles
    Then debe mostrar un mensaje indicando que no se encontraron tablas de usuario
```

---

## Feature 2.2: Listado de objetos en SQL Server

```gherkin
Feature: Listado de objetos en SQL Server
  Como usuario desarrollador
  Quiero ver tablas y stored procedures
  Para explorar la estructura de una base SQL Server

  Scenario: Listar tablas y stored procedures
    Given que el sistema está conectado a una base SQL Server
    When el sistema solicita los objetos disponibles
    Then debe mostrar las tablas disponibles
    And debe mostrar los stored procedures disponibles

  Scenario: Listar objetos agrupados por schema
    Given que el sistema está conectado a una base SQL Server
    When el sistema solicita los objetos disponibles
    Then debe mostrar los objetos agrupados por schema
```

---

# Épica 3: Selección de objeto

Como usuario desarrollador,
quiero seleccionar una tabla, función o stored procedure,
para que el sistema pueda mostrar metadata y preparar la generación.

## Feature 3.1: Selección de tabla

```gherkin
Feature: Selección de tabla
  Como usuario desarrollador
  Quiero seleccionar una tabla
  Para leer sus campos y tipos de datos

  Scenario: Usuario selecciona una tabla válida
    Given que el sistema muestra un listado de tablas
    When el usuario selecciona una tabla
    Then el sistema debe leer la estructura de la tabla
    And debe mostrar sus campos
    And debe mostrar sus tipos de datos

  Scenario: Error al leer estructura de tabla
    Given que el usuario selecciona una tabla
    And ocurre un error al leer sus columnas
    When el sistema procesa la selección
    Then debe mostrar un mensaje de error
    And no debe generar el comando del generador
```

---

## Feature 3.2: Selección de stored procedure o función

```gherkin
Feature: Selección de stored procedure o función
  Como usuario desarrollador
  Quiero seleccionar un stored procedure o función
  Para inspeccionar su metadata

  Scenario: Usuario selecciona un stored procedure en SQL Server
    Given que el sistema muestra stored procedures disponibles
    When el usuario selecciona un stored procedure
    Then el sistema debe mostrar su nombre
    And debe mostrar su schema
    And debe indicar que la generación de API para stored procedures está pendiente si aún no está soportada

  Scenario: Usuario selecciona una función en PostgreSQL
    Given que el sistema muestra funciones disponibles
    When el usuario selecciona una función
    Then el sistema debe mostrar su nombre
    And debe mostrar su schema
    And debe indicar que la generación de API para funciones está pendiente si aún no está soportada
```

---

# Épica 4: Lectura de estructura de tabla

Como usuario desarrollador,
quiero que el sistema lea columnas y tipos de datos de una tabla,
para construir automáticamente los parámetros del generador de APIs.

## Feature 4.1: Lectura de columnas y tipos

```gherkin
Feature: Lectura de columnas y tipos de datos
  Como usuario desarrollador
  Quiero ver las columnas y tipos de una tabla
  Para validar la estructura antes de generar código

  Scenario: Lectura exitosa de columnas
    Given que el usuario selecciona una tabla existente
    When el sistema consulta la metadata de la tabla
    Then debe mostrar cada columna de la tabla
    And debe mostrar el tipo de dato de cada columna
    And debe mostrar si cada columna permite valores nulos

  Scenario: Tabla sin columnas legibles
    Given que el usuario selecciona una tabla existente
    And el sistema no puede obtener columnas
    When el sistema consulta la metadata de la tabla
    Then debe mostrar un mensaje indicando que no fue posible leer la estructura
    And no debe generar el comando del generador
```

---

## Feature 4.2: Normalización de tipos

```gherkin
Feature: Normalización de tipos de datos
  Como sistema
  Quiero convertir tipos específicos de base de datos a tipos normalizados
  Para construir un comando compatible con el generador de APIs

  Scenario: Normalizar tipo de PostgreSQL
    Given que una columna PostgreSQL tiene tipo "integer"
    When el sistema normaliza el tipo
    Then debe convertirlo al tipo esperado por el generador

  Scenario: Normalizar tipo de SQL Server
    Given que una columna SQL Server tiene tipo "datetime2"
    When el sistema normaliza el tipo
    Then debe convertirlo al tipo esperado por el generador

  Scenario: Tipo desconocido
    Given que una columna tiene un tipo no soportado
    When el sistema normaliza el tipo
    Then debe usar un tipo por defecto configurable
    And debe advertir al usuario que el tipo fue inferido
```

---

# Épica 5: Gestión de soft delete

Como usuario desarrollador,
quiero que el sistema detecte o configure un campo `deleted_at`,
para que el API generado pueda soportar eliminación lógica.

## Feature 5.1: Detección automática de `deleted_at`

```gherkin
Feature: Detección automática de deleted_at
  Como usuario desarrollador
  Quiero que el sistema detecte automáticamente el campo deleted_at
  Para evitar configuración manual

  Scenario: La tabla tiene campo deleted_at compatible
    Given que el usuario selecciona una tabla
    And la tabla contiene una columna llamada "deleted_at"
    And la columna tiene un tipo de fecha compatible
    When el sistema analiza la estructura de la tabla
    Then debe marcar esa columna como campo de soft delete
    And no debe preguntar al usuario por otro campo deleted_at

  Scenario: La tabla tiene campo deleted_at con tipo incompatible
    Given que el usuario selecciona una tabla
    And la tabla contiene una columna llamada "deleted_at"
    But la columna no tiene un tipo de fecha compatible
    When el sistema analiza la estructura de la tabla
    Then debe advertir que existe un campo deleted_at incompatible
    And debe preguntar si se desea seleccionar otro campo
```

---

## Feature 5.2: Selección manual de campo `deleted_at`

```gherkin
Feature: Selección manual de campo deleted_at
  Como usuario desarrollador
  Quiero seleccionar manualmente un campo de soft delete
  Para usar una columna existente aunque no se llame deleted_at

  Scenario: Usuario selecciona campo existente
    Given que la tabla no tiene un campo deleted_at compatible
    When el sistema pregunta qué campo será usado para soft delete
    And el usuario selecciona una columna existente compatible
    Then el sistema debe usar esa columna como campo deleted_at lógico

  Scenario: Usuario no selecciona ningún campo
    Given que la tabla no tiene un campo deleted_at compatible
    When el sistema pregunta qué campo será usado para soft delete
    And el usuario no selecciona ningún campo
    Then el sistema debe preguntar si desea generar endpoint delete
```

---

## Feature 5.3: Decisión sobre endpoint delete

```gherkin
Feature: Decisión sobre endpoint delete
  Como usuario desarrollador
  Quiero decidir si el API tendrá endpoint delete
  Para controlar si se requiere soft delete

  Scenario: Usuario no desea endpoint delete
    Given que la tabla no tiene campo deleted_at compatible
    And el usuario no seleccionó un campo alternativo
    When el sistema pregunta si desea generar endpoint delete
    And el usuario responde que no
    Then el sistema debe generar el comando sin endpoint delete
    And no debe requerir campo deleted_at

  Scenario: Usuario desea endpoint delete sin campo deleted_at
    Given que la tabla no tiene campo deleted_at compatible
    And el usuario no seleccionó un campo alternativo
    When el sistema pregunta si desea generar endpoint delete
    And el usuario responde que sí
    Then el sistema debe informar que deleted_at es indispensable para soft delete
    And debe preguntar si desea crear el campo deleted_at
```

---

## Feature 5.4: Creación segura del campo `deleted_at`

```gherkin
Feature: Creación segura del campo deleted_at
  Como usuario desarrollador
  Quiero revisar y confirmar el SQL antes de alterar una tabla
  Para evitar modificaciones accidentales en la base de datos

  Scenario: Usuario acepta crear campo deleted_at y revisa SQL
    Given que el usuario desea generar endpoint delete
    And la tabla seleccionada no tiene un campo deleted_at compatible
    When el sistema pregunta si desea crear el campo deleted_at
    And el usuario responde que sí
    Then el sistema debe generar el SQL de ALTER TABLE
    And debe mostrar el SQL al usuario
    And no debe ejecutar el SQL automáticamente

  Scenario: Usuario cancela alteración de tabla
    Given que el sistema muestra el SQL de ALTER TABLE
    When el usuario selecciona la opción "Cancelar"
    Then el sistema no debe ejecutar ningún SQL
    And debe mantener intacta la base de datos
    And debe preguntar si desea continuar sin endpoint delete

  Scenario: Usuario copia SQL para ejecutarlo manualmente
    Given que el sistema muestra el SQL de ALTER TABLE
    When el usuario selecciona la opción "Copiar SQL"
    Then el sistema debe copiar el SQL al portapapeles si es posible
    And debe mostrar el SQL en pantalla
    And debe permitir refrescar la estructura de la tabla posteriormente

  Scenario: Usuario ejecutó manualmente el SQL
    Given que el usuario copió el SQL
    And el usuario indica que ya ejecutó el SQL manualmente
    When el sistema refresca la estructura de la tabla
    Then debe verificar que el campo deleted_at existe
    And si el campo existe debe continuar con la generación del comando
    And si el campo no existe debe mostrar una advertencia

  Scenario: Usuario confirma ejecución automática del ALTER TABLE
    Given que el sistema muestra el SQL de ALTER TABLE
    And el usuario revisó el SQL
    When el usuario selecciona la opción "Ejecutar ALTER TABLE"
    Then el sistema debe solicitar confirmación explícita
    And si el usuario confirma debe ejecutar el SQL
    And debe esperar el resultado de la base de datos

  Scenario: ALTER TABLE ejecutado exitosamente
    Given que el usuario confirmó la ejecución del ALTER TABLE
    When la base de datos ejecuta el SQL correctamente
    Then el sistema debe mostrar un mensaje de éxito
    And debe refrescar la estructura de la tabla
    And debe verificar que el campo deleted_at existe
    And debe continuar con la generación del comando

  Scenario: ALTER TABLE falla
    Given que el usuario confirmó la ejecución del ALTER TABLE
    When la base de datos devuelve un error
    Then el sistema debe mostrar el mensaje de error
    And debe mostrar el SQL que se intentó ejecutar
    And no debe continuar como si el campo deleted_at existiera
```

---

# Épica 6: Generación de comando

Como usuario desarrollador,
quiero que el sistema construya automáticamente el string de ejecución,
para invocar el generador de APIs sin escribir manualmente campos y tipos.

## Feature 6.1: Construcción del comando base

```gherkin
Feature: Construcción del comando del generador
  Como usuario desarrollador
  Quiero generar un comando a partir de una tabla
  Para invocar el generador externo de APIs

  Scenario: Generar comando desde tabla seleccionada
    Given que el usuario seleccionó una tabla llamada "Usuario"
    And la tabla tiene los campos "id:int", "nombre:string" y "email:string"
    When el sistema construye el comando
    Then debe generar el string "gen.exe Usuario id:int,nombre:string,email:string"

  Scenario: Generar comando con campo deleted_at
    Given que el usuario seleccionó una tabla llamada "Usuario"
    And la tabla tiene los campos "id:int", "nombre:string" y "deleted_at:datetime"
    And deleted_at fue definido como campo de soft delete
    When el sistema construye el comando
    Then debe incluir "deleted_at:datetime" en el string generado
```

---

## Feature 6.2: Vista previa del comando

```gherkin
Feature: Vista previa del comando
  Como usuario desarrollador
  Quiero ver el comando antes de ejecutarlo
  Para confirmar que la generación será correcta

  Scenario: Mostrar comando antes de ejecutar
    Given que el sistema construyó el comando del generador
    When el comando está listo
    Then el sistema debe mostrar el comando completo al usuario
    And debe solicitar confirmación antes de ejecutarlo

  Scenario: Usuario cancela ejecución
    Given que el sistema muestra el comando generado
    When el usuario cancela la ejecución
    Then el sistema no debe invocar el generador externo
    And debe regresar al flujo de selección o edición
```

---

# Épica 7: Ejecución del generador externo

Como usuario desarrollador,
quiero que el sistema invoque el generador externo,
para crear el código del API automáticamente.

## Feature 7.1: Ejecución del comando generado

```gherkin
Feature: Ejecución del comando generado
  Como usuario desarrollador
  Quiero ejecutar el comando generado
  Para iniciar la generación del API

  Scenario: Ejecución exitosa del generador
    Given que el usuario confirmó la ejecución del comando
    And el comando generado es válido
    When el sistema invoca el generador externo
    Then debe esperar a que el proceso termine
    And debe mostrar la salida del generador
    And debe indicar que el proceso terminó exitosamente si el código de salida es cero

  Scenario: Error al ejecutar el generador
    Given que el usuario confirmó la ejecución del comando
    And el comando generado no puede ejecutarse
    When el sistema intenta invocar el generador externo
    Then debe mostrar un mensaje de error
    And debe mostrar la salida de error si existe
```

---

## Feature 7.2: Espera de finalización del proceso

```gherkin
Feature: Espera de finalización del proceso externo
  Como sistema
  Quiero esperar a que el generador termine
  Para conocer el resultado real de la generación

  Scenario: El generador termina correctamente
    Given que el generador externo está en ejecución
    When el proceso termina con código de salida cero
    Then el sistema debe mostrar un mensaje de éxito

  Scenario: El generador termina con error
    Given que el generador externo está en ejecución
    When el proceso termina con código de salida diferente de cero
    Then el sistema debe mostrar un mensaje de error
    And debe mostrar el código de salida
```

---

## Feature 7.3: Mensaje final del generador

```gherkin
Feature: Recepción de mensaje final del generador
  Como usuario desarrollador
  Quiero recibir confirmación de que el código fue generado
  Para saber que el proceso concluyó correctamente

  Scenario: El generador emite mensaje de éxito
    Given que el generador externo terminó su proceso
    And el generador emitió un mensaje indicando éxito
    When el sistema procesa la salida del generador
    Then debe mostrar el mensaje de éxito al usuario

  Scenario: El generador ejecuta un comando posterior
    Given que el generador externo tiene configurado un comando posterior
    When el generador termina la generación de código
    And ejecuta el comando posterior
    Then el sistema debe esperar el resultado final
    And debe mostrar el mensaje final recibido
```

---

# Épica 8: Manejo de errores

Como usuario desarrollador,
quiero recibir mensajes claros cuando algo falle,
para corregir el problema sin inspeccionar logs complejos.

## Feature 8.1: Validación de datos de conexión

```gherkin
Feature: Validación de datos de conexión
  Como sistema
  Quiero validar los datos mínimos de conexión
  Para evitar intentos inválidos

  Scenario: Faltan datos obligatorios
    Given que el usuario captura una conexión manual
    And falta el host o la base de datos
    When el usuario intenta conectarse
    Then el sistema debe indicar qué datos son obligatorios

  Scenario: Motor no soportado
    Given que el archivo de configuración indica un motor no soportado
    When el sistema carga la configuración
    Then debe mostrar un mensaje indicando que el motor no está soportado
```

---

## Feature 8.2: Manejo de errores de metadata

```gherkin
Feature: Manejo de errores al leer metadata
  Como sistema
  Quiero manejar errores al consultar metadata
  Para evitar que la aplicación se cierre inesperadamente

  Scenario: Permisos insuficientes
    Given que el usuario se conectó a una base de datos
    But el usuario no tiene permisos para leer metadata
    When el sistema intenta listar objetos
    Then debe mostrar un mensaje indicando permisos insuficientes

  Scenario: Pérdida de conexión
    Given que el sistema estaba conectado a la base de datos
    When se pierde la conexión durante la lectura de metadata
    Then debe mostrar un mensaje de pérdida de conexión
    And debe permitir reintentar la operación
```

---

# Épica 9: Seguridad en operaciones estructurales

Como usuario desarrollador,
quiero que cualquier modificación estructural requiera revisión y confirmación,
para evitar cambios accidentales en la base de datos.

## Feature 9.1: Protección ante operaciones DDL

```gherkin
Feature: Protección ante operaciones DDL
  Como usuario desarrollador
  Quiero revisar cualquier SQL que modifique estructura
  Para evitar cambios accidentales

  Scenario: Sistema genera operación DDL
    Given que el sistema necesita modificar la estructura de una tabla
    When genera una sentencia DDL
    Then debe mostrar la sentencia al usuario
    And debe solicitar confirmación explícita
    And no debe ejecutarla automáticamente

  Scenario: Usuario no confirma operación DDL
    Given que el sistema muestra una sentencia DDL
    When el usuario no confirma explícitamente
    Then el sistema no debe ejecutar la sentencia
```

---

## 13. Requerimientos no funcionales

## RNF-001: Separación de lógica y UI

La lógica de conexión, introspección, normalización, generación de SQL, generación de comandos y ejecución de procesos deberá estar desacoplada de la interfaz de usuario.

---

## RNF-002: Reutilización del core

El núcleo deberá poder usarse desde:

* CLI.
* TUI.
* GUI futura.
* Tests automatizados.

---

## RNF-003: Seguridad de credenciales

El sistema no deberá exponer contraseñas en logs, errores o salidas visibles.

---

## RNF-004: Extensibilidad

El sistema deberá permitir agregar nuevos motores de base de datos en el futuro sin reescribir la lógica principal.

---

## RNF-005: Configuración del generador externo

El ejecutable del generador deberá ser configurable.

Valor por defecto:

```txt
gen.exe
```

---

## RNF-006: Observabilidad básica

El sistema deberá registrar eventos relevantes:

* Inicio de conexión.
* Conexión exitosa.
* Error de conexión.
* Lectura de metadata.
* Generación de SQL.
* Generación de comando.
* Inicio de ejecución del generador.
* Finalización del generador.
* Error de ejecución.

---

## RNF-007: Portabilidad

El sistema deberá considerar diferencias entre Windows, Linux y macOS para ejecución de comandos externos.

---

## RNF-008: Confirmación de operaciones sensibles

Toda operación que modifique estructura de base de datos deberá requerir confirmación explícita.

Operaciones sensibles:

* `ALTER TABLE`
* `DROP`
* `TRUNCATE`
* `CREATE INDEX`
* `CREATE TABLE`
* Cualquier DDL futura.

En esta versión solo se contempla ejecutar:

```sql
ALTER TABLE ... ADD deleted_at ...
```

---

## RNF-009: Mensajes claros

Los errores deberán ser comprensibles para un usuario desarrollador.

Cada error debe indicar, cuando sea posible:

* Qué falló.
* En qué etapa falló.
* Qué acción puede tomar el usuario.

---

## 14. Arquitectura sugerida

El sistema debería organizarse en módulos independientes.

```txt
core/
  connection/
  introspection/
  metadata/
  type_mapper/
  soft_delete/
  ddl_builder/
  command_builder/
  process_runner/

adapters/
  postgres/
  sqlserver/

ui/
  cli/
  tui/
```

---

## 15. Responsabilidades por módulo

## 15.1 connection

Responsable de:

* Leer configuración.
* Validar datos.
* Crear conexión.
* Probar conexión.

---

## 15.2 introspection

Responsable de:

* Listar schemas.
* Listar tablas.
* Listar funciones.
* Listar stored procedures.
* Leer columnas.
* Leer primary keys.

---

## 15.3 type_mapper

Responsable de:

* Convertir tipos de PostgreSQL a tipos normalizados.
* Convertir tipos de SQL Server a tipos normalizados.
* Detectar tipos desconocidos.
* Proveer fallback configurable.

---

## 15.4 soft_delete

Responsable de:

* Detectar `deleted_at`.
* Validar compatibilidad del tipo.
* Solicitar selección manual.
* Decidir si se requiere crear campo.
* Mantener configuración final de soft delete.

---

## 15.5 ddl_builder

Responsable de:

* Generar SQL `ALTER TABLE`.
* Escapar identificadores correctamente.
* Diferenciar sintaxis PostgreSQL y SQL Server.
* Entregar SQL revisable antes de ejecución.

---

## 15.6 command_builder

Responsable de:

* Construir string para `gen.exe`.
* Aplicar formato de tabla y campos.
* Incluir tipos normalizados.
* Incluir `deleted_at` cuando aplique.

---

## 15.7 process_runner

Responsable de:

* Ejecutar comando externo.
* Esperar finalización.
* Capturar stdout.
* Capturar stderr.
* Capturar código de salida.

---

## 15.8 ui

Responsable de:

* Mostrar opciones al usuario.
* Permitir selección de objetos.
* Mostrar vistas previas.
* Solicitar confirmaciones.
* Mostrar errores y resultados.

---

## 16. Criterios de aceptación generales

La primera versión será aceptada cuando:

1. Permita ingresar conexión manual.
2. Permita leer conexión desde archivo.
3. Permita conectar a PostgreSQL.
4. Permita conectar a SQL Server.
5. Liste schemas y tablas.
6. Liste funciones en PostgreSQL.
7. Liste stored procedures en SQL Server.
8. Permita seleccionar una tabla.
9. Lea columnas y tipos de datos.
10. Normalice tipos para el generador.
11. Detecte `deleted_at` automáticamente.
12. Permita seleccionar manualmente un campo para soft delete.
13. Pregunte si se desea endpoint `delete`.
14. Genere SQL para crear `deleted_at` si se requiere.
15. Muestre el SQL antes de ejecutarlo.
16. Permita copiar el SQL.
17. Permita cancelar sin modificar la base de datos.
18. Ejecute `ALTER TABLE` solo con confirmación explícita.
19. Refresque metadata después de alterar la tabla.
20. Verifique que `deleted_at` existe.
21. Genere el comando para `gen.exe`.
22. Muestre el comando antes de ejecutarlo.
23. Permita copiar o cancelar el comando.
24. Ejecute el comando solo con confirmación.
25. Espere a que el comando termine.
26. Muestre stdout, stderr y código de salida.
27. No exponga contraseñas.
28. Maneje errores sin cerrar inesperadamente la aplicación.

---

## 17. Ejemplo de flujo esperado

```txt
Selecciona origen de conexión:
[1] Captura manual
[2] Archivo de configuración

Motor:
[1] PostgreSQL
[2] SQL Server

Conectando...
Conexión exitosa.

Objetos encontrados:

public
  tables
    users
    products
    orders
  functions
    calculate_total

Selecciona una tabla:
> users

Leyendo estructura...

Campos:
id:int
name:string
email:string
created_at:datetime
updated_at:datetime

No se encontró campo deleted_at compatible.

¿Deseas seleccionar un campo existente para soft delete?
[1] Sí
[2] No

> No

¿Deseas que el API tenga endpoint delete?
[1] Sí
[2] No

> Sí

El endpoint delete requiere un campo deleted_at.

¿Deseas crear el campo deleted_at en la tabla?
[1] Sí
[2] No

> Sí

SQL generado:

ALTER TABLE "public"."users"
ADD COLUMN "deleted_at" TIMESTAMP NULL;

Esta operación modificará la estructura de la tabla seleccionada.
Revisa cuidadosamente el SQL antes de continuar.

Opciones:
[1] Ejecutar ALTER TABLE
[2] Copiar SQL
[3] Cancelar

> Ejecutar ALTER TABLE

Confirma escribiendo: ALTERAR

> ALTERAR

Ejecutando SQL...
Campo deleted_at creado correctamente.

Refrescando estructura...
Campo deleted_at verificado.

Comando generado:

gen.exe users id:int,name:string,email:string,created_at:datetime,updated_at:datetime,deleted_at:datetime

¿Deseas ejecutarlo?
[1] Ejecutar comando
[2] Copiar comando
[3] Cancelar

> Ejecutar comando

Ejecutando generador...

Código generado exitosamente.
```

---

## 18. Riesgos identificados

1. Diferencias entre tipos de datos PostgreSQL y SQL Server.
2. Permisos insuficientes para leer metadata.
3. Permisos insuficientes para ejecutar `ALTER TABLE`.
4. Tablas sin primary key.
5. Tablas con nombres reservados.
6. Tablas o columnas con espacios o caracteres especiales.
7. Tipos de datos no soportados por el generador.
8. Stored procedures con parámetros complejos.
9. Funciones PostgreSQL con firmas sobrecargadas.
10. Diferencias entre sistemas operativos al ejecutar `gen.exe`.
11. Riesgo de modificar estructura en una base productiva.
12. Ambigüedad futura sobre generación desde funciones o procedures.

---

## 19. Prioridad sugerida

## Alta prioridad

1. Conexión a PostgreSQL.
2. Conexión a SQL Server.
3. Listado de tablas.
4. Lectura de columnas.
5. Normalización de tipos.
6. Detección de `deleted_at`.
7. Generación segura de SQL `ALTER TABLE`.
8. Vista previa y confirmación de SQL.
9. Generación de comando.
10. Ejecución de comando externo.

---

## Media prioridad

1. Listado de funciones PostgreSQL.
2. Listado de stored procedures SQL Server.
3. Selección manual de campo `deleted_at`.
4. Copia de SQL al portapapeles.
5. Copia de comando al portapapeles.
6. Logs de ejecución.
7. Archivo de configuración.

---

## Baja prioridad inicial

1. Generación desde stored procedures.
2. Generación desde funciones.
3. Persistencia segura de múltiples conexiones.
4. Editor visual de plantillas.
5. Interfaz gráfica avanzada.
6. Diagrama de relaciones.

---

## 20. Decisiones pendientes

Antes de iniciar implementación, se deben definir:

1. Formato exacto del archivo de configuración:

   * JSON.
   * TOML.
   * YAML.
   * ENV.

2. Formato exacto esperado por `gen.exe` para:

   * Nombre de tabla.
   * Nombre de schema.
   * Tipos de datos.
   * Activación de endpoint `delete`.
   * Campo `deleted_at`.

3. Si el comando debe recibir schema:

```txt
gen.exe public.users id:int,name:string
```

o solo tabla:

```txt
gen.exe users id:int,name:string
```

4. Si el endpoint `delete` se activa implícitamente por presencia de `deleted_at` o mediante argumento explícito.

5. Si el SQL de PostgreSQL debe usar `TIMESTAMP` o `TIMESTAMPTZ`.

6. Si se soportará selección múltiple de tablas en la primera versión.

7. Si las credenciales se guardarán localmente.

8. Si la primera interfaz será:

   * CLI.
   * TUI con Ratatui.
   * GUI con Tauri.

---

## 21. Recomendación para primera implementación

Se recomienda implementar primero el núcleo desacoplado y una interfaz CLI/TUI mínima.

Orden sugerido:

```txt
1. Crear modelos internos.
2. Crear lector de configuración.
3. Implementar conexión PostgreSQL.
4. Implementar conexión SQL Server.
5. Implementar listado de tablas.
6. Implementar lectura de columnas.
7. Implementar normalización de tipos.
8. Implementar detección de deleted_at.
9. Implementar generación de SQL ALTER TABLE.
10. Implementar confirmación y ejecución segura del ALTER.
11. Implementar command_builder para gen.exe.
12. Implementar process_runner.
13. Implementar UI CLI/TUI.
14. Agregar tests del core.
```

---

## 22. Definición de terminado

Una historia, feature o módulo se considerará terminado cuando:

1. Tenga implementación funcional.
2. Tenga manejo básico de errores.
3. No exponga credenciales.
4. Tenga pruebas unitarias cuando aplique.
5. Tenga al menos una prueba manual documentada.
6. Respete las reglas de confirmación para operaciones sensibles.
7. Sea independiente de la UI cuando pertenezca al core.
8. Devuelva mensajes claros al usuario.

---

## 23. Nombre interno sugerido

Nombre funcional temporal:

```txt
Schema API Launcher
```

Nombres alternativos posibles dentro del ecosistema:

```txt
Aleph Schema Studio
Aleph API Forge
Funes Schema Forge
Zahir API Cartographer
```
