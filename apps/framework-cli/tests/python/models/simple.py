from dataclasses import dataclass
from enum import Enum
from typing import List, Optional
from datetime import datetime



type Key[T: (str, int)] = T 

class Status(Enum):
    OK = "ok"
    ERROR = "error"

class MySubModel:
    name: str
    age: int


@dataclass 
class MyModel:
    name: Key[str]
    age: int
    flag: bool
    status: Status
    test_key: str
    arr: List[str]
    opt: Optional[str]
    sub: MySubModel
    date: datetime

