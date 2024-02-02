use serde::Serialize;
use tinytemplate::TinyTemplate;

use super::TypescriptPackage;

pub static PACKAGE_JSON_TEMPLATE: &str = r#"
\{
    "name": "{package_name}",
    "version": "0.0.0",
    "description": "",
    "main": "dist/index.js",
    "types": "dist/index.d.ts",
    "scripts": \{
        "build": "tsc --build",
        "clean": "tsc --build --clean"
      },
    "keywords": [],
    "author": "moose-cli",
    "license": "ISC",
    "devDependencies": \{
        "@types/node": "^18.*.*",
        "typescript": "^5.*.*"
    },
    "dependencies": \{

    }
}
"#;

#[derive(Serialize)]
pub struct PackageJsonContext {
    package_name: String,
    // package_version: String,
    // package_author: String,
}

impl PackageJsonContext {
    fn new(package_name: String) -> PackageJsonContext {
        PackageJsonContext {
            package_name,
            // package_version,
            // package_author,
        }
    }
}

pub struct PackageJsonTemplate;

impl PackageJsonTemplate {
    pub fn build(package: &TypescriptPackage) -> String {
        let mut tt = TinyTemplate::new();
        tt.add_template("package_json", PACKAGE_JSON_TEMPLATE)
            .unwrap();
        let context = PackageJsonContext::new(package.name.clone());

        tt.render("package_json", &context).unwrap()
    }
}

// I'm using the same pattern since we may want to allow the user to configure this in the future.
pub static TS_CONFIG_TEMPLATE: &str = r#"
\{
    "compilerOptions": \{
        "target": "ES2017",
        "module": "esnext",
        "moduleResolution": "node",
        "lib": ["es6"],
        "strict": true,
        "declaration": true,
        "removeComments": false,
        "outDir": "./dist",
    }
}
"#;

#[derive(Serialize)]
pub struct TsConfigContext;

impl TsConfigContext {
    fn new() -> TsConfigContext {
        TsConfigContext
    }
}

pub struct TsConfigTemplate;

impl TsConfigTemplate {
    pub fn build() -> String {
        let mut tt = TinyTemplate::new();
        tt.add_template("ts_config", TS_CONFIG_TEMPLATE).unwrap();
        let context = TsConfigContext::new();

        tt.render("ts_config", &context).unwrap()
    }
}
