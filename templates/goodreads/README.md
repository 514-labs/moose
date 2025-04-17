This is a [Moose](https://docs.fiveonefour.com/moose) project bootstrapped with [`moose init`](https://docs.fiveonefour.com/moose/reference/moose-cli#init) or [`aurora init`](https://docs.fiveonefour.com/aurora/cli-reference#init)

## Getting Started

Prerequisites
* [Docker Desktop](https://www.docker.com/products/docker-desktop/)
* [Node](https://nodejs.org/en)
* [A Kaggle API Key](https://www.kaggle.com/docs/api)
* [An Anthropic API Key](https://docs.anthropic.com/en/api/getting-started)
* [Cursor](https://www.cursor.com/) or [Claude Desktop](https://claude.ai/download)

1. Install Moose / Aurora: `bash -i <(curl -fsSL https://fiveonefour.com/install.sh) moose,aurora`
2. Create project `aurora init books goodreads`
3. Install dependencies: `cd books && npm install`
4. Add your Kaggle Settings file to authenticate (For more information, see https://www.kaggle.com/docs/api)
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

# Template: Goodreads

This project contains data models for storing and processing Goodreads book information.

It has example workflows, data models and streaming functions. If you want to explore egress primitives, try using [Aurora](https://docs.fiveonefour.com/aurora) to generate them.

## License

This template is MIT licenced.
