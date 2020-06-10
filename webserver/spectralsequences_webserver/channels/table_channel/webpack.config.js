const path = require('path');
const webpack = require('webpack');

context = "../../../..";
module.exports = {
    entry: {
      table : './src/table.js',
    },
    output: {
        path: path.resolve(__dirname, 'dist'),
        filename: '[name].js',
        strictModuleExceptionHandling: true,
        devtoolModuleFilenameTemplate (info) {
          const rel = path.relative(context, info.absoluteResourcePath);
          // console.log(`rel: ${rel}`, `localhost:8000/debug/${rel}`);
          // return `webpack:///${rel}`
          return `../debug/table/${rel}`;
        },
    },
    mode : "development",
    devtool: 'eval-source-map',
    // mode : "production",
    plugins: [],
    module: {
      rules: [
        {
            test: /\.(woff|woff2|ttf|eot|svg)(\?v=\d+\.\d+\.\d+)?$/,
            use: ['url-loader']
        },
        {
            test: /\.css$/,
            use: ['to-string-loader', 'css-loader']
        }
      ],
    },

    resolve: {
        modules: [
          "node_modules"
        ],
        alias: {
          // Utilities: path.resolve(__dirname, 'src/utilities/'),
          'd3': path.resolve(__dirname, '../../../../chart/client/dist/d3.min.js'),
          "chart" : path.resolve(__dirname, "../../../../chart/client/src"),
          // "katex" : path.resolve(__dirname, "./node_modules/katex/dist")
        }, 
      }
};