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

pub static PYTHON_BASE_FLOW_TEMPLATE: &str = r#"
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

pub static PTYHON_BASE_AGG_SAMPLE_TEMPLATE: &str = r#"
from dataclasses import dataclass

# Here is a sample aggregation query that calculates the number of daily active users
# based on the number of unique users who complete a sign-in activity each day.

@dataclass
Aggregation:
    select: str
    order_by: str

sql = """
SELECT 
uniqState(userId) as dailyActiveUsers,
toStartOfDay(timestamp) as date
FROM ParsedActivity_0_0
WHERE activity = 'Login' 
GROUP BY toStartOfDay(timestamp)
"""

agg = Aggregation(select=sql, order_by="date")
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
