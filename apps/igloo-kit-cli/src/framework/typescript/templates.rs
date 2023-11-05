use serde::Serialize;
use tinytemplate::TinyTemplate;

use crate::framework::sdks::TypescriptObjects;

use super::{InterfaceField, TypescriptInterface};

pub static INTERFACE_TEMPLATE: &str = r#"
export interface {name} \{
    {{for field in fields}}{field.name}{{if field.is_optional}}?{{endif}}: {field.field_type},{{endfor}}
}
"#;

#[derive(Serialize)]
struct InterfaceContext {
    name: String,
    file_name: String,
    var_name: String,
    fields: Vec<InterfaceFieldContext>,
}

impl InterfaceContext {
    fn new(interface: &TypescriptInterface) -> InterfaceContext {
        InterfaceContext {
            name: interface.name.clone(),
            file_name: interface.file_name(),
            var_name: interface.var_name(),
            fields: interface
                .fields
                .clone()
                .into_iter()
                .map(InterfaceFieldContext::new)
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
    pub fn new(interface: &TypescriptInterface) -> String {
        let mut tt = TinyTemplate::new();
        tt.add_template("interface", INTERFACE_TEMPLATE).unwrap();
        let context = InterfaceContext::new(interface);

        tt.render("interface", &context).unwrap()
    }
}

pub static SEND_FUNC_TEMPLATE: &str = r#"
import \{ {interface_context.name} } from './{interface_context.file_name}';
import fetch from 'node-fetch';

export async function {declaration_name}({interface_context.var_name}: {interface_context.name}) \{
    return fetch('{server_url}/{api_route_name}', \{
        method: 'POST',
        headers: \{
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({interface_context.var_name})
    })
}
"#;

#[derive(Serialize)]
pub struct SendFunctionContext {
    interface_context: InterfaceContext,
    declaration_name: String,
    file_name: String,
    server_url: String,
    api_route_name: String,
}

impl SendFunctionContext {
    fn new(
        interface: &TypescriptInterface,
        server_url: String,
        api_route_name: String,
    ) -> SendFunctionContext {
        SendFunctionContext {
            interface_context: InterfaceContext::new(interface),
            declaration_name: interface.send_function_name(),
            file_name: interface.send_function_file_name(),
            server_url,
            api_route_name,
        }
    }
}

pub struct SendFunctionTemplate;

impl SendFunctionTemplate {
    pub fn new(
        interface: &TypescriptInterface,
        server_url: String,
        api_route_name: String,
    ) -> String {
        let mut tt = TinyTemplate::new();
        tt.add_template("send", SEND_FUNC_TEMPLATE).unwrap();
        let context = SendFunctionContext::new(interface, server_url, api_route_name);

        tt.render("send", &context).unwrap()
    }
}

pub static INDEX_TEMPLATE: &str = r#"
{{- for ts_object in ts_objects}}
import \{ {ts_object.interface_context.name} } from './{ts_object.interface_context.name}';
import \{ {ts_object.send_function_context.declaration_name} } from './{ts_object.send_function_context.file_name}';
{{endfor}}

{{for ts_object in ts_objects}}
export \{ {ts_object.interface_context.name} };
export \{ {ts_object.send_function_context.declaration_name} };
{{- endfor}}
"#;

#[derive(Serialize)]
struct TypescriptObjectsContext {
    interface_context: InterfaceContext,
    send_function_context: SendFunctionContext,
}

impl TypescriptObjectsContext {
    fn new(ts_objects: &TypescriptObjects) -> TypescriptObjectsContext {
        TypescriptObjectsContext {
            interface_context: InterfaceContext::new(&ts_objects.interface),
            send_function_context: SendFunctionContext::new(
                &ts_objects.interface,
                ts_objects.send_function.server_url.clone(),
                ts_objects.send_function.api_route_name.clone(),
            ),
        }
    }
}

#[derive(Serialize)]
struct IndexContext {
    ts_objects: Vec<TypescriptObjectsContext>,
}
impl IndexContext {
    fn new(ts_objects: &Vec<TypescriptObjects>) -> IndexContext {
        IndexContext {
            ts_objects: ts_objects
                .iter()
                .map(TypescriptObjectsContext::new)
                .collect::<Vec<TypescriptObjectsContext>>(),
        }
    }
}
pub struct IndexTemplate;

impl IndexTemplate {
    pub fn new(ts_objects: &Vec<TypescriptObjects>) -> String {
        let mut tt = TinyTemplate::new();
        tt.add_template("index", INDEX_TEMPLATE).unwrap();
        let context = IndexContext::new(ts_objects);

        tt.render("index", &context).unwrap()
    }
}
