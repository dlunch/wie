import path from "path";

import webpack from "webpack";
import { merge } from "webpack-merge";

import "webpack-dev-server";

// @ts-ignore: allowImportingTsExtensions
import commonConfig from "./webpack.config.common.ts";

const config: webpack.Configuration = merge(commonConfig("development"), {
  mode: "development",
  devtool: "eval-source-map",
  devServer: {
    open: false,
    static: [
      path.join(import.meta.dirname, "dist"),
      path.join(import.meta.dirname, "public"),
    ],
    watchFiles: {
      paths: ["src/**/*.*"],
      options: {
        usePolling: true,
      },
    },
  },
});

export default config;
