from dataclasses import dataclass

@dataclass
class Aggregations:
    sql: str
    order_by: str

agg = Aggregations(
    sql="SELECT * FROM table",
    order_by="name"
)

