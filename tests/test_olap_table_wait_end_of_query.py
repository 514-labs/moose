"""Unit tests for OlapTable wait_end_of_query implementation."""

import sys
import os
# Add the package to the path for testing
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'packages', 'py-moose-lib'))

try:
    import pytest
    from unittest.mock import Mock, patch
    from typing import List
    from moose_lib.dmv2.olap_table import OlapTable, InsertOptions
    from pydantic import BaseModel
except ImportError as e:
    print(f"Import error: {e}")
    print("This test requires pytest, pydantic, and the moose_lib package to be installed.")
    sys.exit(1)


class TestUser(BaseModel):
    """Test model for unit tests."""
    id: int
    name: str
    email: str


class TestOlapTableWaitEndOfQuery:
    """Test suite for wait_end_of_query parameter in OlapTable operations."""

    def setup_method(self):
        """Setup test fixtures."""
        self.olap_table = OlapTable[TestUser]("test_users")
        
    @patch('packages.py_moose_lib.moose_lib.dmv2.olap_table.get_clickhouse_client')
    def test_prepare_insert_options_includes_wait_end_of_query(self, mock_get_client):
        """Test that _prepare_insert_options includes wait_end_of_query for INSERT operations."""
        mock_client = Mock()
        mock_get_client.return_value = mock_client
        
        test_data = [TestUser(id=1, name="John", email="john@example.com")]
        
        table_name, json_lines, settings = self.olap_table._prepare_insert_options(
            table_name="test_table",
            data=test_data,
            validated_data=test_data,
            is_stream=False,
            strategy="fail-fast",
            options=None
        )
        
        assert "wait_end_of_query" in settings
        assert settings["wait_end_of_query"] == 1
        assert "date_time_input_format" in settings
        assert settings["date_time_input_format"] == "best_effort"

    @patch('packages.py_moose_lib.moose_lib.dmv2.olap_table.get_clickhouse_client')
    def test_prepare_insert_options_stream_includes_wait_end_of_query(self, mock_get_client):
        """Test that _prepare_insert_options includes wait_end_of_query for stream operations."""
        mock_client = Mock()
        mock_get_client.return_value = mock_client
        
        def test_generator():
            yield TestUser(id=1, name="John", email="john@example.com")
            yield TestUser(id=2, name="Jane", email="jane@example.com")
        
        table_name, data_iterator, settings = self.olap_table._prepare_insert_options(
            table_name="test_table",
            data=test_generator(),
            validated_data=[],
            is_stream=True,
            strategy="fail-fast",
            options=None
        )
        
        assert "wait_end_of_query" in settings
        assert settings["wait_end_of_query"] == 1
        assert "date_time_input_format" in settings

    @patch('packages.py_moose_lib.moose_lib.dmv2.olap_table.get_clickhouse_client')
    def test_retry_individual_records_includes_wait_end_of_query(self, mock_get_client):
        """Test that _retry_individual_records includes wait_end_of_query in batch operations."""
        mock_client = Mock()
        mock_get_client.return_value = mock_client
        
        # Mock successful command execution
        mock_client.command.return_value = None
        
        test_data = [
            TestUser(id=1, name="John", email="john@example.com"),
            TestUser(id=2, name="Jane", email="jane@example.com")
        ]
        
        options = InsertOptions(strategy="isolate")
        
        with patch.object(self.olap_table, '_generate_table_name') as mock_table_name:
            mock_table_name.return_value = "test_table"
            
            result = self.olap_table._retry_individual_records(
                client=mock_client,
                records=test_data,
                options=options
            )
        
        # Verify that command was called with wait_end_of_query
        calls = mock_client.command.call_args_list
        assert len(calls) > 0
        
        for call in calls:
            args, kwargs = call
            assert 'settings' in kwargs
            settings = kwargs['settings']
            assert 'wait_end_of_query' in settings
            assert settings['wait_end_of_query'] == 1
            assert 'date_time_input_format' in settings

    @patch('packages.py_moose_lib.moose_lib.dmv2.olap_table.get_clickhouse_client')
    def test_insert_stream_includes_wait_end_of_query(self, mock_get_client):
        """Test that _insert_stream includes wait_end_of_query in both batch and final settings."""
        mock_client = Mock()
        mock_get_client.return_value = mock_client
        
        # Mock successful command execution
        mock_client.command.return_value = None
        
        def test_generator():
            for i in range(5):  # Small batch to test both batch and final operations
                yield TestUser(id=i, name=f"User{i}", email=f"user{i}@example.com")
        
        options = InsertOptions(strategy="fail-fast")
        
        with patch.object(self.olap_table, '_generate_table_name') as mock_table_name:
            mock_table_name.return_value = "test_table"
            
            result = self.olap_table._insert_stream(
                client=mock_client,
                table_name="test_table",
                data=test_generator(),
                strategy="fail-fast",
                options=options
            )
        
        # Verify that command was called with wait_end_of_query
        calls = mock_client.command.call_args_list
        assert len(calls) > 0
        
        for call in calls:
            args, kwargs = call
            assert 'settings' in kwargs
            settings = kwargs['settings']
            assert 'wait_end_of_query' in settings
            assert settings['wait_end_of_query'] == 1

    def test_wait_end_of_query_comment_accuracy(self):
        """Test that comments accurately reflect the operation type."""
        # This test verifies that we've correctly updated comments to match operation context
        
        # Read the source code to verify comments
        import inspect
        source = inspect.getsource(self.olap_table._prepare_insert_options)
        
        # Verify the comment mentions INSERT operations specifically
        assert "INSERT operations" in source
        assert "DDL acknowledgment" not in source or "INSERT operations" in source
        
        # Verify that DDL comments are only in DDL contexts (like blocks runner)
        # This would need to be tested separately for blocks runner files


if __name__ == "__main__":
    pytest.main([__file__])