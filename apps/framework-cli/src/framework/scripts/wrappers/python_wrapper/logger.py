import logging
import requests
import re

class HTTPLogHandler(logging.Handler):
    def __init__(self, url, workflow_id, run_id):
        super().__init__()
        self.url = url
        self.workflow_id = workflow_id
        self.run_id = run_id
        self.setFormatter(logging.Formatter('%(message)s'))

    def emit(self, record):
        log_entry = self.format(record)
        log_entry = log_entry.replace('\n', ' ').replace('\r', ' ')
        log_entry = re.sub(r'\s+', ' ', log_entry).strip()

        if '(' in log_entry and log_entry.endswith(')'):
            log_entry = log_entry[:log_entry.rfind('(')].strip()
        
        id_prefix = f"'{self.workflow_id}' ID:{self.run_id}:"
        if not log_entry.startswith(id_prefix):
            log_entry = f"{id_prefix} {log_entry}"
        
        payload = {
            "message_type": "Info",
            "action": "Scripts",
            "message": log_entry
        }
        
        try:
            requests.post(self.url, json=payload)
        except Exception as e:
            pass

def initialize_logger(workflow_id, run_id):
    http_handler = HTTPLogHandler("http://localhost:5001/logs", workflow_id, run_id)

    # This is how we route log lines from temporal to moose
    temporal_logger = logging.getLogger('temporalio')
    temporal_logger.setLevel(logging.DEBUG)
    temporal_logger.handlers = [http_handler]
    temporal_logger.propagate = False