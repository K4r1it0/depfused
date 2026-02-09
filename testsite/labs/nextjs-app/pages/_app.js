import React, { useEffect } from "react";
import { Header, Footer } from "@xq9zk7823/design-system";
import { trackPageView, initAnalytics } from "@xq9zk7823/analytics-tracker";
import { AuthProvider } from "@xq9zk7823/auth-sdk";

if (typeof window !== "undefined") {
  initAnalytics({ appId: "nextjs-testlab", env: "production" });
}

function MyApp({ Component, pageProps }) {
  useEffect(() => {
    trackPageView(window.location.pathname);
  }, []);

  const headerHtml = Header({ title: "DepFused Test Lab" });
  const footerHtml = Footer({ text: "DepFused Test Lab - AcmeCorp 2024" });
  const authCtx = AuthProvider({ children: "app" });

  return (
    <div>
      <div dangerouslySetInnerHTML={{ __html: headerHtml }} />
      <div data-auth-context={authCtx}>
        <main style={{ minHeight: "80vh", padding: "2rem" }}>
          <Component {...pageProps} />
        </main>
      </div>
      <div dangerouslySetInnerHTML={{ __html: footerHtml }} />
    </div>
  );
}

export default MyApp;