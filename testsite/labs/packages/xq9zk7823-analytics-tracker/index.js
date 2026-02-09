"use strict";

var trackEvent = function trackEvent(name, data) { console.log("[analytics]", name, data); };
var trackPageView = function trackPageView(page) { console.log("[analytics] pageview:", page); };
var initAnalytics = function initAnalytics(config) { return { ready: true, trackEvent: trackEvent, trackPageView: trackPageView }; };
var init = function(config) { return { ready: true }; };
var VERSION = "1.0.0";

exports.trackEvent = trackEvent;
exports.trackPageView = trackPageView;
exports.initAnalytics = initAnalytics;
exports.init = init;
exports.VERSION = VERSION;
exports.default = { trackEvent: trackEvent, trackPageView: trackPageView, initAnalytics: initAnalytics, init: init, VERSION: VERSION };