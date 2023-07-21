struct Directory {
    name: String,
    path: String,
    subdirectories: Vec<Directory>,
    files: Vec<File>,
}

pub enum ApplicationStructure {
    Ingestion(Directory),
    Dataframes(Directory),
    Insights(Directory),
}