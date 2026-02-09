// Auth service - uses auth-sdk and api-client with decorators and async/await
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
function _export(target, all) {
    for(var name in all)Object.defineProperty(target, name, {
        enumerable: true,
        get: Object.getOwnPropertyDescriptor(all, name).get
    });
}
_export(exports, {
    get AuthService () {
        return AuthService;
    },
    get default () {
        return _default;
    }
});
var _authsdk = require("@xq9zk7823/auth-sdk");
var _apiclient = require("@xq9zk7823/api-client");
function asyncGeneratorStep(gen, resolve, reject, _next, _throw, key, arg) {
    try {
        var info = gen[key](arg);
        var value = info.value;
    } catch (error) {
        reject(error);
        return;
    }
    if (info.done) {
        resolve(value);
    } else {
        Promise.resolve(value).then(_next, _throw);
    }
}
function _async_to_generator(fn) {
    return function() {
        var self = this, args = arguments;
        return new Promise(function(resolve, reject) {
            var gen = fn.apply(self, args);
            function _next(value) {
                asyncGeneratorStep(gen, resolve, reject, _next, _throw, "next", value);
            }
            function _throw(err) {
                asyncGeneratorStep(gen, resolve, reject, _next, _throw, "throw", err);
            }
            _next(undefined);
        });
    };
}
function _class_call_check(instance, Constructor) {
    if (!(instance instanceof Constructor)) {
        throw new TypeError("Cannot call a class as a function");
    }
}
function _defineProperties(target, props) {
    for(var i = 0; i < props.length; i++){
        var descriptor = props[i];
        descriptor.enumerable = descriptor.enumerable || false;
        descriptor.configurable = true;
        if ("value" in descriptor) descriptor.writable = true;
        Object.defineProperty(target, descriptor.key, descriptor);
    }
}
function _create_class(Constructor, protoProps, staticProps) {
    if (protoProps) _defineProperties(Constructor.prototype, protoProps);
    if (staticProps) _defineProperties(Constructor, staticProps);
    return Constructor;
}
function _ts_decorate(decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for(var i = decorators.length - 1; i >= 0; i--)if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
}
function _ts_generator(thisArg, body) {
    var f, y, t, _ = {
        label: 0,
        sent: function() {
            if (t[0] & 1) throw t[1];
            return t[1];
        },
        trys: [],
        ops: []
    }, g = Object.create((typeof Iterator === "function" ? Iterator : Object).prototype), d = Object.defineProperty;
    return d(g, "next", {
        value: verb(0)
    }), d(g, "throw", {
        value: verb(1)
    }), d(g, "return", {
        value: verb(2)
    }), typeof Symbol === "function" && d(g, Symbol.iterator, {
        value: function() {
            return this;
        }
    }), g;
    function verb(n) {
        return function(v) {
            return step([
                n,
                v
            ]);
        };
    }
    function step(op) {
        if (f) throw new TypeError("Generator is already executing.");
        while(g && (g = 0, op[0] && (_ = 0)), _)try {
            if (f = 1, y && (t = op[0] & 2 ? y["return"] : op[0] ? y["throw"] || ((t = y["return"]) && t.call(y), 0) : y.next) && !(t = t.call(y, op[1])).done) return t;
            if (y = 0, t) op = [
                op[0] & 2,
                t.value
            ];
            switch(op[0]){
                case 0:
                case 1:
                    t = op;
                    break;
                case 4:
                    _.label++;
                    return {
                        value: op[1],
                        done: false
                    };
                case 5:
                    _.label++;
                    y = op[1];
                    op = [
                        0
                    ];
                    continue;
                case 7:
                    op = _.ops.pop();
                    _.trys.pop();
                    continue;
                default:
                    if (!(t = _.trys, t = t.length > 0 && t[t.length - 1]) && (op[0] === 6 || op[0] === 2)) {
                        _ = 0;
                        continue;
                    }
                    if (op[0] === 3 && (!t || op[1] > t[0] && op[1] < t[3])) {
                        _.label = op[1];
                        break;
                    }
                    if (op[0] === 6 && _.label < t[1]) {
                        _.label = t[1];
                        t = op;
                        break;
                    }
                    if (t && _.label < t[2]) {
                        _.label = t[2];
                        _.ops.push(op);
                        break;
                    }
                    if (t[2]) _.ops.pop();
                    _.trys.pop();
                    continue;
            }
            op = body.call(thisArg, _);
        } catch (e) {
            op = [
                6,
                e
            ];
            y = 0;
        } finally{
            f = t = 0;
        }
        if (op[0] & 5) throw op[1];
        return {
            value: op[0] ? op[1] : void 0,
            done: true
        };
    }
}
function _ts_metadata(k, v) {
    if (typeof Reflect === "object" && typeof Reflect.metadata === "function") return Reflect.metadata(k, v);
}
function log(target, key, descriptor) {
    var original = descriptor.value;
    descriptor.value = function() {
        for(var _len = arguments.length, args = new Array(_len), _key = 0; _key < _len; _key++){
            args[_key] = arguments[_key];
        }
        console.log('Calling ' + key + ' with', args);
        return original.apply(this, args);
    };
    return descriptor;
}
function injectable(target) {
    target._injectable = true;
    return target;
}
var AuthService = /*#__PURE__*/ function() {
    "use strict";
    function AuthService() {
        _class_call_check(this, AuthService);
        this.auth = (0, _authsdk.init)({
            provider: 'oauth2'
        });
        this.api = (0, _apiclient.init)({
            baseUrl: 'https://api.internal.acmecorp.com'
        });
        this.versions = {
            auth: _authsdk.VERSION,
            api: _apiclient.VERSION
        };
    }
    _create_class(AuthService, [
        {
            key: "login",
            value: function login(username, password) {
                return _async_to_generator(function() {
                    return _ts_generator(this, function(_state) {
                        switch(_state.label){
                            case 0:
                                return [
                                    4,
                                    new Promise(function(resolve) {
                                        return setTimeout(resolve, 100);
                                    })
                                ];
                            case 1:
                                _state.sent();
                                return [
                                    2,
                                    {
                                        token: 'fake-jwt-token',
                                        user: username
                                    }
                                ];
                        }
                    });
                })();
            }
        },
        {
            key: "fetchUserProfile",
            value: function fetchUserProfile(token) {
                return _async_to_generator(function() {
                    return _ts_generator(this, function(_state) {
                        switch(_state.label){
                            case 0:
                                return [
                                    4,
                                    new Promise(function(resolve) {
                                        return setTimeout(resolve, 50);
                                    })
                                ];
                            case 1:
                                _state.sent();
                                return [
                                    2,
                                    {
                                        id: 1,
                                        name: 'Test User',
                                        token: token
                                    }
                                ];
                        }
                    });
                })();
            }
        },
        {
            key: "refreshToken",
            value: function refreshToken(oldToken) {
                return _async_to_generator(function() {
                    var result;
                    return _ts_generator(this, function(_state) {
                        switch(_state.label){
                            case 0:
                                return [
                                    4,
                                    this.login('refresh', oldToken)
                                ];
                            case 1:
                                result = _state.sent();
                                return [
                                    2,
                                    result.token
                                ];
                        }
                    });
                }).call(this);
            }
        }
    ]);
    return AuthService;
}();
_ts_decorate([
    log,
    _ts_metadata("design:type", Function),
    _ts_metadata("design:paramtypes", [
        void 0,
        void 0
    ]),
    _ts_metadata("design:returntype", Promise)
], AuthService.prototype, "login", null);
_ts_decorate([
    log,
    _ts_metadata("design:type", Function),
    _ts_metadata("design:paramtypes", [
        void 0
    ]),
    _ts_metadata("design:returntype", Promise)
], AuthService.prototype, "fetchUserProfile", null);
_ts_decorate([
    log,
    _ts_metadata("design:type", Function),
    _ts_metadata("design:paramtypes", [
        void 0
    ]),
    _ts_metadata("design:returntype", Promise)
], AuthService.prototype, "refreshToken", null);
AuthService = _ts_decorate([
    injectable,
    _ts_metadata("design:type", Function),
    _ts_metadata("design:paramtypes", [])
], AuthService);
var _default = AuthService;

//# sourceMappingURL=auth-service.js.map