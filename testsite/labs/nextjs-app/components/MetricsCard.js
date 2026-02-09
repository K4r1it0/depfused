import React from "react";
import { Card } from "@xq9zk7823/design-system";
import _ from "lodash";

export default function MetricsCard({ title, value, trend }) {
  const formattedValue = _.isNumber(value) ? value.toLocaleString() : value;
  const trendIcon = trend > 0 ? "up" : trend < 0 ? "down" : "flat";

  return (
    <Card title={title}>
      <div style={{ textAlign: "center" }}>
        <p style={{ fontSize: "2rem", fontWeight: "bold" }}>{formattedValue}</p>
        <p style={{ color: trend > 0 ? "green" : trend < 0 ? "red" : "gray" }}>
          {trendIcon} ({trend > 0 ? "+" : ""}{trend}%)
        </p>
      </div>
    </Card>
  );
}
