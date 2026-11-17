# AGENTS.md — plugin-configarr

> **Start at the roadmap and board on [lemonfiber.app](https://lemonfiber.app),
> rendered from the report of where every unreleased version stands. Then the
> rules** every repository shares:
> [working in the repositories](https://github.com/lemonfiber/spec/blob/main/50-governance/working-in-the-repositories.md)
> and [the rules for agents](https://github.com/lemonfiber/spec/blob/main/50-governance/ai-contributors.md).
> This file holds only what is true of this repository.

## What this repo is

Configarr as a first-party lemonfiber plugin, offered in Recyclarr's place as
quality sync: `plugin.toml` and the adapter that speaks the `quality.sync@1`
contract in front of Configarr, by rewriting the `config.yml` it reads in its
configuration directory. The adapter is the Rust crate at the root, built on
`lemonfiber-adapter-kit` from the core at the commit `Cargo.toml` pins, into the
image the `Dockerfile` builds. `release.yml` publishes that image from every
commit on `main` and from the train's tag. The
[bundle contract](https://github.com/lemonfiber/spec/blob/main/20-architecture/contracts/bundle.md)
says how the catalogue pins this repository and how lemonfiber carries it.

## The rules you cannot break

- **The plugin id is the service's id** (`ARCH-R266`). The container, the
  configuration directory, the settings and the journal keep their names.
- **`configarr` is a scheduled service** (`ARCH-R431`): it runs to completion at
  the times the `configarr:schedule` setting names and is never kept running, so
  it speaks no contract, publishes no port and provides nothing. It runs as the
  operator's uid under the confined posture, with `/tmp` in memory, sends no
  usage data, and holds its configuration to its schema strictly.
- **Its proofs' recordings sit under `recordings/configarr/`**, and its starting
  files under `config/configarr/` (`ARCH-R271`). Each proof is a run with no
  network, so only the lines it keeps are recorded.
- **Every service declares `reaches` and `asks_for`** (`ARCH-R270`).
- **No Python here.** The reader is spec's `first-party-plugin` workflow's.
- **No `unwrap`, `expect`, `panic` or indexing** outside tests. The lints in
  `Cargo.toml` are the core's.
- **A preset rewrites one curator's section and nothing else**: its `include:`
  entry naming a TRaSH Guides quality profile by `trash_id`, the `type` of its
  `quality_definition`, its `base_url`, its `api_key`, which names a `!secret`,
  and its `enabled`. Every comment, every other section, every entry naming
  another template and every key nested beneath another stays as it was. A
  section no curator was pointed at stays disabled, so a run never asks a
  curator it holds no key for.
- **A curator's key goes into `secrets.yml`, owner-only** (`0600`), and every
  file is reached through `lemonfiber_adapter_kit::shared`.
- **A configuration changed by hand is never written over by `apply`.** The first
  line of one the adapter writes carries the digest of the rest. One whose digest
  no longer holds and that is not the shipped file is named in the refusal. A
  hand edit to `lemonfiber-presets.json` is refused the same way.
- **Only `reapply` writes over a hand edit, and `edited` shows it first.** Both
  answer the whole file on disk beside the whole file putting the profiles back
  writes, named by its path in the configuration directory, each at most 64 KiB;
  the core masks and compares them, so the adapter builds no diff. What is
  written starts from the shipped file, with every other curator held to the
  preset `lemonfiber-presets.json` last kept for it.
- **The core's crates move with the train.** The `rev` in `Cargo.toml` is the
  core commit `targets.toml` names, the same for every `lemonfiber-*` crate.
- **Moving Configarr's digest** means recording every response again against the
  new image in the same change, moving `RELEASE` in `src/adapter.rs` with it
  (`build.rs` reads `DIGEST` from `plugin.toml`, the digest's one home), and
  committing the `proofs.json` the checks write.
