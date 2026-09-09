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

    const root = path.dirname(this.crateDir);
    const compilerDir = path.join(root, "wie-arm-wasm/compiler");
    const sourceDirs = [
      path.join(this.crateDir, "src/rust"),
      path.join(root, "wie-arm-jit/src"),
      path.join(root, "wie-arm-wasm/src"),
      path.join(compilerDir, "src"),
      path.join(root, "wie-core-arm/src"),
    ];
    const manifests = [root, this.crateDir, path.join(root, "wie-arm-jit"), path.join(root, "wie-arm-wasm"), compilerDir, path.join(root, "wie-core-arm")]
      .map(dir => path.join(dir, "Cargo.toml"));
    manifests.push(path.join(root, "Cargo.lock"), path.join(compilerDir, "Cargo.lock"));

    let needsBuild = true;

    compiler.hooks.watchRun.tap("WasmPackPlugin", () => {
      const modified = compiler.modifiedFiles;
      if (!modified) return;
      needsBuild ||= [...modified].some(f => manifests.includes(f) || sourceDirs.some(dir => f === dir || f.startsWith(dir + path.sep)));
    });

    compiler.hooks.beforeCompile.tapPromise("WasmPackPlugin", async () => {
      if (!needsBuild) return;
      for (const crateDir of [compilerDir, this.crateDir]) {
        await new Promise<void>((resolve, reject) => {
          const args = ["build", crateDir, "--target", "bundler", dev ? "--dev" : "--release"];
          const proc = spawn("wasm-pack", args, { stdio: "inherit", env });
          proc.on("exit", code => code === 0 ? resolve() : reject(new Error(`wasm-pack exited with code ${code}`)));
          proc.on("error", reject);
        });
      }
      needsBuild = false;
    });

    compiler.hooks.afterCompile.tap("WasmPackPlugin", compilation => {
      for (const dir of sourceDirs) compilation.contextDependencies.add(dir);
      for (const manifest of manifests) compilation.fileDependencies.add(manifest);
    });
  }
}


const commonConfig = (mode: "development" | "production"): webpack.Configuration => ({
  context: import.meta.dirname,
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
      "@wie-arm-worker": path.resolve(import.meta.dirname, "../wie-arm-wasm/src/worker.js"),
      "@wie-arm-compiler": path.resolve(import.meta.dirname, "../wie-arm-wasm/compiler/pkg/wie_arm_wasm_compiler.js"),
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
