from dataclasses import dataclass
from datetime import datetime
from moose_lib import Key, moose_data_model

@moose_data_model
@dataclass
class HistoricalStargazer:
    starred_at: datetime
    login: Key[str]
    avatar_url: str
    repos_url: str