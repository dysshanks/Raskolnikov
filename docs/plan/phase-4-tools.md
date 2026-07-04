# Phase 4 — Tool System

**Objective:** Implement the 4 launch tools (nmap, gobuster/ffuf, nikto, sqlmap) with the executor, output parsing, wordlist resolution, and startup availability checking.

---

## Files to Create / Modify

| File | Status | Purpose |
|------|--------|---------|
| `src/tools/mod.rs` | Create | `Tool` trait, `ToolRegistry`, `check_all_available()` |
| `src/tools/executor.rs` | Create | Async subprocess runner with streaming, timeout, interrupt |
| `src/tools/nmap.rs` | Create | nmap command builder + XML output parser |
| `src/tools/gobuster.rs` | Create | gobuster command builder + ffuf fallback + wordlist resolution |
| `src/tools/nikto.rs` | Create | nikto command builder + output parser |
| `src/tools/sqlmap.rs` | Create | sqlmap command builder + injection confirmation parser |

---

## Data Structures (src/tools/mod.rs)

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &'static str;
    fn check_available(&self) -> Result<Option<SemVer>>;
    fn build_command(&self, args: &ToolArgs) -> Command;
    fn parse_output(&self, raw: &str) -> ToolResult;
}

pub struct ToolArgs {
    pub target: String,
    pub port: Option<u16>,
    pub wordlist: Option<PathBuf>,
    pub extra_flags: Vec<String>,
}

pub enum ToolResult {
    Nmap(NmapResult),
    Gobuster(Vec<WebPath>),
    Nikto(Vec<NiktoFinding>),
    Sqlmap(Vec<SqlmapFinding>),
}

pub struct ToolRunResult {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration: Duration,
    pub was_interrupted: bool,
}
```

---

## Executor Design (src/tools/executor.rs)

```rust
pub async fn run_tool(
    tool: &dyn Tool,
    args: ToolArgs,
    output_tx: tokio::sync::mpsc::Sender<String>,
    interrupt_rx: tokio::sync::watch::Receiver<bool>,
) -> ToolRunResult
```

### Behaviour

| Event | Action |
|-------|--------|
| Tool starts | Spawn `tokio::process::Command`, pipe stdout/stderr line by line into `output_tx` channel |
| Line received | Send to TUI via channel, append to internal buffer |
| Interrupt signal (`interrupt_rx` changes to `true`) | Send SIGTERM to child process PID |
| SIGTERM no response after 5s | Send SIGKILL |
| 30-minute timeout hit | Same SIGTERM → SIGKILL sequence |
| Process exits | Collect exit code, duration, return `ToolRunResult` |
| Partial output on interrupt | All buffered output preserved in `ToolRunResult.stdout` |

### Timeout implementation

```rust
tokio::select! {
    status = child.wait() => { /* normal exit */ }
    _ = tokio::time::sleep(Duration::from_secs(30 * 60)) => { /* timeout */ }
    _ = interrupt_rx.changed() => { /* user interrupt */ }
}
```

---

## Tool Details

### nmap

| Field | Value |
|-------|-------|
| Check | `nmap --version` → parse `Nmap version 7.94` |
| Output flag | `-oX -` (XML to stdout) |
| Default flags | `-sV -sC -T{n}` (n from config, default 4) |
| Full scan | `-p- -sV -T3` when operator requests thorough |
| Script scan | `--script <name>` for targeted NSE |
| Parsing | `quick-xml` reader: extract `<port protocol="tcp"> <portid>80</portid> <state state="open"/> <service name="http" product="Apache httpd" version="2.4.52"/>` |
| Output | `Vec<NmapPort>` — port, protocol, state, service, version, extra_info (script output) |
| Save | Raw XML to `session/tools/nmap_output.xml` |

### gobuster / ffuf

| Field | Value |
|-------|-------|
| Check | `gobuster --version` or `ffuf --version` |
| Default tool | gobuster; `prefer_ffuf = true` in config switches |
| Fallback | If gobuster not found, try ffuf |
| Wordlist resolution | Spec order → fallback to built-in 500-entry list |
| gobuster flags | `dir -u {url} -w {wordlist} -q -t 20` |
| ffuf flags | `-u {url}/FUZZ -w {wordlist} -mc 200,301,302,403 -t 20 -of json -o -` |
| Extension flags | `-x php,html,txt` added when web stack detected |
| Exclude flags | `-b 404,403` |
| Output parsing | Lines like `/{path} (Status: {code})` (gobuster) or JSON array (ffuf) |
| Path flagging | Paths containing `admin`, `login`, `dashboard`, `wp-admin`, `config` flagged as "admin-pattern" |

### nikto

| Field | Value |
|-------|-------|
| Check | `nikto -Version` |
| Flags | `-h {host} -p {port} -Format txt -nointeractive -Plugins @@DEFAULT` |
| Port | Omitted if target is standard 80 |
| Output parsing | Lines starting with `+` |
| CVE extraction | Regex `CVE-\d+-\d{4,}` |
| Output | `Vec<NiktoFinding>` — description, cve (optional), osvdb (optional) |
| Save | Raw to `session/tools/nikto_output.txt` |

### sqlmap

| Field | Value |
|-------|-------|
| Check | `sqlmap --version` |
| Flags | `-u {url} --crawl=2 --batch --level={n} --risk={n} --output-dir={session_dir} --forms --threads=4` |
| --batch | Always set — sqlmap never prompts interactively |
| Level/risk | From config (default level=2, risk=1) |
| Output parsing | Lines containing `Parameter:` + `Type:` + `Payload:` |
| Output | `Vec<SqlmapFinding>` — parameter, injection_type, technique, payload |
| Save | Full output to `session/tools/sqlmap_output.txt` |

---

## Startup Availability Check

On every launch, `ToolRegistry::check_all_available()` runs:

```
tools  ✓ nmap 7.94  ✓ gobuster 3.6.0  ✓ nikto 2.1.6  ✓ sqlmap 1.7.8
```

- Each tool runs its version command (captured, not displayed)
- Missing tools show `✗ nmap (not found)` but don't block startup
- Agent receives the list of available tools and won't propose steps requiring missing tools
- Verdict stored in `AppState.available_tools: HashMap<&str, Option<SemVer>>`

---

## Built-in Wordlist (gobuster fallback)

A `const` array of ~500 common web paths embedded in `gobuster.rs`. Used when no wordlist file is found in any of the standard locations. The agent warns the operator:

```
⚠ No wordlist found in standard locations. Using built-in minimal wordlist (500 entries).
  Install seclists for better coverage: sudo apt install seclists
