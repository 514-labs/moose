# Contributing

## Ways to contribute

You can help the project several ways

- Give us feedback on your experience with the ecosystem through the [Moose slack community](https://join.slack.com/t/moose-community/shared_invite/zt-2fjh5n3wz-cnOmM9Xe9DYAgQrNu8xKxg) or [Github Discussions](https://github.com/514-labs/moose/discussions)
- Report bugs through [Github Issues](https://github.com/514-labs/moose/issues)
- Propose changes by [opening an RFD](./rfd/0001/README.mdx)
- Give you perspective on an existing opened RFD
- Contribute to the documentation
- Contribute features and functionality by picking up issues. A good way to start is to pick up issues marked as `Good First Issue`

## Code Contributions

### Setup

Requirements:

- `Rust`: We recommend using [rustup](https://rustup.rs/) to manage your rust toolchain
- `Node`: We recommend using [nvm](https://github.com/nvm-sh/nvm#nvmrc) to manage your node versions
- [`Pnpm`](https://pnpm.io/installation)
- [`TurboRepo`](https://turbo.build/repo/docs/installing): `pnpm install turbo --global`

```bash
$ pnpm install
```

### Build

```bash
turbo build
```

### Debug

To debug the CLI, create a `.vscode/launch.json` & change the inputs according to your needs. 

```
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "CLI (OSX)",
            "preLaunchTask": "CLI build",
            "program": "${workspaceRoot}/apps/framework-cli/target/debug/moose-cli",
            "cwd": "${input:enterMooseDir}",
            "args": [
                "${input:enterCommand}",
                // "${input:enterOptions}"
            ]
        }
    ],
    "inputs": [
        {
            "id": "enterMooseDir",
            "type": "promptString",
            "description": "Directory of your moose app",
            "default": "${env:HOME}"
        },
        {
            "id": "enterCommand",
            "type": "pickString",
            "description": "CLI command to run",
            "options": ["init", "dev", "update", "stop", "clean", "help"],
            "default": "dev"
        },
        {
            "id": "enterOptions",
            "type": "promptString",
            "description": "Extra options for the command",
            "default": ""
        }
    ]
}
```

## Versioning Scheme.

We use [semantic versioning](https://semver.org/) to denote versions of the different components of the system.

We have automation that helps keep a really high cadence as we develop the initial version of the framework and ecosystem.

> We might change how we release later in the lifecycle of the project.

Currently we release every time code that changes the framework or the CLI is merged to the `main` branch. We are [releasing from the trunk](https://trunkbaseddevelopment.com/release-from-trunk/).

- By Default, every commit on `main` will increase the patch version. ie in `x.y.(z+1)`
- if the commit message contains `[minor-version]`, the bot will pick up on it and will set the version of the released packages as `x.(y + 1).0`
- if the commit message contains `[major-version]`, the bot will pick up on it and will set the version of the released packages as `(x+1).0.0`
- if the commit message contains `[no-release]` in the commit message. We do not release anything.

We ensure that we don't have conflicts as branches get merged in main by leveraging git linear history as well as the [`concurrency`](https://docs.github.com/en/actions/using-jobs/using-concurrency) github actions concept to enforce order.
