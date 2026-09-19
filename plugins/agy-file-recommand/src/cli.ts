import * as fs from "node:fs";
import { randomUUID } from "node:crypto";
import { HookInput, Episode } from "./types.js";
import { parseLastTurn } from "./parser.js";
import { getGitModifiedFiles } from "./git.js";
import { saveEpisode, loadEpisodes, getStats } from "./store.js";
import { recommendFiles } from "./recommender.js";
import { loadConfig, saveConfig, DEFAULT_CONFIG, isPluginEnabled } from "./config.js";

async function readStdin(timeoutMs = 1500): Promise<string> {
  if (process.stdin.isTTY) return "";
  return new Promise((resolve) => {
    const chunks: Buffer[] = [];
    const timer = setTimeout(() => {
      resolve(Buffer.concat(chunks).toString("utf8"));
    }, timeoutMs);

    process.stdin.on("data", (chunk) => {
      chunks.push(Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk));
    });
    process.stdin.on("end", () => {
      clearTimeout(timer);
      resolve(Buffer.concat(chunks).toString("utf8"));
    });
    process.stdin.on("error", () => {
      clearTimeout(timer);
      resolve("");
    });
  });
}

function getWorkspaceRoot(input: HookInput): string | undefined {
  if (input.workspacePaths && input.workspacePaths.length > 0 && fs.existsSync(input.workspacePaths[0])) {
    return input.workspacePaths[0];
  }
  if (fs.existsSync(process.cwd())) {
    return process.cwd();
  }
  return undefined;
}

async function handlePreInvocation() {
  try {
    const rawInput = await readStdin();
    if (!rawInput || !rawInput.trim()) {
      console.log(JSON.stringify({ injectSteps: [] }));
      return;
    }

    let input: HookInput;
    try {
      const cleaned = rawInput.replace(/^\uFEFF/, "").trim();
      input = JSON.parse(cleaned);
    } catch {
      console.log(JSON.stringify({ injectSteps: [] }));
      return;
    }

    const workspaceRoot = getWorkspaceRoot(input);
    const config = loadConfig(workspaceRoot);

    if (!config || !config.recommend.enabled) {
      console.log(JSON.stringify({ injectSteps: [] }));
      return;
    }

    const transcriptPath = input.transcriptPath;
    if (!transcriptPath || !fs.existsSync(transcriptPath)) {
      console.log(JSON.stringify({ injectSteps: [] }));
      return;
    }

    const parsed = parseLastTurn(transcriptPath, workspaceRoot || process.cwd());
    if (!parsed || !parsed.query) {
      console.log(JSON.stringify({ injectSteps: [] }));
      return;
    }

    const recommendation = recommendFiles(parsed.query, workspaceRoot, config, input.conversationId);
    if (recommendation.mode === "SUGGESTION_HINT" && recommendation.items.length > 0) {
      const xmlItems = recommendation.items.map(
        (item) => `  <file path="${item.path}" confidence="${item.score.toFixed(2)}" reason="${item.reasons.join(",")}" />`
      );

      const message = `<context_recommendations source="file-recommand">\n${xmlItems.join("\n")}\n</context_recommendations>`;

      console.log(
        JSON.stringify({
          injectSteps: [
            {
              ephemeralMessage: message,
            },
          ],
        })
      );
      return;
    }

    console.log(JSON.stringify({ injectSteps: [] }));
  } catch {
    console.log(JSON.stringify({ injectSteps: [] }));
  }
}

