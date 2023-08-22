# Fift Playground

## About

A simple browser REPL for [`fift`](https://github.com/broxus/fift).

## Usage

### Local Development

This website is built using `Vue` and `Rust`.

### Prerequisites

- Rust 1.70+ with installed target `wasm32-unknown-unknown`
- wasm-pack
- binaryen 99+ (for `wasm-opt`)
- Node.js 18+

### Installation

```bash
git clone https://github.com/Rexagon/fift-playground.git
cd everscale-web-tools
npm install
npm run wasm
```

### Run

```
npm run dev
```

This command starts a local development server and opens up a browser window (http://localhost:5173).
Most changes are reflected live without having to restart the server.

### Build

```bash
npm run build
```

This command generates static content into the `dist` directory and can be served using any static contents hosting service.

## Contributing

We welcome contributions to the project! If you notice any issues or errors, feel free to open an issue or submit a pull request.

## License

Licensed under GNU Lesser General Public License v2.1 ([/cli/LICENSE](./cli/LICENSE) or <https://www.gnu.org/licenses/old-licenses/lgpl-2.1.html>)

Uses a modified version of [original fift libraries](https://github.com/ton-blockchain/ton/tree/master/crypto/fift/lib) ([LGPL-2.1](./fift-wasm/src/lib/LICENSE)).
