from datetime import datetime

from typing import Literal, Tuple, Union, Any, Optional, get_origin, get_args
from pydantic import BaseModel

type Key[T: (str, int)] = T
type JWT[T] = T


class EnumValue(BaseModel):
    name: str
    value: int | str


class DataEnum(BaseModel):
    name: str
    values: list[EnumValue]


class Nested(BaseModel):
    name: str
    columns: list["Column"]
    jwt: bool = False


class ArrayType(BaseModel):
    element_type: "DataType"
    element_nullable: bool


type DataType = str | DataEnum | ArrayType | Nested


def handle_jwt(field_type: type) -> Tuple[bool, type]:
    if hasattr(field_type, "__origin__") and field_type.__origin__ is JWT:
        return True, field_type.__args__[0]  # type: ignore
    return False, field_type


def handle_optional(field_type: type) -> Tuple[bool, type]:
    if hasattr(field_type, "__origin__") and field_type.__origin__ is Union:
        args = field_type.__args__  # type: ignore
        if type(None) in args and len(args) == 2:
            return True, next(t for t in args if t is not type(None))
    return False, field_type


def handle_key(field_type: type) -> Tuple[bool, type]:
    if hasattr(field_type, "__origin__") and field_type.__origin__ is Key:
        return True, field_type.__args__[0]  # type: ignore
    return False, field_type


class Column(BaseModel):
    name: str
    data_type: str
    required: bool
    unique: Literal[False]
    primary_key: bool


def is_array(t: type) -> Optional[type]:
    if get_origin(t) is list:
        return get_args(t)[0]
    else:
        return None


def py_type_to_column_type(t: type) -> Tuple[bool, DataType]:
    optional, t = handle_optional(t)

    if t is str:
        data_type = "String"
    elif t is int:
        data_type = "Integer"
    elif t is float:
        data_type = "Float"
    elif t is bool:
        data_type = "Boolean"
    elif t is datetime:
        data_type = "DateTime"
    elif get_origin(t) is list:
        inner_optional, inner_type = py_type_to_column_type(get_args(t)[0])
        data_type = ArrayType(element_type=inner_type, element_nullable=inner_optional)
    elif issubclass(t, BaseModel):
        data_type = Nested(
            name=t.__name__,
            columns=_to_columns(t),
        )
    else:
        raise ValueError(f"Unknown type {t}")
    return optional, data_type


def _to_columns(model: type[BaseModel]) -> list[Column]:
    """Convert Pydantic model fields to Column definitions."""
    columns = []
    for field_name, field_info in model.model_fields.items():
        # Get the field type annotation
        field_type = field_info.annotation
        if field_type is None:
            raise ValueError(f"Missing type for {field_name}")
        primary_key, field_type = handle_key(field_type)
        is_jwt, field_type = handle_jwt(field_type)

        optional, data_type = py_type_to_column_type(field_type)

        columns.append(
            Column(
                name=field_name,
                data_type=data_type,
                required=not optional,
                unique=False,
                primary_key=primary_key
            )
        )
    return columns
