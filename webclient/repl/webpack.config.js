const path = require('path');
const webpack = require('webpack');
const CopyPlugin = require('copy-webpack-plugin');
const WebpackShellPlugin = require('webpack-shell-plugin');
const ExtraWatchWebpackPlugin = require('extra-watch-webpack-plugin');

module.exports = {
    entry: {
      index : "./src/index.js",
      pyodide_worker : "./src/pyodide.worker.js"
    },
    output: {
        path: path.resolve(__dirname),
        filename: 'dist/[name].bundle.js',
        strictModuleExceptionHandling: true,
    },
    module: {
        rules: [
          {
            test: /\.py$/,
            use: 'raw-loader',
          },          
        ],
    },
    watchOptions: {
      ignored: ["**/python_imports.js"]
    },
    plugins : [
      new webpack.DllReferencePlugin({
        context: path.resolve(__dirname),
        manifest: require(path.resolve(__dirname, 'monaco.json'))
      }),
      new CopyPlugin({
        patterns: [
          { from: 'src/index.html', to: 'dist/index.html' },
        ],
      }),
      new WebpackShellPlugin({
        onBuildStart: ["./scripts/prebuild.sh"],
        dev : false // Rerun prebuild everytime webpack-dev-server rebuilds please.
        // onBuildEnd: ['python script.py && node script.js']
      }),
      new ExtraWatchWebpackPlugin({
        // files: [ './src/python/*' ],
        dirs: [ './src/python' ],
      }),
    ],  
    mode : "development",
    devtool: 'eval-source-map',
    // mode : "production",
    resolve: {
        alias: {
          "pyodide" : path.resolve(__dirname, "pyodide-build-0.15.0"),
        }
    },
    devServer: {
        compress: true,
        port: 9200
    }      
};
