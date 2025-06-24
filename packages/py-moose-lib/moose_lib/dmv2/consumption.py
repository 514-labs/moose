"""
Consumption (Egress) API definitions for Moose Data Model v2 (dmv2).

This module provides classes for defining and configuring consumption APIs
that allow querying data through user-defined functions.
"""
import os
from typing import Any, Callable, Optional, Tuple, Generic

import requests
from pydantic import BaseModel
from pydantic.json_schema import JsonSchemaValue

from .types import BaseTypedResource, T, U
from ._registry import _egress_apis

# Global base URL configuration
_global_base_url: Optional[str] = None


def set_moose_base_url(url: str) -> None:
    """Set the global base URL for consumption API calls.
    
    Args:
        url: The base URL to use for API calls
    """
    global _global_base_url
    _global_base_url = url


def get_moose_base_url() -> Optional[str]:
    """Get the configured base URL from global setting or environment variable.
    
    Returns:
        The base URL if configured, None otherwise
    """
    # Priority: programmatically set > environment variable
    if _global_base_url:
        return _global_base_url
    return os.getenv('MOOSE_BASE_URL')


class EgressConfig(BaseModel):
    """Configuration for Consumption (Egress) APIs.

    Attributes:
        version: Optional version string.
        metadata: Optional metadata for the consumption API.
    """
    version: Optional[str] = None
    metadata: Optional[dict] = None


class ConsumptionApi(BaseTypedResource, Generic[U]):
    """Represents a Consumption (Egress) API endpoint.

    Allows querying data, typically powered by a user-defined function.
    Requires two Pydantic models: `T` for query parameters and `U` for the response body.

    Args:
        name: The name of the consumption API endpoint.
        query_function: The callable that executes the query logic.
                      It receives parameters matching model `T` (and potentially
                      other runtime utilities) and should return data matching model `U`.
        config: Optional configuration (currently only `version`).
        t: A tuple containing the input (`T`) and output (`U`) Pydantic models
           (passed via `ConsumptionApi[InputModel, OutputModel](...)`).

    Attributes:
        config (EgressConfig): Configuration for the API.
        query_function (Callable[..., U]): The handler function for the API.
        name (str): The name of the API.
        model_type (type[T]): The Pydantic model for the input/query parameters.
        return_type (type[U]): The Pydantic model for the response body.
    """
    config: EgressConfig
    query_function: Callable[..., U]
    _u: type[U]

    def __class_getitem__(cls, items):
        # Handle two type parameters
        if not isinstance(items, tuple) or len(items) != 2:
            raise ValueError(f"Use `{cls.__name__}[T, U](name='...')` to supply both input and output types")
        input_type, output_type = items

        def curried_constructor(*args, **kwargs):
            return cls(t=(input_type, output_type), *args, **kwargs)

        return curried_constructor

    def __init__(self, name: str, query_function: Callable[..., U], config: EgressConfig = EgressConfig(), **kwargs):
        super().__init__()
        self._set_type(name, self._get_type(kwargs))
        self.config = config
        self.query_function = query_function
        self.metadata = config.metadata
        _egress_apis[name] = self

    @classmethod
    def _get_type(cls, keyword_args: dict):
        t = keyword_args.get('t')
        if not isinstance(t, tuple) or len(t) != 2:
            raise ValueError(f"Use `{cls.__name__}[T, U](name='...')` to supply both input and output types")

        input_type, output_type = t
        if not isinstance(input_type, type) or not issubclass(input_type, BaseModel):
            raise ValueError(f"Input type {input_type} is not a Pydantic model")
        if not isinstance(output_type, type) or not issubclass(output_type, BaseModel):
            raise ValueError(f"Output type {output_type} is not a Pydantic model")
        return t

    def _set_type(self, name: str, t: Tuple[type[T], type[U]]):
        input_type, output_type = t
        self._t = input_type
        self._u = output_type
        self.name = name

    def return_type(self) -> type[U]:
        """Get the Pydantic model type for the API's response body."""
        return self._u

    def get_response_schema(self) -> JsonSchemaValue:
        """Generates the JSON schema for the API's response body model (`U`).

        Returns:
            A dictionary representing the JSON schema.
        """
        from pydantic.type_adapter import TypeAdapter
        return TypeAdapter(self.return_type).json_schema(
            ref_template='#/components/schemas/{model}'
        )

    def call(self, params: T, base_url: Optional[str] = None) -> U:
        """Call the consumption API with the given parameters.
        
        Args:
            params: Parameters matching the input model T
            base_url: Optional base URL override. If not provided, uses the global
                     base URL set via set_base_url() or MOOSE_BASE_URL environment variable.
            
        Returns:
            Response data matching the output model U
            
        Raises:
            ValueError: If no base URL is configured
        """
        # Determine which base URL to use
        effective_base_url = base_url or get_moose_base_url()
        if not effective_base_url:
            raise ValueError(
                "No base URL configured. Set it via set_base_url(), "
                "MOOSE_BASE_URL environment variable, or pass base_url parameter."
            )

        # Construct the API endpoint URL
        url = f"{effective_base_url.rstrip('/')}/consumption/{self.name}"

        # Convert Pydantic model to dictionary
        params_dict = params.model_dump()

        # Build query parameters, handling lists as repeated params
        query_params = []
        for key, value in params_dict.items():
            if isinstance(value, list):
                # For list values, add each item as a separate query param
                for item in value:
                    query_params.append((key, str(item)))
            elif value is not None:
                query_params.append((key, str(value)))

        # Make the HTTP request
        response = requests.get(url, params=query_params)
        response.raise_for_status()  # Raise an exception for bad status codes

        # Parse JSON response and return as the expected type
        response_data = response.json()
        return self._u.model_validate(response_data)
