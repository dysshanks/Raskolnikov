# Phase 5 — Agent Shell

**Objective:** Build the core conversation loop that wires the TUI, AI provider, and tools together. This is the brain of Raskolnikov — it receives operator input, calls the AI, parses tool requests, confirms with the operator, executes tools, and feeds results back to the AI.

---

## Files to Create / Modify

| File | Status | Purpose |
|------|--------|---------|
| `src/agent/mod.rs` | Rewrite | Re-export shell, context, prompt |
| `src/agent/shell.rs` | Create | Main conversation loop — the orchestration core |
| `src/agent/context.rs` | Create | `EngagementContext` — accumulated findings, ports, paths |
| `src/agent/prompt.rs` | Create | System prompt builder — role, tools, context |

---

## Conversation Loop (src/agent/shell.rs)

```
loop {
    // 1. Wait for operator input (idle state)
    let input = wait_for_input().await;

    if input == "/quit" { handle_quit(); break; }

    // 2. Add operator message to history
    history.push(Message::user(input));
    logger.log(Event::Operator(input));

    // 3. Build full context + call AI
    let system_prompt = PromptBuilder::build(&context, &available_tools);
    let response = provider.chat(&system_prompt, &history).await?;

    // 4. Stream response to conversation panel
    stream_to_tui(response.content.clone()).await;
    history.push(Message::assistant(&response.content));

    // 5. Check if AI proposed a tool
    if let Some(tool_request) = ToolRequestParser::parse(&response.content) {
        // 5a. State = AwaitingConfirmation
        tui.set_state(AppState::AwaitingConfirm);
        let operator_response = wait_for_operator_approval(tool_request).await;

        match operator_response {
            Approval::Yes => { /* proceed */ }
            Approval::Modified(cmd) => { /* use modified args */ }
            Approval::No => { /* skip, loop back */ }
            Approval::Redirect(new_input) => {
                /* inject new_input as operator message, loop */
                continue;
            }
        }

        // 5b. State = ToolRunning
        tui.set_state(AppState::ToolRunning);
        let output_tx = tui.get_tool_output_channel();
        let interrupt_rx = tui.get_interrupt_channel();
        let result = tools::executor::run_tool(
            &tool, tool_request.args, output_tx, interrupt_rx
        ).await;

        // 5c. Parse output
        let tool_result = tool.parse_output(&result.stdout);

        // 5d. Add to context
        context.merge(tool_result);

        // 5e. Add tool result as a message
        history.push(Message::tool(result.stdout));

        // 5f. Call AI again with tool result
        let follow_up = provider.chat(&system_prompt, &history).await?;
        stream_to_tui(follow_up.content).await;
        history.push(Message::assistant(&follow_up.content));
    }

    // 6. Check context window usage
    context_manager.check_and_summarise(&mut history).await;
}
```

### Tool Request Parsing

The AI signals a tool request within its response. Two possible formats (the system prompt specifies which):

**Option A: Structured JSON block**
```
I'll scan the target with nmap.

```tool
{
  "tool": "nmap",
  "args": {
    "target": "10.0.0.1",
    "flags": ["-sV", "-sC", "-T4"]
  }
}
```
```

**Option B: Markdown code block with command**
```
I'll run nmap to discover open ports.

`nmap -sV -sC -T4 10.0.0.1` — run this?
```

The parser checks for ` ```tool ` JSON blocks first, then falls back to scanning for code blocks containing known tool names. The `— run this?` suffix is a convention but not required — any code block matching a known tool is treated as a proposal.

### EngagementContext (src/agent/context.rs)

```rust
pub struct EngagementContext {
    pub ports: Vec<Port>,
    pub web_paths: Vec<WebPath>,
    pub findings: Vec<Finding>,
    pub targets: Vec<String>,
}

impl EngagementContext {
    pub fn new() -> Self;
    pub fn merge(&mut self, result: ToolResult);  // add parsed tool output
    pub fn to_context_string(&self) -> String;     // formatted for system prompt
    pub fn deduplicate(&mut self);                 // dedup by key+value
}
```

Deduplication rules:
- Ports: dedup by `(port, protocol)`
- Web paths: dedup by `(path, status_code)`
- Findings: dedup by `(description, source)`

### System Prompt (src/agent/prompt.rs)

The system prompt is assembled from components:

```
You are Raskolnikov, a security agent running in a terminal.
You are assisting a security operator with penetration testing.

