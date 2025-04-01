"""
Streaming function discovery and execution for Python Moose projects.

This module provides functionality to:
1. Discover streaming functions in a Python Moose project
2. Execute transformations when data arrives
3. Support both single-destination and multi-destination transforms
"""

import importlib.util
import os
import sys
import json
from typing import Any, Callable, Dict, List, Optional, Tuple, Union

from .dmv2 import Stream, _streams, _RoutedMessage
from .internal import to_infra_map

def get_streaming_functions() -> Dict[str, Callable]:
    """
    Discovers all streaming functions defined in the project.
    
    Similar to the TypeScript version, this extracts all transformation functions
    from the registered streams and prepares them for execution.
    
    Returns:
        A dictionary mapping transformation keys to transformation functions.
        The key format is "{source_stream_name}_0_0_{target_stream_name}_0_0"
    """
    transform_functions = {}
    
    # Extract single-destination transforms
    for stream_name, stream in _streams.items():
        for dest_name, (destination, transform_func) in stream.transformations.items():
            # Use same key format as TypeScript for compatibility
            transform_key = f"{stream_name}_0_0_{dest_name}_0_0"
            transform_functions[transform_key] = transform_func
    
    # Also handle multi-destination transforms
    for stream_name, stream in _streams.items():
        if stream._multipleTransformations:
            # For multi-transforms, we need a wrapper that handles the routing
            def create_multi_transform_wrapper(stream, multi_transform):
                def wrapper(data):
                    # Call the multi-transform function to get routed messages
                    routed_msgs = multi_transform(data)
                    # Process each routed message
                    results = []
                    for routed_msg in routed_msgs:
                        if isinstance(routed_msg, _RoutedMessage):
                            dest_stream = routed_msg.destination
                            values = routed_msg.values
                            if values is not None:
                                # If it's a list, extend results; otherwise append
                                if isinstance(values, list):
                                    results.extend(values)
                                else:
                                    results.append(values)
                    return results
                return wrapper
            
            # Create a wrapper for the multi-transform function
            multi_transform_func = create_multi_transform_wrapper(
                stream, stream._multipleTransformations
            )
            
            # Create a special key for multi-transforms
            multi_transform_key = f"{stream_name}_multi_transform"
            transform_functions[multi_transform_key] = multi_transform_func
    
    return transform_functions

def load_transforms(module_path: Optional[str] = None) -> Dict[str, Callable]:
    """
    Loads transformations from a specified module.
    
    Args:
        module_path: Path to the module to load (can be a file path or module name).
                   If None, looks for 'main.py' in the current directory.
    
    Returns:
        A dictionary mapping transformation keys to transformation functions.
    """
    if module_path is None:
        # Try common module paths, similar to what's in internal.py
        possible_paths = [
            os.path.join(os.getcwd(), "main.py"),
            os.path.join(os.getcwd(), "app", "main.py"),
            os.path.join(os.getcwd(), "models.py"),
            os.path.join(os.getcwd(), "app", "models.py")
        ]
        
        for path in possible_paths:
            if os.path.exists(path):
                module_path = path
                break
                
        if module_path is None:
            raise FileNotFoundError(
                "Could not find model definitions. Please specify the module path."
            )
    
    # Check if it's a file path or module name
    if os.path.exists(module_path):
        # Load from file path
        dir_path = os.path.dirname(os.path.abspath(module_path))
        if dir_path not in sys.path:
            sys.path.insert(0, dir_path)
            
        module_name = os.path.basename(module_path).replace(".py", "")
        spec = importlib.util.spec_from_file_location(module_name, module_path)
        if spec and spec.loader:
            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)
    else:
        # Try to import as a module name
        try:
            importlib.import_module(module_path)
        except ImportError as e:
            raise ImportError(f"Could not load module: {module_path}")
    
    # Return the discovered streaming functions
    return get_streaming_functions()

def execute_transform(transform_func: Callable, data: Any) -> Any:
    """
    Executes a transformation function on the provided data.
    
    Args:
        transform_func: The transformation function to execute
        data: The data to transform
        
    Returns:
        The transformed data
    """
    try:
        result = transform_func(data)
        return result
    except Exception as e:
        raise

def get_transform_info() -> Dict[str, Any]:
    """
    Gets information about all available transforms in the project.
    
    Returns:
        A dictionary with information about the transforms, including
        source and destination streams.
    """
    # Get the infrastructure map which contains information about streams and their transforms
    infra_map = to_infra_map()
    
    # Extract streaming function info from the topics (streams)
    transform_info = {}
    for topic_name, topic_config in infra_map.get("topics", {}).items():
        # Get transformation targets (single-destination transforms)
        for target in topic_config.get("transformationTargets", []):
            if target.get("kind") == "stream":
                transform_key = f"{topic_name}_0_0_{target['name']}_0_0"
                transform_info[transform_key] = {
                    "source": topic_name,
                    "destination": target["name"],
                    "type": "single"
                }
        
        # Check for multi-transforms
        if topic_config.get("hasMultiTransform", False):
            multi_transform_key = f"{topic_name}_multi_transform"
            transform_info[multi_transform_key] = {
                "source": topic_name,
                "type": "multi"
            }
    
    return transform_info

def print_transform_info():
    """
    Prints information about all available transforms in the project.
    
    This is useful for debugging and for the CLI to discover transforms.
    Prints in the format expected by the infrastructure system.
    """
    transform_info = get_transform_info()
    print("___MOOSE_TRANSFORMS___start", json.dumps(transform_info), "end___MOOSE_TRANSFORMS___") 