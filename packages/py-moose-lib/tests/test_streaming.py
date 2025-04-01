"""
Tests for the streaming functionality in moose_lib.
"""

import json
import unittest
from unittest.mock import patch, MagicMock

from moose_lib.dmv2 import Stream, StreamConfig, OlapTable, OlapConfig
from moose_lib.streaming import get_streaming_functions, get_transform_info
from pydantic import BaseModel

class TestModel(BaseModel):
    name: str
    value: int

class TestDestModel(BaseModel):
    processed_name: str
    processed_value: int

class TestStreamingFunctions(unittest.TestCase):
    """Test the streaming functionality."""
    
    def setUp(self):
        # Reset the registry before each test
        from moose_lib.dmv2 import _streams
        _streams.clear()
        
        # Create test streams and transformations
        self.source_stream = Stream[TestModel](name="test_source", config=StreamConfig())
        self.dest_stream = Stream[TestDestModel](name="test_dest", config=StreamConfig())
        
        # Create a test transformation
        def transform_func(data: TestModel) -> TestDestModel:
            return TestDestModel(
                processed_name=f"processed_{data.name}",
                processed_value=data.value * 2
            )
            
        # Add the transformation to the source stream
        self.source_stream.add_transform(self.dest_stream, transform_func)
        
    def test_get_streaming_functions(self):
        """Test that get_streaming_functions correctly identifies transformations."""
        # Get the streaming functions
        functions = get_streaming_functions()
        
        # Check that our transformation was found
        self.assertIn("test_source_0_0_test_dest_0_0", functions)
        
        # Test that the transformation function works correctly
        transform = functions["test_source_0_0_test_dest_0_0"]
        result = transform(TestModel(name="test", value=10))
        
        self.assertEqual(result.processed_name, "processed_test")
        self.assertEqual(result.processed_value, 20)
        
    def test_multi_destination_transform(self):
        """Test multi-destination transformations."""
        # Create a third stream for multi-destination test
        third_stream = Stream[TestModel](name="test_third", config=StreamConfig())
        
        # Create a multi-destination transformation function
        def multi_transform(data: TestModel):
            return [
                self.dest_stream.routed(TestDestModel(
                    processed_name=f"dest_{data.name}",
                    processed_value=data.value * 2
                )),
                third_stream.routed(TestModel(
                    name=f"third_{data.name}",
                    value=data.value * 3
                ))
            ]
            
        # Set the multi-transform on the source stream
        self.source_stream.set_multi_transform(multi_transform)
        
        # Get the streaming functions
        functions = get_streaming_functions()
        
        # Check that our multi-transform was found
        self.assertIn("test_source_multi_transform", functions)
        
        # Test that the multi-transform function works
        transform = functions["test_source_multi_transform"]
        results = transform(TestModel(name="test", value=10))
        
        # We should have two results
        self.assertEqual(len(results), 2)
        
        # First result should be the destination model
        self.assertEqual(results[0].processed_name, "dest_test")
        self.assertEqual(results[0].processed_value, 20)
        
        # Second result should be the third model
        self.assertEqual(results[1].name, "third_test")
        self.assertEqual(results[1].value, 30)
        
    def test_get_transform_info(self):
        """Test that get_transform_info correctly identifies transformations."""
        with patch("moose_lib.streaming.to_infra_map") as mock_infra_map:
            # Setup mock return value
            mock_infra_map.return_value = {
                "topics": {
                    "test_source": {
                        "transformationTargets": [
                            {"kind": "stream", "name": "test_dest"}
                        ],
                        "hasMultiTransform": True
                    }
                }
            }
            
            # Get the transform info
            info = get_transform_info()
            
            # Check the single-destination transform
            self.assertIn("test_source_0_0_test_dest_0_0", info)
            self.assertEqual(info["test_source_0_0_test_dest_0_0"]["source"], "test_source")
            self.assertEqual(info["test_source_0_0_test_dest_0_0"]["destination"], "test_dest")
            self.assertEqual(info["test_source_0_0_test_dest_0_0"]["type"], "single")
            
            # Check the multi-destination transform
            self.assertIn("test_source_multi_transform", info)
            self.assertEqual(info["test_source_multi_transform"]["source"], "test_source")
            self.assertEqual(info["test_source_multi_transform"]["type"], "multi")

if __name__ == "__main__":
    unittest.main() 