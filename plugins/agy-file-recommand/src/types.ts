export interface HookInput {
  conversationId: string;
  workspacePaths: string[];
  transcriptPath: string;
  artifactDirectoryPath: string;
  modelName?: string;
  invocationNum?: number;
  initialNumSteps?: number;
  stepIdx?: number;
}

export interface Episode {
  id: string;
  timestamp: string;
  conversationId: string;
  query: string;
  thinkingTokens?: string[];
  readFiles: string[];
  editedFiles: string[];
  citedFiles: string[];
  gitStatus: string[];
}

export interface TranscriptStep {
  step_index?: number;
  source?: string;
  type?: string;
  status?: string;
  created_at?: string;
  content?: string;
  thinking?: string;
  tool_calls?: Array<{
    name: string;
    args?: Record<string, unknown>;
  }>;
}

export interface RecommendationItem {
  path: string;
  score: number;
  reasons: string[];
}

export interface RecommendationResult {
  items: RecommendationItem[];
  mode: "SUGGESTION_HINT" | "NONE";
}
