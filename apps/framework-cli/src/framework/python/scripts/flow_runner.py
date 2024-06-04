import argparse
import dataclasses
from datetime import datetime
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

sys.path.append(args.flow_file_path)
import flow

# Get all the named flows in the flow file and make sure the flow is of type Flow
flows = [f for f in dir(flow) if getattr(flow, f).__class__.__name__ == 'Flow']

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
        security_protocol=args.security_protocol
    )
else:
    producer = KafkaProducer(
        bootstrap_servers=broker,
        max_in_flight_requests_per_connection=1
    )

consumer.subscribe([source_topic])

# Print each message that is consumed
for message in consumer:
    # Parse the message into the input type
    input_data = parse_input(message.value)

    # Run the flow
    output_data = flow_run(input_data)

    # Send the output to the target topic
    producer.send(target_topic, json.dumps(output_data, cls=EnhancedJSONEncoder).encode('utf-8'))




