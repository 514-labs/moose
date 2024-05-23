import os
import sys

# Get the path of the current working directory
# cwd = os.getcwd()

test_dir = '/Users/timdelisle/Dev/igloo-stack/apps/framework-cli/tests/python'

project_root_dir = test_dir

framework_object_dirs = {
    'models': os.path.join(project_root_dir, 'models'),
    'flows': os.path.join(project_root_dir, 'flows'),
    'aggregations': os.path.join(project_root_dir, 'aggregations')
}

sys.path.append(framework_object_dirs['models'])
import simple

simple_contents = dir(simple)

print(simple_contents)
    