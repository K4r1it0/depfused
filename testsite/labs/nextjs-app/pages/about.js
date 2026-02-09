import React, { useState, useEffect } from "react";
import dynamic from "next/dynamic";
import { Card, Button } from "@xq9zk7823/design-system";
import { trackPageView } from "@xq9zk7823/analytics-tracker";
import axios from "axios";

// Dynamic import of auth SDK
const AuthSection = dynamic(
  () => import("@xq9zk7823/auth-sdk").then(mod => {
    return function AuthSectionInner() {
      const authHtml = mod.LoginForm();
      const authState = mod.useAuth();
      return React.createElement("div", null,
        React.createElement("h3", null, "Authentication (" + (authState.isAuthenticated ? "logged in" : "logged out") + ")"),
        React.createElement("div", { dangerouslySetInnerHTML: { __html: authHtml } })
      );
    };
  }),
  { loading: () => React.createElement("p", null, "Loading auth..."), ssr: false }
);

// Dynamic import of payment gateway
const PaymentSection = dynamic(
  () => import("@xq9zk7823/payment-gateway").then(mod => {
    return function PaymentDemo() {
      const [payment, setPayment] = React.useState(null);
      return React.createElement("div", null,
        React.createElement("h3", null, "Payment Gateway Demo"),
        React.createElement("button", {
          onClick: () => {
            var p = mod.createPayment(99.99, "USD");
            setPayment(mod.processPayment(p.id));
          }
        }, "Process Payment"),
        payment && React.createElement("pre", null, JSON.stringify(payment, null, 2))
      );
    };
  }),
  { loading: () => React.createElement("p", null, "Loading payments..."), ssr: false }
);

export default function AboutPage({ teamData }) {
  const [apiData, setApiData] = useState(null);

  useEffect(() => {
    trackPageView("/about");
    axios.get("/api/health").then(res => setApiData(res.data)).catch(() => {});
  }, []);

  return (
    <div>
      <h1>About DepFused Test Lab</h1>
      <p>This is a test application for dependency confusion scanning.</p>
      <div dangerouslySetInnerHTML={{ __html: Card({ title: "Team: " + teamData.team }) }} />
      <h2>Authentication</h2>
      <AuthSection />
      <h2>Payments</h2>
      <PaymentSection />
      {apiData && (
        <div>
          <h3>API Health</h3>
          <pre>{JSON.stringify(apiData, null, 2)}</pre>
        </div>
      )}
    </div>
  );
}

export async function getServerSideProps() {
  const logger = require("private-logger");
  const enterpriseSdk = require("enterprise-sdk");

  logger.log("About page SSR render");
  enterpriseSdk.log("Loading team data");

  return {
    props: {
      teamData: {
        team: "AcmeCorp Security",
        version: "1.0.0"
      }
    }
  };
}