// ***************************************************************************************
// |docname| - Define the `webpack configuration <https://webpack.js.org/configuration/>`_
// ***************************************************************************************
// .. toctree::
//  :caption: Related contents
//
//  webpack.index.js

const path = require("path");

const CompressionPlugin = require("compression-webpack-plugin");
const CssMinimizerPlugin = require("css-minimizer-webpack-plugin");
const HtmlWebpackPlugin = require("html-webpack-plugin");
const MiniCssExtractPlugin = require("mini-css-extract-plugin");

module.exports = (env, argv) => {
    const is_dev_mode = argv.mode === "development";

    return {
        // Cache build results between builds in development mode, per the `docs <https://webpack.js.org/configuration/cache/>`__.
        cache: is_dev_mode
            ? {
                  type: "filesystem",
              }
            : false,
        // Per the `docs <https://webpack.js.org/concepts/entry-points/>`__, the main source file.
        entry: "./src/CodeChat-editor.ts",
        // See `mode <https://webpack.js.org/configuration/mode/>`_ for the conditional statement below.
        devtool: is_dev_mode ? "inline-source-map" : "source-map",
        module: {
            rules: [
                {
                    test: /\.css$/i,
                    use: [MiniCssExtractPlugin.loader, "css-loader"],
                },
                {
                    test: /\.(png|jpe?g|gif|svg|eot|ttf|woff|woff2)$/i,
                    // For more information, see `Asset Modules <https://webpack.js.org/guides/asset-modules/>`_.
                    type: "asset",
                },
                {
                    // See the `Webpack TypeScript docs <https://webpack.js.org/guides/typescript/>`_.
                    test: /\.tsx?$/,
                    use: 'ts-loader',
                    exclude: /node_modules/,
                },
            ],
        },
        output: {
            path: path.resolve(__dirname, "../static/webpack"),
            // _`Output file naming`: see the `caching guide <https://webpack.js.org/guides/caching/>`_. This provides a hash for dynamic imports as well, avoiding caching out-of-date JS. Putting the hash in a query parameter (such as ``[name].js?v=[contenthash]``) causes the compression plugin to not update zipped files.
            filename: "[name].[contenthash].bundle.js",
            // Node 17.0 reports ``Error: error:0308010C:digital envelope routines::unsupported``. Per `SO <https://stackoverflow.com/a/69394785/16038919>`_, this error is produced by using an old, default hash that OpenSSL removed support for. The `webpack docs <https://webpack.js.org/configuration/output/#outputhashfunction>`__ says that ``xxhash64`` is a faster algorithm.
            hashFunction: "xxhash64",
            // Delete everything in the output directory on each build for production; keep files when doing development.
            clean: is_dev_mode ? false : true,
        },
        // See the `SplitChunksPlugin docs <https://webpack.js.org/guides/code-splitting/#splitchunksplugin>`_.
        optimization: {
            // CSS for production was copied from `Minimizing For Production <https://webpack.js.org/plugins/mini-css-extract-plugin/#minimizing-for-production>`_.
            minimizer: [
                // For webpack@5 you can use the ``...`` syntax to extend existing minimizers (i.e. ``terser-webpack-plugin``), uncomment the next line.
                "...",
                new CssMinimizerPlugin(),
            ],
            moduleIds: "deterministic",
            // Collect all the webpack import runtime into a single file, which is named ``???.bundle.js``.
            runtimeChunk: "single",
            splitChunks: {
                cacheGroups: {
                    // From the `TinyMCE webpack docs <https://www.tiny.cloud/docs/advanced/usage-with-module-loaders/webpack/webpack_es6_npm/>`_.
                    tinymceVendor: {
                        test: /[\\/]node_modules[\\/](tinymce)[\\/](.*js|.*skin.css)|[\\/]plugins[\\/]/,
                        name: 'tinymce',
                    },
                },
                chunks: "all",
            },
        },
        plugins: [
            // _`webpack_static_imports`: Instead of HTML, produce a list of static imports as JSON. The server will then read this file and inject these imports when creating each page.
            new HtmlWebpackPlugin({
                filename: "webpack_static_imports.json",
                // Don't prepend the ``<head>`` tag and data to the output.
                inject: false,
                // The template to create JSON.
                templateContent: ({ htmlWebpackPlugin }) =>
                    JSON.stringify({
                        js: htmlWebpackPlugin.files.js,
                        css: htmlWebpackPlugin.files.css,
                    }),
            }),
            new MiniCssExtractPlugin({
                // See `output file naming`_.
                filename: "[name].[contenthash].css",
                chunkFilename: "[id].css",
            }),
            // Copied from the `webpack docs <https://webpack.js.org/plugins/compression-webpack-plugin>`_. This creates ``.gz`` versions of all files. The webserver in use needs to be configured to send this instead of the uncompressed versions.
            new CompressionPlugin(),
        ],
        resolve: {
            // Otherwise, TypeScript modules aren't found.
            extensions: ['.ts', '.js'],
        },
    };
};
