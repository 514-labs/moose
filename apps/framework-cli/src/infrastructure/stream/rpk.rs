// A library to interact with the RPK CLI
pub enum RPKCommand {
    Topic(TopicCommand),
    Profile,
    Options,
    Cloud,
    Container,
    Debug,
    Redpanda,
}

pub enum TopicCommand {
    Create { topic_name: String },
    Delete { topic_name: String },
    Consume,
    Describe,
    List,
    AddPartitions,
    AlterConfig,
    DescribeStorage,
    Produce,
    TrimPrefix,
}

// Creates the RPK command arguments that would be passed to the RPK CLI after calling `rpk`
pub fn create_rpk_command_args(rpk_command: RPKCommand) -> Vec<String> {
    match rpk_command {
        RPKCommand::Topic(command) => match command {
            TopicCommand::Create { topic_name } => {
                vec!["topic".to_string(), "create".to_string(), topic_name]
            }
            TopicCommand::Delete { topic_name } => {
                vec!["topic".to_string(), "delete".to_string(), topic_name]
            }
            _ => {
                todo!("Command not yet implemented, implement other commands")
            }
        },
        _ => {
            todo!("not topic, implement other commands")
        }
    }
}