async function handlePostInvocation() {
  try {
    const rawInput = await readStdin();
    if (!rawInput || !rawInput.trim()) {
      console.log(JSON.stringify({}));
      return;
    }

    let input: HookInput;
    try {
      const cleaned = rawInput.replace(/^\uFEFF/, "").trim();
      input = JSON.parse(cleaned);
    } catch {
      console.log(JSON.stringify({}));
      return;
    }

    const workspaceRoot = getWorkspaceRoot(input);
    const config = loadConfig(workspaceRoot);

    if (!config || !config.record.enabled) {
      console.log(JSON.stringify({}));
      return;
    }

    const transcriptPath = input.transcriptPath;
    if (!transcriptPath || !fs.existsSync(transcriptPath)) {
      console.log(JSON.stringify({}));
      return;
    }

    const parsed = parseLastTurn(transcriptPath, workspaceRoot || process.cwd());
    if (parsed && parsed.query) {
      // Noise filter & Action deduplication:
      // Only record episodes that have concrete file operations or citations
      const hasAction =
        parsed.readFiles.length > 0 ||
        parsed.editedFiles.length > 0 ||
        parsed.citedFiles.length > 0;

      if (hasAction) {
        const gitFiles = getGitModifiedFiles(workspaceRoot);

        const episode: Episode = {
          id: randomUUID().replace(/-/g, "").slice(0, 12),
          timestamp: new Date().toISOString(),
          conversationId: input.conversationId || "unknown",
          query: parsed.query,
          thinkingTokens: parsed.thinkingTokens,
          readFiles: parsed.readFiles,
          editedFiles: parsed.editedFiles,
          citedFiles: parsed.citedFiles,
          gitStatus: gitFiles,
        };

        saveEpisode(workspaceRoot, episode);
      }
    }

    console.log(JSON.stringify({}));
  } catch {
    console.log(JSON.stringify({}));
  }
}

function handleInit() {
  const workspaceRoot = process.cwd();
  if (isPluginEnabled(workspaceRoot)) {
    console.log("ℹ️  agy-file-recommand is already initialized in this project (.agents/file-recommand/config.json).");
    return;
  }
  saveConfig(DEFAULT_CONFIG, workspaceRoot);
  console.log("✅ Initialized agy-file-recommand for this project!");
  console.log("Created: .agents/file-recommand/config.json");
}

function handleStats() {
  const workspaceRoot = process.cwd();
  const config = loadConfig(workspaceRoot);
  if (!config) {
    console.log("⚠️  Plugin is not enabled in this project. Run `node bin/recommand.cjs init` to enable.");
    return;
  }

  const stats = getStats(workspaceRoot);

  console.log("\n📊 === agy-file-recommand Statistics ===");
  console.log(`Total Episodes Recorded: ${stats.totalEpisodes}`);
  console.log(`Avg Read Files / Turn  : ${stats.avgReadFilesPerTurn}`);
  console.log(`Avg Edited Files / Turn: ${stats.avgEditedFilesPerTurn}`);

  console.log("\n🔥 Top Read Files:");
  if (stats.topReadFiles.length === 0) console.log("  (None)");
  else stats.topReadFiles.forEach((f, i) => console.log(`  ${i + 1}. [${f.count}x] ${f.file}`));

  console.log("\n✏️  Top Edited Files:");
  if (stats.topEditedFiles.length === 0) console.log("  (None)");
  else stats.topEditedFiles.forEach((f, i) => console.log(`  ${i + 1}. [${f.count}x] ${f.file}`));

  console.log("\n💬 Top Cited Files in AI Responses:");
  if (stats.topCitedFiles.length === 0) console.log("  (None)");
  else stats.topCitedFiles.forEach((f, i) => console.log(`  ${i + 1}. [${f.count}x] ${f.file}`));
  console.log("");
}

function handleList(limit = 10) {
  const workspaceRoot = process.cwd();
  const config = loadConfig(workspaceRoot);
  if (!config) {
    console.log("⚠️  Plugin is not enabled in this project. Run `node bin/recommand.cjs init` to enable.");
    return;
  }

  const episodes = loadEpisodes(workspaceRoot, limit);

  console.log(`\n📜 === Last ${episodes.length} Episodes ===`);
  if (episodes.length === 0) {
    console.log("  No episodes recorded yet.");
    return;
  }

  for (const ep of episodes) {
    console.log(`\n[${ep.id}] ${ep.timestamp} (Conv: ${ep.conversationId.slice(0, 8)})`);
    console.log(`  Query : ${ep.query.slice(0, 80)}`);
    console.log(`  Read  : ${ep.readFiles.length > 0 ? ep.readFiles.join(", ") : "(None)"}`);
    console.log(`  Edit  : ${ep.editedFiles.length > 0 ? ep.editedFiles.join(", ") : "(None)"}`);
    console.log(`  Cited : ${ep.citedFiles.length > 0 ? ep.citedFiles.join(", ") : "(None)"}`);
    console.log(`  Git   : ${ep.gitStatus.length > 0 ? ep.gitStatus.join(", ") : "(Clean)"}`);
  }
  console.log("");
}

