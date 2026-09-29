import * as fs from "node:fs";
import * as path from "node:path";
import { randomUUID } from "node:crypto";
import { HookInput, Episode } from "./types.js";
import { parseLastTurn } from "./parser.js";
import { getGitModifiedFiles } from "./git.js";
import {
  saveEpisode,
  loadEpisodes,
  getStats,
  saveSessionSnapshot,
  loadRecentSessions,
  saveLastSessionState,
  loadLastSessionState,
  loadCustomMessage,
  saveCustomMessage,
  loadSnippets,
  saveSnippets,
} from "./store.js";
import { recommendFiles } from "./recommender.js";
import { evaluateSessionContinuity } from "./continuity.js";
import {
  loadConfig,
  saveConfig,
  DEFAULT_CONFIG,
  isPluginEnabled,
  formatConfidence,
  initWorkspace,
  findWorkspaceRoot,
} from "./config.js";

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

    if (!config) {
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

    const injectMessages: string[] = [];

    // 1. Session Continuity Check (Handover only on 1st turn of new session)
    if (config.continuity?.enabled) {
      const continuityResult = evaluateSessionContinuity(
        input.conversationId,
        workspaceRoot,
        config
      );
      if (
        continuityResult.isContinuity &&
        continuityResult.recentSessions &&
        continuityResult.recentSessions.length > 0
      ) {
        const sessionLines = continuityResult.recentSessions.map((s) => {
          const queryAttr = s.lastQuery
            ? ` last_query="${s.lastQuery.replace(/"/g, "&quot;")}"`
            : "";
          const updatedAttr = s.updatedAt
            ? ` updated_at="${s.updatedAt}"`
            : "";
          return `  <session id="${s.conversationId}"${queryAttr}${updatedAttr} />`;
        });
        const continuityXml = `<recent_sessions>\n${sessionLines.join("\n")}\n</recent_sessions>`;
        injectMessages.push(continuityXml);
      }
    }

    // 2. Custom Snippet Injection (Injected only when user query triggers #<tag>)
    const snippets = loadSnippets(workspaceRoot);
    if (snippets.size > 0 && parsed.query) {
      const matches = Array.from(parsed.query.matchAll(/#([a-zA-Z0-9_\u4e00-\u9fa5\-]+)/g));
      const matchedTags = Array.from(new Set(matches.map((m) => m[1].toLowerCase())));
      const triggeredItems: string[] = [];

      for (const tag of matchedTags) {
        if (snippets.has(tag)) {
          const content = snippets.get(tag)!;
          triggeredItems.push(`  <item tag="#${tag}">${content}</item>`);
        }
      }

      if (triggeredItems.length > 0) {
        const reqXml = `<additional_requirements>\n${triggeredItems.join("\n")}\n</additional_requirements>`;
        injectMessages.push(reqXml);
      }
    }

    // 3. File Recommendations
    if (config.recommend.enabled) {
      const recommendation = recommendFiles(parsed.query, workspaceRoot, config, input.conversationId);
      if (recommendation.mode === "SUGGESTION_HINT" && recommendation.items.length > 0) {
        const format = config.recommend.confidenceFormat ?? "categorical";
        const xmlItems = recommendation.items.map((item) => {
          const conf = formatConfidence(item.score, format, config.recommend.confidenceTiers);
          const confAttr = conf !== null ? ` confidence="${conf}"` : "";
          return `  <file path="${item.path}"${confAttr} reason="${item.reasons.join(",")}" />`;
        });

        const recMessage = `<context_recommendations source="context-flow">\n${xmlItems.join("\n")}\n</context_recommendations>`;
        injectMessages.push(recMessage);
      }
    }

    if (injectMessages.length > 0) {
      console.log(
        JSON.stringify({
          injectSteps: [
            {
              ephemeralMessage: injectMessages.join("\n\n"),
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
      const nowIso = new Date().toISOString();
      const currentConvId = input.conversationId || "unknown";
      const gitFiles = getGitModifiedFiles(workspaceRoot);

      // Noise filter & Action deduplication:
      // Only record episodes that have concrete file operations or citations
      const hasAction =
        parsed.readFiles.length > 0 ||
        parsed.editedFiles.length > 0 ||
        parsed.citedFiles.length > 0;

      if (hasAction) {
        const episode: Episode = {
          id: randomUUID().replace(/-/g, "").slice(0, 12),
          timestamp: nowIso,
          conversationId: currentConvId,
          query: parsed.query,
          thinkingTokens: parsed.thinkingTokens,
          readFiles: parsed.readFiles,
          editedFiles: parsed.editedFiles,
          citedFiles: parsed.citedFiles,
          gitStatus: gitFiles,
        };

        saveEpisode(workspaceRoot, episode);
      }

      // Always update Last Session Snapshot for Session Continuity
      const maxSessions = config.continuity?.maxSessions ?? 5;
      saveSessionSnapshot(
        workspaceRoot,
        {
          conversationId: currentConvId,
          updatedAt: nowIso,
          lastQuery: parsed.query,
        },
        maxSessions
      );
    }


    console.log(JSON.stringify({}));
  } catch {
    console.log(JSON.stringify({}));
  }
}

function handleInit() {
  const workspaceRoot = findWorkspaceRoot() || process.cwd();
  
  let binRelPath = "plugins/agy-context-flow/bin/context-flow.cjs";
  if (process.argv[1]) {
    const executedScript = path.resolve(process.argv[1]);
    const relToWs = path.relative(workspaceRoot, executedScript).replace(/\\/g, "/");
    if (!relToWs.startsWith("..") && !path.isAbsolute(relToWs)) {
      binRelPath = relToWs;
    }
  }

  const result = initWorkspace(workspaceRoot, binRelPath);

  if (result.configCreated) {
    console.log("✅ Created configuration: .agents/context-flow/config.json");
  } else {
    console.log("ℹ️  Configuration already present: .agents/context-flow/config.json");
  }

  if (result.hooksUpdated) {
    console.log("✅ Registered lifecycle hooks in: .agents/hooks.json");
  }

  if (result.snippetsCreated) {
    console.log("✅ Created initial snippets: .agents/context-flow/snippets.md (#op, #rg)");
  } else {
    console.log("ℹ️  Snippets file already present: .agents/context-flow/snippets.md");
  }

  console.log("🚀 agy-context-flow initialized and active!");
}

function handleStats() {
  const workspaceRoot = process.cwd();
  const config = loadConfig(workspaceRoot);
  if (!config) {
    console.log("⚠️  Plugin is not enabled in this project. Run `node plugins/agy-context-flow/bin/context-flow.cjs init` to enable.");
    return;
  }

  const stats = getStats(workspaceRoot);

  console.log("\n📊 === agy-context-flow Statistics ===");
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
    console.log("⚠️  Plugin is not enabled in this project. Run `node plugins/agy-context-flow/bin/context-flow.cjs init` to enable.");
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
    console.log("⚠️  Plugin is not enabled in this project. Run `node plugins/agy-context-flow/bin/context-flow.cjs init` to enable.");
    return;
  }

  console.log(`\n🔍 Evaluating recommendation for query: "${query}"...`);
  const result = recommendFiles(query, workspaceRoot, config);

  if (result.mode === "NONE" || result.items.length === 0) {
    console.log("  Result: No file recommendations (gating closed or below threshold).");
  } else {
    console.log(`  Result (${result.items.length} suggestions):`);
    result.items.forEach((item, idx) => {
      const conf = formatConfidence(
        item.score,
        config.recommend.confidenceFormat ?? "categorical",
        config.recommend.confidenceTiers
      );
      const confLabel = conf !== null ? ` [${conf}]` : "";
      console.log(`  ${idx + 1}. ${item.path}${confLabel} (Score: ${(item.score * 100).toFixed(0)}%)`);
      item.reasons.forEach((r) => console.log(`     - ${r}`));
    });
  }
  console.log("");
}

function handleSessions() {
  const workspaceRoot = process.cwd();
  const list = loadRecentSessions(workspaceRoot);
  if (list.length === 0) {
    console.log("\nℹ️  No recorded sessions in .agents/context-flow/last_session.json\n");
    return;
  }
  console.log(`\n📋 === Recent Sessions Buffer (${list.length}) ===`);
  list.forEach((s, idx) => {
    console.log(`  ${idx + 1}. [${s.conversationId}] (${s.updatedAt})`);
    console.log(`     Last Query: "${s.lastQuery}"`);
  });
  console.log("");
}

function handleCustomMessage(args: string[]) {
  const workspaceRoot = process.cwd();
  const snippets = loadSnippets(workspaceRoot);

  if (args.length === 0) {
    if (snippets.size === 0) {
      console.log("\nℹ️  No snippets configured in .agents/context-flow/snippets.md");
      console.log("Usage: context-flow snippet set <tag> <content>");
      console.log("Example: context-flow snippet set op \"你的看法是?\"\n");
      return;
    }
    console.log(`\n📝 === Active Snippets (${snippets.size}) [.agents/context-flow/snippets.md] ===`);
    for (const [tag, content] of snippets.entries()) {
      console.log(`  #${tag} => "${content}"`);
    }
    console.log("\n(Triggered dynamically when your prompt includes #<tag>)\n");
    return;
  }

  if (args[0] === "clear") {
    saveSnippets(workspaceRoot, new Map());
    saveCustomMessage(workspaceRoot, "");
    console.log("✅ Cleared all snippets (.agents/context-flow/snippets.md cleared).");
    return;
  }

  if (args[0] === "set" && args.length >= 3) {
    const rawTag = args[1].replace(/^#+/, "").toLowerCase();
    const content = args.slice(2).join(" ");
    snippets.set(rawTag, content);
    saveSnippets(workspaceRoot, snippets);
    console.log(`✅ Saved snippet #${rawTag} to .agents/context-flow/snippets.md`);
    return;
  }

  if (args[0] === "delete" && args.length >= 2) {
    const rawTag = args[1].replace(/^#+/, "").toLowerCase();
    if (snippets.delete(rawTag)) {
      saveSnippets(workspaceRoot, snippets);
      console.log(`✅ Deleted snippet #${rawTag}`);
    } else {
      console.log(`⚠️ Snippet #${rawTag} not found.`);
    }
    return;
  }

  // Fallback for simple message setting
  const rawTag = "msg";
  const messageText = args.join(" ");
  snippets.set(rawTag, messageText);
  saveSnippets(workspaceRoot, snippets);
  console.log(`✅ Saved snippet #${rawTag}: "${messageText}" to .agents/context-flow/snippets.md`);
}

function handleConfig(args: string[]) {
  const workspaceRoot = process.cwd();
  const config = loadConfig(workspaceRoot);

  if (args.length === 0) {
    if (!config) {
      console.log("\n⚠️  Status: Disabled in this project (No .agents/context-flow/config.json).");
      console.log("Run `node plugins/agy-context-flow/bin/context-flow.cjs init` to enable.\n");
      return;
    }
    console.log("\n⚙️ === Current agy-context-flow Configuration ===");
    console.log(JSON.stringify(config, null, 2));
    console.log("");
    return;
  }

  const action = args[0];
  if (action === "set") {
    const key = args[1];
    const value = args[2];

    if (!key || value === undefined) {
      console.log(
        "Usage: context-flow config set <record.enabled|recommend.enabled|recommend.threshold|recommend.maxItems|recommend.confidenceFormat|recommend.confidenceTiers.high|recommend.confidenceTiers.medium|recommend.confidenceTiers.low|continuity.enabled|continuity.maxSessions> <value>"
      );
      return;
    }

    const currentConfig = config || { ...DEFAULT_CONFIG };
    if (!currentConfig.recommend.confidenceTiers) {
      currentConfig.recommend.confidenceTiers = { high: 0.75, medium: 0.5, low: 0.35 };
    }
    if (!currentConfig.continuity) {
      currentConfig.continuity = { enabled: true, maxSessions: 5 };
    }

    if (key === "record.enabled") {
      currentConfig.record.enabled = value === "true" || value === "1";
    } else if (key === "recommend.enabled") {
      currentConfig.recommend.enabled = value === "true" || value === "1";
    } else if (key === "recommend.threshold") {
      currentConfig.recommend.threshold = parseFloat(value);
    } else if (key === "recommend.maxItems") {
      currentConfig.recommend.maxItems = parseInt(value, 10);
    } else if (key === "recommend.confidenceFormat") {
      if (value !== "categorical" && value !== "numeric" && value !== "hidden") {
        console.log("Invalid format. Must be 'categorical', 'numeric', or 'hidden'.");
        return;
      }
      currentConfig.recommend.confidenceFormat = value;
    } else if (key === "recommend.confidenceTiers.high") {
      currentConfig.recommend.confidenceTiers.high = parseFloat(value);
    } else if (key === "recommend.confidenceTiers.medium") {
      currentConfig.recommend.confidenceTiers.medium = parseFloat(value);
    } else if (key === "recommend.confidenceTiers.low") {
      currentConfig.recommend.confidenceTiers.low = parseFloat(value);
    } else if (key === "continuity.enabled") {
      currentConfig.continuity.enabled = value === "true" || value === "1";
    } else if (key === "continuity.maxSessions") {
      currentConfig.continuity.maxSessions = parseInt(value, 10);
    } else {
      console.log(`Unknown config key: ${key}`);
      return;
    }

    saveConfig(currentConfig, workspaceRoot);
    console.log(`✅ Config updated in .agents/context-flow/config.json: ${key} = ${value}`);
  }
}

async function main() {
  const args = process.argv.slice(2);
  const command = args[0];
  const subCommand = args[1];

  if (command === "hook") {
    if (subCommand === "pre-invocation") {
      await handlePreInvocation();
    } else if (subCommand === "post-invocation" || subCommand === "stop") {
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
  } else if (command === "sessions") {
    handleSessions();
  } else if (command === "message" || command === "custom-message" || command === "snippet" || command === "snippets") {
    handleCustomMessage(args.slice(1));
  } else if (command === "recommend") {
    const query = args.slice(1).join(" ");
    if (!query) {
      console.error("Please provide a query: context-flow recommend '<query>'");
      process.exit(1);
    }
    handleRecommend(query);
  } else if (command === "config") {
    handleConfig(args.slice(1));
  } else {
    console.log("Usage: context-flow [init | config [set <k> <v>] | snippet [set <tag> <text>|delete <tag>|clear] | message [<text>|clear] | sessions | hook <pre-invocation|post-invocation> | stats | list | recommend <query>]");
  }
}

main().catch((err) => {
  console.error("CLI error:", err);
  process.exit(1);
});
