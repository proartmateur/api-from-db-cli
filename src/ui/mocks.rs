use crate::app::generation_service::GenerationPreview;
use crate::domain::{
    ColumnSchema, DatabaseEngine, DatabaseObject, DatabaseObjectType, ProcessResult, TableSchema,
};
use crate::ui::state::{Catalog, CatalogMode, MockTable};

pub(crate) fn external_process_result(preview: Option<&GenerationPreview>) -> ProcessResult {
    let command = preview
        .map(|item| item.generated_command.raw_command.clone())
        .unwrap_or_else(|| "gen.exe demo id:int".to_string());

    ProcessResult {
        exit_code: 0,
        stdout: format!(
            "Comando copiado para ejecucion externa:\n{command}\nPuedes correrlo en otra herramienta y volver despues."
        ),
        stderr: String::new(),
        success: true,
    }
}

pub(crate) fn catalog(engine: DatabaseEngine) -> Catalog {
    match engine {
        DatabaseEngine::PostgreSql => Catalog {
            mode: CatalogMode::Mock,
            objects: vec![
                DatabaseObject {
                    name: "users".to_string(),
                    schema: Some("public".to_string()),
                    object_type: DatabaseObjectType::Table,
                    engine,
                },
                DatabaseObject {
                    name: "orders".to_string(),
                    schema: Some("sales".to_string()),
                    object_type: DatabaseObjectType::Table,
                    engine,
                },
                DatabaseObject {
                    name: "calculate_total".to_string(),
                    schema: Some("sales".to_string()),
                    object_type: DatabaseObjectType::Function,
                    engine,
                },
            ],
            tables: vec![
                MockTable {
                    schema: TableSchema::new(
                        "public",
                        "users",
                        vec![
                            ColumnSchema::new("id", "integer", false, 1),
                            ColumnSchema::new("name", "varchar(50)", false, 2),
                            ColumnSchema::new("email", "varchar(255)", false, 3),
                            ColumnSchema::new("created_at", "timestamp", false, 4),
                            ColumnSchema::new("updated_at", "timestamp", true, 5),
                        ],
                        vec!["id".to_string()],
                    ),
                },
                MockTable {
                    schema: TableSchema::new(
                        "sales",
                        "orders",
                        vec![
                            ColumnSchema::new("id", "bigint", false, 1),
                            ColumnSchema::new("customer_name", "text", false, 2),
                            ColumnSchema::new("total", "numeric(10,2)", false, 3),
                            ColumnSchema::new("deleted_at", "timestamptz", true, 4),
                        ],
                        vec!["id".to_string()],
                    ),
                },
            ],
        },
        DatabaseEngine::SqlServer => Catalog {
            mode: CatalogMode::Mock,
            objects: vec![
                DatabaseObject {
                    name: "Employees".to_string(),
                    schema: Some("dbo".to_string()),
                    object_type: DatabaseObjectType::Table,
                    engine,
                },
                DatabaseObject {
                    name: "Invoices".to_string(),
                    schema: Some("billing".to_string()),
                    object_type: DatabaseObjectType::Table,
                    engine,
                },
                DatabaseObject {
                    name: "sp_CloseInvoice".to_string(),
                    schema: Some("billing".to_string()),
                    object_type: DatabaseObjectType::StoredProcedure,
                    engine,
                },
            ],
            tables: vec![
                MockTable {
                    schema: TableSchema::new(
                        "dbo",
                        "Employees",
                        vec![
                            ColumnSchema::new("EmployeeId", "int", false, 1),
                            ColumnSchema::new("FullName", "varchar(80)", false, 2),
                            ColumnSchema::new("Email", "varchar(120)", true, 3),
                            ColumnSchema::new("CreatedAt", "datetime2", false, 4),
                        ],
                        vec!["EmployeeId".to_string()],
                    ),
                },
                MockTable {
                    schema: TableSchema::new(
                        "billing",
                        "Invoices",
                        vec![
                            ColumnSchema::new("InvoiceId", "bigint", false, 1),
                            ColumnSchema::new("CustomerName", "nvarchar(120)", false, 2),
                            ColumnSchema::new("TotalAmount", "decimal(18,2)", false, 3),
                            ColumnSchema::new("DeletedAt", "datetime2", true, 4),
                        ],
                        vec!["InvoiceId".to_string()],
                    ),
                },
            ],
        },
    }
}
