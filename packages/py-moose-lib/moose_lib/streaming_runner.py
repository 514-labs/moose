"""
Streaming function runner for Python Moose projects.

This module provides functionality to:
1. Run streaming functions on incoming data
2. Connect to Kafka for message processing
3. Handle both single and multi-destination transforms

This is the Python equivalent of the runner.ts file in ts-moose-lib.
"""

import json
import logging
import os
import signal
import sys
import time
import threading
from datetime import datetime
from typing import Any, Callable, Dict, List, Optional, Union

try:
    from kafka import KafkaConsumer, KafkaProducer
    KAFKA_AVAILABLE = True
except ImportError:
    KAFKA_AVAILABLE = False

from .streaming import get_streaming_functions, load_transforms

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

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger("moose-streaming-runner")

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
            
        consumer = KafkaConsumer(self.source_topic, **config)
        logger.info(f"Consumer connected to {self.broker} listening on {self.source_topic}")
        return consumer
        
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
            
        producer = KafkaProducer(**config)
        logger.info(f"Producer connected to {self.broker}")
        return producer
    
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
        except Exception as e:
            logger.error(f"Error transforming message: {str(e)}")
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
                    logger.warning("Received message with no value, skipping...")
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
                    
        except Exception as e:
            logger.error(f"Error in consumer thread: {str(e)}")
        finally:
            if producer:
                producer.close()
            consumer.close()
    
    def _log_metrics(self):
        """Log metrics information."""
        logger.info(
            f"Metrics - In: {self.metrics['count_in']}, "
            f"Out: {self.metrics['count_out']}, "
            f"Bytes: {self.metrics['bytes_processed']}"
        )
    
    def start(self):
        """Start the streaming runner."""
        if self._running:
            logger.warning("Streaming runner is already running")
            return
            
        self._running = True
        
        # Create consumer threads
        for _ in range(self.max_subscriber_count):
            thread = threading.Thread(target=self._consumer_thread)
            thread.daemon = True
            thread.start()
            self._threads.append(thread)
            
        logger.info(
            f"Started streaming runner with {self.max_subscriber_count} "
            f"consumers for {self.source_topic} -> {self.target_topic or 'multiple destinations'}"
        )
        
    def stop(self):
        """Stop the streaming runner."""
        logger.info("Stopping streaming runner...")
        self._running = False
        
        # Wait for threads to terminate
        for thread in self._threads:
            thread.join(timeout=10)
            
        self._threads = []
        self._log_metrics()
        logger.info("Streaming runner stopped")

def run_streaming_function(
    source_topic: str,
    target_topic: Optional[str],
    transform_key: str,
    broker: str = "localhost:9092",
    max_subscriber_count: int = 1,
    module_path: Optional[str] = None,
    sasl_username: Optional[str] = None,
    sasl_password: Optional[str] = None,
    sasl_mechanism: Optional[str] = None,
    security_protocol: Optional[str] = None
):
    """
    Run a streaming function for a specific source and target.
    
    Args:
        source_topic: Name of the source Kafka topic
        target_topic: Name of the target Kafka topic (optional for multi-destination)
        transform_key: Key identifying the transform function
        broker: Kafka broker address
        max_subscriber_count: Maximum number of consumers
        module_path: Path to the module containing the streaming functions
        sasl_username: SASL username for authentication
        sasl_password: SASL password for authentication
        sasl_mechanism: SASL mechanism for authentication
        security_protocol: Security protocol
    """
    # Load transforms
    transforms = load_transforms(module_path)
    
    if transform_key not in transforms:
        available_keys = ", ".join(transforms.keys())
        raise ValueError(
            f"Transform key '{transform_key}' not found. Available keys: {available_keys}"
        )
    
    transform_func = transforms[transform_key]
    
    # Create runner
    runner = StreamingRunner(
        source_topic=source_topic,
        target_topic=target_topic,
        broker=broker,
        transform_function=transform_func,
        max_subscriber_count=max_subscriber_count,
        sasl_username=sasl_username,
        sasl_password=sasl_password,
        sasl_mechanism=sasl_mechanism,
        security_protocol=security_protocol
    )
    
    # Handle graceful shutdown
    def signal_handler(sig, frame):
        logger.info("Received shutdown signal")
        runner.stop()
        sys.exit(0)
        
    signal.signal(signal.SIGINT, signal_handler)
    signal.signal(signal.SIGTERM, signal_handler)
    
    # Start runner
    runner.start()
    
    # Keep main thread alive
    try:
        while True:
            time.sleep(1)
    except KeyboardInterrupt:
        runner.stop()

if __name__ == "__main__":
    # This can be used for direct command-line execution
    # Arguments would be parsed here
    pass 