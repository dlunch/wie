import path from "path";

import webpack from "webpack";
import { merge } from "webpack-merge";

import "webpack-dev-server";

// @ts-ignore: allowImportingTsExtensions
import commonConfig from "./webpack.config.common.ts";

const config = (env: { native?: boolean } = {}): webpack.Configuration => merge(commonConfig("development", env.native), {
  mode: "development",
  devtool: "eval-source-map",
  devServer: {
    open: false,
    host: process.env.TAURI_DEV_HOST ?? "localhost",
    port: env.native ? 1420 : 8080,
    static: env.native ? false : [
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
