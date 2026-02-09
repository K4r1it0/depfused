import React from "react";
import { Button, Card, VERSION as designVersion } from "@xq9zk7823/design-system";
import { trackEvent } from "@xq9zk7823/analytics-tracker";
import _ from "lodash";
import { format } from "company-internal-utils";

export default function HomePage({ serverData }) {
  const handleClick = () => {
    trackEvent("button_click", { page: "home", action: "cta" });
  };

  const features = _.chunk([
    "Dependency Scanning",
    "Confusion Detection",
    "Bundle Analysis",
    "Real-time Alerts",
    "CI/CD Integration",
    "Report Generation"
  ], 3);

  const buttonHtml = Button({ children: "Get Started" });
  const cardHtml = Card({ title: "Features" });

  return (
    <div>
      <h1>DepFused Test Lab - Home</h1>
      <p>Design System v{designVersion}</p>
      <p>Server timestamp: {serverData.timestamp}</p>
      <p>Formatted: {format("  Hello from internal utils  ")}</p>
      <div dangerouslySetInnerHTML={{ __html: cardHtml }} />
      <div>
        {features.map((row, i) => (
          <div key={i} style={{ display: "flex", gap: "1rem", marginBottom: "1rem" }}>
            {row.map((feat, j) => (
              <div key={j} dangerouslySetInnerHTML={{ __html: Card({ title: feat }) }} />
            ))}
          </div>
        ))}
      </div>
      <div dangerouslySetInnerHTML={{ __html: buttonHtml }} />
      <button onClick={handleClick}>Track Event</button>
    </div>
  );
}

export async function getServerSideProps() {
  const logger = require("private-logger");
  const sdk = require("enterprise-sdk");
  const utils = require("company-internal-utils");

  logger.log("Home page SSR render");
  sdk.log("Enterprise SDK initialized for home page");

  return {
    props: {
      serverData: {
        timestamp: new Date().toISOString(),
        formatted: utils.format("  server-side data  ")
      }
    }
  };
}