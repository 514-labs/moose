# Workflow Orchestration

## Overview
Workflows in Moose enable you to automate sequences of tasks in a maintainable, reliable way. They provide a powerful way to orchestrate complex data processing pipelines, ETL jobs, and scheduled tasks. Moose workflows are built on top of Temporal, providing enterprise-grade durability and reliability.

## Basic Workflow Setup

```python
from moose_lib.dmv2.workflow import Workflow, Task, WorkflowConfig, TaskConfig
from pydantic import BaseModel
from typing import Optional

class WorkflowInput(BaseModel):
    user_id: str
    start_date: str
    end_date: str

class WorkflowOutput(BaseModel):
    processed_records: int
    errors: list[str]

class ValidationResult(BaseModel):
    valid: bool
    message: str

# Define task functions
async def validate_input(input: WorkflowInput) -> ValidationResult:
    """Validate input data"""
    if not input.user_id or not input.start_date or not input.end_date:
        return ValidationResult(valid=False, message="Missing required input fields")
    return ValidationResult(valid=True, message="Input validated successfully")

async def process_data(input: WorkflowInput) -> WorkflowOutput:
    """Process user data"""
    # Simulate data processing
    processed_count = 100  # This would be actual processing logic
    return WorkflowOutput(processed_records=processed_count, errors=[])

# Create tasks with typed inputs and outputs
validate_task = Task[WorkflowInput, ValidationResult](
    name="validateInput",
    config=TaskConfig(
        run=validate_input,
        timeout="30s",
        retries=3
    )
)

process_task = Task[WorkflowInput, WorkflowOutput](
    name="processData", 
    config=TaskConfig(
        run=process_data,
        timeout="5m",
        retries=2
    )
)

# Chain tasks using on_complete
validate_task.config.on_complete = [process_task]

# Create workflow
workflow = Workflow(
    name="DataProcessingWorkflow",
    config=WorkflowConfig(
        starting_task=validate_task,
        timeout="1h",
        retries=3
    )
)
```

## Task Configuration

The `TaskConfig` class provides comprehensive configuration options:

```python
from typing import Union, Callable, Awaitable
from moose_lib.dmv2.workflow import TaskConfig, Task

# TaskConfig accepts different function signatures:
TaskConfig(
    run: Union[
        Callable[[], None],                          # No input, no output
        Callable[[], Awaitable[OutputModel]],        # No input, with output  
        Callable[[InputModel], None],                # With input, no output
        Callable[[InputModel], Awaitable[OutputModel]]  # With input, with output
    ],
    on_complete: Optional[list[Task]] = None,        # Tasks to run after completion
    timeout: Optional[str] = None,                   # Timeout string (e.g., "5m", "1h")
    retries: Optional[int] = None                    # Number of retry attempts
)
```

### Task Dependencies and Chaining

```python
# Create a chain of dependent tasks
task1 = Task[InputModel, IntermediateModel](
    name="firstTask",
    config=TaskConfig(run=first_function)
)

task2 = Task[IntermediateModel, OutputModel](
    name="secondTask", 
    config=TaskConfig(run=second_function)
)

task3 = Task[OutputModel, None](
    name="finalTask",
    config=TaskConfig(run=final_function)
)

# Chain tasks
task1.config.on_complete = [task2]
task2.config.on_complete = [task3]

# Create workflow starting with first task
workflow = Workflow(
    name="ChainedWorkflow",
    config=WorkflowConfig(starting_task=task1)
)
```

## Workflow Operations

### Working with the Workflow Registry

All workflows are automatically registered and can be accessed:

```python
from moose_lib.dmv2._registry import _workflows

# Get workflow by name
my_workflow = _workflows.get("DataProcessingWorkflow")

# List all task names in workflow
task_names = my_workflow.get_task_names()
print("Tasks:", task_names)

# Find specific task
specific_task = my_workflow.get_task("validateInput")
```

### Running Workflows via CLI

Workflows are executed using the Moose CLI:

```bash
# Run a workflow
npx moose-cli workflow run DataProcessingWorkflow

# Run with input data
npx moose-cli workflow run DataProcessingWorkflow --input '{"user_id": "123", "start_date": "2024-01-01", "end_date": "2024-01-31"}'

# Check workflow status
npx moose-cli workflow status DataProcessingWorkflow

# List all workflows
npx moose-cli workflow list

# Terminate a running workflow
npx moose-cli workflow terminate DataProcessingWorkflow
```

