from moose_lib import Key
from pydantic import BaseModel

class DeadLetterQueue(BaseModel):
    time: Key[str]
    error_location: str
    payload: str
    error_message: str
  