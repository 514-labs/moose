# Igloo-stack

Supports Linux and MacOS

## Generate a new igloo project

```bash
$ npx create-igloo-app
```

## Install the CLI (Node)

```bash
$ npm install -g @514labs/igloo-cli
```

## Config file

The config file is located in `~/.igloo-config.toml`

You can create one with the following content

```toml
# Feature flags to hide ongoing feature work on the CLI
[features]

# Coming soon wall on all the CLI commands as we build the MVP.
# if you want to try features as they are built, set this to false
coming_soon_wall=true
```

## Versioning

All packages and builds are synced in version when published to repositories. It makes it easy to understand the compatibility levels in the different components. 

Prior to 1.0.0, there is no guaranteed compatibility between components of different versions, beside the components with the exact same version numbers. 

i.e. You can expect same major and minor and patch versions of components to be compatible with each other.
