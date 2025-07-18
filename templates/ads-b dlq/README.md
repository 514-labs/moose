This is a [Moose](https://docs.fiveonefour.com/moose) project bootstrapped with [`moose init`](https://docs.fiveonefour.com/moose/reference/moose-cli#init) or [`aurora init`](https://docs.fiveonefour.com/aurora/cli-reference#init)

## Getting Started

Prerequisites
* [Docker Desktop](https://www.docker.com/products/docker-desktop/)
* [Node](https://nodejs.org/en)
* [An Anthropic API Key](https://docs.anthropic.com/en/api/getting-started)
* [Cursor](https://www.cursor.com/) or [Claude Desktop](https://claude.ai/download)

1. Install Moose / Aurora: `bash -i <(curl -fsSL https://fiveonefour.com/install.sh) moose,aurora`
2. Create project `aurora init aircraft ads-b`
3. Install dependencies: `cd aircraft && npm install`
5. Run Moose: `moose dev`

You are ready to go!

You can start editing the app by modifying primitives in the `app` subdirectory. The dev server auto-updates as you edit the file.

This project gets data from http://adsb.lol.

## Learn More

To learn more about Moose, take a look at the following resources:

- [Moose Documentation](https://docs.fiveonefour.com/moose) - learn about Moose.
- [Aurora Documentation](https://docs.fiveonefour.com/aurora) - learn about Aurora, the MCP interface for data engineering.

You can check out [the Next.js GitHub repository](https://github.com/vercel/next.js) - your feedback and contributions are welcome!

## Deploy on Boreal

The easiest way to deploy your Moose app is to use the [Boreal](https://www.fiveonefour.com/boreal) from Fiveonefour, the creators of Moose and Aurora.

[Sign up](https://www.boreal.cloud/sign-up).

# Template: ADS-B DLQ Version

This project processes and transforms aircraft tracking data from various sources into a standardized format for analysis and visualization. It is currently only pulling from military aircraft.

It has example workflows, data models and streaming functions. If you want to explore egress primitives, try using [Aurora](https://docs.fiveonefour.com/aurora) to generate them.

## DLQ Feature Demonstration

This project intentionally implements an ingestion pipeline that successfully processes approximately ~90% of incoming aircraft data, with the remaining 10% being routed to a Dead Letter Queue (DLQ) (where the `alt_baro` field returns `ground` instead of an integer). This is to showcase Moose's DLQ capabilities and demonstrates how AI can be leveraged to handle data that doesn't conform to standard schemas or validation rules.

## License

This template is MIT licenced.
