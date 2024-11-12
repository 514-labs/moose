from moose_lib import Key, moose_data_model
from dataclasses import dataclass
from datetime import datetime


@moose_data_model
@dataclass
class DeadLetterQueue:
    time: Key[str]
    error_location: str
    payload: str
    error_message: str
  