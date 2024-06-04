from dataclasses import dataclass
from typing import Callable
from datetime import datetime

@dataclass
class Flow:
    run: Callable

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

def my_func(dm: UserActivity) -> ParsedActivity:
    print(dm)

my_flow = Flow(
    run=my_func
)

