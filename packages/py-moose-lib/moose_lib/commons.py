import logging
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

class Logger:
    default_action = "Custom"

    def __init__(self, action: Optional[str] = None, is_moose_task: bool = False):
        self.action = action or Logger.default_action
        self._is_moose_task = is_moose_task

    def _log(self, message: str, message_type: str) -> None:
        # We have a task decorator in the lib that initializes a logger
        if self._is_moose_task:
            # Temporal already has a named logger. We customize that logger's level
            # and handler to send logs to moose. That handler has some sanitization
            # to make the logs more readable. This is just re-using all the same config.
            temporal_logger = logging.getLogger('temporalio')
            temporal_logger.info(message)
        else:
            cli_log(CliLogData(action=self.action, message=message, message_type=message_type))

    def info(self, message: str) -> None:
        self._log(message, "Info")

    def success(self, message: str) -> None:
        self._log(message, "Success")

    def error(self, message: str) -> None:
        self._log(message, "Error")

    def highlight(self, message: str) -> None:
        self._log(message, "Highlight")
