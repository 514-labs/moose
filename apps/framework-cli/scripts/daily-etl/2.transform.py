from moose_lib import task

@task
def transform():  # The name of your script
    """
    Description of what this script does
    """
    # The body of your script goes here

    # The return value is the output of the script.
    # The return value should be a dictionary with at least:
    # - step: the step name (e.g., "extract", "transform")
    # - data: the actual data being passed to the next step
    return {
        "step": "transform",  # The step name is the name of the script
        "data": None     # The data being passed to the next step (4MB limit)
    }
