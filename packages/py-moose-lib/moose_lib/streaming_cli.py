"""
Command-line interface for running streaming functions.

This module provides a CLI for:
1. Listing available streaming functions
2. Running a streaming function
3. Exporting streaming function information

This is used by the framework-cli to run Python streaming functions.
"""

import argparse
import json
import os
import sys
from typing import Dict, List, Optional

from .streaming import get_streaming_functions, load_transforms, print_transform_info, get_transform_info
from .streaming_runner import run_streaming_function

def parse_args():
    """Parse command line arguments."""
    parser = argparse.ArgumentParser(description="Moose Python Streaming Functions CLI")
    subparsers = parser.add_subparsers(dest="command", help="Command to run")
    
    # List transforms command
    list_parser = subparsers.add_parser("list", help="List available streaming functions")
    list_parser.add_argument("--module", help="Path to the module with the model definitions")
    list_parser.add_argument("--json", action="store_true", help="Output in JSON format")
    
    # Run transform command
    run_parser = subparsers.add_parser("run", help="Run a streaming function")
    run_parser.add_argument("--module", help="Path to the module with the model definitions")
    run_parser.add_argument("--source-topic", required=True, help="Source Kafka topic")
    run_parser.add_argument("--target-topic", help="Target Kafka topic (not required for multi-transforms)")
    run_parser.add_argument("--transform-key", required=True, help="Transform function key")
    run_parser.add_argument("--broker", default="localhost:9092", help="Kafka broker address")
    run_parser.add_argument("--max-subscribers", type=int, default=1, help="Maximum number of subscribers")
    run_parser.add_argument("--sasl-username", help="SASL username")
    run_parser.add_argument("--sasl-password", help="SASL password")
    run_parser.add_argument("--sasl-mechanism", help="SASL mechanism")
    run_parser.add_argument("--security-protocol", help="Security protocol")
    
    # Export transforms command
    export_parser = subparsers.add_parser("export", help="Export streaming function information")
    export_parser.add_argument("--module", help="Path to the module with the model definitions")
    
    return parser.parse_args()

def list_transforms(module_path: Optional[str] = None, json_output: bool = False):
    """
    List available transforms.
    
    Args:
        module_path: Path to the module with the model definitions
        json_output: Whether to output in JSON format
    """
    transforms = load_transforms(module_path)
    transform_info = get_transform_info()
    
    if json_output:
        print(json.dumps(transform_info, indent=2))
    else:
        print(f"Found {len(transforms)} streaming functions:")
        for key, info in transform_info.items():
            source = info.get('source', 'unknown')
            dest = info.get('destination', 'multiple destinations')
            transform_type = info.get('type', 'unknown')
            
            print(f"  {key}:")
            print(f"    Source: {source}")
            if transform_type == 'single':
                print(f"    Destination: {dest}")
            else:
                print(f"    Type: Multi-destination transform")
            print()

def export_transforms(module_path: Optional[str] = None):
    """
    Export transform information in the format expected by the infrastructure system.
    
    Args:
        module_path: Path to the module with the model definitions
    """
    # Load transforms to ensure they're registered
    load_transforms(module_path)
    print_transform_info()

def main():
    """Main entry point for the CLI."""
    args = parse_args()
    
    if args.command == "list":
        list_transforms(args.module, args.json)
    elif args.command == "run":
        run_streaming_function(
            source_topic=args.source_topic,
            target_topic=args.target_topic,
            transform_key=args.transform_key,
            broker=args.broker,
            max_subscriber_count=args.max_subscribers,
            module_path=args.module,
            sasl_username=args.sasl_username,
            sasl_password=args.sasl_password,
            sasl_mechanism=args.sasl_mechanism,
            security_protocol=args.security_protocol
        )
    elif args.command == "export":
        export_transforms(args.module)
    else:
        print("No command specified. Use --help for usage information.")
        return 1
        
    return 0

if __name__ == "__main__":
    sys.exit(main()) 