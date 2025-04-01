"""
Streaming function runner for Python Moose projects.

This module provides functionality to:
1. Run streaming functions on incoming data
2. Connect to Kafka for message processing
3. Handle both single and multi-destination transforms

This is the Python equivalent of the runner.ts file in ts-moose-lib.
"""

import json
import os
import signal
import sys
import time
import threading
from datetime import datetime
from typing import Any, Callable, Dict, List, Optional, Union, Tuple

try:
    from kafka import KafkaConsumer, KafkaProducer
    KAFKA_AVAILABLE = True
except ImportError:
    KAFKA_AVAILABLE = False

from .streaming import get_streaming_functions, load_transforms, execute_transform

# Configurable constants
AUTO_COMMIT_INTERVAL_MS = 5000
PARTITIONS_CONSUMED_CONCURRENTLY = 3
MAX_RETRIES = 150
MAX_RETRY_TIME_MS = 1000
MAX_RETRIES_PRODUCER = 150
MAX_RETRIES_CONSUMER = 150
SESSION_TIMEOUT_CONSUMER = 30000
HEARTBEAT_INTERVAL_CONSUMER = 3000
RETRY_FACTOR_PRODUCER = 0.2
DEFAULT_MAX_STREAMING_CONCURRENCY = 100

# Get maximum concurrency from environment or use default
MAX_STREAMING_CONCURRENCY = int(os.environ.get('MAX_STREAMING_CONCURRENCY', 
                                              DEFAULT_MAX_STREAMING_CONCURRENCY))

