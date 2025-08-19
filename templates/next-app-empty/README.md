This is a [Moose](https://docs.fiveonefour.com/moose) project bootstrapped with [`moose init`](https://docs.fiveonefour.com/moose/reference/moose-cli#init) or [`sloan init`](https://docs.fiveonefour.com/sloan/cli-reference#init)

This project is structured as follows
```
next-app-empty/
├── frontend/ # Frontend placeholder in Node
├── moose/ # Backend services
└── README.md # Project documentation
```

## Getting Started

Prerequisites
* [Docker Desktop](https://www.docker.com/products/docker-desktop/)
* [Node](https://nodejs.org/en)
* [An Anthropic API Key](https://docs.anthropic.com/en/api/getting-started)
* [Cursor](https://www.cursor.com/) or [Claude Desktop](https://claude.ai/download)

1. Install Moose / Sloan: `bash -i <(curl -fsSL https://fiveonefour.com/install.sh) moose,sloan`
2. Create project `sloan init <project-name> next-app-empty`
3. Install dependencies: `cd <project-name>/moose && npm install`
4. Run Moose: `moose dev`
5. In a new terminal, install frontend dependencies `cd <project-name>/frontend && npm install`
7. Run frontend: `npm run dev`

You are ready to go!

You can start editing the app by modifying primitives in the `app` subdirectory. The dev server auto-updates as you edit the file.

This project gets data from http://adsb.lol.

## Learn More

To learn more about Moose, take a look at the following resources:

- [Moose Documentation](https://docs.fiveonefour.com/moose) - learn about Moose.
- [Sloan Documentation](https://docs.fiveonefour.com/sloan) - learn about Sloan, the MCP interface for data engineering.

You can check out [the Moose GitHub repository](https://github.com/514-labs/moose) - your feedback and contributions are welcome!

## Community

You can join the Moose community [on Slack](https://join.slack.com/t/moose-community/shared_invite/zt-2fjh5n3wz-cnOmM9Xe9DYAgQrNu8xKxg). Check out the [MooseStack repo on GitHub](https://github.com/514-labs/moose).

## Deploy on Boreal

The easiest way to deploy your Moose app is to use [Boreal](https://www.fiveonefour.com/boreal) from 514 Labs, the creators of Moose.

[Sign up](https://www.boreal.cloud/sign-up).

## License

This template is MIT licenced.

