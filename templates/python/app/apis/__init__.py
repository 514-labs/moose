from moose_lib.dmv2 import ConsumptionApi
import app.apis.bar as bar

bar = ConsumptionApi[bar.QueryParams, bar.QueryResult](name="bar", query_function=bar.run)

__all__ = ["bar"]