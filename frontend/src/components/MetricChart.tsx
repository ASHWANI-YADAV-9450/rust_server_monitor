
import {
  ResponsiveContainer,
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
} from 'recharts'

// Supports CPU, memory, disk and network chart data.
interface ChartPoint {
  time: string
  value?: number
  rx?: number
  tx?: number
}

interface MetricChartProps {
  title: string
  data: ChartPoint[]
  color: string
  unit: string
  dataKey?: string
  secondDataKey?: string
  secondColor?: string
}

function MetricChart({
  title,
  data,
  color,
  unit,
  dataKey = 'value',
  secondDataKey,
  secondColor = '#10b981',
}: MetricChartProps) {
  return (
    <div className="chart-card">
      <h3>{title}</h3>

      <ResponsiveContainer width="100%" height={250}>
        <LineChart data={data}>
          <CartesianGrid strokeDasharray="3 3" />

          <XAxis dataKey="time" />

          <YAxis />

          <Tooltip
            formatter={(value) => [`${value} ${unit}`]}
          />

          {secondDataKey && <Legend />}

          <Line
            type="monotone"
            dataKey={dataKey}
            name="Receive"
            stroke={color}
            strokeWidth={2}
            dot={false}
          />

          {secondDataKey && (
            <Line
              type="monotone"
              dataKey={secondDataKey}
              name="Transmit"
              stroke={secondColor}
              strokeWidth={2}
              dot={false}
            />
          )}
        </LineChart>
      </ResponsiveContainer>
    </div>
  )
}

export default MetricChart
