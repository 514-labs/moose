import logging
import requests
import json
from typing import Optional, Literal

class CliLogData:
    INFO = "Info"
    SUCCESS = "Success"
    ERROR = "Error"
    HIGHLIGHT = "Highlight"

    def __init__(self, action: str, message: str, message_type: Optional[Literal[INFO, SUCCESS, ERROR, HIGHLIGHT]] = INFO):
        self.message_type = message_type
        self.action = action
        self.message = message


def cli_log(log: CliLogData) -> None:
    try:
        # When dmv2 starts up, it imports all the dmv2 definitions. In python,
        # import_module executes code at the module level (but not inside functions).
        # If the user has a function being called at the module level, and that function
        # tries to send logs when moose hasn't fully started, the requests will fail.
        # The try catch is to ignore those errors.
        url = "http://localhost:5001/logs"
        headers = {'Content-Type': 'application/json'}
        requests.post(url, data=json.dumps(log.__dict__), headers=headers)
    except:
        pass


class Logger:
    default_action = "Custom"
    
    def __init__(self, action: Optional[str] = None, is_moose_task: bool = False):
        self.action = action or Logger.default_action
        self._is_moose_task = is_moose_task
    
    def _log(self, message: str, message_type: str) -> None:
        if self._is_moose_task:
            # We have a task decorator in the lib that initializes a logger
            # This re-uses the same logger in moose scripts runner
            moose_scripts_logger = logging.getLogger("moose-scripts")
            if message_type == CliLogData.INFO:
                moose_scripts_logger.info(message)
            elif message_type == CliLogData.SUCCESS:
                moose_scripts_logger.success(message)
            elif message_type == CliLogData.ERROR:
                moose_scripts_logger.error(message)
            elif message_type == CliLogData.HIGHLIGHT:
                moose_scripts_logger.warning(message)
        else:
            cli_log(CliLogData(action=self.action, message=message, message_type=message_type))

    def info(self, message: str) -> None:
        self._log(message, CliLogData.INFO)

    def success(self, message: str) -> None:
        self._log(message, CliLogData.SUCCESS)

    def error(self, message: str) -> None:
        self._log(message, CliLogData.ERROR)

    def highlight(self, message: str) -> None:
        self._log(message, CliLogData.HIGHLIGHT)