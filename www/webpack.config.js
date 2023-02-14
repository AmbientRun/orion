const path = require('path');
const webpack = require('webpack');

module.exports = {
  entry: "./bootstrap.js",
  // resolve: {
  //   modules: ['node_modules'],
  //   alias: {
  //     '@bjorn3/browser_wasi_shim': '@bjorn3/browser_wasi_shim/src/index.js'
  //   }
  // },
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "bootstrap.js",
  },
  externals: {
    'wasmer_wasi_js_bg.wasm': true
  },
  module: {
    rules: [
      {
        test: /\.wasm/,
        type: "asset/resource",
        // use: {
        //   loader: "@wasm-tool/wasi"
        // }
      }
    ]
  },
  mode: "development",

  plugins: [
    // Work around for Buffer is undefined:
    // https://github.com/webpack/changelog-v5/issues/10
    new webpack.ProvidePlugin({
      Buffer: ['buffer', 'Buffer'],
    }),
    // new webpack.ProvidePlugin({
    //   process: 'process/browser',
    // }),
  ],
  // plugins: [
  //   new HtmlWebpackPlugin({
  //     // template: path.resolve(__dirname, "src", "index.html")
  //   })
  // ],
};
