---
name: tbp-engineering-protocol
description: Controls task gating, risk analysis, and engineering dossier verification logic.
---

# TBP Engineering Protocol Skill

## Core Instruction
- Execute gate audit when target triggers are met.
- Refer to [DOSSIER_TEMPLATE.md](file:///{PROJECT_ROOT}/.agents/skills/tbp-engineering-protocol/examples/DOSSIER_TEMPLATE.md) to initialize the dossier.
- Always run the dossier validation script before submitting plans.
- When the user explicitly requests to save state, update memory, or archive progress (e.g. "Save", "Save Progress", "Save Conversation", "Save Chat", "Save State"), execute the **CSD (Consented State Distillation) Protocol**:
  1. The host agent should directly distill a clean JSON array of completed milestones and open problems from the active conversation context. (Spawning subagents `SynthesisAgent` and `CriticAgent` to analyze transcript files is optional and should only be used as a fallback if the host agent's context is truncated or lacks detail).
  2. Execute the CSD save state script using:
     `powershell -ExecutionPolicy Bypass -File {PROJECT_ROOT}/.agents/skills/tbp-engineering-protocol/scripts/Save-SessionStateWithCSD.ps1 -ConversationId <current_conversation_id> -CompletedMilestonesJson '<milestones_json>' -OpenProblemsJson '<problems_json>'`
