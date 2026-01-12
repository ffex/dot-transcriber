# Claude Working Prompts

This file contains prompt templates and guidelines for Claude to use throughout the Dot project development.

---

## 📋 Update WhereAreWe.md Prompt

**When to use**: After completing tasks, making decisions, or changing project direction

**Prompt**:
```
Update WhereAreWe.md with the following:
- Mark completed tasks as [x]
- Update phase status if phase is complete (🟡 In Progress, 🟢 Complete)
- Add entry to "Notes & Decisions Log" with today's date and summary
- Update "Current Status" section with latest progress
- Update "Next Steps" with immediate action items
- Add any new questions to "Questions & Open Items"
```

---

## 🔨 Starting a New Task Prompt

**When to use**: Before beginning work on a new feature or phase

**Prompt**:
```
Before implementing [TASK_NAME]:
1. Read WhereAreWe.md to understand current project state
2. Read relevant code files to understand existing implementation
3. Create todo list with subtasks
4. Identify dependencies and potential issues
5. Ask user for clarification on any ambiguous requirements
6. Implement step-by-step, updating todos as you go
7. Update WhereAreWe.md when complete
```

---

## ✅ Task Completion Checklist

**When to use**: After completing any implementation work

**Checklist**:
- [ ] Code compiles without errors
- [ ] Code follows Rust best practices
- [ ] Error handling is implemented
- [ ] Todos are marked as complete
- [ ] WhereAreWe.md is updated
- [ ] Any new dependencies are added to Cargo.toml
- [ ] Configuration changes are documented
- [ ] Consider: Are tests needed?

---

## 🐛 Debugging Prompt

**When to use**: When encountering errors or unexpected behavior

**Prompt**:
```
Debug [ISSUE_DESCRIPTION]:
1. Read error messages carefully
2. Check recent changes that might have caused the issue
3. Verify configuration and dependencies
4. Add logging/println for debugging if needed
5. Test fix incrementally
6. Document the issue and solution in WhereAreWe.md if significant
```

---

## 🔍 Research Task Prompt

**When to use**: When researching libraries, APIs, or approaches

**Prompt**:
```
Research [TOPIC]:
1. Search for official documentation
2. Compare alternatives (pros/cons)
3. Check Rust ecosystem crates on crates.io
4. Consider: ease of use, maintenance, community support
5. Make recommendation with reasoning
6. Document findings in WhereAreWe.md under relevant phase
```

---

## 🎯 Phase Completion Prompt

**When to use**: When finishing a major phase

**Prompt**:
```
Complete Phase [N]:
1. Verify all phase tasks are checked off
2. Test all phase functionality end-to-end
3. Update phase status to 🟢 Complete
4. Add detailed summary to "Notes & Decisions Log"
5. Review next phase and prepare initial task breakdown
6. Update "Next Steps" with concrete actions for next phase
7. Commit changes with descriptive message
```

---

## 💬 Communication Guidelines

**When working with user**:
- Be concise and technical (user is a developer)
- Show code and explain reasoning
- Ask for decisions on open questions before implementing
- Suggest best practices but defer to user preferences
- Keep Italian language context in mind for transcription features
- Think about real-world usage (car, walking, etc.)

**Avoid**:
- Creating unnecessary files or over-engineering
- Adding features not requested
- Making decisions on unclear requirements without asking
- Using overly complex solutions when simple ones work

---

## 🚀 Quick Reference Commands

**Check current status**:
1. Read WhereAreWe.md
2. Check git status
3. Review current phase tasks

**Start new implementation**:
1. Create todo list
2. Read relevant code
3. Implement incrementally
4. Update documentation

**Wrap up session**:
1. Mark todos complete
2. Update WhereAreWe.md
3. Suggest next steps
4. Commit if user requests
