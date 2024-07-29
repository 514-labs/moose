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
cd moose && npm install

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
This application architecture helps you build a simple product analytics solution using Moose. It sets up infrastructure for ingesting, processing, storing, and visualizing clickstream events. Components include a web server for data ingestion and consumption, a streaming data platform (Redpanda), an OLAP database (Clickhouse), and a NextJS dashboard for data visualization.

For more details, [please refer to the full Template documentation](https://docs.moosejs.com/templates/product-analytics).
# Product Analytics Template

## Introduction

Adding live analytics to apps can be complex, costly, and time-consuming. That's why we created a specific solution using MooseJS to make adding analytics to your app easier, faster and cheaper. This ready-made solution provides all the tools needed to quickly understand and leverage user data for driving your product strategy.

## Getting Started

To initiate a new project with the Product Analytics template, run the following command in your terminal:

```bash
npx create-moose-app@latest <YOUR_PROJECT_NAME> --template product-analytics
```

Replace `<YOUR_PROJECT_NAME>` with your desired project name. This should be a unique identifier for your new project.

## Project Structure

Executing the command will create a new directory on your local machine, named after your project. This directory includes two main components:

- Backend Data Service (Moose application): This folder contains the backend logic and data processing elements of your Product Analytics application, including pre-built data models and transformation pipelines for ingesting and processing your event data streams.
- Frontend Analytics Dashboard (Next application): This folder hosts the analytics dashboard application, designed for interactive data visualization and analysis.

These components work together to form an end-to-end product analytics solution, providing the infrastructure and front-end interfaces necessary for analyzing user interactions with your product.

## Running the Template Locally

### Start Your Moose Development Server

Start your Moose development server by executing the following commands in your terminal. Ensure you are executing these commands from the root directory of your analytics project.

```bash
cd moose && npx @514labs/moose-cli@latest dev
```

Running this sequence of commands will spin up all the backend infrastructure powering your backend data service (running on port 4000).

Here is the default `PageViewEvent` data model you are provided in your `/datamodels` folder.

```typescript
// models.ts

interface PageViewEvent {
    eventId    Key<string>
    timestamp  DateTime
    session_id String
    user_agent String
    locale     String
    location   String
    href       String
    pathname   String
    referrer   String
}
```

### Start Your NextJS Dashboard Application

From your project’s root directory in your terminal, run the following commands to navigate to your NextJS project and start the NextJS development server:

```bash
cd next && npm run dev
```

Port Configuration: Note the port number used (typically 3000), as it is needed for setting up event tracking in Step 3.

## Integrating the Analytics Script

The analytics template includes an integrated script that automatically sends PageView event data to the Moose application, capturing basic user interactions. The provided script additionally automatically enriches your event data with common web analytics properties. All of these properties are defined in the PageViewEvent data model schema showcased above.

### Locating the Script

The `script.js` file is located in the `/public` folder of your Next.js analytics dashboard application.
When the Next.js application runs, it automatically serves this tracking script, making it available for inclusion in the web application you wish to instrument event tracking.

### Add the Script to Your Application:

Insert the following script tag into the `<head>` section of your web application's HTML:

```html
<script
  data-host="http://localhost:4000"
  data-event="PageViewEvent/0.0"
  src="http://localhost:<PORT_NUMBER>/script.js"
></script>
```

Replace `<PORT_NUMBER>` with the port number where your Next.js server is running.
The data-host attribute in the script tag points to the URL where your Moose analytics service is running, ensuring all your captured data is successfully transmitted to your backend.

## Tracking Custom Events

### Define a New Data Model

To track custom events effectively, start by defining a new data model based on your specific requirements. Begin with the global properties found in the `PageViewEvent` model, then add custom properties relevant to the specific events you want to track.

#### Example: Button Click Event

For a button click event, you might create a data model named ButtonClickEvent with properties that capture both the context and user interaction details:

```typescript
interface ButtonClickEvent {
    eventId    Key<string>
    timestamp  DateTime
    session_id String
    user_agent String
    locale     String
    location   String
    href       String
    pathname   String
    referrer   String
    cta_copy   String // Text on the button clicked
    cta_target   String // Target link redirected by the button
}
```

### Sending Custom Events

To send custom events, leverage the `script.js` that is already integrated into your application.

#### Example: Tracking Button Click Events

Add the following code to the `onClick()` event handler for the button you want to track:

```typescript
window.MooseAnalytics.trackEvent("ButtonClickEvent", {
  cta_copy: button_text, // Dynamically capture the button text
  cta_target: target_href, // Capture the target URL
});
```

#### Key Points:

- The first argument to `trackEvent()` corresponds to the data model name, ensuring the right model is used for transmitting your data to Moose.
- The second argument is an object that includes the properties unique to the `ButtonClickEvent`, as specified in your data model.
- The integrated script enriches the event data with common web analytics properties defined in the PageViewEvent model. This means you do not need to manually set these properties, as they are automatically appended to your event data payload in the tracking script.
