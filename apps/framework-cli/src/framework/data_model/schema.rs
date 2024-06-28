use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::framework::core::infrastructure::table::{Column, Table, TableType};
use crate::framework::core::infrastructure_map::{PrimitiveSignature, PrimitiveTypes};

use super::config::DataModelConfig;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct DataModel {
    pub columns: Vec<Column>,
    pub name: String,
    #[serde(default)]
    pub config: DataModelConfig,
    pub abs_file_path: PathBuf,
    pub version: String,
}

impl DataModel {
    // TODO this probably should be on the Table object itself which can be built from
    // multiplle sources. The Aim will be to have DB Blocks provision some tables as well.
    pub fn to_table(&self) -> Table {
        Table {
            table_type: TableType::Table,
            name: format!("{}_{}", self.name, self.version.replace('.', "_")),
            columns: self.columns.clone(),
            order_by: self.config.storage.order_by_fields.clone(),
            version: self.version.clone(),
            source_primitive: PrimitiveSignature {
                name: self.name.clone(),
                primitive_type: PrimitiveTypes::DataModel,
            },
        }
    }
}
