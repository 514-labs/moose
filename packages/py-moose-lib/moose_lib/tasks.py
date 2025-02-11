from functools import wraps
from typing import TypeVar, Callable, Any
from .commons import Logger

T = TypeVar('T')

def task(retries: int = 1) -> Callable[[Callable[..., T]], Callable[..., T]]:
    """Decorator to mark a function as a Moose task"""
    def decorator(func: Callable[..., T]) -> Callable[..., T]:
        @wraps(func)
        def wrapper(*args, **kwargs) -> T:
            return func(*args, **kwargs)

        # Add the marker to the wrapper
        wrapper._is_moose_task = True
        wrapper._retries = retries
        wrapper.logger = Logger(is_moose_task=True)
        return wrapper

    return decorator
