const MOCK_DATA = [
  { day: "Mon", calls: 120 },
  { day: "Tue", calls: 340 },
  { day: "Wed", calls: 280 },
  { day: "Thu", calls: 450 },
  { day: "Fri", calls: 380 },
  { day: "Sat", calls: 90 },
  { day: "Sun", calls: 60 },
];

const maxCalls = Math.max(...MOCK_DATA.map((d) => d.calls));

export default function UsageChart() {
  return (
    <div className="dash-card">
      <div className="dash-card-title">Usage this week</div>
      <div className="usage-chart">
        {MOCK_DATA.map(({ day, calls }) => (
          <div key={day} className="usage-bar-col">
            <span className="usage-bar-value">{calls}</span>
            <div
              className="usage-bar"
              style={{ height: `${(calls / maxCalls) * 100}%` }}
            />
            <span className="usage-bar-label">{day}</span>
          </div>
        ))}
      </div>
    </div>
  );
}