```

---

## User Stories

| ID | Title | Description |
|----|-------|-------------|
| US-4.1 | Tool availability shown on startup | As a user, I see which tools are installed and their versions when launching `rsk` |
| US-4.2 | Missing tools don't crash | As a user, if nmap isn't installed, I can still launch and use other tools |
| US-4.3 | Tool runs and streams output | As a user, when the agent runs nmap, I see live output streaming in the left panel |
| US-4.4 | Tool output is parsed | As a user, after nmap finishes, the agent knows which ports are open and their services |
| US-4.5 | Long-running tool can be interrupted | As a user, I can press Ctrl+C to stop a sqlmap that's taking too long |
| US-4.6 | Wordlist auto-resolved | As a user, gobuster/ffuf finds a wordlist automatically without me specifying one |
| US-4.7 | Agent skips unavailable tools | As a user, if nikto is not installed, the agent never proposes nikto commands |

---

## Acceptance Criteria

| ID | Criterion | How to Verify |
|----|-----------|---------------|
| AC-4.1 | `nmap --version` → tool detected and version parsed | Run tool check, inspect output |
| AC-4.2 | Missing tool shows `✗` warning but app doesn't crash | Uninstall a tool, launch app |
| AC-4.3 | nmap with `-oX -` produces valid XML that parses into port structs | Run nmap, inspect parsed output |
| AC-4.4 | gobuster finds wordlist from standard paths automatically | Run gobuster, check log for wordlist path |
| AC-4.5 | gobuster falls back to built-in wordlist when none found | Remove wordlists, run gobuster, check for fallback warning |
| AC-4.6 | ffuf used when gobuster not installed and `prefer_ffuf = false` | Uninstall gobuster, run, verify ffuf used |
| AC-4.7 | ffuf used when `prefer_ffuf = true` regardless of gobuster | Set config, run, verify ffuf used |
| AC-4.8 | nikto output parsed: `+` lines extracted, CVEs identified | Run nikto on a test target, inspect findings |
| AC-4.9 | sqlmap runs with `--batch` and output goes to session dir | Run sqlmap, check output dir |
| AC-4.10 | sqlmap output parsed: injection confirmations extracted | Run sqlmap on vulnerable target, inspect findings |
| AC-4.11 | Tool execution streams lines to TUI in real-time | Watch left panel during tool run |
| AC-4.12 | Ctrl+C sends SIGTERM to running tool, output preserved | Start long scan, Ctrl+C, check partial output saved |
| AC-4.13 | 30-min timeout kills the tool and notifies operator | (Test by mocking timeout value) |
| AC-4.14 | Tool output saved to `session/tools/{tool}_output.{ext}` | Check session directory after tool runs |

---

## Parsing Tests (Unit Tests with Fixtures)

Each tool module has unit tests that parse canned output:

| Test | Fixture | Verifies |
|------|---------|----------|
| `nmap_parses_basic_scan` | Sample nmap XML with 3 open ports | Port extraction |
| `nmap_parses_no_open_ports` | nmap XML with all filtered | Empty result |
| `nmap_parses_script_output` | nmap XML with NSE script data | Script extraction |
| `gobuster_parses_stdout` | Sample gobuster output lines | Path extraction |
| `ffuf_parses_json` | Sample ffuf JSON output | Path extraction |
| `nikto_parses_plus_lines` | Sample nikto output | Finding extraction |
| `nikto_extracts_cve` | nikto output with CVE refs | CVE extraction |
| `sqlmap_parses_injection` | sqlmap output with confirmations | Injection extraction |
| `sqlmap_handles_no_injection` | sqlmap output with no findings | Empty result |

---

## Order of Implementation

1. `src/tools/mod.rs` — Tool trait, ToolRegistry, ToolArgs, ToolResult enums
2. `src/tools/executor.rs` — subprocess runner with streaming, timeout, interrupt
3. `src/tools/nmap.rs` — command builder + XML parser
4. `src/tools/gobuster.rs` — command builder + wordlist resolver + ffuf fallback + built-in wordlist
5. `src/tools/nikto.rs` — command builder + output parser
6. `src/tools/sqlmap.rs` — command builder + injection parser
7. Unit tests with fixtures (canned output files)
8. Integration test: run a real tool (if available), verify parsing
