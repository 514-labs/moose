from dataclasses import dataclass
from typing import List

@dataclass
class Blocks:
    teardown: List[str]
    setup: List[str]

# Define the SQL queries for setup and teardown
setup_queries = [
    """
    CREATE VIEW userLanguages AS
    SELECT DISTINCT 
        username AS user, 
        arrayFirst(x->x IS NOT NULL, language_object.language) AS language, 
        arrayFirst(x->x IS NOT NULL, language_object.bytes) AS bytes 
    FROM 
        local.ProcessedStarEvent_0_0
    ARRAY JOIN 
    languages AS language_object
    """
]

teardown_queries = [
    "DROP VIEW userLanguages"
]

# Create the block with setup and teardown queries
block = Blocks(teardown=teardown_queries, setup=setup_queries)
