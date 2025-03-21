from importlib import import_module
from .internal import load_models

import_module("app.main")

load_models()
