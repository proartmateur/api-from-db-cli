use crate::domain::{ColumnSchema, TableSchema};

pub(crate) fn sqlserver_tables() -> Vec<TableSchema> {
    vec![
        TableSchema::new(
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
        TableSchema::new(
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
    ]
}
