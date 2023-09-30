import path from "path";
import webpack from "webpack";
import HtmlBundlerPlugin from "html-bundler-webpack-plugin";
import WasmPackPlugin from "@wasm-tool/wasm-pack-plugin";
import TsConfigPathsPlugin from "tsconfig-paths-webpack-plugin";
import "webpack-dev-server";

const config: webpack.Configuration = {
  mode: "development",
  experiments: {
    futureDefaults: true,
  },
  output: {
    path: path.resolve(__dirname, "dist"),
    clean: true,
  },
  resolve: {
    extensions: [".ts", ".js"],
    plugins: [
      new TsConfigPathsPlugin({
        configFile: "./tsconfig.json",
      }),
    ],
  },
  module: {
    rules: [
      {
        test: /\.ts$/,
        loader: "ts-loader",
        exclude: /node_modules/,
        options: {
          onlyCompileBundledFiles: true,
        },
      },
      {
        test: /\.(css|sass|scss)$/,
        use: ["css-loader", "sass-loader"],
      },
      {
        test: /\.(ico|png|jp?g|webp|svg)$/,
        type: "asset/resource",
        generator: {
          filename: "assets/img/",
        },
      },
    ],
  },
  plugins: [
    new HtmlBundlerPlugin({
      entry: {
        index: {
          import: "src/html/index.html",
        },
      },
      js: {
        filename: "assets/js/[name].[contenthash:8].js",
      },
      css: {
        filename: "assets/css/[name].[contenthash:8].css",
      },
    }),
    new WasmPackPlugin({
      crateDirectory: ".",
      outDir: "./src/ts/pkg",
    }),
  ],
};

export default config;
