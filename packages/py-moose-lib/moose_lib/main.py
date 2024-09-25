from clickhouse_connect.driver.client import Client
from dataclasses import dataclass, asdict
from enum import Enum
from string import Formatter
from typing import Any, Callable, Dict, Generic, Optional, TypeVar, Union, overload
import sys
import os
import json

type Key[T: (str, int)] = T


@dataclass
class StreamingFunction:
    run: Callable


class IngestionFormat(Enum):
    JSON = "JSON"
    JSON_ARRAY = "JSON_ARRAY"


@dataclass
class IngestionConfig:
    format: Optional[IngestionFormat] = None


@dataclass
class StorageConfig:
    enabled: Optional[bool] = None
    order_by_fields: Optional[list[str]] = None


@dataclass
class DataModelConfig:
    ingestion: Optional[IngestionConfig] = None
    storage: Optional[StorageConfig] = None


class CustomEncoder(json.JSONEncoder):
    def default(self, obj):
        if isinstance(obj, Enum):
            return obj.value
        return super().default(obj)


_DC = TypeVar("_DC", bound=type)


@overload
def moose_data_model(arg: Optional[DataModelConfig]) -> Callable[[_DC], _DC]:
    ...


@overload
def moose_data_model(arg: _DC) -> _DC:
    ...


def moose_data_model(arg: Any = None) -> Any:
    def get_file(t: type) -> Optional[str]:
        module = sys.modules.get(t.__module__)
        if module and hasattr(module, '__file__'):
            return module.__file__
        return None

    def remove_null(d: dict) -> dict:
        return {key: remove_null(value) if isinstance(value, dict) else value for key, value in d.items() if
                not (value is None)}

    def decorator(data_class: type) -> type:
        expected_file_name = os.environ.get("MOOSE_PYTHON_DM_DUMP")
        if expected_file_name and expected_file_name == get_file(data_class):
            output = {
                'class_name': data_class.__name__
            }
            if arg:
                output["config"] = remove_null(asdict(arg))
            output_json = json.dumps(output, cls=CustomEncoder, indent=4)
            print(output_json, "___DATAMODELCONFIG___", sep="")
        return data_class

    if isinstance(arg, type):
        return moose_data_model(None)(arg)
    return decorator


JWTPayload = Dict[str, Any]

@dataclass
class ConsumptionApiResult:
    status: int
    body: Any

class MooseClient:
    def __init__(self, ch_client: Client):
        self.ch_client = ch_client

    def query(self, input, variables):
        params = {}
        values = {}

        for i, (_, variable_name, _, _) in enumerate(Formatter().parse(input)):
            if variable_name:
                value = variables[variable_name]
                if isinstance(value, list) and len(value) == 1:
                    # handling passing the value of the query string dict directly to variables
                    value = value[0]

                t = 'String' if isinstance(value, str) else \
                    'Int64' if isinstance(value, int) else \
                        'Float64' if isinstance(value, float) else "String"  # unknown type

                params[variable_name] = f'{{p{i}: {t}}}'
                values[f'p{i}'] = value
        clickhouse_query = input.format_map(params)

        val = self.ch_client.query(clickhouse_query, values)

        return list(val.named_results())


class Sql:
    def __init__(self, raw_strings: list[str], raw_values: list['RawValue']):
        if len(raw_strings) - 1 != len(raw_values):
            if len(raw_strings) == 0:
                raise TypeError("Expected at least 1 string")
            raise TypeError(f"Expected {len(raw_strings)} strings to have {len(raw_strings) - 1} values")

        values_length = sum(1 if not isinstance(value, Sql) else len(value.values) for value in raw_values)

        self.values: list['Value'] = [None] * values_length
        self.strings: list[str] = [None] * (values_length + 1)

        self.strings[0] = raw_strings[0]

        i = 0
        pos = 0
        while i < len(raw_values):
            child = raw_values[i]
            raw_string = raw_strings[i + 1]

            if isinstance(child, Sql):
                self.strings[pos] += child.strings[0]

                for child_index in range(len(child.values)):
                    self.values[pos] = child.values[child_index]
                    pos += 1
                    self.strings[pos] = child.strings[child_index + 1]

                self.strings[pos] += raw_string
            else:
                self.values[pos] = child
                pos += 1
                self.strings[pos] = raw_string

            i += 1


def sigterm_handler():
    print("SIGTERM received")
    sys.exit(0)


def stub():
    print("Hello from moose-lib!")