## Advanced Configuration

### Scheduled Workflows

```python
# Workflow with scheduled execution
scheduled_workflow = Workflow(
    name="DailyCleanupWorkflow",
    config=WorkflowConfig(
        starting_task=cleanup_task,
        schedule="0 0 * * *",  # Daily at midnight (cron format)
        timeout="2h",
        retries=3
    )
)
```

### Error Handling and Retries

```python
async def risky_operation(input: DataModel) -> ResultModel:
    """Function that might fail and needs retries"""
    try:
        # Risky operation that might fail
        result = await external_api_call(input)
        return ResultModel(success=True, data=result)
    except Exception as e:
        # Log error for debugging
        print(f"Operation failed: {e}")
        raise  # Re-raise to trigger retry logic

risky_task = Task[DataModel, ResultModel](
    name="riskyOperation",
    config=TaskConfig(
        run=risky_operation,
        retries=5,           # Retry up to 5 times
        timeout="10m"        # 10 minute timeout per attempt
    )
)
```

### Parallel Task Execution

```python
# Create parallel tasks that don't depend on each other
parallel_task1 = Task[InputModel, Output1](
    name="parallelTask1",
    config=TaskConfig(run=function1)
)

parallel_task2 = Task[InputModel, Output2](
    name="parallelTask2", 
    config=TaskConfig(run=function2)
)

# Both tasks can run in parallel from the starting task
starting_task.config.on_complete = [parallel_task1, parallel_task2]
```

## Best Practices

### 1. Type Safety
```python
# Always use typed models for inputs and outputs
class UserData(BaseModel):
    user_id: str
    email: str
    created_at: datetime

class ProcessingResult(BaseModel):
    success: bool
    records_processed: int
    errors: list[str]

# Typed task ensures compile-time safety
typed_task = Task[UserData, ProcessingResult](
    name="processUser",
    config=TaskConfig(run=process_user_data)
)
```

### 2. Task Granularity
```python
# Keep tasks focused and single-purpose
async def validate_email(user: UserData) -> ValidationResult:
    """Single purpose: validate email format"""
    if "@" not in user.email:
        return ValidationResult(valid=False, error="Invalid email format")
    return ValidationResult(valid=True, error=None)

async def send_welcome_email(user: UserData) -> EmailResult:
    """Single purpose: send welcome email"""
    # Email sending logic
    return EmailResult(sent=True, message_id="12345")
```

### 3. Timeout and Retry Configuration
```python
# Configure appropriate timeouts and retries
TaskConfig(
    run=database_operation,
    timeout="30s",      # Short timeout for database operations
    retries=3           # Moderate retries for transient failures
)

TaskConfig(
    run=external_api_call,
    timeout="2m",       # Longer timeout for external APIs
    retries=5           # More retries for network issues
)

TaskConfig(
    run=file_processing,
    timeout="10m",      # Very long timeout for file processing
    retries=1           # Fewer retries for resource-intensive operations
)
```

### 4. Resource Management
```python
async def process_large_dataset(input: DatasetInfo) -> ProcessingResult:
    """Handle large datasets efficiently"""
    try:
        # Process in chunks to avoid memory issues
        results = []
        for chunk in input.get_chunks(chunk_size=1000):
            chunk_result = await process_chunk(chunk)
            results.append(chunk_result)
        
        return ProcessingResult(
            total_processed=sum(r.count for r in results),
            errors=[e for r in results for e in r.errors]
        )
    except Exception as e:
        # Clean up resources on failure
        await cleanup_temp_files()
        raise
```

## Integration with Moose Data Models

### Ingesting Data from Workflows

```python
import httpx
from moose_lib.dmv2.workflow import Task, TaskConfig

async def ingest_processed_data(result: ProcessingResult) -> None:
    """Send processed data to Moose ingestion endpoint"""
    
    # Transform result to match your data model
    ingestion_data = {
        "id": result.id,
        "timestamp": result.processed_at.isoformat(),
        "records_count": result.records_processed,
        "success": result.success
    }
    
    # Send to ingestion endpoint
    async with httpx.AsyncClient() as client:
        response = await client.post(
            "http://localhost:4000/ingest/ProcessingResults",
            json=ingestion_data,
            timeout=30.0
        )
        
        if not response.is_success:
            raise Exception(f"Failed to ingest data: {response.status_code}")

# Create ingestion task
ingest_task = Task[ProcessingResult, None](
    name="ingestResults",
    config=TaskConfig(
        run=ingest_processed_data,
        retries=3,
        timeout="1m"
    )
)
```

