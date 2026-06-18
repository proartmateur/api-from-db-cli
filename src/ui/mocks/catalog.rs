use crate::domain::DatabaseEngine;
use crate::ui::state::{Catalog, CatalogMode, MockTable};

use super::{postgresql_objects, postgresql_tables, sqlserver_objects, sqlserver_tables};

pub(crate) fn catalog_fn(engine: DatabaseEngine) -> Catalog {
    match engine {
        DatabaseEngine::PostgreSql => Catalog {
            mode: CatalogMode::Mock,
            objects: postgresql_objects::postgre_objects(),
            tables: postgresql_tables::postgresql_tables()
                .into_iter()
                .map(|schema| MockTable { schema })
                .collect(),
        },
        DatabaseEngine::SqlServer => Catalog {
            mode: CatalogMode::Mock,
            objects: sqlserver_objects::sqlserver_objects(),
            tables: sqlserver_tables::sqlserver_tables()
                .into_iter()
                .map(|schema| MockTable { schema })
                .collect(),
        },
    }
}
