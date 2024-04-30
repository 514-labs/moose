from dataclasses import dataclass, field
from enum import Enum
from typing import List, Optional


type Key[T: (str, int)] = T 


# Might be a solid idea to use the field specifier parameters to rename errand fields?
# https://typing.readthedocs.io/en/latest/spec/dataclasses.html#field-specifier-parameters

class Status(Enum):
    OK = "ok"
    ERROR = "error"


@dataclass 
class MyModel:
    name: Key[str]
    age: int
    flag: bool
    status: Status
    test_key: str = field(alias="test-key")
    arr: List[str]
    opt: Optional[str]