=== AVAILABLE TOOLS ===
{nmap description, flags, capabilities}
{gobuster description, ...}
{nikto description, ...}
{sqlmap description, ...}

=== RULES ===
1. Always explain your reasoning before proposing a tool.
2. Always state the exact command you want to run.
3. Never execute a tool without operator approval.
4. Wait for "yes" or "go ahead" before proceeding.
5. If the operator says "no" or changes direction, adapt.

=== CURRENT CONTEXT ===
{ports discovered so far}
{web paths found}
{findings}
{current target}

=== CONVERSATION ===
{full message history}
```

The prompt is rebuilt every turn with the latest context. Old context is appended to the conversation history as a new system message.

---

## Input Queuing (during tool execution)

- When `AppState == ToolRunning` and operator presses Enter:
  - If no queued message: store input in `queued_message`, show `[1 queued]` indicator
  - If message already queued: overwrite it
- When tool finishes:
  - If `queued_message` exists: inject as operator input → go to step 3
  - If no queued message: go to idle state and wait for input

---

## User Stories

| ID | Title | Description |
|----|-------|-------------|
| US-5.1 | Agent responds to natural language | As a user, I can type "scan 10.0.0.1" and the agent proposes appropriate nmap flags |
| US-5.2 | Agent asks before running | As a user, the agent always shows the exact command and asks for approval before executing |
| US-5.3 | Operator approves with "yes" | As a user, I can type "yes" to approve the proposed command |
| US-5.4 | Operator modifies command | As a user, I can say "add -p- to that" or "use -T2 instead" and the agent adjusts |
| US-5.5 | Operator denies and redirects | As a user, I can say "no, skip nmap and run gobuster instead" and the agent adapts |
| US-5.6 | Tool output is fed back to agent | As a user, after nmap finishes, the agent sees the results and suggests next steps |
| US-5.7 | Agent tracks findings across session | As a user, the agent remembers what was found earlier and can report it when asked |
| US-5.8 | Operator can ask "what have we found" | As a user, I can ask the agent to summarise all findings so far |
| US-5.9 | Input queues during tool run | As a user, I can type while a tool runs and my message is sent when it finishes |
| US-5.10 | Mid-session redirection | As a user, I can say "forget web stuff, check MySQL" mid-session and the agent pivots |

---

## Acceptance Criteria

| ID | Criterion | How to Verify |
|----|-----------|---------------|
| AC-5.1 | Typing "scan 10.0.0.1" produces an nmap proposal with explanation | Type input, observe agent response |
| AC-5.2 | Proposed command is shown in a code block | Visual inspection |
| AC-5.3 | Typing "yes" starts the tool | Approve, verify tool executes |
| AC-5.4 | Typing "no" skips the tool, agent asks what to do next | Deny, verify agent responds |
| AC-5.5 | Agent stays in AwaitingConfirm state until "yes"/"no"/modified | Type something ambiguous, verify agent clarifies |
| AC-5.6 | After tool completes, agent summarises results and proposes next step | Run nmap, observe follow-up |
| AC-5.7 | Asking "what have we found" returns a summary of all findings | After several tool runs, ask |
| AC-5.8 | Typing during a tool run queues the message; delivered after tool finishes | Start long tool, type message, wait for completion |
| AC-5.9 | Queued message indicator shows in input bar | Visual during tool run |
| AC-5.10 | Mid-session "stop, check MySQL instead" causes agent to pivot | Request mid-session redirect, verify next proposal is MySQL-related |
| AC-5.11 | All messages logged to session.log in real-time | Check session.log file during conversation |
| AC-5.12 | `/quit` at idle ends session and saves | Type `/quit`, confirm |
| AC-5.13 | `/quit` during tool run is queued, processed after tool finishes | Start tool, type `/quit`, verify action after tool ends |

---

## Order of Implementation

1. `src/agent/context.rs` — EngagementContext struct with merge/dedup
2. `src/agent/prompt.rs` — system prompt builder with all components
3. `src/agent/shell.rs` — main conversation loop (steps 1-6 above)
4. Wire shell into TUI — replace TUI's standalone event loop with shell-integrated loop
5. Tool request parser (JSON block + markdown code block)
6. Input queuing during tool execution
7. Integration test: mock AI → mock tool → verify conversation flow
