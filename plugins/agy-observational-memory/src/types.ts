export const RELEVANCE_VALUES = ["low", "medium", "high", "critical"] as const;
export type Relevance = (typeof RELEVANCE_VALUES)[number];

export interface Observation {
  id: string; // 12-char hex hash
  content: string;
  timestamp: string; // YYYY-MM-DD HH:MM
  relevance: Relevance;
  sourceStepIndices: number[];
  tokenCount: number;
  conversationId?: string;
}

export interface Reflection {
  id: string; // 12-char hex hash
  content: string;
  supportingObservationIds: string[];
  tokenCount: number;
}

export interface SessionLedger {
  version: number;
  conversationId: string;
  workspacePath?: string;
  createdAt: string;
  updatedAt: string;
  activeObservations: Observation[];
  allObservations: Observation[]; // historical ledger
  reflections: Reflection[];
  droppedObservationIds: string[];
}

export interface ProjectBaseline {
  workspacePath: string;
  updatedAt: string;
  reflections: Reflection[];
}

export interface StoragePaths {
  projectReflectionsPath?: string;
  sessionStoreDir: string;
  activeLedgerPath: string;
  activeProjectionPath: string;
  workspaceRoot?: string;
  workspaceHash: string;
}
