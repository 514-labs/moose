from functools import wraps
from typing import TypeVar, Callable, Any
from .commons import Logger

T = TypeVar('T')

def task(func: Callable[..., T] = None, *, retries: int = 3) -> Callable[..., T]:
    """Decorator to mark a function as a Moose task
    
    Args:
        func: The function to decorate
        retries: Number of times to retry the task if it fails
    """
    def decorator(f: Callable[..., T]) -> Callable[..., T]:
        @wraps(f)
        def wrapper(*args, **kwargs) -> T:
            result = f(*args, **kwargs)
            
            # Ensure proper return format
            if not isinstance(result, dict):
                raise ValueError("Task must return a dictionary with 'step' and 'data' keys")
            
            if "step" not in result or "data" not in result:
                raise ValueError("Task result must contain 'step' and 'data' keys")
            
            return result
            
        # Add the markers to the wrapper
        wrapper._is_moose_task = True
        wrapper._retries = retries
        wrapper.logger = Logger(is_moose_task=True)
        return wrapper
    
    # Handle both @task and @task() syntax
    if func is None:
        return decorator
    return decorator(func)
