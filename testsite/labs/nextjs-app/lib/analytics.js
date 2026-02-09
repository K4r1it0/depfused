// Analytics wrapper using internal packages
const { trackEvent, trackPageView, initAnalytics } = require("@xq9zk7823/analytics-tracker");
const { log } = require("private-logger");

function setupAnalytics(config) {
  log("Setting up analytics with config: " + JSON.stringify(config));
  return initAnalytics(config);
}

function track(event, data) {
  log("Tracking: " + event);
  trackEvent(event, data);
}

function pageView(page) {
  log("Page view: " + page);
  trackPageView(page);
}

module.exports = { setupAnalytics, track, pageView };
