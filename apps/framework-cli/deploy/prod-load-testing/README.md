# Prod load testing

For load testing we're using [K6 from Graphana Labs](https://k6.io/docs/).

Download the CLI on Mac via Homebrew:

```sh
brew install k6
```

Once installed create and cd into a folder you'll use for testing:

```sh
mkdir /Users/cjus/dev/moose/apps/framework-cli/deploy/prod-load-testing
cd /Users/cjus/dev/moose/apps/framework-cli/deploy/prod-load-testing
k6 new
```

A new `script.js` file will be generated.
We're using the modified `script.js` file included in this repo folder.

You can take k6 for a test run with:

```sh
k6 run script.js
```

In this next run we'll create 10 virtual users (vus) and allow the each user to run for 30 seconds.

```sh
k6 run --vus 10 --duration 30s script.js
```

