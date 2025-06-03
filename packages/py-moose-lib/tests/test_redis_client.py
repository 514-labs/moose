import os
import sys
import subprocess
import pytest
from moose_lib import MooseCache

@pytest.mark.integration
def test_atexit_cleanup():
    # Create a test script that will be run in a separate process
    test_script = """
import sys
from moose_lib.clients.redis_client import MooseCache

# Create instance
cache = MooseCache()
print("Created cache instance")

# Force exit without calling disconnect
sys.exit(0)
"""

    # Write the test script to a temporary file
    with open("test_atexit.py", "w") as f:
        f.write(test_script)

    try:
        # Run the script and capture output
        result = subprocess.run([sys.executable, "test_atexit.py"],
                              capture_output=True,
                              text=True)

        # Check if we see both the connection and disconnection messages
        output = result.stdout + result.stderr
        print("\nTest output:")
        print("------------")
        print(output)
        print("------------")

        assert "Python Redis client connected" in output
        assert "Python Redis client disconnected" in output
        print("\nTest passed! Verified atexit cleanup is working.")

    finally:
        # Clean up the test script
        os.remove("test_atexit.py")

if __name__ == "__main__":
    test_atexit_cleanup()