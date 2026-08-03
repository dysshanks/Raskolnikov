# AI Providers

Raskolnikov supports local and cloud AI providers through a common
`#[async_trait] Provider` trait. Cloud providers are opt-in — no API key is
required by default, and the tool works fully air-gapped with Ollama.

## Provider resolution

On startup, providers are detected in this order:

1. Ollama (local — checked via `http://localhost:11434`)
2. `ANTHROPIC_API_KEY`
3. `OPENAI_API_KEY`
4. `OPENROUTER_API_KEY`
5. `GROQ_API_KEY`
6. `NOUS_API_KEY`
7. `LLAMA_API_KEY`
8. `TOGETHER_API_KEY`

The first detected provider becomes default unless overridden in
`config.toml` or via `--provider`. If no provider is usable, Raskolnikov
offers an Ollama fallback or continues without AI.

## Ollama (default)

No account, no API key, no network required after the model pull.

```bash
ollama pull qwen3
rsk config provider ollama
rsk config model qwen3
```

Recommended models:

| Model | Pull | Context | Notes |
|-------|------|---------|-------|
| `qwen3` | `ollama pull qwen3` | 32k | Recommended |
| `nous-hermes3` | `ollama pull nous-hermes3` | 128k | Strong tool-call reasoning |
| `deepseek-r1` | `ollama pull deepseek-r1` | 64k | Chain-of-thought, slower |
| `llama3.3` | `ollama pull llama3.3` | 128k | Strong general baseline |
| `phi4` | `ollama pull phi4` | 16k | Strong reasoning for size |
| `mistral` | `ollama pull mistral` | 8k | Lightweight fallback |

**Hardware:**

| Model size | Min RAM | GPU |
|------------|---------|-----|
| 7–8B | 8 GB | Optional |
| 14B | 16 GB | Recommended |
| 30B+ | 32 GB | Required for acceptable speed |

**Local alternatives to Ollama:** llama.cpp server, LM Studio, and Jan all
expose OpenAI-compatible endpoints and work via the `openai` provider with
`base_url` pointed at localhost.

## Anthropic

```bash
export ANTHROPIC_API_KEY=sk-ant-...
rsk config provider anthropic
rsk config model claude-sonnet-4-6
```

| Model | Notes |
|-------|-------|
| `claude-sonnet-4-6` | Recommended — best balance |
| `claude-opus-4-6` | Most capable, higher cost |
| `claude-haiku-4-5` | Fastest, lowest cost |

Best for highest reasoning quality and long complex sessions (200k context).

## OpenAI

```bash
export OPENAI_API_KEY=sk-...
rsk config provider openai
rsk config model gpt-4o
```

| Model | Notes |
|-------|-------|
| `gpt-4o` | Recommended |
| `gpt-4o-mini` | Lightweight, low cost |
| `o3-mini` | Strong reasoning at lower cost |

The `openai` provider also handles Groq, Llama API, and Together — they share
an OpenAI-compatible API with different `base_url`s.

## OpenRouter

Routes to many providers via a single API key.

```bash
export OPENROUTER_API_KEY=sk-or-...
rsk config provider openrouter
rsk config model meta-llama/llama-3.3-70b-instruct
```

## Groq

Very fast token generation on open models.

```bash
export GROQ_API_KEY=gsk_...
rsk config provider groq
rsk config model llama-3.3-70b-versatile
```

## Nous Research

Hermes models fine-tuned for agentic behaviour, accessed via the Nous API
directly (not through OpenRouter).

```bash
export NOUS_API_KEY=sk-nous-...
rsk config provider nous
rsk config model hermes-3-llama-3.1-70b
```

For local use without an API key: `ollama pull nous-hermes3`.

## Llama API (Meta)

```bash
export LLAMA_API_KEY=...
rsk config provider llama-api
rsk config model llama3.3-70b
```

## Together AI

```bash
export TOGETHER_API_KEY=...
rsk config provider together
rsk config model meta-llama/Llama-3.3-70B-Instruct-Turbo
```

## Provider comparison

| Provider | Cost | Speed | Privacy | Context | Best For |
|----------|------|-------|---------|---------|----------|
| Ollama | Free | GPU-dependent | Full local | Up to 128k | Default, air-gapped |
| Nous API | Paid | Fast | API | 128k | Hermes direct — tool-call reliability |
| Groq | Cheap | Very fast | API | 128k | Fast open models |
| Anthropic | Paid | Fast | API | 200k | Highest reasoning quality |
| OpenAI | Paid | Fast | API | 128k | GPT family |
| OpenRouter | Paid | Varies | API | Varies | Non-Hermes model switching |
| Llama API | Paid | Fast | API | 128k | Meta models direct |
| Together AI | Paid | Fast | API | Varies | Open model variety |

## Context window management

1. Full history is passed while within 80% of the model's context limit.
2. At 80%, older tool outputs are summarised and replaced in history.
3. The operator is warned when summarisation occurs.
4. Raw outputs are always preserved on disk in the session `tools/` directory.
5. Operator and agent messages are never summarised.

**Minimum recommended:** 32k tokens. **Recommended:** 128k+.
