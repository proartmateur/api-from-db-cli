use crate::domain::{ColumnSchema, TableSchema};

pub(crate) fn postgresql_tables() -> Vec<TableSchema> {
    vec![
        TableSchema::new(
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
        TableSchema::new(
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
    ]
}
