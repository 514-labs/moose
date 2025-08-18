This is a [Moose](https://docs.fiveonefour.com/moose) project bootstrapped with [`moose init`](https://docs.fiveonefour.com/moose/reference/moose-cli#init) or [`sloan init`](https://docs.fiveonefour.com/sloan/cli-reference#init)

This project is structured as follows
```
ads-b-frontend/
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
2. Create project `sloan init aircraft ads-b-frontend`
3. Install dependencies: `cd aircraft/moose && npm install`
5. Run Moose: `moose dev`
6. In a new terminal, install frontend dependencies `cd aircraft/frontend && npm install`
7. Run frontend: `npm run dev`

You are ready to go!

You can start editing the app by modifying primitives in the `app` subdirectory. The dev server auto-updates as you edit the file.

This project gets data from http://adsb.lol.

## Learn More

To learn more about Moose, take a look at the following resources:

- [Moose Documentation](https://docs.fiveonefour.com/moose) - learn about Moose.
- [Sloan Documentation](https://docs.fiveonefour.com/sloan) - learn about Sloan, the MCP interface for data engineering.

You can check out [the Next.js GitHub repository](https://github.com/vercel/next.js) - your feedback and contributions are welcome!

## Deploy on Boreal

The easiest way to deploy your Moose app is to use [Boreal](https://www.fiveonefour.com/boreal) from 514 Labs, the creators of Moose.

[Sign up](https://www.boreal.cloud/sign-up).

# Template: ADS-B

This project processes and transforms aircraft tracking data from various sources into a standardized format for analysis and visualization. It is currently only pulling from military aircraft.

It has example workflows, data models and streaming functions. If you want to explore egress primitives, try using [Sloan](https://docs.fiveonefour.com/sloan) to generate them.

This project also has a seed frontend written in Node, to be used when generating frontend applications on top of this.

## License

This template is MIT licenced.

