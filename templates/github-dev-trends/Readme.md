# GitHub Trending Topics Template

<a href="https://www.docs.fiveonefour.com/moose"><img src="https://raw.githubusercontent.com/514-labs/moose/main/logo-m-light.png" alt="moose logo" height="100px"></a>

This template provides a real-time dashboard tracking trending repositories and topics on GitHub. It collects and analyzes Star Events from the public GitHub events feed using Moose for the backend and a separate dashboard application for the frontend.

**Language:** TypeScript (Backend - Moose), JavaScript/TypeScript (Frontend - Next.js)
**Stack:** Moose, Node.js, Kafka, ClickHouse, Next.js/React (Frontend)

**Documentation:** [Template Documentation](https://docs.fiveonefour.com/templates/github)

## Prerequisites

*   Node.js (v20+ recommended)
*   npm (or yarn/pnpm)
*   Moose CLI
*   Docker Desktop (For underlying services like Kafka/ClickHouse if used by `moose dev`)

## Project Structure

```
.
├── dashboard/        # Next.js dashboard application
├── moose-backend/    # Moose backend service (TypeScript)
├── Readme.md         # This file
└── template.config.toml # Template specific configuration
```

## Setup

If you haven't already, install the Moose CLI:
```bash copy
bash -i <(curl -fsSL https://fiveonefour.com/install.sh) moose
```

### 1. Initialize the Project
```bash copy
moose init moose-github-dev-trends github-dev-trends
```

### 2. Moose Backend Setup

*   Navigate into the backend directory and install dependencies:
    ```bash copy
    cd moose-github-dev-trends/moose-backend && npm install
    ```
*   Set the GitHub Personal Access Token:
    *   This project requires a [GitHub Personal Access Token](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens#creating-a-personal-access-token-classic) to access the GitHub API for fetching event data.
    *   Set the `GITHUB_TOKEN` environment variable in your `.env.example` file to your GitHub token.
* Start your Moose local dev server:
    ```bash copy
    moose dev
    ```
    *   This will start the Moose backend service containing the local infrastructure (Redpanda, ClickHouse, Temporal, Webserver) and the GitHub event poller.
    *   You can verify that it's running by checking the Moose logs in your terminal.

### 2. Frontend Dashboard Setup

*   Open a **new terminal** and navigate into the dashboard directory:
    ```bash copy
    cd moose-github-dev-trends/dashboard && npm install
    ```
*   Start the frontend dashboard:
    ```bash copy
    npm run dev
    ```
    *   Navigate to the URL provided (usually `http://localhost:3000`) in your browser to view the dashboard.


## Deployment

Deploying this project involves deploying the Moose backend service and the frontend dashboard separately.

**Prerequisites:**

*   A GitHub account and your project code pushed to a GitHub repository.
*   A [Boreal](https://boreal.cloud/signup) account for the backend.
*   A [Vercel](https://vercel.com/signup) account (or similar platform) for the frontend.

### 1. Deploying the Moose Backend (Boreal)

*   **Push to GitHub:** Ensure your latest backend code (the contents of the `moose-backend` directory) is committed and pushed to your GitHub repository.
*   **Create Boreal Project:**
    *   Log in to your Boreal account and create a new project.
    *   Connect Boreal to your GitHub account and select the repository containing your project.
    *   Configure the project settings, ensuring Boreal points to the `moose-backend` directory if your repository root contains both backend and frontend.
*   **Configure Environment Variables:**
    *   In the Boreal project settings, add the `GITHUB_TOKEN` environment variable with your GitHub Personal Access Token as the value.
*   **Deploy:** Boreal should automatically build and deploy your Moose service based on your repository configuration. It will also typically start any polling sources (like the GitHub event poller) defined in your `moose.config.toml`.
*   **Note API URL:** Once deployed, Boreal will provide a public URL for your Moose backend API. You will need this for the frontend deployment.

### 2. Deploying the Frontend Dashboard (Vercel)

*   **Push to GitHub:** Ensure your latest frontend code (the contents of the `dashboard` directory) is committed and pushed to your GitHub repository.
*   **Create Vercel Project:**
    *   Log in to your Vercel account and create a new project.
    *   Connect Vercel to your GitHub account and select the repository containing your project.
*   **Configure Project Settings:**
    *   Set the **Root Directory** in Vercel to `dashboard` (or wherever your frontend code resides within the repository).
    *   Vercel should automatically detect it's a Next.js project and configure the build command (`npm run build`) and output directory correctly. Adjust if necessary.
*   **Configure Environment Variables:**
    *   This is crucial: The frontend needs to know where the deployed backend API is located.
    *   Add an environment variable in Vercel to point to your Boreal API URL. The variable name depends on how the frontend code expects it (e.g., `NEXT_PUBLIC_MOOSE_URL`). Check the frontend code (`dashboard/`) for the exact variable name.
        ```
        # Example Vercel Environment Variable
        NEXT_PUBLIC_API_URL=https://your-boreal-project-url.boreal.cloud
        ```
*   **Deploy:** Vercel will build and deploy your Next.js frontend.

Once both backend and frontend are deployed and configured correctly, your live GitHub Trends Dashboard should be accessible via the Vercel deployment URL.