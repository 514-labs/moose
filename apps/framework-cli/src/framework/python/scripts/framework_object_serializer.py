import dataclasses
from enum import Enum
import json
import os
import re
import sys
import typing
import inspect

# Get the path of the current working directory
# cwd = os.getcwd()

test_dir = '/Users/timdelisle/Dev/igloo-stack/apps/framework-cli/tests/python'

project_root_dir = test_dir

framework_object_dirs = {
    'models': os.path.join(project_root_dir, 'models'),
    'flows': os.path.join(project_root_dir, 'flows'),
    'aggregations': os.path.join(project_root_dir, 'aggregations')
}

sys.path.append(framework_object_dirs['models'])
import simple

simple_contents = dir(simple)


objects = {
    'enums': {},
    'dataclasses': {}
}

def serialize_enum(enum):
    return {'name': enum.name, 'value': enum.value}

def serialize_type(type_):
    type_str = str(type_)
    class_match = re.match(r"<class '(.*)'>", type_str)
    enum_match = re.match(r"<enum '(.*)'>", type_str)
    if class_match:
        return {'kind': 'class', 'name': class_match.group(1)}
    elif enum_match:
        return {'kind': 'enum', 'name': enum_match.group(1)}
    else:
        # Handle complex typing types
        typing_match = re.match(r"typing\.(.*)\[(.*)\]", type_str)
        if typing_match:
            return {'kind': typing_match.group(1), 'type': typing_match.group(2)}
        else:
            return {'kind': 'unknown', 'name': type_str}

for item in simple_contents:
    attributes = getattr(simple, item)
    if inspect.isclass(attributes) and issubclass(attributes, Enum):
        # check if the enum is not empty
        if len(attributes) > 0:
            objects['enums'][item] = {
                'name': item,
                'type': serialize_type(attributes),
                'members': [serialize_enum(member) for member in attributes]
            }
    if dataclasses.is_dataclass(attributes):
        objects['dataclasses'][item] = {
            'name': item,
            'type': serialize_type(attributes),
            'fields': [{'name': field, 'type': serialize_type(type_)} for field, type_ in typing.get_type_hints(attributes).items()]
        }
 

print(json.dumps(objects))
