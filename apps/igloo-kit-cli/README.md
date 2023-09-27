# Igloo CLI
The Igloo CLI is your entrypoint to a seamless, local development experience for you data-intensive application. It's written in rust and supports building applications in TypeScript and Python.

## Installation
Before getting started you'll need to install some dependencies on your machine:
1. Python or Node
2. Docker

The Igloo CLI is available as an NPM or Pip package. To install the CLI, run the following command:

```bash
npm install -g @igloo-kit/cli
```

or 

```bash
pip install igloo-cli
```

## Usage
The Igloo CLI is a command-line tool that allows you to create, manage, and deploy your Igloo applications.

### Creating a new application
To create a new application, run the following command:

```bash
igloo init
```

This will prompt you to enter a name for your application. Once you have entered a name, the CLI will create a new directory with the name you provided and scaffold out a new Igloo application.

#### Application structure
The CLI will create the following files and directories:

```
├── .igloo
│   ├── .clickhouse
│   ├── .redpanda
│   └── ...
├── .gitignore
├── README.md
├── package.json or requirements.txt
├── app
│   ├── index.ts or index.py
│   ├── dataframes
│   │   └── ...
│   ├── flows
│   │   └── ...
│   ├── ingests
│   │   └── ...
│   ├── insights
│   │   ├── dashboards
│   │   ├── metrics
│   │   ├── models
│   │   └── ...
│   └── utils
```

##### `app` directory
The `app` directory contains all of the code for your application. This includes all of the dataframes, flows, ingests, and insights that make up your application. This directory is where you will spend most of your time developing your application.

##### `.igloo` for contributors only
The `.igloo` directory contains all of the configuration files for an application. This includes configuration for the local state of the application as well as any required local infrastructure. This directory is only used by contributors to the application and should not be committed to source control. Modifying files in this directory may cause unexpected behavior.

### Running your application in development mode
To run your application, run the following command:

```bash
igloo dev
```

This will start all of the required infrastructure and run your application in development mode. You can now make changes to your application and see them reflected in real-time.

### Seeing your application in action
Once your application is running, you can see it in action by navigating to the following URLs:
- [http://localhost:4000](http://localhost:4000) - Your application's interface if you've created one
- [http://localhost:4000/console](http://localhost:4000/console) - A console to see the data flowing through your application and the objects you've created

#### The console
The console is a great way to see the data flowing through your application. It allows you to see the data flowing through your application in real-time and interact with the objects you've created. You can also use the console to run queries against your application's data.