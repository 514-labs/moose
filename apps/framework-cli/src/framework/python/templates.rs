use handlebars::Handlebars;

use crate::project::python_project::PythonProject;

#[derive(Debug, thiserror::Error)]
#[error("Failed to generate Typescript code")]
#[non_exhaustive]
pub enum PythonRenderingError {
    HandlebarError(#[from] handlebars::RenderError),
}

pub static PYTHON_BASE_MODEL_TEMPLATE: &str = r#"
from dataclasses import dataclass
import datetime

type Key[T: (str, int)] = T 

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
from datetime import datetime
from app.datamodels.models import UserActivity, ParsedActivity
from dataclasses import dataclass
from typing import Callable

@dataclass
class Flow:
    run: Callable

def parse_activity(activity: UserActivity) -> ParsedActivity:
    return ParsedActivity(
        eventId=activity.eventId,
        timestamp=datetime.fromisoformat(activity.timestamp),
        userId=activity.userId,
        activity=activity.activity,
    )

my_flow = Flow(
    run=parse_activity
)
"#;

pub static PYTHON_BASE_API_SAMPLE: &str = r#"
def run(client, params):
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
from dataclasses import dataclass
from typing import List

# This file is where you can define your SQL queries to shape and manipulate batches
# of data using Blocks. Blocks can also manage materialized views to store the results of 
# your queries for improved performance. A materialized view is the recommended approach for aggregating
# data. For more information on the types of aggregate functions you can run on your existing data, 
# consult the Clickhouse documentation: https://clickhouse.com/docs/en/sql-reference/aggregate-functions

@dataclass
class Blocks:
    teardown: List[str]
    setup: List[str]

teardown_queries = []

setup_queries = []

block = Blocks(teardown=teardown_queries, setup=setup_queries)
"#;

pub static PYTHON_BASE_BLOCKS_SAMPLE: &str = r#"
from dataclasses import dataclass
from typing import List

# Here is a sample aggregation query that calculates the number of daily active users
# based on the number of unique users who complete a sign-in activity each day.

@dataclass
class Blocks:
    teardown: List[str]
    setup: List[str]

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

teardown_queries = [
    f"""
    DROP VIEW IF EXISTS {materialized_view}
    """,
    f"""
    DROP TABLE IF EXISTS {destination_table}
    """
]

setup_queries = [
    f"""
    CREATE TABLE IF NOT EXISTS {destination_table}
    (
        date Date,
        dailyActiveUsers AggregateFunction(uniq, String)
    )
    ENGINE = AggregatingMergeTree()
    ORDER BY date
    """,
    f"""
    CREATE MATERIALIZED VIEW IF NOT EXISTS {materialized_view}
    TO {destination_table}
    AS {select_sql}
    """,
    f"""
    INSERT INTO {destination_table}
    {select_sql}
    """
]

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
