import path from "path";
import os from "os";
import { spawn } from "child_process";
import { mkdir } from "fs/promises";

import webpack from "webpack";
import TsConfigPathsPlugin from "tsconfig-paths-webpack-plugin";
import CopyPlugin from "copy-webpack-plugin";
import { createSentrySDK } from "sentry";

class WasmPackPlugin {
  readonly crateDir: string;

  constructor(crateDir: string) {
    this.crateDir = crateDir;
  }

  apply(compiler: webpack.Compiler) {
    const dev = compiler.options.mode !== "production";

    const cargoBin = path.join(os.homedir(), ".cargo", "bin");
    const env = { ...process.env, PATH: `${cargoBin}${path.delimiter}${process.env.PATH ?? ""}` };

    const rustDir = path.join(this.crateDir, "src/rust");
    const cargoToml = path.join(this.crateDir, "Cargo.toml");

    let needsBuild = true;

    compiler.hooks.watchRun.tap("WasmPackPlugin", () => {
      const modified = compiler.modifiedFiles;
      if (!modified) return;
      needsBuild = [...modified].some(f => f === cargoToml || f.startsWith(rustDir + path.sep));
    });

    compiler.hooks.beforeCompile.tapPromise("WasmPackPlugin", async () => {
      if (!needsBuild) return;
      needsBuild = false;
      await new Promise<void>((resolve, reject) => {
        const args = ["build", this.crateDir, "--target", "bundler", dev ? "--dev" : "--release"];
        const proc = spawn("wasm-pack", args, { stdio: "inherit", env });
        proc.on("exit", code => code === 0 ? resolve() : reject(new Error(`wasm-pack exited with code ${code}`)));
        proc.on("error", reject);
      });
      if (!dev) {
        const debugDir = path.resolve(this.crateDir, "../target/sentry");
        await mkdir(debugDir, { recursive: true });
        await createSentrySDK()["wasm-split"]({
          input: path.join(this.crateDir, "pkg/wie_web_bg.wasm"),
          debugOut: path.join(debugDir, "wie_web.debug.wasm"),
          strip: true,
          stripNames: true,
        });
      }
    });

    compiler.hooks.afterCompile.tap("WasmPackPlugin", compilation => {
      compilation.contextDependencies.add(rustDir);
      compilation.fileDependencies.add(cargoToml);
    });
  }
}

const commonConfig: webpack.Configuration = {
  context: import.meta.dirname,
  entry: { index: "./src/html/index.html" },
  experiments: { html: true, css: true },
  output: {
    path: path.resolve(import.meta.dirname, "dist"),
    filename: "assets/js/[name].[contenthash:8].js",
    cssFilename: "assets/css/[name].[contenthash:8].css",
    htmlFilename: "[name].html",
    clean: true,
  },
  ignoreWarnings: [
    /"global" has been used, it will be undefined in next major version./,
  ],
  resolve: {
    alias: {
      "@css": path.resolve(import.meta.dirname, "src/css"),
      "@ts": path.resolve(import.meta.dirname, "src/ts"),
    },
    extensions: [".ts", ".js"],
    plugins: [
      new TsConfigPathsPlugin({
        configFile: path.resolve(import.meta.dirname, "./tsconfig.json"),
        extensions: [".ts", ".js"],
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
        test: /\.(ico|png|jp?g|webp|svg)$/,
        type: "asset/resource",
        generator: {
          filename: "assets/img/[name][ext]",
        },
      },
      {
        test: /\.ttf$/,
        type: "asset/resource",
        generator: {
          filename: "assets/font/[name].[contenthash:8][ext]",
        },
      },
    ],
  },
  plugins: [
    new webpack.EnvironmentPlugin({
      ENVIRONMENT: "development",
      SENTRY_DSN: "",
      SENTRY_RELEASE: "local",
    }),
    new WasmPackPlugin(import.meta.dirname),
    new CopyPlugin({
      patterns: [
        { from: path.resolve(import.meta.dirname, "public"), to: "." },
        { from: path.resolve(import.meta.dirname, "../assets/1mgm.sf2"), to: "." },
        { from: path.resolve(import.meta.dirname, "../assets/1mgm.txt"), to: "." },
        {
          from: path.resolve(import.meta.dirname, "../node_modules/spessasynth_lib/dist/spessasynth_processor.min.js"),
          to: ".",
        },
      ],
    }),
  ],
};

export default commonConfig;
