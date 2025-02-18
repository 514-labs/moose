from functools import wraps
from typing import TypeVar, Callable, Any, Awaitable
from .commons import Logger
import asyncio

T = TypeVar('T')

def task(func: Callable[..., T] = None, *, retries: int = 3) -> Callable[..., T]:
    """Decorator to mark a function as a Moose task
    
    Args:
        func: The function to decorate
        retries: Number of times to retry the task if it fails
    """
    def validate_result(result: Any) -> None:
        """Ensure proper return format"""
        if not isinstance(result, dict):
            raise ValueError("Task must return a dictionary with 'task' and 'data' keys")
        if "task" not in result or "data" not in result:
            raise ValueError("Task result must contain 'task' and 'data' keys")

    def decorator(f: Callable[..., T]) -> Callable[..., T]:
        if asyncio.iscoroutinefunction(f):
            @wraps(f)
            async def wrapper(*args, **kwargs) -> T:
                result = await f(*args, **kwargs)
                validate_result(result)
                return result
        else:
            @wraps(f)
            def wrapper(*args, **kwargs) -> T:
                result = f(*args, **kwargs)
                validate_result(result)
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
