from clickhouse_connect.driver.client import Client
from dataclasses import dataclass
from enum import Enum
from string import Formatter
from typing import Callable, Generic, List, Optional, TypeVar, Union
import sys

type Key[T: (str, int)] = T 

@dataclass
class StreamingFunction:
    run: Callable

class IngestionFormat(Enum):
    JSON = "JSON"
    JSON_ARRAY = "JSON_ARRAY"

T = TypeVar('T')

@dataclass
class IngestionConfig:
    format: Optional[IngestionFormat] = None

@dataclass
class StorageConfig(Generic[T]):
    enabled: Optional[bool] = None
    order_by_fields: Optional[List[str]] = None

@dataclass
class DataModelConfig(Generic[T]):
    ingestion: Optional[IngestionConfig] = None
    storage: Optional[StorageConfig[T]] = None


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
    def __init__(self, raw_strings: List[str], raw_values: List['RawValue']):
        if len(raw_strings) - 1 != len(raw_values):
            if len(raw_strings) == 0:
                raise TypeError("Expected at least 1 string")
            raise TypeError(f"Expected {len(raw_strings)} strings to have {len(raw_strings) - 1} values")

        values_length = sum(1 if not isinstance(value, Sql) else len(value.values) for value in raw_values)

        self.values: List['Value'] = [None] * values_length
        self.strings: List[str] = [None] * (values_length + 1)

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