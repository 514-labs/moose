from temporalio.client import Client, TLSConfig
from typing import Dict, Any
import os

async def create_temporal_connection(temporal_url: str, client_cert: str, client_key: str, api_key: str) -> Client:
    """
    Create a connection to the Temporal server with appropriate authentication.
    """
    namespace = "default"
    if "localhost" not in temporal_url:
        # Extract namespace from URL for non-local connections
        host_part = temporal_url.split(":")[0]
        import re
        match = re.match(r"^([^.]+\.[^.]+)", host_part)
        if match and match.group(1):
            namespace = match.group(1)

    print(f"Using namespace from URL: {namespace}")
    
    client_options: Dict[str, Any] = {
        "target_host": temporal_url,
        "namespace": namespace,
    }
    
    if "localhost" not in temporal_url:
        # Handle non-local connections with TLS or API key
        print("Using TLS for non-local Temporal")
        
        if client_cert and client_key:
            with open(client_cert, "rb") as f:
                cert = f.read()
            with open(client_key, "rb") as f:
                key = f.read()
                
            client_options["tls"] = TLSConfig(
                client_cert=cert,
                client_private_key=key,
            )
        elif api_key:
            # Use regional endpoint for API key authentication
            print("Using API key for non-local Temporal")
            client_options["target_host"] = "us-west1.gcp.api.temporal.io:7233"
            client_options["api_key"] = api_key
            client_options["tls"] = True
    
    print(f"Connecting to Temporal at {temporal_url}")
    client = await Client.connect(**client_options)
    print("Connected to Temporal server")
    return client