use serde::Serialize;
use tinytemplate::TinyTemplate;

use super::{InterfaceField, TypescriptInterface};

pub static INTERFACE_TEMPLATE: &str = r#"
export interface {name} \{
    {{for field in fields}}{field.name}{{if field.is_optional}}?{{endif}}: {field.field_type},
    {{endfor}}
}
"#;

#[derive(Serialize)]
struct InterfaceContext {
    name: String,
    fields: Vec<InterfaceFieldContext>,
}

impl InterfaceContext {
    fn new(interface: TypescriptInterface) -> InterfaceContext {
        InterfaceContext {
            name: interface.name,
            fields: interface
                .fields
                .into_iter()
                .map(|field| InterfaceFieldContext::new(field))
                .collect::<Vec<InterfaceFieldContext>>(),
        }
    }
}

#[derive(Serialize)]
struct InterfaceFieldContext {
    name: String,
    field_type: String,
    is_optional: bool,
}

impl InterfaceFieldContext {
    fn new(interface_field: InterfaceField) -> InterfaceFieldContext {
        InterfaceFieldContext {
            name: interface_field.name,
            field_type: interface_field.field_type.to_string(),
            is_optional: interface_field.is_optional,
        }
    }
}

pub struct InterfaceTemplate;

impl InterfaceTemplate {
    pub fn new(interface: TypescriptInterface) -> String {
        let mut tt = TinyTemplate::new();
        tt.add_template("interface", INTERFACE_TEMPLATE).unwrap();
        let context = InterfaceContext::new(interface);
        let rendered = tt.render("interface", &context).unwrap();
        rendered
    }
}
