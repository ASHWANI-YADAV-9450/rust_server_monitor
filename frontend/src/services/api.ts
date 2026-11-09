
const API_BASE_URL = '/api'

export type TimeRange = '1m' | '5m' | '15m' | '1h'

const TIME_RANGE_LIMITS: Record<TimeRange, number> = {
  '1m': 12,
  '5m': 60,
  '15m': 180,
  '1h': 720,
}

// Generic API request helper
async function fetchApi<T>(endpoint: string): Promise<T> {
  const response = await fetch(`${API_BASE_URL}${endpoint}`)

  if (!response.ok) {
    throw new Error(
      `API request failed: ${response.status} ${response.statusText}`
    )
  }

  return response.json() as Promise<T>
}

// Types matching Rust SystemMetrics
export interface DiskMetrics {
  name: string
  mount_point: string
  total_space: number
  available_space: number
  used_space: number
  usage_percent: number
}

export interface NetworkMetrics {
  interface: string
  received_bytes: number
  transmitted_bytes: number
  total_received_bytes: number
  total_transmitted_bytes: number
}

export interface SystemMetrics {
  cpu_usage: number
  memory_used: number
  memory_total: number
  disks: DiskMetrics[]
  networks: NetworkMetrics[]
}

// Historical metric types
export interface CpuHistory {
  cpu_usage: number
  timestamp: number
}

export interface MemoryHistory {
  memory_used: number
  memory_total: number
  memory_usage_percent: number
  timestamp: number
}

export interface DiskHistory {
  disk_used: number
  disk_total: number
  disk_usage_percent: number
  timestamp: number
}

export interface NetworkHistory {
  network_rx_bytes: number
  network_tx_bytes: number
  rx_speed_mb_per_sec: number
  tx_speed_mb_per_sec: number
  timestamp: number
}

// API functions
// API functions
export const getCurrentMetrics = () =>
  fetchApi<SystemMetrics>('/metrics')

export const getCpuHistory = (range: TimeRange = '5m') =>
  fetchApi<{ host: string; count: number; data: CpuHistory[] }>(
    `/metrics/cpu/history?limit=${TIME_RANGE_LIMITS[range]}`
  )

export const getMemoryHistory = (range: TimeRange = '5m') =>
  fetchApi<{ host: string; count: number; data: MemoryHistory[] }>(
    `/metrics/memory/history?limit=${TIME_RANGE_LIMITS[range]}`
  )

export const getDiskHistory = (range: TimeRange = '5m') =>
  fetchApi<{ host: string; count: number; data: DiskHistory[] }>(
    `/metrics/disk/history?limit=${TIME_RANGE_LIMITS[range]}`
  )

export const getNetworkHistory = (range: TimeRange = '5m') =>
  fetchApi<{ host: string; count: number; data: NetworkHistory[] }>(
    `/metrics/network/history?limit=${TIME_RANGE_LIMITS[range]}`
  )
