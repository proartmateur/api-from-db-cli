use crate::domain::{DatabaseEngine, DatabaseObject, DatabaseObjectType};

pub(crate) fn postgre_objects() -> Vec<DatabaseObject> {
    vec![
        DatabaseObject {
            name: "users".to_string(),
            schema: Some("public".to_string()),
            object_type: DatabaseObjectType::Table,
            engine: DatabaseEngine::PostgreSql,
        },
        DatabaseObject {
            name: "orders".to_string(),
            schema: Some("sales".to_string()),
            object_type: DatabaseObjectType::Table,
            engine: DatabaseEngine::PostgreSql,
        },
        DatabaseObject {
            name: "calculate_total".to_string(),
            schema: Some("sales".to_string()),
            object_type: DatabaseObjectType::Function,
            engine: DatabaseEngine::PostgreSql,
        },
    ]
}
