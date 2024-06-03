use crate::framework::data_model::schema::{Column, ColumnDefaults, ColumnType};

use super::parser::PythonParserError;

#[derive(Default)]
pub struct ColumnBuilder {
    pub name: Option<String>,
    pub data_type: Option<ColumnType>,
    pub required: Option<bool>,
    pub unique: Option<bool>,
    pub primary_key: Option<bool>,
    pub path: Option<String>,
    pub default: Option<ColumnDefaults>,
}

impl ColumnBuilder {
    pub fn build(self) -> Result<Column, PythonParserError> {
        let name = self.name.ok_or(PythonParserError::ClassParseError {
            message: "Class builder property, name, not set properly".to_string(),
        })?;

        let data_type = self
            .data_type
            .ok_or(PythonParserError::UnsupportedDataTypeError {
                type_name: "Class builder property, data_type, unsupported or not set properly"
                    .to_string(),
            })?;

        let required = self.required.unwrap_or(true);

        let unique = self.unique.unwrap_or(false);

        let primary_key = self.primary_key.unwrap_or(false);

        Ok(Column {
            name,
            data_type,
            required,
            unique,
            primary_key,
            default: self.default,
        })
    }
}
