use std::{path::PathBuf, fs, env, io::{self, Write}};
use std::process::Command;


use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Optional name to operate on
    name: Option<String>,

    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// does testing things
    Init {
        #[arg(short, long)]
        list: bool,
    },
    Add(AddArgs) ,
}

#[derive(Debug, Args)]
#[command()]
struct AddArgs {
    #[command(subcommand)]
    command: Option<AddableObjects>,
}

#[derive(Debug, Subcommand)]
enum AddableObjects {
    IngestPoint,
    Dataframe,
    Metric,
    Dashboard,
    Model,

}

fn cli_run() {
    let cli = Cli::parse();

    // You can check the value provided by positional arguments, or option arguments
    if let Some(name) = cli.name.as_deref() {
        println!("Value for name: {name}");
    }

    if let Some(config_path) = cli.config.as_deref() {
        println!("Value for config: {}", config_path.display());
    }

    // You can see how many times a particular flag or argument occurred
    // Note, only flags can have multiple occurrences
    match cli.debug {
        0 => println!("Debug mode is off"),
        1 => println!("Debug mode is kind of on"),
        2 => println!("Debug mode is on"),
        _ => println!("Don't be crazy"),
    }

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Some(Commands::Init { list }) => {
            if *list {
                println!("Printing testing lists...");
            } else {
                println!("Not printing testing lists...");
            }
        }
        Some(Commands::Add(add_arg)) => {
            match &add_arg.command {
                Some(AddableObjects::IngestPoint) => {
                    println!("Adding ingestion point...");
                }
                Some(AddableObjects::Dataframe) => {
                    println!("Adding dataframe...");
                }
                Some(AddableObjects::Metric) => {
                    println!("Adding metric...");
                }
                Some(AddableObjects::Dashboard) => {
                    println!("Adding dashboard...");
                }
                Some(AddableObjects::Model) => {
                    println!("Adding model...");
                }
                None => {}
            }
        }
        None => {}
    }
}


fn remove_docker_network() {
    let output = Command::new("docker")
        .arg("network")
        .arg("rm")
        .arg("panda-house")
        .output();

    match output {
        Ok(_) => println!("Removed docker network"),
        Err(_) => println!("Failed to remove docker network"),
    }
}

fn delete_mount_volume() {
    let current_dir = env::current_dir().unwrap();
    let mount_dir = current_dir.join(".panda_house");
    
    let output = fs::remove_dir_all(mount_dir.clone());

    match output {
        Ok(_) => println!("Removed mount directory at {}", mount_dir.display()),
        Err(err) => {
            println!("Failed to remove mount directory at {}", mount_dir.display());
            println!("error: {}", err)
        },
    }
}


fn stop_container() {
    let output = Command::new("docker")
        .arg("stop")
        .arg("redpanda-1")
        .output();

    match output {
        Ok(_) => println!("Stopped docker container"),
        Err(_) => println!("Failed to stop docker container"),
    }
}

fn infra_teardown() {
    stop_container();
    remove_docker_network();
    delete_mount_volume();
}


fn create_mount_volume() {
    let current_dir = env::current_dir().unwrap();
    let mount_dir = current_dir.join(".panda_house");
    
    // This function will fail silently if the directory already exists
    let output = fs::create_dir_all(mount_dir.clone());

    match output {
        Ok(_) => println!("Created mount directory at {}", mount_dir.display()),
        Err(err) => {
            println!("Failed to create mount directory at {}", mount_dir.display());
            println!("error: {}", err)
        },
    }
}

fn create_docker_network() {
    let output = Command::new("docker")
        .arg("network")
        .arg("create")
        .arg("panda-house")
        .output();

    match output {
        Ok(_) => println!("Created docker network"),
        Err(_) => println!("Failed to create docker network"),
    }
}