## Monitoring and Debugging

### Workflow Visibility

```python
# Access workflow structure for monitoring
workflow = _workflows["DataProcessingWorkflow"]

# Get all task names for monitoring dashboards
all_tasks = workflow.get_task_names()
print(f"Workflow has {len(all_tasks)} tasks: {all_tasks}")

# Find specific tasks for targeted monitoring
critical_task = workflow.get_task("processPayment")
if critical_task:
    print(f"Critical task timeout: {critical_task.config.timeout}")
    print(f"Critical task retries: {critical_task.config.retries}")
```

### Debugging with Temporal UI

Moose workflows integrate with Temporal's debugging tools:

1. **Timeline View**: Visual representation of workflow execution
2. **Event History**: Detailed log of all workflow events  
3. **Retry Information**: Track retry attempts and failures
4. **Task Status**: Real-time status of individual tasks

Access the Temporal UI at `http://localhost:8080` during development.

## Example: Complete E-commerce Order Processing

```python
from moose_lib.dmv2.workflow import Workflow, Task, WorkflowConfig, TaskConfig
from pydantic import BaseModel
from datetime import datetime
from typing import Optional

# Data models
class Order(BaseModel):
    order_id: str
    user_id: str
    items: list[dict]
    total_amount: float

class PaymentResult(BaseModel):
    success: bool
    transaction_id: Optional[str] = None
    error_message: Optional[str] = None

class InventoryResult(BaseModel):
    available: bool
    reserved_items: list[str]

class ShippingResult(BaseModel):
    shipping_id: str
    estimated_delivery: datetime

class OrderResult(BaseModel):
    order_id: str
    status: str
    payment_id: Optional[str] = None
    shipping_id: Optional[str] = None

# Task functions
async def validate_order(order: Order) -> Order:
    """Validate order data"""
    if not order.items or order.total_amount <= 0:
        raise ValueError("Invalid order data")
    return order

async def check_inventory(order: Order) -> InventoryResult:
    """Check and reserve inventory"""
    # Simulate inventory check
    return InventoryResult(
        available=True,
        reserved_items=[item["id"] for item in order.items]
    )

async def process_payment(order: Order) -> PaymentResult:
    """Process payment"""
    # Simulate payment processing
    return PaymentResult(
        success=True,
        transaction_id=f"txn_{order.order_id}"
    )

async def arrange_shipping(order: Order) -> ShippingResult:
    """Arrange shipping"""
    return ShippingResult(
        shipping_id=f"ship_{order.order_id}",
        estimated_delivery=datetime.now()
    )

async def finalize_order(order: Order) -> OrderResult:
    """Finalize order and send to ingestion"""
    # Create final result
    result = OrderResult(
        order_id=order.order_id,
        status="completed"
    )
    
    # Ingest into Moose
    async with httpx.AsyncClient() as client:
        await client.post(
            "http://localhost:4000/ingest/Orders",
            json=result.dict()
        )
    
    return result

# Create tasks
validate_task = Task[Order, Order](
    name="validateOrder",
    config=TaskConfig(run=validate_order, timeout="30s", retries=2)
)

inventory_task = Task[Order, InventoryResult](
    name="checkInventory", 
    config=TaskConfig(run=check_inventory, timeout="1m", retries=3)
)

payment_task = Task[Order, PaymentResult](
    name="processPayment",
    config=TaskConfig(run=process_payment, timeout="2m", retries=2)
)

shipping_task = Task[Order, ShippingResult](
    name="arrangeShipping",
    config=TaskConfig(run=arrange_shipping, timeout="1m", retries=3)
)

finalize_task = Task[Order, OrderResult](
    name="finalizeOrder",
    config=TaskConfig(run=finalize_order, timeout="30s", retries=2)
)

# Chain tasks
validate_task.config.on_complete = [inventory_task]
inventory_task.config.on_complete = [payment_task, shipping_task]  # Parallel execution
payment_task.config.on_complete = [finalize_task]
shipping_task.config.on_complete = [finalize_task]

# Create workflow
order_workflow = Workflow(
    name="OrderProcessingWorkflow",
    config=WorkflowConfig(
        starting_task=validate_task,
        timeout="30m",
        retries=1
    )
)
```

This comprehensive example demonstrates a real-world order processing workflow with proper error handling, parallel execution, and integration with Moose's data ingestion system.

