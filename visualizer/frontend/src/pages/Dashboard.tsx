import ApiKeyCard from "../components/dashboard/ApiKeyCard";
import UsageChart from "../components/dashboard/UsageChart";
import QuickStart from "../components/dashboard/QuickStart";
import PricingCards from "../components/dashboard/PricingCards";
import "./Dashboard.css";

export default function Dashboard() {
  return (
    <div className="dash">
      <header className="dash-header">
        <h1 className="dash-title">API Dashboard</h1>
        <span className="dash-badge">Preview</span>
      </header>
      <div className="dash-grid">
        <ApiKeyCard />
        <UsageChart />
        <QuickStart />
        <PricingCards />
      </div>
    </div>
  );
}
