use std::io::Error;
use diagnostics::Diagnostics;

use schema_ast::{parse_schema, ast::SchemaAst};

use crate::infrastructure::db::clickhouse::ast_mapper;

pub fn parse_schema_file()  {
    let path = "/Users/timdelisle/Dev/igloo-stack/apps/igloo-kit-cli/tests/psl/simple.psl";
    let schema_file = std::fs::read_to_string(path).unwrap();

    let mut diagnostics = Diagnostics::default();

    let ast = parse_schema(&schema_file, &mut diagnostics);

    let table = ast_mapper(ast);
    println!("{:#?}", table);
}