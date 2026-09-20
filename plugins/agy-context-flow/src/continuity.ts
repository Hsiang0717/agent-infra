import { ContinuityResult, LastSessionState, SessionSnapshot } from "./types.js";
import { loadRecentSessions } from "./store.js";
import { ContextFlowConfig } from "./config.js";

/**
 * Evaluates whether the current invocation is the first turn of a new session,
 * and if so, returns the list of recent session snapshots for multi-session handover.
 */
export function evaluateSessionContinuity(
  currentConversationId: string | undefined,
  workspaceRoot?: string,
  config?: ContextFlowConfig | null,
  overrideRecentSessions?: SessionSnapshot[] | LastSessionState | null
): ContinuityResult {
  if (!config || !config.continuity || !config.continuity.enabled) {
    return { isContinuity: false };
  }

  let recentList: SessionSnapshot[];
  if (overrideRecentSessions !== undefined) {
    if (Array.isArray(overrideRecentSessions)) {
      recentList = overrideRecentSessions;
    } else if (overrideRecentSessions && overrideRecentSessions.conversationId) {
      recentList = [overrideRecentSessions];
    } else {
      recentList = [];
    }
  } else {
    recentList = loadRecentSessions(workspaceRoot);
  }

  if (recentList.length === 0) {
    return { isContinuity: false };
  }

  // If the top session is the current session, it means we are in the same session (suppress handover)
  const topSession = recentList[0];
  if (!currentConversationId || topSession.conversationId === currentConversationId) {
    return { isContinuity: false };
  }

  // Filter out any entries that match currentConversationId
  const handoverSessions = recentList.filter((s) => s.conversationId !== currentConversationId);
  if (handoverSessions.length === 0) {
    return { isContinuity: false };
  }

  return {
    isContinuity: true,
    recentSessions: handoverSessions,
    previousId: handoverSessions[0].conversationId,
    lastQuery: handoverSessions[0].lastQuery,
    updatedAt: handoverSessions[0].updatedAt,
  };
}

