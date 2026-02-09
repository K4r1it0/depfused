// Logger module - uses private-logger and lodash
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
    get createLogger () {
        return createLogger;
    },
    get formatMessage () {
        return formatMessage;
    },
    get mergeLogConfigs () {
        return mergeLogConfigs;
    }
});
var _lodash = /*#__PURE__*/ _interop_require_default(require("lodash"));
function _array_like_to_array(arr, len) {
    if (len == null || len > arr.length) len = arr.length;
    for(var i = 0, arr2 = new Array(len); i < len; i++)arr2[i] = arr[i];
    return arr2;
}
function _array_without_holes(arr) {
    if (Array.isArray(arr)) return _array_like_to_array(arr);
}
function _interop_require_default(obj) {
    return obj && obj.__esModule ? obj : {
        default: obj
    };
}
function _iterable_to_array(iter) {
    if (typeof Symbol !== "undefined" && iter[Symbol.iterator] != null || iter["@@iterator"] != null) return Array.from(iter);
}
function _non_iterable_spread() {
    throw new TypeError("Invalid attempt to spread non-iterable instance.\\nIn order to be iterable, non-array objects must have a [Symbol.iterator]() method.");
}
function _to_consumable_array(arr) {
    return _array_without_holes(arr) || _iterable_to_array(arr) || _unsupported_iterable_to_array(arr) || _non_iterable_spread();
}
function _unsupported_iterable_to_array(o, minLen) {
    if (!o) return;
    if (typeof o === "string") return _array_like_to_array(o, minLen);
    var n = Object.prototype.toString.call(o).slice(8, -1);
    if (n === "Object" && o.constructor) n = o.constructor.name;
    if (n === "Map" || n === "Set") return Array.from(n);
    if (n === "Arguments" || /^(?:Ui|I)nt(?:8|16|32)(?:Clamped)?Array$/.test(n)) return _array_like_to_array(o, minLen);
}
var privateLogger = require('private-logger');
function createLogger(namespace) {
    return {
        info: function info(msg) {
            return privateLogger.log('[INFO][' + namespace + '] ' + msg);
        },
        error: function error(msg) {
            return privateLogger.log('[ERROR][' + namespace + '] ' + msg);
        },
        debug: function debug(msg) {
            return privateLogger.log('[DEBUG][' + namespace + '] ' + msg);
        }
    };
}
function formatMessage(template, data) {
    var formatted = privateLogger.format(template);
    return _lodash.default.template(formatted)(data);
}
function mergeLogConfigs() {
    for(var _len = arguments.length, configs = new Array(_len), _key = 0; _key < _len; _key++){
        configs[_key] = arguments[_key];
    }
    var _$_;
    return (_$_ = _lodash.default).merge.apply(_$_, [
        {}
    ].concat(_to_consumable_array(configs)));
}

//# sourceMappingURL=logger.js.map