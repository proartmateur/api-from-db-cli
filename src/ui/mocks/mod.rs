pub mod catalog;
pub(crate) use crate::ui::mocks::catalog::catalog_fn;
mod postgresql_objects;
mod postgresql_tables;
pub mod process;
mod sqlserver_objects;
mod sqlserver_tables;
