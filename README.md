# Rust Server Monitor

A real-time system monitoring dashboard built using Rust and React. It collects system performance metrics and stores historical data in GreptimeDB for visualization.

## Dashboard Preview

<!-- Add dashboard overview screenshot here -->
<img width="2047" height="1117" alt="image" src="https://github.com/user-attachments/assets/daa969f6-0ce8-4591-8bf4-df1d2aa85a9b" />


## Features

- Real-time CPU usage monitoring
- Memory and disk usage tracking
- Network traffic monitoring
- Historical performance charts
- Time-range selection (1 Min, 5 Min, 15 Min, 1 Hour)
- Network interface details
- Automatic metric refresh

## Tech Stack

- **Backend:** Rust, Tokio, Axum
- **System Metrics:** sysinfo
- **Database:** GreptimeDB
- **Frontend:** React, TypeScript, Vite
- **Charts:** Recharts
- **Infrastructure:** Docker Compose

## Project Structure

```text
rust-server-monitor/
├── backend/
│   ├── src/
│   ├── migrations/
│   └── Cargo.toml
├── frontend/
│   ├── src/
│   └── package.json
├── docker-compose.yml
└── README.md
```

## Getting Started

**1. Start GreptimeDB**

```bash
docker compose up -d
```

**2. Start the Rust backend**

```bash
cd backend
cargo run
```

**3. Start the React frontend**

```bash
cd frontend
npm install
npm run dev
```

Open http://localhost:5173 in your browser.

## Screenshots

### Performance History

<!-- Add performance history screenshot here -->
<img width="2047" height="1125" alt="image" src="https://github.com/user-attachments/assets/87f655e8-d41c-4e73-a5df-d6904cf8a12f" />


### System Details

<!-- Add system details screenshot here -->
<img width="2048" height="1114" alt="image" src="https://github.com/user-attachments/assets/c56043f4-b9b9-47c8-a836-371542b13112" />


## Purpose

This project demonstrates system-level monitoring, asynchronous programming in Rust, REST API development, time-series data storage, and real-time data visualization.
