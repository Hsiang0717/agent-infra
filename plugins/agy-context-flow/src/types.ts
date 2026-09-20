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

export type ConfidenceFormat = "categorical" | "numeric" | "hidden";

export interface ConfidenceTiers {
  high: number;
  medium: number;
  low: number;
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

export interface ContinuityConfig {
  enabled: boolean;
  maxSessions?: number;
}

export interface SessionSnapshot {
  conversationId: string;
  updatedAt: string;
  lastQuery: string;
}

export type LastSessionState = SessionSnapshot | SessionSnapshot[];

export interface ContinuityResult {
  isContinuity: boolean;
  recentSessions?: SessionSnapshot[];
  previousId?: string;
  lastQuery?: string;
  updatedAt?: string;
}


