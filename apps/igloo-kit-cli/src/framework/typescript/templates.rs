use serde::Serialize;
use tinytemplate::TinyTemplate;

use super::{TypescriptInterface, InterfaceField};

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
    fn new(interface: TypescriptInterface) -> InterfaceContext {
        InterfaceContext {
            name: interface.name.clone(),
            file_name: interface.file_name(),
            var_name: interface.var_name(),
            fields: interface.fields.into_iter().map(|field| InterfaceFieldContext::new(field)).collect::<Vec<InterfaceFieldContext>>(),
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


pub static SEND_TEMPLATE: &str = r#"
import \{ {interface_context.name} } from './{interface_context.file_name}';

export async function send{interface_context.name}({interface_context.var_name}: {interface_context.name}) \{
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
    server_url: String,
    api_route_name: String,
}

pub struct SendFunctionTemplate;

impl SendFunctionTemplate {
    pub fn new(interface: TypescriptInterface, server_url: String, api_route_name: String) -> String {
        let mut tt = TinyTemplate::new();
        tt.add_template("send", SEND_TEMPLATE).unwrap();
        let context = SendFunctionContext {
            interface_context: InterfaceContext::new(interface),
            server_url,
            api_route_name,
        };
        let rendered = tt.render("send", &context).unwrap();
        rendered
    }
}

pub static INDEX_TEMPLATE: &str = r#"
import \{ {interface_context.name} } from './{interface_context.file_name}';
import \{ send{interface_context.name} } from './send{interface_context.file_name}';

export \{ {interface_context.name}, send{interface_context.name} };
"#;

#[derive(Serialize)]
struct IndexContext {
    interface_context: InterfaceContext,
}
impl IndexContext {
    fn new(interface: TypescriptInterface) -> IndexContext {
        IndexContext {
            interface_context: InterfaceContext::new(interface),
        }
    }
}
pub struct IndexTemplate;

impl IndexTemplate {
    pub fn new(interface: TypescriptInterface) -> String {
        let mut tt = TinyTemplate::new();
        tt.add_template("index", INDEX_TEMPLATE).unwrap();
        let context = IndexContext::new(interface);
        let rendered = tt.render("index", &context).unwrap();
        rendered
    }
}


pub static PACKAGE_JSON_TEMPLATE: &str = r#"
\{
    "name": "{package_name}",
    "version": "{package_version}",
    "description": "",
    "main": "index.js",
    "scripts": {
        "build": "tsc --build",
        "clean": "tsc --build --clean"
      },
    "keywords": [],
    "author": "{package_author}",
    "license": "ISC",
}
"#;

#[derive(Serialize)]
struct PackageJsonContext {
    package_name: String,
    package_version: String,
    package_author: String,
}

impl PackageJsonContext {
    fn new(package_name: String, package_version: String, package_author: String) -> PackageJsonContext {
        PackageJsonContext {
            package_name,
            package_version,
            package_author,
        }
    }
}

pub struct PackageJsonTemplate;

impl PackageJsonTemplate {
    pub fn new(package_name: String, package_version: String, package_author: String) -> String {
        let mut tt = TinyTemplate::new();
        tt.add_template("package_json", PACKAGE_JSON_TEMPLATE).unwrap();
        let context = PackageJsonContext::new(package_name, package_version, package_author);
        let rendered = tt.render("package_json", &context).unwrap();
        rendered
    }
}


// I'm using the same pattern since we may want to allow the user to configure this in the future.
pub static TS_CONFIG_TEMPLATE: &str = r#"
\{
    "compilerOptions": \{
        "target": "ES2017",
        "module": "commonjs",
        "lib": ["es6"],
        "strict": true,
        "declaration": true,
        "removeComments": false,
        "outDir": "./dist",
    }
}
"#;

#[derive(Serialize)]
struct TsConfigContext;

impl TsConfigContext {
    fn new() -> TsConfigContext {
        TsConfigContext
    }
}

pub struct TsConfigTemplate;

impl TsConfigTemplate {
    pub fn new() -> String {
        let mut tt = TinyTemplate::new();
        tt.add_template("ts_config", TS_CONFIG_TEMPLATE).unwrap();
        let context = TsConfigContext::new();
        let rendered = tt.render("ts_config", &context).unwrap();
        rendered
    }
}