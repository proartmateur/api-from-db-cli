use crate::domain::{DatabaseEngine, DatabaseObject, DatabaseObjectType};

pub(crate) fn sqlserver_objects() -> Vec<DatabaseObject> {
    vec![
        DatabaseObject {
            name: "Employees".to_string(),
            schema: Some("dbo".to_string()),
            object_type: DatabaseObjectType::Table,
            engine: DatabaseEngine::SqlServer,
        },
        DatabaseObject {
            name: "Invoices".to_string(),
            schema: Some("billing".to_string()),
            object_type: DatabaseObjectType::Table,
            engine: DatabaseEngine::SqlServer,
        },
        DatabaseObject {
            name: "sp_CloseInvoice".to_string(),
            schema: Some("billing".to_string()),
            object_type: DatabaseObjectType::StoredProcedure,
            engine: DatabaseEngine::SqlServer,
        },
    ]
}
