
import { useCallback, useEffect, useState } from 'react'
import SystemDetails from './components/SystemDetails'
import MetricCard from './components/MetricCard'
import MetricChart from './components/MetricChart'

import {
  Cpu,
  MemoryStick,
  HardDrive,
  Network,
  Activity,
  Server,
} from 'lucide-react'

import {
  getCurrentMetrics,
  getCpuHistory,
  getMemoryHistory,
  getDiskHistory,
  getNetworkHistory,
} from './services/api'

import type {
  TimeRange,
  SystemMetrics,
  CpuHistory,
  MemoryHistory,
  DiskHistory,
  NetworkHistory,
} from './services/api'

import './App.css'

interface ChartPoint {
  time: string
  value: number
}

interface NetworkChartPoint {
  time: string
  rx: number
  tx: number
}

const TIME_RANGES: { label: string; value: TimeRange }[] = [
  { label: '1 Min', value: '1m' },
  { label: '5 Min', value: '5m' },
  { label: '15 Min', value: '15m' },
  { label: '1 Hour', value: '1h' },
]

function formatTime(timestamp: number): string {
  return new Date(timestamp).toLocaleTimeString()
}

function App() {
  const [timeRange, setTimeRange] = useState<TimeRange>('5m')

  const [metrics, setMetrics] = useState<SystemMetrics | null>(null)

  const [cpuHistory, setCpuHistory] = useState<ChartPoint[]>([])
  const [memoryHistory, setMemoryHistory] = useState<ChartPoint[]>([])
  const [diskHistory, setDiskHistory] = useState<ChartPoint[]>([])
  const [networkHistory, setNetworkHistory] = useState<NetworkChartPoint[]>([])

  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [lastUpdated, setLastUpdated] = useState<Date | null>(null)

  const loadMetrics = useCallback(async () => {
    try {
      const [current, cpu, memory, disk, network] = await Promise.all([
        getCurrentMetrics(),
        getCpuHistory(timeRange),
        getMemoryHistory(timeRange),
        getDiskHistory(timeRange),
        getNetworkHistory(timeRange),
      ])

      setMetrics(current)

      const formatHistory = <
        T extends CpuHistory | MemoryHistory | DiskHistory
      >(
        data: T[],
        getValue: (item: T) => number,
      ): ChartPoint[] => {
        return data
          .map((item) => ({
            time: formatTime(item.timestamp),
            value: getValue(item),
          }))
          .reverse()
      }

      setCpuHistory(
        formatHistory(cpu.data, (item) => item.cpu_usage),
      )

      setMemoryHistory(
        formatHistory(memory.data, (item) => item.memory_usage_percent),
      )

      setDiskHistory(
        formatHistory(disk.data, (item) => item.disk_usage_percent),
      )

      setNetworkHistory(
        network.data
          .map((item: NetworkHistory) => ({
            time: formatTime(item.timestamp),
            rx: item.rx_speed_mb_per_sec,
            tx: item.tx_speed_mb_per_sec,
          }))
          .reverse(),
      )

      setLastUpdated(new Date())
      setError(null)
    } catch (err) {
      console.error('Failed to fetch metrics:', err)
      setError('Unable to fetch server metrics. Check backend connection.')
    } finally {
      setLoading(false)
    }
  }, [timeRange])

  // Fetch metrics immediately and refresh every 5 seconds.
  useEffect(() => {
    loadMetrics()

    const interval = setInterval(loadMetrics, 5000)

    return () => clearInterval(interval)
  }, [loadMetrics])

  const rootDisk = metrics?.disks.find(
    (disk) => disk.mount_point === '/',
  )

  const memoryUsage =
    metrics && metrics.memory_total > 0
      ? (metrics.memory_used / metrics.memory_total) * 100
      : 0

  const networkUsage =
    metrics?.networks.reduce(
      (total, item) =>
        total + item.received_bytes + item.transmitted_bytes,
      0,
    ) ?? 0

  const networkMB = networkUsage / 1024 / 1024

  return (
    <div className="dashboard">
      <header className="topbar">
        <div className="brand">
          <div className="brand-icon">
            <Activity size={22} />
          </div>

          <div>
            <h1>Server Monitor</h1>
            <p>System performance dashboard</p>
          </div>
        </div>

        <div className="server-status">
          <span className="status-dot"></span>
          Server Monitoring
        </div>
      </header>

      <main className="main-content">
        <section className="page-heading">
          <div>
            <h2>Overview</h2>
            <p>Monitor your server performance in real time.</p>
          </div>

        </section>

        {loading && !metrics && <p>Loading server metrics...</p>}

        {error && <p className="error-message">{error}</p>}

        {metrics && (
          <>
            <section className="metric-grid">
              <MetricCard
                title="CPU Usage"
                value={metrics.cpu_usage.toFixed(2)}
                unit="%"
                icon={Cpu}
                status="Live"
              />

              <MetricCard
                title="Memory Usage"
                value={memoryUsage.toFixed(2)}
                unit="%"
                icon={MemoryStick}
                status="Live"
              />

              <MetricCard
                title="Disk Usage"
                value={rootDisk?.usage_percent.toFixed(2) ?? 'N/A'}
                unit="%"
                icon={HardDrive}
                status="Live"
              />

              <MetricCard
                title="Network Traffic"
                value={networkMB.toFixed(3)}
                unit="MB/s"
                icon={Network}
                status="Active"
              />
            </section>

            <section className="charts-section">
              <div className="section-heading">
                <div>
                  <h2>Performance History</h2>
                  <p>Historical metrics from GreptimeDB</p>
                </div>
              </div>

              {/* Time-range selector */}
              <div className="time-range-selector">
                {TIME_RANGES.map((range) => (
                  <button
                    key={range.value}
                    type="button"
                    className={
                      timeRange === range.value ? 'active' : ''
                    }
                    aria-pressed={timeRange === range.value}
                    onClick={() => setTimeRange(range.value)}
                  >
                    {range.label}
                  </button>
                ))}
              </div>

              <div className="chart-grid">
                <MetricChart
                  title="CPU Usage"
                  data={cpuHistory}
                  color="#3b82f6"
                  unit="%"
                />

                <MetricChart
                  title="Memory Usage"
                  data={memoryHistory}
                  color="#a855f7"
                  unit="%"
                />

                <MetricChart
                  title="Disk Usage"
                  data={diskHistory}
                  color="#f59e0b"
                  unit="%"
                />

                <MetricChart
                  title="Network Throughput"
                  data={networkHistory}
                  color="#3b82f6"
                  secondDataKey="tx"
                  secondColor="#10b981"
                  dataKey="rx"
                  unit="MB/s"
                />
              </div>
            </section>

            <SystemDetails metrics={metrics} />

            <div className="dashboard-footer">
              <Server size={16} />
              Rust Server Monitor · Powered by Axum and GreptimeDB

              {lastUpdated && (
                <span>
                  · Updated {lastUpdated.toLocaleTimeString()}
                </span>
              )}
            </div>
          </>
        )}
      </main>
    </div>
  )
}

export default App
