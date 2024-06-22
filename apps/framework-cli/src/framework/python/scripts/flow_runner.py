import argparse
import dataclasses
from datetime import datetime
from importlib import import_module
import json
import sys
from kafka import KafkaConsumer, KafkaProducer
class EnhancedJSONEncoder(json.JSONEncoder):
    def default(self, o):
        if isinstance(o, datetime):
            return o.isoformat()
        if dataclasses.is_dataclass(o):
            return dataclasses.asdict(o)
        return super().default(o)


parser = argparse.ArgumentParser(description='Run a flow')

parser.add_argument('source_topic', type=str, help='The source topic for the flow')
parser.add_argument('target_topic', type=str, help='The target topic for the flow')
parser.add_argument('target_topic_config', type=str, help='The streaming server config for target topic')
parser.add_argument('flow_file_path', type=str, help='The file of the flow to run')
parser.add_argument('broker', type=str, help='The broker to use for the flow')
# The following arguments are optional
parser.add_argument('--sasl_username', type=str, help='The SASL username to use for the flow')
parser.add_argument('--sasl_password', type=str, help='The SASL password to use for the flow')
parser.add_argument('--sasl_mechanism', type=str, help='The SASL mechanism to use for the flow')
parser.add_argument('--security_protocol', type=str, help='The security protocol to use for the flow')

args = parser.parse_args()

source_topic = args.source_topic
target_topic = args.target_topic
target_topic_config = args.target_topic_config
flow_file_path = args.flow_file_path
broker = args.broker
sasl_mechanism = args.sasl_mechanism

# Setup SASL config w/ supported mechanisms
if args.sasl_mechanism is not None:
    if args.sasl_mechanism not in ['PLAIN', 'SCRAM-SHA-256']:
        raise Exception(f"Unsupported SASL mechanism: {args.sasl_mechanism}")
    if args.sasl_username is None or args.sasl_password is None:
        raise Exception("SASL username and password must be provided if a SASL mechanism is specified")
    if args.security_protocol is None:
        raise Exception("Security protocol must be provided if a SASL mechanism is specified")

sasl_config = {
    'username': args.sasl_username,
    'password': args.sasl_password,
    'mechanism': args.sasl_mechanism
}

log_prefix = f"{args.source_topic} -> {args.target_topic}"
def log(msg):
    print(f"{log_prefix}: {msg}")

def error(msg):
    raise Exception(f"{log_prefix}: {msg}")


# message.max.bytes is a broker setting that applies to all topics.
# max.message.bytes is a per-topic setting.
# 
# In general, max.message.bytes should be less than or equal to message.max.bytes.
# If max.message.bytes is larger than message.max.bytes, the broker will still reject
# any message that is larger than message.max.bytes, even if it's sent to a topic
# where max.message.bytes is larger. So we take the minimum of the two values,
# or default to 1MB if either value is not set. 1MB is the server's default.
def get_max_message_size(config_json: str) -> int:
    config = json.loads(config_json)
    
    max_message_bytes = int(config.get("max.message.bytes", 1024 * 1024))
    message_max_bytes = int(config.get("message.max.bytes", 1024 * 1024))

    return min(max_message_bytes, message_max_bytes)


sys.path.append(args.flow_file_path)
log(f"Importing flow from {flow_file_path}")

try:
    flow = import_module('flow', package=flow_file_path)
    flow_def = flow.Flow
except Exception as e:
    error(f"Error importing flow: {e} in file {flow_file_path}")

# Get all the named flows in the flow file and make sure the flow is of type Flow
flows = [f for f in dir(flow) if isinstance(getattr(flow, f), flow_def)]

# Make sure that there is only one flow in the file
if len(flows) != 1:
    error(f"Expected one flow in the file, but got {len(flows)}")

# Get the dataclass that's the input to the flow run function

# get the flow definition
flow_def = getattr(flow, flows[0])

# get the run function
flow_run = flow_def.run

# get run input type that doesn't rely on the name of the input parameter
run_input_type = flow_run.__annotations__[list(flow_run.__annotations__.keys())[0]]

# parse json into the input type
def parse_input(json_input):
    return run_input_type(**json_input)

flow_id = f'flow-{source_topic} -> {target_topic}'
max_message_size = get_max_message_size(target_topic_config)

if sasl_config['mechanism'] is not None:
    consumer = KafkaConsumer(
        source_topic,
        client_id= "python_flow_consumer",
        group_id=flow_id,
        bootstrap_servers=broker,
        sasl_plain_username=sasl_config['username'],
        sasl_plain_password=sasl_config['password'],
        sasl_mechanism=sasl_config['mechanism'],
        security_protocol=args.security_protocol,
        # consumer_timeout_ms=10000,
        value_deserializer=lambda m: json.loads(m.decode('utf-8'))
    )
else:
    print("No sasl mechanism specified. Using default consumer.")
    consumer = KafkaConsumer(
        source_topic,
        client_id= "python_flow_consumer",
        group_id=flow_id,
        bootstrap_servers=broker,
        # consumer_timeout_ms=10000,
        value_deserializer=lambda m: json.loads(m.decode('utf-8'))
    )

# Doesn't look like python producers can be idempotent
if sasl_config['mechanism'] is not None:
    producer = KafkaProducer(
        bootstrap_servers=broker,
        sasl_plain_username=sasl_config['username'],
        sasl_plain_password=sasl_config['password'],
        sasl_mechanism=sasl_config['mechanism'],
        security_protocol=args.security_protocol,
        max_request_size=max_message_size
    )
else:
    producer = KafkaProducer(
        bootstrap_servers=broker,
        max_in_flight_requests_per_connection=1,
        max_request_size=max_message_size
    )


consumer.subscribe([source_topic])

while True:
    msg_pack = consumer.poll(timeout_ms=1000)
    transformed_messages = []
    
    total_messages = sum(len(messages) for messages in msg_pack.values())
    if total_messages > 0:
        log(f"Received a batch of {total_messages} messages")

    for tp, messages in msg_pack.items():
        for message in messages:
            # Parse the message into the input type
            input_data = parse_input(message.value)

            # Run the flow
            output_data = flow_run(input_data)

            # Handle flow function returning an array or a single object
            output_data_list = output_data if isinstance(output_data, list) else [output_data]

            for item in output_data_list:
                # Ignore flow function returning null
                if item is not None: 
                    transformed_message = json.dumps(item, cls=EnhancedJSONEncoder).encode('utf-8')
                    transformed_messages.append(transformed_message)

    # send() is asynchronous. When called it adds the record to a buffer of pending record sends 
    # and immediately returns. This allows the producer to batch together individual records
    if len(transformed_messages) > 0:
        for transformed_message in transformed_messages:
            producer.send(target_topic, transformed_message)

        log(f"Sent {len(transformed_messages)} transformed messages")

        # Ensure all messages are sent
        producer.flush()
