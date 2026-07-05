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
  1. Spawn a `SynthesisAgent` subagent to analyze the session transcript (from `{SESSION_TRANSCRIPT_PATH}`) to distill a clean JSON array of completed milestones.
  2. Spawn a `CriticAgent` subagent to analyze the same session transcript (from `{SESSION_TRANSCRIPT_PATH}`) to distill a clean JSON array of open problems and risk areas.
  3. Execute the CSD save state script using:
     `powershell -ExecutionPolicy Bypass -File {PROJECT_ROOT}/.agents/skills/tbp-engineering-protocol/scripts/Save-SessionStateWithCSD.ps1 -ConversationId <current_conversation_id> -CompletedMilestonesJson '<milestones_json>' -OpenProblemsJson '<problems_json>'`
