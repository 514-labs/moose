from functools import wraps
from typing import TypeVar, Callable, Any

T = TypeVar('T')

def task(func: Callable[..., T]) -> Callable[..., T]:
    """Decorator to mark a function as a Moose task"""
    @wraps(func)
    def wrapper(*args, **kwargs) -> T:
        # Mark this function as a Moose task
        wrapper._is_moose_task = True
        return func(*args, **kwargs)
    
    # Add the marker to the wrapper
    wrapper._is_moose_task = True
    return wrapper
