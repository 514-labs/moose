use handlebars::Handlebars;

use crate::project::python_project::PythonProject;

#[derive(Debug, thiserror::Error)]
#[error("Failed to generate Typescript code")]
#[non_exhaustive]
pub enum PythonRenderingError {
    HandlebarError(#[from] handlebars::RenderError),
}

pub static PYTHON_BASE_MODEL_TEMPLATE: &str = r#"
from moose_lib import Key, moose_data_model
from dataclasses import dataclass
from datetime import datetime
from typing import Optional
@moose_data_model
@dataclass
class Foo:
    primary_key: Key[str]
    timestamp: float
    optional_text: Optional[str]

@moose_data_model
@dataclass
class Bar:
    primary_key: Key[str]
    utc_timestamp: datetime
    has_text: bool
    text_length: int
"#;

pub static SETUP_PY_TEMPLATE: &str = r#"
from setuptools import setup

setup(
    name='{{name}}',
    version='{{version}}',
    install_requires=[
        {{#each dependencies}}
        "{{{ this }}}",
        {{/each}}
    ],
)
"#;

pub static PYTHON_BASE_STREAMING_FUNCTION_SAMPLE: &str = r#"
from moose_lib import StreamingFunction
from app.datamodels.models import Foo, Bar

def transform(foo: Foo) -> Bar:
    return Bar(
        primary_key=foo.primary_key,
        utc_timestamp=foo.timestamp,
        has_text=foo.optional_text is not None,
        text_length=len(foo.optional_text) if foo.optional_text else 0
    )

Foo__Bar = StreamingFunction(run=transform)
"#;

pub static PYTHON_BASE_STREAMING_FUNCTION_TEMPLATE: &str = r#"
# Import your Moose data models to use in the streaming function
{{source_import}}
{{destination_import}}
from moose_lib import StreamingFunction
from typing import Optional
from datetime import datetime

def fn(source: {{source}}) -> Optional[{{destination}}]:
    return {{destination_object}}

my_function = StreamingFunction(
    run=fn
)
"#;

pub static PYTHON_BASE_CONSUMPTION_TEMPLATE: &str = r#"
# This file is where you can define your API templates for consuming your data
#Query params are passed in as Pydantic models and are used within the sql tag to parameterize you queries

from moose_lib import MooseClient
from pydantic import BaseModel

class QueryParams(BaseModel):
    limit: Optional[int] = 10

def run(client: MooseClient, params: QueryParams):
    return client.query.execute("SELECT 1", {})
"#;

pub static PYTHON_BASE_API_SAMPLE: &str = r#"
from moose_lib import MooseClient
from pydantic import BaseModel, Field
from datetime import datetime
from typing import Optional

class QueryParams(BaseModel):
    return_as: str = Field(
        default="both",
        pattern=r"^(both|count|percentage)$",
        description="Must be one of: both, count, percentage"
    )
    
    
def run(client: MooseClient, params: QueryParams):
    query = "SELECT countMerge(total_rows) as total"
    if params.return_as in ("both", "count"):
        query += ", countMerge(rows_with_text) as count_with_text"
    if params.return_as in ("both", "percentage"):
        query += ", countMerge(rows_with_text) / total as percentage_with_text"
    
    query += " FROM BarAggregated"        
   
    return client.query.execute(query, {})
"#;

pub static PYTHON_BASE_BLOCKS_TEMPLATE: &str = r#"
# This file is where you can define your SQL queries to shape and manipulate batches
# of data using Blocks. Blocks can also manage materialized views to store the results of 
# your queries for improved performance. A materialized view is the recommended approach for aggregating
# data. For more information on the types of aggregate functions you can run on your existing data, 
# consult the Clickhouse documentation: https://clickhouse.com/docs/en/sql-reference/aggregate-functions

from moose_lib import (
    AggregationCreateOptions,
    AggregationDropOptions,
    Blocks,
    ClickHouseEngines,
    TableCreateOptions,
    create_aggregation,
    drop_aggregation,
)

teardown_queries = []

setup_queries = []

block = Blocks(teardown=teardown_queries, setup=setup_queries)
"#;

pub static PYTHON_BASE_BLOCKS_SAMPLE: &str = r#"
# Here is a sample aggregation query that sets up a materialized view to store the results of the query
# It uses Clickhouse's aggregate functions to count the number of rows and the number of rows with text
# The materialized view is named BarAggregated_MV and the table is named BarAggregated

from moose_lib import (
  Blocks, 
  create_aggregation,
  AggregationCreateOptions,
  TableCreateOptions, 
  drop_aggregation, 
  AggregationDropOptions, 
  ClickHouseEngines as CH
)

TABLE_NAME = "BarAggregated"
MV_NAME = "BarAggregated_MV"

QUERY = """
SELECT
  toStartOfDay(utc_timestamp) as date,
  countState(primary_key) as total_rows,
  countIfState(has_text) as rows_with_text,
  sumState(text_length) as total_text_length
FROM Bar_0_0
GROUP BY toStartOfDay(utc_timestamp)
"""

setup = create_aggregation(AggregationCreateOptions(
    table_create_options=TableCreateOptions(
      name=TABLE_NAME,
      columns={
        "date": "Date",
        "total_rows": "AggregateFunction(count, String)",
        "rows_with_text": "AggregateFunction(count, Int64)",
        "total_text_length": "AggregateFunction(sum, Int64)"
      },
      engine=CH.AggregatingMergeTree,
      order_by="date"
    ),
    materialized_view_name=MV_NAME,
    select=QUERY
))

teardown = drop_aggregation(
  options=AggregationDropOptions(
    view_name=MV_NAME,
    table_name=TABLE_NAME
  )
)
block = Blocks(teardown=teardown, setup=setup)
"#;

pub static PYTHON_BASE_SCRIPT_TEMPLATE: &str = r#"from moose_lib import task

@task()
def {{name}}(data: dict):  # The name of your script
    """
    Description of what this script does
    """
    # The body of your script goes here

    # The return value is the output of the script.
    # The return value should be a dictionary with at least:
    # - step: the step name (e.g., "extract", "transform")
    # - data: the actual data being passed to the next step
    return {
        "step": "{{name}}",  # The step name is the name of the script
        "data": None     # The data being passed to the next step (4MB limit)
    }
"#;

pub fn render_setup_py(project: PythonProject) -> Result<String, PythonRenderingError> {
    let reg = Handlebars::new();

    let template_context = serde_json::json!({
        "name": project.name,
        "version": project.version,
        "dependencies": project.dependencies,
    });

    Ok(reg.render_template(SETUP_PY_TEMPLATE, &template_context)?)
}
