pub static PYTHON_BASE_MODEL_TEMPLATE: &str = r#"
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
