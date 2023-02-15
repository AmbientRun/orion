const path = require('path');

module.exports = {
  entry: "./bootstrap.js",
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "bootstrap.js",
  },
  // module: {
  //   rules: [
  //     {
  //       test: /\.wasm/,
  //       type: "asset/resource",
  //       // use: {
  //       //   loader: "@wasm-tool/wasi"
  //       // }
  //     }
  //   ]
  // },
  mode: "development",

};
