import webpack from "webpack";
import { merge } from "webpack-merge";
import { sentryWebpackPlugin } from "@sentry/webpack-plugin";

// @ts-ignore: allowImportingTsExtensions
import commonConfig from "./webpack.config.common.ts";

const config: webpack.Configuration = merge(commonConfig("production"), {
  mode: "production",
  devtool: "hidden-source-map",
  plugins: [
    sentryWebpackPlugin({
      // Fail the build on upload errors, which the plugin otherwise only logs.
      errorHandler(error) { throw error; },
      sourcemaps: {
        filesToDeleteAfterUpload: ["wie-web/dist/**/*.map"],
      },
    }),
  ],
});

export default config;