function handleRecommend(query: string) {
  const workspaceRoot = process.cwd();
  const config = loadConfig(workspaceRoot);
  if (!config) {
    console.log("⚠️  Plugin is not enabled in this project. Run `node bin/recommand.cjs init` to enable.");
    return;
  }

  console.log(`\n🔍 Evaluating recommendation for query: "${query}"...`);
  const result = recommendFiles(query, workspaceRoot, config);

  if (result.mode === "NONE" || result.items.length === 0) {
    console.log("  Result: No file recommendations (gating closed or below threshold).");
  } else {
    console.log(`  Result (${result.items.length} suggestions):`);
    result.items.forEach((item, idx) => {
      console.log(`  ${idx + 1}. ${item.path} (Score: ${(item.score * 100).toFixed(0)}%)`);
      item.reasons.forEach((r) => console.log(`     - ${r}`));
    });
  }
  console.log("");
}

function handleConfig(args: string[]) {
  const workspaceRoot = process.cwd();
  const config = loadConfig(workspaceRoot);

  if (args.length === 0) {
    if (!config) {
      console.log("\n⚠️  Status: Disabled in this project (No .agents/file-recommand/config.json).");
      console.log("Run `node bin/recommand.cjs init` to enable.\n");
      return;
    }
    console.log("\n⚙️ === Current agy-file-recommand Configuration ===");
    console.log(JSON.stringify(config, null, 2));
    console.log("");
    return;
  }

  const action = args[0];
  if (action === "set") {
    const key = args[1];
    const value = args[2];

    if (!key || value === undefined) {
      console.log("Usage: file-recommand config set <record.enabled|recommend.enabled|recommend.threshold|recommend.maxItems> <value>");
      return;
    }

    const currentConfig = config || { ...DEFAULT_CONFIG };

    if (key === "record.enabled") {
      currentConfig.record.enabled = value === "true" || value === "1";
    } else if (key === "recommend.enabled") {
      currentConfig.recommend.enabled = value === "true" || value === "1";
    } else if (key === "recommend.threshold") {
      currentConfig.recommend.threshold = parseFloat(value);
    } else if (key === "recommend.maxItems") {
      currentConfig.recommend.maxItems = parseInt(value, 10);
    } else {
      console.log(`Unknown config key: ${key}`);
      return;
    }

    saveConfig(currentConfig, workspaceRoot);
    console.log(`✅ Config updated in .agents/file-recommand/config.json: ${key} = ${value}`);
  }
}

async function main() {
  const args = process.argv.slice(2);
  const command = args[0];
  const subCommand = args[1];

  if (command === "hook") {
    if (subCommand === "pre-invocation") {
      await handlePreInvocation();
    } else if (subCommand === "post-invocation") {
      await handlePostInvocation();
    } else {
      console.error(`Unknown hook command: ${subCommand}`);
      process.exit(1);
    }
  } else if (command === "init") {
    handleInit();
  } else if (command === "stats") {
    handleStats();
  } else if (command === "list") {
    const limit = args[1] ? parseInt(args[1], 10) : 10;
    handleList(limit);
  } else if (command === "recommend") {
    const query = args.slice(1).join(" ");
    if (!query) {
      console.error("Please provide a query: file-recommand recommend '<query>'");
      process.exit(1);
    }
    handleRecommend(query);
  } else if (command === "config") {
    handleConfig(args.slice(1));
  } else {
    console.log("Usage: file-recommand [init | config [set <k> <v>] | hook <pre-invocation|post-invocation> | stats | list | recommend <query>]");
  }
}

main().catch((err) => {
  console.error("CLI error:", err);
  process.exit(1);
});
