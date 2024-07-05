from string import Formatter
from typing import List, Union
import sys


class MooseClient:
    def __init__(self, ch_client):
        self.ch_client = ch_client

    def query(self, input, variables):
        fieldnames = [fname for _, fname, _, _ in Formatter().parse(input) if fname]

        # Create a dictionary with keys as fieldnames and values as the corresponding string format
        params = {fname: f'{{p{i}: String}}' for i, fname in enumerate(fieldnames)}

        values = {f'p{i}': variables[fname][0] if isinstance(variables[fname], list) and len(variables[fname]) == 1 else variables[fname] for i, fname in enumerate(fieldnames) if fname in variables}
        clickhouse_query = input.format_map(params)
       
        val = self.ch_client.query(clickhouse_query, values)
        return val.result_rows
    
    
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