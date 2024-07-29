# Moose Product Analytics

This repository contains the code and resources you need to set up a simple product analytics solution using the Moose framework.

## Overview

Moose provisions and configures the essential infrastructure components for capturing, processing, storing, and visualizing clickstream events. The solution includes a web server with ingest and consumption API routes, a streaming data platform, an OLAP database, and a background task orchestrator.

## Getting Started

To get started, please refer to our comprehensive guide in the [documentation](https://docs.moosejs.com/templates/product-analytics). This guide will walk you through the steps to set up Moose, instrument your web application, process events, and visualize data on a dashboard.

## Prerequisites

Ensure you have Node.js and Docker installed on your machine:

```bash
docker --version && node --version
```

If you don't have them installed, you can download them from the following links:

- [Node.js](https://nodejs.org/en/download/)
- [Docker](https://www.docker.com/products/docker-desktop)

## Create a New Project

Run the following command to create a new project:

```bash
npx create-moose-app@latest moose-product-analytics --template product-analytics

```

## Running the Moose Application Locally

### Start Docker

Ensure Docker is running:

```bash
docker info
```

### Start Moose Development Server

Navigate to the `moose` directory and install dependencies:

```bash
cd moose-product-analytics/moose && npm install

```

Start the Moose development server:

```bash
npx moose-cli dev

```

### Start NextJS Dev Server

Navigate to the `next` directory and install dependencies:

```bash
cd ../next && npm install

```

Start the NextJS development server:

```bash
npm run dev

```

## Documentation

For detailed instructions and further information, please refer to the [documentation](https://docs.moosejs.com/templates/product-analytics).
