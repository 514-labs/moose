use protobuf::MessageField;
use serde::{Deserialize, Serialize};

use crate::framework::data_model::model::DataModel;
use crate::proto::infrastructure_map::View as ProtoView;
use crate::proto::infrastructure_map::ViewType as ProtoViewType;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ViewType {
    TableAlias { source_table_name: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct View {
    pub name: String,
    pub version: String,
    pub view_type: ViewType,
}

impl View {
    // This is only to be used in the context of the new core
    // currently name includes the version, here we are separating that out.
    pub fn id(&self) -> String {
        format!("{}_{}", self.name, self.version.replace('.', "_"))
    }

    pub fn expanded_display(&self) -> String {
        self.short_display()
    }

    pub fn short_display(&self) -> String {
        format!("View: {} Version {}", self.name, self.version)
    }

    pub fn alias_view(data_model: &DataModel, source_data_model: &DataModel) -> Self {
        View {
            name: data_model.name.clone(),
            version: data_model.version.clone(),
            view_type: ViewType::TableAlias {
                source_table_name: source_data_model.id(),
            },
        }
    }

    pub fn to_proto(&self) -> ProtoView {
        ProtoView {
            name: self.name.clone(),
            version: self.version.clone(),
            view_type: MessageField::some(self.view_type.to_proto()),
            special_fields: Default::default(),
        }
    }
}

impl ViewType {
    fn to_proto(&self) -> ProtoViewType {
        match self {
            ViewType::TableAlias { source_table_name } => ProtoViewType {
                source_table_name: source_table_name.clone(),
                special_fields: Default::default(),
            },
        }
    }
}
