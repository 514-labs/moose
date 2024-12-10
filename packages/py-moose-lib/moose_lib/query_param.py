import dataclasses
from dataclasses import dataclass, fields, is_dataclass
from datetime import datetime
from typing import Optional, Literal, Union, Any
import inspect

scalar_types = Literal['String', 'Float', 'Int', 'Boolean', 'DateTime']


@dataclass
class ArrayType:
    element_type: scalar_types


@dataclass
class QueryField:
    name: str
    data_type: Union[scalar_types, ArrayType]
    required: bool


def to_scalar_type(t: type) -> scalar_types:
    if t == str:
        return 'String'
    elif t == int:
        return 'Int'
    elif t == float:
        return 'Float'
    elif t == bool:
        return 'Boolean'
    elif t == datetime:
        return 'DateTime'
    raise ValueError(f"Unsupported type: {t}")


def parse_scalar_value(value: str, t: scalar_types) -> Any:
    match t:
        case 'String':
            return value
        case 'Int':
            return int(value)
        case 'Float':
            return float(value)
        case 'Boolean':
            value_lower = value.lower()
            if value_lower not in ('true', 'false'):
                raise ValueError(f"Boolean value must be 'true' or 'false', got: {value}")
            return value_lower == 'true'
        case 'DateTime':
            return datetime.fromisoformat(value)
        case _:
            raise ValueError(f"Unsupported type: {t}")


def convert_dataclass_definition(cls: type) -> list[QueryField]:
    """Convert a dataclass into a list of QueryField definitions."""
    if not is_dataclass(cls):
        raise ValueError(f"Input {cls.__name__} must be a dataclass")

    fields_list = []
    for field_def in fields(cls):
        field_name = field_def.name
        field_type = field_def.type

        # Handle Optional types
        # Field is not required if it has a default value or is Optional
        required = field_def.default == field_def.default_factory == dataclasses.MISSING

        if hasattr(field_type, "__origin__"):
            if field_type.__origin__ is Optional:
                field_type = field_type.__args__[0]
                required = False
            elif field_type.__origin__ is list:
                element_type = field_type.__args__[0]
                scala_type = to_scalar_type(element_type)
                fields_list.append(QueryField(field_name, ArrayType(scala_type), required))
                continue

        scala_type = to_scalar_type(field_type)
        fields_list.append(QueryField(field_name, scala_type, required))

    return fields_list


def convert_consumption_api_param(module) -> Optional[tuple[type, list[QueryField]]]:
    run_func = module.run
    params_arg = inspect.getfullargspec(run_func).args[1]
    param_class = run_func.__annotations__.get(params_arg)
    if not param_class:
        return None
    return param_class, convert_dataclass_definition(param_class)
