"use strict";
// @xq9zk7823/auth-sdk - internal package stub

function AuthProvider(props) {
  return "<div data-auth=provider>" + (props && props.children || "") + "</div>";
}

function LoginForm() {
  return "<form class=acme-login><input type=email placeholder=Email /><button type=submit>Login</button></form>";
}

function useAuth() {
  return { user: null, isAuthenticated: false, login: function(){}, logout: function(){} };
}

var init = function(config) { return { ready: true }; };
var VERSION = "1.0.0";

exports.AuthProvider = AuthProvider;
exports.LoginForm = LoginForm;
exports.useAuth = useAuth;
exports.init = init;
exports.VERSION = VERSION;