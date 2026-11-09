import type { LucideIcon } from 'lucide-react'

interface MetricCardProps {
  title: string
  value: string
  unit: string
  icon: LucideIcon
  status: string
}

function MetricCard({
  title,
  value,
  unit,
  icon: Icon,
  status,
}: MetricCardProps) {
  return (
    <div className="metric-card">
      <div className="metric-header">
        <span>{title}</span>
        <Icon size={20} />
      </div>

      <div className="metric-value">
        {value}
        <span>{unit}</span>
      </div>

      <div className="metric-status">{status}</div>
    </div>
  )
}

export default MetricCard