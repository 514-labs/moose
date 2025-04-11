
# from app.ingest import models, transforms
# import app.apis.bar as bar_api
# import app.views.bar_aggregated as bar_view

from app.apis.getHR import get_hr_api, QueryParams, QueryResult

from app.datamodels.RawAntHRPacket import RawAntHRPacket
from app.datamodels.BluetoothHRPacket import BluetoothHRPacket
from app.datamodels.ProcessedAntHRPacket import ProcessedAntHRPacket
from app.datamodels.DeadLetterQueue import DeadLetterQueue
from app.datamodels.UNIFIED_HR_MODEL import UNIFIED_HR_MODEL

from app.views.aggregatePerSecondHR import aggregateHeartRateSummaryPerSecondMV


from moose_lib import IngestPipeline, IngestPipelineConfig, MaterializedView, MaterializedViewOptions, ClickHouseEngines, ConsumptionApi

from app.streaming_functions.RawAntHRPacket__ProcessedAntHRPacket import RawAntHRPacket__ProcessedAntHRPacket
from app.streaming_functions.ProcessedAntHRPacket__UNIFIED_HR_MODEL import processedAntHRPacket__UNIFIED_HR_MODEL

rawAntHRPacketModel = IngestPipeline[RawAntHRPacket]("RawAntHRPacket", IngestPipelineConfig(
    ingest=True,
    stream=True,
    table=True
))

bluetoothHRPacketModel = IngestPipeline[BluetoothHRPacket]("BluetoothHRPacket", IngestPipelineConfig(
    ingest=True,
    stream=True,
    table=True
))

processedAntHRPacketModel = IngestPipeline[ProcessedAntHRPacket]("ProcessedAntHRPacket", IngestPipelineConfig(
    ingest=True,
    stream=True,
    table=True
))

deadLetterQueueModel = IngestPipeline[DeadLetterQueue]("DeadLetterQueue", IngestPipelineConfig(
    ingest=True,
    stream=True,
    table=True
))

unifiedHRModel = IngestPipeline[UNIFIED_HR_MODEL]("UNIFIED_HR_MODEL", IngestPipelineConfig(
    ingest=True,
    stream=True,
    table=True
))


# Transform RawAntHRPacket to ProcessedAntHRPacket in stream
rawAntHRPacketModel.get_stream().add_transform(
    destination=processedAntHRPacketModel.get_stream(),
    transformation=RawAntHRPacket__ProcessedAntHRPacket
)

processedAntHRPacketModel.get_stream().add_transform(
    destination=unifiedHRModel.get_stream(),
    transformation=processedAntHRPacket__UNIFIED_HR_MODEL
)


getHR_api = ConsumptionApi[QueryParams, QueryResult](name="getHR", query_function=get_hr_api)
