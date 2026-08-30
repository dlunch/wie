import webpack from "webpack";
import { merge } from "webpack-merge";

// @ts-ignore: allowImportingTsExtensions
import commonConfig from "./webpack.config.common.ts";

const config: webpack.Configuration = merge(commonConfig("production"), {
  mode: "production",
  devtool: false,
});

export default config;
