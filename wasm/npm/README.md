# Karlsen WASM SDK

An integration wrapper around [`karlsen-wasm`](https://www.npmjs.com/package/karlsen-wasm) module that uses [`websocket`](https://www.npmjs.com/package/websocket) W3C adaptor for WebSocket communication.

This is a Node.js module that provides bindings to the Karlsen WASM SDK strictly for use in the Node.js environment. The web browser version of the SDK is available as part of official SDK releases at [https://github.com/karlsen-network/rusty-karlsen/releases](https://github.com/karlsen-network/rusty-karlsen/releases)

## Usage

Karlsen NPM module exports include all WASM32 bindings.
```javascript
const karlsen = require('karlsen');
console.log(karlsen.version());
```

## Documentation

Documentation is available at [https://karlsen.aspectron.org/docs/](https://karlsen.aspectron.org/docs/)


## Building from source & Examples

SDK examples as well as information on building the project from source can be found at [https://github.com/karlsen-network/rusty-karlsen/tree/master/wasm](https://github.com/karlsen-network/rusty-karlsen/tree/master/wasm)

## Releases

Official releases as well as releases for Web Browsers are available at [https://github.com/karlsen-network/rusty-karlsen/releases](https://github.com/karlsen-network/rusty-karlsen/releases).

Nightly / developer builds are available at: [https://aspectron.org/en/projects/karlsen-wasm.html](https://aspectron.org/en/projects/karlsen-wasm.html)

