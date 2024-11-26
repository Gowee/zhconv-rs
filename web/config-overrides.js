const path = require('path');

// const MonacoWebpackPlugin = require('monaco-editor-webpack-plugin');
const { removeModuleScopePlugin } = require('customize-cra');
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

module.exports = function override(config, env) {
  config.plugins.push(
    // new MonacoWebpackPlugin({
    //   languages: ['json', 'python', 'rust']
    // }),
    new WasmPackPlugin({
      crateDirectory: path.resolve(__dirname, "../"),
      withTypeScript: true,
      extraArgs: "-- --features opencc",
      // NOTE: it is 'index' by default, different from the default (package name) of wasm-pack
      outName: 'zhconv'
    }));

  const wasmExtensionRegExp = /\.wasm$/;

  config.resolve.extensions.push('.wasm');

  config.module.rules.forEach(rule => {
    (rule.oneOf || []).forEach(oneOf => {
      if (oneOf.loader && oneOf.loader.indexOf('file-loader') >= 0) {
        // Make file-loader ignore WASM files
        oneOf.exclude.push(wasmExtensionRegExp);
      }
    });
  });

  // Add a dedicated loader for WASM
  config.module.rules.push({
    test: wasmExtensionRegExp,
    include: path.resolve(__dirname, 'src'),
    use: [{ loader: require.resolve('wasm-loader'), options: {} }]
  });

  removeModuleScopePlugin()(config);

  return config;
}

// Ref:
//  https://github.com/rustwasm/rust-webpack-template/issues/43#issuecomment-426597176
//  https://prestonrichey.com/blog/react-rust-wasm/
