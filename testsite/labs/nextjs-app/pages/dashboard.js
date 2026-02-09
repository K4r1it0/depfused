import React, { useState, useEffect, useCallback } from "react";
import dynamic from "next/dynamic";
import { Card, Button } from "@xq9zk7823/design-system";
import { useAuth } from "@xq9zk7823/auth-sdk";
import { trackEvent, trackPageView } from "@xq9zk7823/analytics-tracker";
import _ from "lodash";
import { format } from "company-internal-utils";

const AnalyticsDashboard = dynamic(
  () => import("@xq9zk7823/analytics-tracker").then(mod => {
    return function AnalyticsWidget() {
      var evtCount = React.useState(0);
      return React.createElement("div", null,
        React.createElement("h3", null, "Analytics Dashboard"),
        React.createElement("p", null, "Events tracked: " + evtCount[0]),
        React.createElement("button", {
          onClick: () => { mod.trackEvent("dashboard_action", { ts: Date.now() }); evtCount[1](evtCount[0] + 1); }
        }, "Track Event")
      );
    };
  }),
  { ssr: false }
);

const PaymentHistory = dynamic(
  () => import("@xq9zk7823/payment-gateway").then(mod => {
    return function History() {
      var payments = [
        mod.createPayment(100, "USD"),
        mod.createPayment(250, "EUR"),
        mod.createPayment(75, "GBP")
      ];
      return React.createElement("div", null,
        React.createElement("h3", null, "Payment History"),
        React.createElement("ul", null,
          payments.map(function(p, i) { return React.createElement("li", { key: i }, p.id + " - " + p.amount + " " + p.currency); })
        )
      );
    };
  }),
  { ssr: false }
);

export default function DashboardPage({ dashboardData }) {
  var authState = useAuth();
  var metricsState = useState(dashboardData.metrics);
  var metrics = metricsState[0];
  var setMetrics = metricsState[1];

  useEffect(function() {
    trackPageView("/dashboard");
  }, []);

  var handleRefresh = useCallback(function() {
    trackEvent("dashboard_refresh", { user: authState.user });
    var newMetrics = _.mapValues(metrics, function(v) { return typeof v === "number" ? v + Math.floor(Math.random() * 10) : v; });
    setMetrics(newMetrics);
  }, [metrics, authState.user]);

  var formattedTitle = format("  Dashboard Overview  ");

  return (
    <div>
      <h1>{formattedTitle}</h1>
      <p>Auth status: {authState.isAuthenticated ? "Authenticated" : "Not authenticated"}</p>
      <div style={{ display: "grid", gridTemplateColumns: "repeat(3, 1fr)", gap: "1rem" }}>
        {Object.entries(metrics).map(function(entry) {
          return (
            <div key={entry[0]} dangerouslySetInnerHTML={{ __html: Card({ title: _.startCase(entry[0]) + ": " + entry[1] }) }} />
          );
        })}
      </div>
      <div style={{ marginTop: "2rem" }}>
        <div dangerouslySetInnerHTML={{ __html: Button({ children: "Refresh" }) }} />
        <button onClick={handleRefresh}>Refresh Metrics</button>
      </div>
      <AnalyticsDashboard />
      <PaymentHistory />
    </div>
  );
}

export async function getServerSideProps() {
  var logger = require("private-logger");
  var sdk = require("enterprise-sdk");
  var utils = require("company-internal-utils");
  var lodash = require("lodash");

  logger.log("Dashboard SSR render");
  sdk.log("Loading dashboard metrics");

  var metrics = {
    totalScans: lodash.random(100, 500),
    vulnerabilities: lodash.random(0, 50),
    dependencies: lodash.random(50, 200),
    alerts: lodash.random(0, 10),
    packages: lodash.random(20, 100),
    score: lodash.random(70, 100)
  };

  return {
    props: {
      dashboardData: {
        metrics: metrics,
        lastUpdated: new Date().toISOString(),
        formatted: utils.format("  dashboard loaded  ")
      }
    }
  };
}