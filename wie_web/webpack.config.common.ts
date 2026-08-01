import path from "path";
import os from "os";
import { spawn } from "child_process";

import webpack from "webpack";
import HtmlBundlerPlugin from "html-bundler-webpack-plugin";
import TsConfigPathsPlugin from "tsconfig-paths-webpack-plugin";
import CopyPlugin from "copy-webpack-plugin";

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

    compiler.hooks.beforeCompile.tapPromise("WasmPackPlugin", () => {
      if (!needsBuild) return Promise.resolve();
      needsBuild = false;
      return new Promise<void>((resolve, reject) => {
        const args = ["build", this.crateDir, "--target", "bundler", dev ? "--dev" : "--release"];
        const proc = spawn("wasm-pack", args, { stdio: "inherit", env });
        proc.on("exit", code => code === 0 ? resolve() : reject(new Error(`wasm-pack exited with code ${code}`)));
        proc.on("error", reject);
      });
    });

    compiler.hooks.afterCompile.tap("WasmPackPlugin", compilation => {
      compilation.contextDependencies.add(rustDir);
      compilation.fileDependencies.add(cargoToml);
    });
  }
}


const commonConfig = (mode: "development" | "production"): webpack.Configuration => ({
  context: import.meta.dirname,
  experiments: {
    futureDefaults: true,
    css: false,
  },
  output: {
    path: path.resolve(import.meta.dirname, "dist"),
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
          data: { adtest: mode !== "production" },
        },
      },
      js: {
        filename: "assets/js/[name].[contenthash:8].js",
      },
      css: {
        filename: "assets/css/[name].[contenthash:8].css",
      },
    }),
    new WasmPackPlugin(import.meta.dirname),
    new CopyPlugin({
      patterns: [
        { from: path.resolve(import.meta.dirname, "public"), to: "." },
        {
          from: path.resolve(import.meta.dirname, "../node_modules/spessasynth_lib/dist/spessasynth_processor.min.js"),
          to: ".",
        },
      ],
    }),
  ],
});

export default commonConfig;
