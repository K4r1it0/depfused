// enterprise-sdk internal package stub
var createClient = function(options) {
  console.log('enterprise-sdk createClient', options);
  return {
    get: function(path) { return Promise.resolve({ data: {}, status: 200 }); },
    post: function(path, body) { return Promise.resolve({ data: body, status: 201 }); },
    ready: true
  };
};
var VERSION = '2.3.1';
exports.createClient = createClient;
exports.VERSION = VERSION;
