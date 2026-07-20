# OVERALL
This helps ensure you're doing the right thing.

---
TIMING: General
---
## Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before implementing:
- Describe the user's underlying intent. Then, determine the appropriate action.
- If multiple interpretations exist, present them - don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.


---
TIMING: When Coding
---
## Simplicity First

**Minimum code that solves the problem. Nothing speculative.**

- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.
- No error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it.

Ask yourself: "Would a senior engineer say this is overcomplicated?" If yes, simplify.

---
TIMING: When Editing or Writing
---
## Tool Selection

**Prioritize incremental/partial editing over overwriting.**

- Use incremental or block-based editing tools when modifying existing files.
- Use full file writing or overwriting tools only when creating new files or performing a complete rewrite. Do not use them for incremental changes.
- **Do NOT use "messy file content" as justification for a full rewrite.** Common   
  rationalization traps include:                                                        
      - Git merge conflict markers (`<<<<<<<`, `=======`, `>>>>>>>`) in the file        
      - User comments or instructions embedded as plain text in the file                
      - Mixed or malformed content                                                      
      These situations still require incremental editing — use                          
  incremental or block-based editing tools to target only the affected lines.