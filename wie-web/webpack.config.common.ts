import path from "path";
import os from "os";
import { execFile, spawn } from "child_process";
import { promisify } from "util";

import webpack from "webpack";
import HtmlBundlerPlugin from "html-bundler-webpack-plugin";
import TsConfigPathsPlugin from "tsconfig-paths-webpack-plugin";
import CopyPlugin from "copy-webpack-plugin";

interface CargoMetadata {
  workspace_root: string;
  packages: {
    manifest_path: string;
    targets: { kind: string[]; src_path: string }[];
    dependencies: { path?: string; kind: string | null }[];
  }[];
}

class WasmPackPlugin {
  readonly crateDir: string;
  readonly rustDir: string;

  constructor(crateDir: string) {
    this.crateDir = crateDir;
    this.rustDir = path.join(crateDir, "src/rust");
  }

  apply(compiler: webpack.Compiler) {
    const dev = compiler.options.mode !== "production";

    const cargoBin = path.join(os.homedir(), ".cargo", "bin");
    const env = { ...process.env, PATH: `${cargoBin}${path.delimiter}${process.env.PATH ?? ""}` };

    const cargoToml = path.join(this.crateDir, "Cargo.toml");
    const rustDirs = new Set([this.rustDir]);
    const rustFiles = new Set([cargoToml]);

    let needsBuild = true;

    compiler.hooks.watchRun.tap("WasmPackPlugin", () => {
      const changed = [...(compiler.modifiedFiles ?? []), ...(compiler.removedFiles ?? [])];
      needsBuild ||= changed.some(f => rustFiles.has(f) || [...rustDirs].some(dir => f === dir || f.startsWith(dir + path.sep)));
    });

    compiler.hooks.beforeCompile.tapPromise("WasmPackPlugin", async () => {
      if (!needsBuild) return;
      const { stdout } = await promisify(execFile)("cargo", ["metadata", "--format-version", "1", "--no-deps", "--manifest-path", cargoToml], { env });
      const metadata: CargoMetadata = JSON.parse(stdout);
      const packages = new Map(metadata.packages.map(pkg => [path.dirname(pkg.manifest_path), pkg]));
      const dependencies = new Set([this.crateDir]);
      rustDirs.clear();
      rustFiles.clear();
      rustFiles.add(path.join(metadata.workspace_root, "Cargo.toml"));
      rustFiles.add(path.join(metadata.workspace_root, "Cargo.lock"));
      for (const dir of dependencies) {
        const pkg = packages.get(dir)!;
        rustFiles.add(pkg.manifest_path);
        for (const target of pkg.targets) {
          if (target.kind.includes("custom-build")) {
            rustFiles.add(target.src_path);
          } else if (target.kind.some(kind => ["lib", "rlib", "cdylib", "staticlib", "dylib", "proc-macro"].includes(kind))) {
            rustDirs.add(path.dirname(target.src_path));
          }
        }
        for (const dependency of pkg.dependencies) {
          if (dependency.kind !== "dev" && dependency.path && packages.has(dependency.path)) {
            dependencies.add(dependency.path);
          }
        }
      }
      await new Promise<void>((resolve, reject) => {
        const args = ["build", this.crateDir, "--target", "bundler", dev ? "--dev" : "--release"];
        const proc = spawn("wasm-pack", args, { stdio: "inherit", env });
        proc.on("exit", code => code === 0 ? resolve() : reject(new Error(`wasm-pack exited with code ${code}`)));
        proc.on("error", reject);
      });
      needsBuild = false;
    });

    compiler.hooks.afterCompile.tap("WasmPackPlugin", compilation => {
      for (const dir of rustDirs) compilation.contextDependencies.add(dir);
      for (const file of rustFiles) compilation.fileDependencies.add(file);
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
