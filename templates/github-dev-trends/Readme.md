# GitHub Development Trends Dashboard

A real-time dashboard that tracks and visualizes development trends from GitHub. This project consists of two main components:

## Project Structure

```
github-dev-trends/
├── live-dev-trends-dashboard/ # Frontend dashboard application
├── moose/ # Backend services
└── README.md # Project documentation
```

## Overview

This project aims to provide insights into GitHub development trends by collecting and visualizing data about repositories, programming languages, frameworks, and other development patterns.

## Getting Started

### Prerequisites

- Node v20+
- Docker Desktop

### Installation

1. Clone the repository:
```bash filename="terminal" copy
npx create-moose-app GitHubDevTrends --template github-dev-trends
cd github-dev-trends
```

2. Set up the backend:
```bash filename="terminal" copy
cd moose
npm install
```

3. Set up the frontend dashboard:
```bash filename="terminal" copy
cd live-dev-trends-dashboard
npm install
```

4. Start the application:
```bash
npm run dev
```
```

## Usage

Once you successfully spin up your Moose dev server, you can start the workflow that polls the GitHub events API for the latest events:

```bash filename="terminal" copy
moose workflow start github-dev-trends
```

You can then view the dashboard by navigating to `http://localhost:3000` in your browser.