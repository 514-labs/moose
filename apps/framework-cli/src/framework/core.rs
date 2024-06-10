/// This module is going to be the command center of moose.
/// it will be responsible for taking a program definition that will be exported by
/// the language speicifc modules and build the representation of what the state of the infrastructure should be.
/// It will then build a representation of the current state of the infrastructure and compare the two to
/// determine what needs to be done.
/// It will finally build a plan of what needs to be done and delegate execution
/// to modules specific to the infrastructure provider.
///
///
/// ┌──────────────┐                     ┌──────────────┐
/// │              │                     │              │
/// │  Python      ├──┐                ┌─►  ClickHouse  │
/// │              │  │  ┌──────────┐  │ │              │
/// └──────────────┘  │  │          │  │ └──────────────┘
///                   │  │          │  │                 
///                   ├──►   Core   ├──┤                 
/// ┌──────────────┐  │  │          │  │ ┌──────────────┐
/// │              │  │  │          │  │ │              │
/// │  Typescript  ├──┘  └──────────┘  └►│   Kafka      │
/// │              │                     │              │
/// └──────────────┘                     └──────────────┘
///
pub mod code_loader;