fn docker_test() {
    let current_dir = env::current_dir().unwrap();
    let mount_dir = current_dir.join(".panda_house");

    println!("Running docker test");
    let mut cmd = Command::new("docker");
    
    let full_cmd = cmd.arg("run")
        .arg("-d")
        .arg("--pull=always")
        .arg("--name=redpanda-1")
        .arg("--rm")
        .arg("--network=panda-house")
        .arg("--volume=".to_owned() + mount_dir.to_str().unwrap() + ":/tmp/panda_house")
        .arg("--publish=9092:9092")
        .arg("--publish=9644:9644")
        .arg("docker.vectorized.io/vectorized/redpanda:latest")
        .arg("redpanda")
        .arg("start")
        .arg("--advertise-kafka-addr redpanda-1")
        .arg("--overprovisioned")
        .arg("--smp 1")
        .arg("--memory 2G")
        .arg("--reserve-memory 200M")
        .arg("--node-id 0")
        .arg("--check=false");

    println!("{:?}", full_cmd);

    full_cmd.spawn().unwrap().wait().unwrap();


    // match output {
    //     Ok(o) => {
    //         io::stdout().write_all(&o.stdout).unwrap();
    //         io::stderr().write_all(&o.stdout).unwrap();
    //     },
    //     Err(_) => println!("Failed to run docker ps"),
    // }
}

fn run_rp_docker_container(debug: bool) {
    let current_dir = env::current_dir().unwrap();
    let mount_dir = current_dir.join(".panda_house");

    let mut command = Command::new("docker");
        
        
    command.arg("run")
        .arg("-d")
        .arg("--pull=always")
        .arg("--name=redpanda-1")
        .arg("--rm")
        .arg("--network=panda-house")
        .arg("--volume=".to_owned() + mount_dir.to_str().unwrap() + ":/tmp/panda_house")
        .arg("--publish=9092:9092")
        .arg("--publish=9644:9644")
        .arg("docker.vectorized.io/vectorized/redpanda:latest")
        .arg("redpanda")
        .arg("start")
        .arg("--advertise-kafka-addr redpanda-1")
        .arg("--overprovisioned")
        .arg("--smp 1")
        .arg("--memory 2G")
        .arg("--reserve-memory 200M")
        .arg("--node-id 0")
        .arg("--check=false");

    println!("{:?}", command);

    let output = command.output();

    match output {
        Ok(o) => {
            if debug {
                println!("Debugging docker container run");
                io::stdout().write_all(&o.stdout).unwrap();
            }
            println!("Successfully ran docker container")
        },
        Err(_) => println!("Failed to run docker container"),
    }
        
}

fn validate_docker_run(debug: bool) {
    let output = Command::new("docker")
        .arg("ps")
        .arg("--filter")
        .arg("name=redpanda-1")
        .output();

    match output {
        Ok(o) => {
            if debug {
                io::stdout().write_all(&o.stdout).unwrap();
            }
            let output = String::from_utf8(o.stdout).unwrap();
            if output.contains("redpanda-1") {
                println!("Successfully validated docker container");
            } else {
                println!("Failed to validate docker container");
            }
        },
        Err(_) => println!("Failed to validate docker container"),
    }
}

fn validate_rp_cluster(debug: bool) {
    let output = Command::new("docker")
        .arg("exec")
        .arg("redpanda-1")
        .arg("rpk")
        .arg("cluster")
        .arg("info")
        .output();

    match output {
        Ok(o) => {
            if debug {
                io::stdout().write_all(&o.stdout).unwrap();
            }
            let output = String::from_utf8(o.stdout).unwrap();
            if output.contains("redpanda-1") {
                println!("Successfully validated docker container");
            } else {
                println!("Failed to validate docker container");
            }
        },
        Err(_) => println!("Failed to validate redpanda cluster"),
    }
}

fn run_ch_docker_container(debug: bool) {
    let mut command = Command::new("docker");
        
        
    command.arg("run")
        .arg("-d")
        .arg("--pull=always")
        .arg("--name=clickhousedb-1")
        .arg("--rm")
        .arg("--network=panda-house")
        .arg("--publish=18123:8123")
        .arg("--ulimit=nofile=262144:262144")
        .arg("docker.io/clickhouse/clickhouse-server");
    
    println!("{:?}", command);

    let output = command.output();

    match  output {
        Ok(o) => {
            if debug {
                println!("Debugging docker container run");
                io::stdout().write_all(&o.stdout).unwrap();
            }
            println!("Successfully ran clickhouse container")
        },
        Err(_) => println!("Failed to run clickhouse container"),
    }
}

fn infra_setup() {
    create_docker_network();
    create_mount_volume();
    run_rp_docker_container(true);
    validate_docker_run(true);
}

fn main() {
    // infra_setup();
    // infra_teardown();    
    // validate_rp_cluster(true);
    run_ch_docker_container(true)
}

