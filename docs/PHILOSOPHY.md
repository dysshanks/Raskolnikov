# Philosophy

Raskolnikov is built on a set of intentional constraints that define what it is
and, equally, what it will never be.

## On the Name

The name is taken from Rodion Raskolnikov, the protagonist of Dostoevsky's
*Crime and Punishment*. Raskolnikov is a former law student who develops a
theory that certain extraordinary individuals are permitted to transgress moral
boundaries in service of a higher purpose. He tests this theory by committing
murder with an axe — and the novel is the account of his subsequent collapse
under the weight of what he has done.

The name fits a security tool not because of the crime, but because of the
question the novel forces on every reader: just because you *can*, does that
mean you *should*? Every penetration test asks the same question. The tool
gives you the axe. Whether and how you wield it is your responsibility alone.

```text
 /\   
(  `.____(_)_
|           |
(           |
 \    ,'|  |
  \  /  |  |
   \(   |  |
    `   |  |
        |  |
        |  |
        |  |
        |  |
        |  |
        |  |
        |  |
        |  |
        |  |
        |  |
        |  |
        |  |
        ╩══╩
```

## Markdown-Driven

All session artifacts are markdown:

- Transcripts are `.md` files (human-readable, diffable, embeddable in reports)
- Findings are `.md` files (copy-paste into client reports, wiki pages, etc.)
- The specification itself is a single `spec-mvp.md`

Markdown is the universal interchange format for security work. No binary
formats, no proprietary databases, no lock-in.

## Not a Chatbot

A chatbot answers questions. Raskolnikov plans, executes, and interprets
results. It runs tools, parses their output, and adapts its next step based on
what it finds. The conversation panel shows the agent's reasoning and your
direction, but the real work happens in the tool output panel.

## Not a Scanner Wrapper

A wrapper maps one CLI flag to one button. Raskolnikov has no buttons. You
describe a target and an objective in natural language. The agent reasons about
which tool to use, with what flags and in what order, based on the engagement
context it has built up.

## Not Another AI CLI

Most AI CLIs are stateless: you prompt, it answers, you prompt again.
Raskolnikov maintains a persistent engagement context — discovered ports, found
paths, extracted findings — and injects it into every prompt automatically. The
agent knows what it has done and adapts its recommendations accordingly.

## Confirm-All Model

The agent never executes a command without operator approval. Every tool
invocation is gated by a confirmation prompt. The operator can approve, modify,
or reject any suggested command. This is not a safety feature — it is the core
interaction model. The agent plans, the operator decides.

## Observability

Every tool invocation and its output is displayed in the tool output panel
before the agent interprets it. The operator sees exactly what ran and what
came back. No hidden reasoning, no black-box decisions.

## Privacy by Design

- API keys from environment variables only — never in config files
- No telemetry, no phone-home, no usage statistics
- All AI provider communication is direct between Raskolnikov and the provider
  — no intermediary, no proxy, no data collection
- Session files local-only unless explicitly copied by the operator
