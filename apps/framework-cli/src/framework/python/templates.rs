use handlebars::Handlebars;

use crate::project::python_project::PythonProject;

#[derive(Debug, thiserror::Error)]
#[error("Failed to generate Typescript code")]
#[non_exhaustive]
pub enum PythonRenderingError {
    HandlebarError(#[from] handlebars::RenderError),
}

pub static PYTHON_BASE_MODEL_TEMPLATE: &str = r#"
from moose_lib import Key
from dataclasses import dataclass
import datetime

@dataclass
class UserActivity:
    eventId: Key[str]
    timestamp: str
    userId: str
    activity: str

@dataclass
class ParsedActivity:
    eventId: Key[str]
    timestamp: datetime
    userId: str
    activity: str
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
from datetime import datetime
from app.datamodels.models import UserActivity, ParsedActivity

def parse_activity(activity: UserActivity) -> ParsedActivity:
    return ParsedActivity(
        eventId=activity.eventId,
        timestamp=datetime.fromisoformat(activity.timestamp),
        userId=activity.userId,
        activity=activity.activity,
    )

my_function = StreamingFunction(
    run=parse_activity
)
"#;

pub static PYTHON_BASE_STREAMING_FUNCTION_TEMPLATE: &str = r#"
# Add your models & start the development server to import these types
{{source_import}}
{{destination_import}}
from moose_lib import StreamingFunction
from typing import Optional
from datetime import datetime

def fn(activity: {{source}}) -> Optional[{{destination}}]:
    return {{destination_object}}

my_function = StreamingFunction(
    run=fn
)
"#;

pub static PYTHON_BASE_API_SAMPLE: &str = r#"
from moose_lib import MooseClient

def run(client: MooseClient, params):
    minDailyActiveUsers = int(params.get('minDailyActiveUsers', [0])[0])
    limit = int(params.get('limit', [10])[0])

    return client.query(
        '''SELECT
            date,
            uniqMerge(dailyActiveUsers) as dailyActiveUsers
        FROM DailyActiveUsers
        GROUP BY date
        HAVING dailyActiveUsers >= {minDailyActiveUsers}
        ORDER BY date
        LIMIT {limit}''',
        {
            "minDailyActiveUsers": minDailyActiveUsers,
            "limit": limit
        }
    )
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
# Here is a sample aggregation query that calculates the number of daily active users
# based on the number of unique users who complete a sign-in activity each day.

from moose_lib import (
    AggregationCreateOptions,
    AggregationDropOptions,
    Blocks,
    ClickHouseEngines,
    TableCreateOptions,
    create_aggregation,
    drop_aggregation,
)

destination_table = "DailyActiveUsers"
materialized_view = "DailyActiveUsers_mv"

select_sql = """
SELECT 
    toStartOfDay(timestamp) as date,
    uniqState(userId) as dailyActiveUsers
FROM ParsedActivity_0_0
WHERE activity = 'Login' 
GROUP BY toStartOfDay(timestamp)
"""

teardown_queries = drop_aggregation(
    AggregationDropOptions(materialized_view, destination_table)
)

table_options = TableCreateOptions(
    name=destination_table,
    columns={"date": "Date", "dailyActiveUsers": "AggregateFunction(uniq, String)"},
    engine=ClickHouseEngines.MergeTree,
    order_by="date",
)

aggregation_options = AggregationCreateOptions(
    table_create_options=table_options,
    materialized_view_name=materialized_view,
    select=select_sql,
)

setup_queries = create_aggregation(aggregation_options)

block = Blocks(teardown=teardown_queries, setup=setup_queries)
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
