# This program is used to serialize the project data into a JSON file.
# Project data is usually defined in the setup object of the project
# and is used to generate the project moose project config. The setup
# object is assumed to live within the setup.py file of the project 
# directory. 

# Read the arguments pass in when running the script from the command line

import argparse
import importlib
import json
import sys

parser = argparse.ArgumentParser(description='Serialize the project data into a JSON file')

parser.add_argument('project_root_dir', type=str, help='The root directory of the project')

args = parser.parse_args()

# Get the path of the current working directory
path = args.project_root_dir
setup_path = path + '/setup.py'

# Import the setup object from the setup.py file
sys.path.append(path)
setup = importlib.import_module('setup')

# Serialize the setup object into a printable json string
setup_object = setup.setup

# Parse the dependencies and turn them into dictionaries that can be serialized
dependencies = setup_object['install_requires']
setup_object['install_requires'] = [{'name': dependency} for dependency in dependencies]

print(json.dumps(setup_object))
