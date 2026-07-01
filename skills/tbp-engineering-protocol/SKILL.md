---
name: tbp-engineering-protocol
description: Controls task gating, risk analysis, and engineering dossier verification logic.
---

# TBP Engineering Protocol Skill

## Core Instruction
- Execute gate audit when target triggers are met.
- Refer to [DOSSIER_TEMPLATE.md](file:///.agents/skills/tbp-engineering-protocol/examples/DOSSIER_TEMPLATE.md) to initialize the dossier.
- Always run the dossier validation script before submitting plans.
- When the user explicitly requests to save state, update memory, or archive progress (e.g. "Save", "Save Progress", "Save Conversation", "Save Chat", "Save State"), execute the save state script: `powershell -ExecutionPolicy Bypass -File .agents/skills/tbp-engineering-protocol/scripts/Save-SessionState.ps1 -ConversationId <current_conversation_id>` (where `<current_conversation_id>` is obtained from the current environment/metadata).
