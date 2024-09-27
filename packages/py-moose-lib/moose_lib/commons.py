import requests
import json
from typing import Optional, Literal

class CliLogData:
    def __init__(self, action: str, message: str, message_type: Optional[Literal["Info", "Success", "Error", "Highlight"]] = "Info"):
        self.message_type = message_type
        self.action = action
        self.message = message

def cli_log(log: CliLogData) -> None:
    url = "http://localhost:5001/logs"
    headers = {'Content-Type': 'application/json'}
    requests.post(url, data=json.dumps(log.__dict__), headers=headers)