class StreamingRunner:
    """
    Main class for running streaming transformations in Moose Python projects.
    
    This handles the lifecycle of consuming messages, applying transformations,
    and producing output messages.
    """
    
    def __init__(self, 
                 source_topic: str,
                 target_topic: Optional[str],
                 broker: str,
                 transform_function: Callable,
                 max_subscriber_count: int = 1,
                 sasl_username: Optional[str] = None,
                 sasl_password: Optional[str] = None,
                 sasl_mechanism: Optional[str] = None,
                 security_protocol: Optional[str] = None):
        """
        Initialize the streaming runner.
        
        Args:
            source_topic: Name of the source Kafka topic
            target_topic: Name of the target Kafka topic (optional for multi-destination)
            broker: Kafka broker address
            transform_function: Function to transform the data
            max_subscriber_count: Maximum number of subscribers
            sasl_username: SASL username for authentication
            sasl_password: SASL password for authentication
            sasl_mechanism: SASL mechanism for authentication
            security_protocol: Security protocol
        """
        if not KAFKA_AVAILABLE:
            raise ImportError(
                "kafka-python-ng is required for streaming functions. "
                "Install it with: pip install kafka-python-ng"
            )
            
        self.source_topic = source_topic
        self.target_topic = target_topic
        self.broker = broker
        self.transform_function = transform_function
        self.max_subscriber_count = max_subscriber_count
        
        # Authentication config
        self.sasl_config = None
        if sasl_username and sasl_password and sasl_mechanism:
            self.sasl_config = {
                'sasl_plain_username': sasl_username,
                'sasl_plain_password': sasl_password,
                'sasl_mechanism': sasl_mechanism,
                'security_protocol': security_protocol or 'SASL_PLAINTEXT'
            }
            
        # Metrics
        self.metrics = {
            'count_in': 0,
            'count_out': 0,
            'bytes_processed': 0
        }
        
        # Thread management
        self._running = False
        self._threads = []
        
    def _create_consumer(self) -> KafkaConsumer:
        """Create and configure a Kafka consumer."""
        config = {
            'bootstrap_servers': self.broker,
            'group_id': f'moose-streaming-{self.source_topic}-{self.target_topic or "multi"}',
            'auto_offset_reset': 'earliest',
            'enable_auto_commit': True,
            'auto_commit_interval_ms': AUTO_COMMIT_INTERVAL_MS,
            'session_timeout_ms': SESSION_TIMEOUT_CONSUMER,
            'heartbeat_interval_ms': HEARTBEAT_INTERVAL_CONSUMER,
            'max_poll_interval_ms': 300000,  # 5 minutes
            'fetch_max_wait_ms': 500,
            'request_timeout_ms': 305000,    # Should be > max_poll_interval_ms
            'value_deserializer': lambda v: json.loads(v.decode('utf-8'))
        }
        
        if self.sasl_config:
            config.update(self.sasl_config)
            
        return KafkaConsumer(self.source_topic, **config)
        
    def _create_producer(self) -> KafkaProducer:
        """Create and configure a Kafka producer."""
        config = {
            'bootstrap_servers': self.broker,
            'acks': 'all',
            'retries': MAX_RETRIES_PRODUCER,
            'value_serializer': lambda v: json.dumps(v).encode('utf-8')
        }
        
        if self.sasl_config:
            config.update(self.sasl_config)
            
        return KafkaProducer(**config)
    
    def _process_message(self, message: Any) -> Union[List[Any], Any, None]:
        """Process a single message through the transformation function."""
        try:
            transformed = self.transform_function(message)
            # Update metrics
            self.metrics['count_in'] += 1
            
            if transformed:
                if isinstance(transformed, list):
                    self.metrics['count_out'] += len(transformed)
                    self.metrics['bytes_processed'] += sum(
                        len(json.dumps(item).encode('utf-8')) for item in transformed
                    )
                    return transformed
                else:
                    self.metrics['count_out'] += 1
                    self.metrics['bytes_processed'] += len(json.dumps(transformed).encode('utf-8'))
                    return transformed
            
            return None
        except Exception:
            return None
    
    def _consumer_thread(self):
        """Main consumer thread function."""
        consumer = self._create_consumer()
        producer = self._create_producer() if self.target_topic else None
        
        try:
            for message in consumer:
                if not self._running:
                    break
                    
                value = message.value
                if value is None:
                    continue
                    
                transformed = self._process_message(value)
                
                if transformed is not None and producer is not None and self.target_topic:
                    if isinstance(transformed, list):
                        # Send each item in the list
                        for item in transformed:
                            producer.send(self.target_topic, item)
                    else:
                        producer.send(self.target_topic, transformed)
                        
                # Print metrics periodically (every 100 messages)
                if self.metrics['count_in'] % 100 == 0:
                    self._log_metrics()
                    
        finally:
            if producer:
                producer.close()
            consumer.close()
    
    def _log_metrics(self):
        """Log metrics information."""
        pass
    
    def start(self):
        """Start the streaming runner."""
        if self._running:
            return
            
        self._running = True
        
        # Create consumer threads
        for _ in range(self.max_subscriber_count):
            thread = threading.Thread(target=self._consumer_thread)
            thread.daemon = True
            thread.start()
            self._threads.append(thread)
        
    def stop(self):
        """Stop the streaming runner."""
        self._running = False
        
        # Wait for threads to terminate
        for thread in self._threads:
            thread.join(timeout=10)
            
        self._threads = []
        self._log_metrics()

    def process_record(self, stream_name: str, record: Any) -> List[Tuple[str, Any]]:
        """
        Process a single record through the transformation pipeline.
        
        Args:
            stream_name: Name of the source stream
            record: The record to process
            
        Returns:
            List of (destination_stream_name, transformed_record) tuples
        """
        results = []
        
        # Check for single-destination transforms
        for transform_key, transform_func in self.transform_function.items():
            if transform_key.startswith(f"{stream_name}_0_0_"):
                try:
                    # Extract destination stream name from transform key
                    dest_stream = transform_key.split("_0_0_")[1].split("_0_0")[0]
                    transformed = execute_transform(transform_func, record)
                    
                    if transformed is not None:
                        if isinstance(transformed, list):
                            for item in transformed:
                                results.append((dest_stream, item))
                        else:
                            results.append((dest_stream, transformed))
                except Exception:
                    raise
        
        # Check for multi-destination transforms
        multi_transform_key = f"{stream_name}_multi_transform"
        if multi_transform_key in self.transform_function:
            try:
                multi_transform = self.transform_function[multi_transform_key]
                transformed = execute_transform(multi_transform, record)
                
                if transformed:
                    for item in transformed:
                        results.append((item.destination.name, item.values))
            except Exception:
                raise
        
        return results

    def get_transform_info(self) -> Dict[str, Dict[str, Any]]:
        """Get information about available transforms."""
        transform_info = {}
        
        for transform_key in self.transform_function:
            if "_0_0_" in transform_key:
                source, dest = transform_key.split("_0_0_")
                dest = dest.split("_0_0")[0]
                transform_info[transform_key] = {
                    "source": source,
                    "destination": dest,
                    "type": "single"
                }
            elif transform_key.endswith("_multi_transform"):
                source = transform_key.replace("_multi_transform", "")
                transform_info[transform_key] = {
                    "source": source,
                    "type": "multi"
                }
        
        return transform_info

def run_streaming_function(transform_key: str, transform_func: Callable, data: Any) -> Any:
    """
    Executes a streaming function with the provided data.
    
    Args:
        transform_key: The key identifying the transform (e.g. "Foo_0_0_Bar_0_0")
        transform_func: The transformation function to execute
        data: The data to transform
        
    Returns:
        The transformed data
    """
    try:
        result = transform_func(data)
        return result
    except Exception:
        raise

if __name__ == "__main__":
    # This can be used for direct command-line execution
    # Arguments would be parsed here
    pass 