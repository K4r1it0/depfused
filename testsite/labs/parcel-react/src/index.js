import React from "react";
import ReactDOM from "react-dom/client";
import _ from "lodash";

// Static imports from fake internal packages
import { init as designInit, VERSION as designVersion } from "@xq9zk7823/design-system";
import { init as apiInit, VERSION as apiVersion } from "@xq9zk7823/api-client";
import { log, format } from "company-internal-utils";

// Use the imported modules
const designResult = designInit({ theme: "dark" });
const apiResult = apiInit({ baseUrl: "https://api.acmecorp.internal" });

log(format("  Loaded design system v" + designVersion + "  "));
log(format("  Loaded API client v" + apiVersion + "  "));

// Use lodash to demonstrate real package usage
const data = _.merge({}, designResult, apiResult, { timestamp: _.now() });

function App() {
  const [dynamicModule, setDynamicModule] = React.useState(null);

  React.useEffect(() => {
    // Dynamic import of internal package
    import("@xq9zk7823/api-client").then((mod) => {
      setDynamicModule(mod);
    });
  }, []);

  return React.createElement(
    "div",
    { className: "app" },
    React.createElement("h1", null, "Parcel React Lab"),
    React.createElement("p", null, "Design System: " + JSON.stringify(designResult)),
    React.createElement("p", null, "API Client: " + JSON.stringify(apiResult)),
    React.createElement("p", null, "Merged Data: " + JSON.stringify(data)),
    React.createElement("p", null, "Lodash version: " + _.VERSION),
    dynamicModule
      ? React.createElement("p", null, "Dynamic module loaded: " + dynamicModule.VERSION)
      : React.createElement("p", null, "Loading dynamic module...")
  );
}

const root = ReactDOM.createRoot(document.getElementById("root"));
root.render(React.createElement(App));
