import * as esbuild from "esbuild";
import * as fs from "node:fs";

// Ensure bin directory exists
if (!fs.existsSync("bin")) fs.mkdirSync("bin", { recursive: true });

async function build() {
  console.log("Building bin/context-flow.cjs...");
  await esbuild.build({
    bundle: true,
    platform: "node",
    target: "node20",
    format: "cjs",
    sourcemap: false,
    entryPoints: ["src/cli.ts"],
    outfile: "bin/context-flow.cjs",
    banner: {
      js: "#!/usr/bin/env node",
    },
  });
  // Normalize CRLF to LF to prevent Unix /usr/bin/env: 'node\r' execution error
  const content = fs.readFileSync("bin/context-flow.cjs", "utf8");
  fs.writeFileSync("bin/context-flow.cjs", content.replace(/\r\n/g, "\n"), "utf8");

  try {
    fs.chmodSync("bin/context-flow.cjs", 0o755);
  } catch {
    // Ignore chmod errors on Windows
  }
  console.log("Build complete!");
}

build().catch((err) => {
  console.error("Build failed:", err);
  process.exit(1);
});
