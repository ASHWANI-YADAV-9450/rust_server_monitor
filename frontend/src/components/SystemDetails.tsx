
import { useState } from 'react'
import {
    MemoryStick,
    HardDrive,
    Network,
} from 'lucide-react'

import type { SystemMetrics } from '../services/api'

interface SystemDetailsProps {
    metrics: SystemMetrics
}

function formatBytes(bytes: number): string {
    if (bytes < 1024) {
        return `${bytes.toFixed(0)} B`
    }

    if (bytes < 1024 ** 2) {
        return `${(bytes / 1024).toFixed(2)} KB`
    }

    if (bytes < 1024 ** 3) {
        return `${(bytes / 1024 ** 2).toFixed(2)} MB`
    }

    return `${(bytes / 1024 ** 3).toFixed(2)} GB`
}

function SystemDetails({ metrics }: SystemDetailsProps) {
    const memoryAvailable = Math.max(
        0,
        metrics.memory_total - metrics.memory_used,
    )

    // Use only the root mount to avoid duplicate APFS storage.
    const rootDisk = metrics.disks.find(
        (disk) => disk.mount_point === '/',
    )

    const activeNetworks = metrics.networks.filter(
        (network) =>
            network.total_received_bytes > 0 ||
            network.total_transmitted_bytes > 0
    )

    const [showAllNetworks, setShowAllNetworks] = useState(false)

    const displayedNetworks = showAllNetworks
    ? metrics.networks
    : activeNetworks

    return (
        <section className="system-details">
            <div className="section-heading">
                <div>
                    <h2>System Details</h2>
                    <p>Detailed information about your machine</p>
                </div>
            </div>

            <div className="details-grid">

                {/* Memory details */}
                <div className="details-card">
                    <div className="details-title">
                        <MemoryStick size={20} />
                        <h3>Memory Information</h3>
                    </div>

                    <div className="detail-row">
                        <span>Total Memory</span>
                        <strong>{formatBytes(metrics.memory_total)}</strong>
                    </div>

                    <div className="detail-row">
                        <span>Used Memory</span>
                        <strong>{formatBytes(metrics.memory_used)}</strong>
                    </div>

                    <div className="detail-row">
                        <span>Available Memory</span>
                        <strong>{formatBytes(memoryAvailable)}</strong>
                    </div>

                    <div className="progress-track">
                        <div
                            className="progress-fill memory-fill"
                            style={{
                                width: `${Math.min(
                                    100,
                                    (metrics.memory_used / Math.max(1, metrics.memory_total)) * 100,
                                )}%`,
                            }}
                        />
                    </div>
                </div>

                {/* Disk details */}
                <div className="details-card">
                    <div className="details-title">
                        <HardDrive size={20} />
                        <h3>Disk Information</h3>
                    </div>

                    {rootDisk ? (
                        <>
                            <div className="detail-row">
                                <span>Mount Point</span>
                                <strong>{rootDisk.mount_point}</strong>
                            </div>

                            <div className="detail-row">
                                <span>Total Capacity</span>
                                <strong>{formatBytes(rootDisk.total_space)}</strong>
                            </div>

                            <div className="detail-row">
                                <span>Used Space</span>
                                <strong>{formatBytes(rootDisk.used_space)}</strong>
                            </div>

                            <div className="detail-row">
                                <span>Available Space</span>
                                <strong>{formatBytes(rootDisk.available_space)}</strong>
                            </div>

                            <div className="progress-track">
                                <div
                                    className="progress-fill disk-fill"
                                    style={{
                                        width: `${Math.min(100, rootDisk.usage_percent)}%`,
                                    }}
                                />
                            </div>
                        </>
                    ) : (
                        <p>No root disk information available.</p>
                    )}
                </div>

                {/* Network details */}
                <div className="details-card network-details-card">
                    <div className="details-title">
                        <Network size={20} />
                        <h3>Network Interfaces</h3>
                        <div className="network-controls">
                            <button
                                className="network-filter-button"
                                onClick={() => setShowAllNetworks((current) => !current)}
                            >
                                {showAllNetworks ? 'Active Only' : 'Show All'}
                            </button>
                        </div>
                    </div>

                    <div className="network-table-wrapper">
                        <table className="network-table">
                            <thead>
                                <tr>
                                    <th>Interface</th>
                                    <th>Received</th>
                                    <th>Transmitted</th>
                                </tr>
                            </thead>

                            <tbody>
                                {displayedNetworks.map((network) => (
                                    <tr key={network.interface}>
                                        <td>
                                            {network.interface}
                                            {network.interface === 'lo0' && (
                                                <span className="interface-label"> (Loopback)</span>
                                            )}
                                        </td>
                                        <td>{formatBytes(network.total_received_bytes)}</td>
                                        <td>{formatBytes(network.total_transmitted_bytes)}</td>
                                    </tr>
                                ))}
                            </tbody>
                        </table>
                    </div>
                </div>

            </div>
        </section>
    )
}

export default SystemDetails
