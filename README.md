# Moose Stack

[![slack](https://img.shields.io/badge/slack-igloo-brightgreen.svg?logo=slack)](https://join.slack.com/t/igloocommunity/shared_invite/zt-25gsnx2x2-9ttVTt4L9LYFrRcM6jimcg)

## Requirements

- OS
  - Linux
  - MacOS
- Languages
  - Typescript
  - Javascript
  - ... want something else? Let us know on [the slack](https://join.slack.com/t/igloocommunity/shared_invite/zt-25gsnx2x2-9ttVTt4L9LYFrRcM6jimcg)!

## Generate a new MooseJS project

```bash
$ npx create-moose-app
```

## NPM Install of the CLI

```bash
$ npm install -g @514labs/moose-cli
```

## Config file

The config file is located in `~/.moose/config.toml`

You can create one with the following content

```toml
# Feature flags to hide ongoing feature work on the CLI
[features]

# Coming soon wall on all the CLI commands as we build the MVP.
# if you want to try features as they are built, set this to false
coming_soon_wall=false
```

## Versioning

All packages and builds are synced in version when published to repositories. It makes it easy to understand the compatibility levels in the different components.

Prior to 1.0.0, there is no guaranteed compatibility between components of different versions, beside the components with the exact same version numbers.

i.e. You can expect same major and minor and patch versions of components to be compatible with each other.
