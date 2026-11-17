//! A quality preset in Configarr's words, and the lines of `config.yml` that carry it
//! for one curator.
//!
//! A preset resolves to one TRaSH Guides quality profile, which a section's `include:`
//! list names by its `trash_id` as a `TRASH` template, and the quality sizes it is
//! paired with, which the section's `quality_definition:` names by `type:`. Holding a
//! curator to a preset rewrites that entry and that type, and pointing it at a curator
//! rewrites the section's `enabled`, `base_url` and `api_key`; every comment, every
//! other section and every entry naming another template stays exactly where it was.

use lemonfiber_ports::media::preset::Preset;
use lemonfiber_ports::media::Kind;

/// What a preset asks the guides for: a quality profile and its quality sizes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Guidance {
    profile: &'static str,
    sizes: &'static str,
}

impl Guidance {
    /// The `trash_id` of the TRaSH Guides quality profile.
    #[must_use]
    pub const fn profile(self) -> &'static str {
        self.profile
    }

    /// The TRaSH Guides quality sizes, as `quality_definition.type` names them.
    #[must_use]
    pub const fn sizes(self) -> &'static str {
        self.sizes
    }
}

/// The source an entry of `include:` names a TRaSH Guides template with.
const SOURCE: &str = "TRASH";

/// WEB-1080p, for series.
const WEB_1080P: Guidance = Guidance {
    profile: "72dae194fc92bf828f32cde7744e51a1",
    sizes: "series",
};

/// WEB-2160p, for series.
const WEB_2160P: Guidance = Guidance {
    profile: "d1498e7d189fbe6c7110ceaabb7473e6",
    sizes: "series",
};

/// \[SQP\] SQP-1 WEB (1080p), for film, with the streaming sizes it is built for.
const SQP_1_WEB_1080P: Guidance = Guidance {
    profile: "90a3370d2d30cbaf08d9c23b856a12c8",
    sizes: "sqp-streaming",
};

/// HD Bluray + WEB, for film.
const HD_BLURAY_WEB: Guidance = Guidance {
    profile: "d1d67249d3890e49bc12e275d989a7e9",
    sizes: "movie",
};

/// Remux + WEB 1080p, for film.
const REMUX_WEB_1080P: Guidance = Guidance {
    profile: "9ca12ea80aa55ef916e3751f4b874151",
    sizes: "movie",
};

/// UHD Bluray + WEB, for film.
const UHD_BLURAY_WEB: Guidance = Guidance {
    profile: "64fb5f9858489bdac2af690e27c8f42f",
    sizes: "movie",
};

/// Every profile a preset resolves to.
const PROFILES: [Guidance; 6] = [
    WEB_1080P,
    WEB_2160P,
    SQP_1_WEB_1080P,
    HD_BLURAY_WEB,
    REMUX_WEB_1080P,
    UHD_BLURAY_WEB,
];

/// What a preset asks the guides for, for the curator of `kind`.
///
/// The guides offer series only a WEB-1080p and a WEB-2160p profile, so the three
/// 1080p presets resolve to the same television guidance. Film has the full range.
#[must_use]
pub const fn guidance(kind: Kind, preset: Preset) -> Guidance {
    match (kind, preset) {
        (Kind::Tv, Preset::SpaceSaving | Preset::Balanced | Preset::HighQuality) => WEB_1080P,
        (Kind::Tv, Preset::Maximum) => WEB_2160P,
        (Kind::Movies, Preset::SpaceSaving) => SQP_1_WEB_1080P,
        (Kind::Movies, Preset::Balanced) => HD_BLURAY_WEB,
        (Kind::Movies, Preset::HighQuality) => REMUX_WEB_1080P,
        (Kind::Movies, Preset::Maximum) => UHD_BLURAY_WEB,
    }
}

/// The top-level section the curator of `kind` is configured under.
#[must_use]
pub const fn section(kind: Kind) -> &'static str {
    match kind {
        Kind::Tv => "sonarr",
        Kind::Movies => "radarr",
    }
}

/// The kind whose section a top-level key names, or none for any other key.
fn of_section(key: &str) -> Option<Kind> {
    Kind::ALL.into_iter().find(|kind| section(*kind) == key)
}

/// The width of a line's leading whitespace.
fn indent_width(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

/// The leading whitespace of `line`.
fn indent(line: &str) -> &str {
    line.get(..indent_width(line)).unwrap_or_default()
}

/// `line` with any trailing inline comment removed.
fn without_comment(line: &str) -> &str {
    line.find(" #")
        .map_or(line, |at| line.get(..at).unwrap_or(line))
}

/// The value `line` gives `key`, where `line`, past its indent and an entry's dash, is
/// that key.
fn value_of<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let body = without_comment(line.trim_start());
    let body = body.strip_prefix("- ").unwrap_or(body);
    body.strip_prefix(key)?.strip_prefix(':').map(str::trim)
}

/// Whether `line` is the bare `key:` that opens a block.
fn opens(line: &str, key: &str) -> bool {
    value_of(line, key).is_some_and(str::is_empty)
}

/// Whether `line` opens a top-level section: a key in the first column, not a comment.
fn top_level_key(line: &str) -> bool {
    line.chars()
        .next()
        .is_some_and(|first| !first.is_whitespace() && first != '#')
}

/// Whether `line` lies inside a block whose key is indented `width`.
fn within(line: &str, width: usize) -> bool {
    line.trim().is_empty() || indent_width(line) > width
}

/// One line of `config`: the kind whose section it lies in, and whether it is a key of
/// an instance in that section.
#[derive(Clone, Copy)]
struct Line<'a> {
    kind: Option<Kind>,
    key: bool,
    text: &'a str,
}

/// Every line of `config`, each marked with the kind whose section it lies in and
/// whether it is a key of one of the section's instances.
fn sectioned(config: &str) -> Vec<Line<'_>> {
    let mut kind = None;
    let mut named: Option<usize> = None;
    let mut keyed: Option<usize> = None;
    config
        .lines()
        .map(|text| {
            let width = indent_width(text);
            let body = text.trim_start();
            let key = if top_level_key(text) {
                kind = of_section(without_comment(text).trim_end().trim_end_matches(':'));
                named = None;
                keyed = None;
                false
            } else if body.is_empty() || body.starts_with('#') {
                false
            } else if named.is_none_or(|at| width <= at) {
                named = Some(width);
                keyed = None;
                false
            } else {
                *keyed.get_or_insert(width) == width
            };
            Line { kind, key, text }
        })
        .collect()
}

/// The lines of the block under a key indented `width`, from `at`, and the line after.
fn block<'a>(lines: &[Line<'a>], mut at: usize, width: usize) -> (Vec<&'a str>, usize) {
    let mut held = Vec::new();
    while let Some(line) = lines.get(at).filter(|line| within(line.text, width)) {
        held.push(line.text);
        at += 1;
    }
    let blank = held
        .iter()
        .rev()
        .take_while(|line| line.trim().is_empty())
        .count();
    (
        held.get(..held.len() - blank).unwrap_or_default().to_vec(),
        at - blank,
    )
}

/// The entries of an `include:` block, each its lines, and the lines between them.
enum Included<'a> {
    /// An entry: its dash line and every line beneath it.
    Entry(Vec<&'a str>),
    /// A comment or blank line between entries.
    Between(&'a str),
}

/// The block of an `include:` split into its entries.
fn entries<'a>(held: &[&'a str]) -> Vec<Included<'a>> {
    let mut out: Vec<Included<'a>> = Vec::new();
    for line in held {
        let continues = matches!(out.last(), Some(Included::Entry(first)) if first
            .first()
            .is_some_and(|dash| !line.trim().is_empty() && indent_width(line) > indent_width(dash)));
        match out.last_mut() {
            Some(Included::Entry(lines)) if continues => lines.push(line),
            _ if line.trim_start().starts_with("- ") => out.push(Included::Entry(vec![line])),
            _ => out.push(Included::Between(line)),
        }
    }
    out
}

/// Whether an entry names one of the profiles a preset resolves to.
fn a_preset(entry: &[&str]) -> bool {
    entry
        .iter()
        .filter_map(|line| value_of(line, "template"))
        .any(|named| PROFILES.iter().any(|known| known.profile == named))
}

/// The entry naming `guidance`'s profile, its dash at `at`.
fn entry(at: &str, guidance: Guidance) -> [String; 2] {
    [
        format!("{at}- template: {}", guidance.profile),
        format!("{at}  source: {SOURCE}"),
    ]
}

/// The `include:` block `held`, under a key indented `key`, with every entry naming a
/// preset's profile replaced by one naming `guidance`'s.
fn included(held: &[&str], key: &str, guidance: Guidance) -> Vec<String> {
    let split = entries(held);
    let at = split
        .iter()
        .find_map(|one| match one {
            Included::Entry(lines) => lines.first().map(|dash| indent(dash).to_owned()),
            Included::Between(_) => None,
        })
        .unwrap_or_else(|| format!("{key}  "));
    let mut out = Vec::new();
    let mut written = false;
    for one in split {
        match one {
            Included::Entry(lines) if a_preset(&lines) => {
                if !written {
                    out.extend(entry(&at, guidance));
                    written = true;
                }
            }
            Included::Entry(lines) => out.extend(lines.into_iter().map(str::to_owned)),
            Included::Between(line) => out.push(line.to_owned()),
        }
    }
    if !written {
        out.extend(entry(&at, guidance));
    }
    out
}

/// The `quality_definition:` block `held`, under a key indented `key`, with its `type`
/// naming `sizes`.
fn defined(held: &[&str], key: &str, sizes: &str) -> Vec<String> {
    let child = held
        .iter()
        .find(|line| !line.trim().is_empty() && !line.trim_start().starts_with('#'))
        .map_or_else(|| format!("{key}  "), |line| indent(line).to_owned());
    let typed = format!("{child}type: {sizes}");
    let mut out: Vec<String> = Vec::new();
    let mut written = false;
    for line in held {
        if indent(line) == child && value_of(line, "type").is_some() {
            if !written {
                out.push(typed.clone());
                written = true;
            }
        } else {
            out.push((*line).to_owned());
        }
    }
    if !written {
        out.insert(0, typed);
    }
    out
}

/// `config` with the section of `kind` held to `preset`: the entries of each
/// `include:` naming a preset's profile replaced by the one `preset` asks for, and the
/// `type` of each `quality_definition:` set to its sizes, every other line kept. Line
/// endings become LF and the result ends in one newline.
#[must_use]
pub fn rewrite(config: &str, kind: Kind, preset: Preset) -> String {
    let asked = guidance(kind, preset);
    let lines = sectioned(config);
    let mut out = String::with_capacity(config.len());
    let mut at = 0;
    while let Some(Line {
        kind: section,
        key,
        text: line,
    }) = lines.get(at).copied()
    {
        at += 1;
        push(&mut out, line);
        if section != Some(kind) || !key {
            continue;
        }
        let rewritten: fn(&[&str], &str, Guidance) -> Vec<String> = if opens(line, "include") {
            included
        } else if opens(line, "quality_definition") {
            |held, key, asked| defined(held, key, asked.sizes)
        } else {
            continue;
        };
        let (held, next) = block(&lines, at, indent_width(line));
        at = next;
        for kept in rewritten(&held, indent(line), asked) {
            push(&mut out, &kept);
        }
    }
    out
}

/// `config` with the section of `kind` pointed at `base_url` and the secret named
/// `secret`, and enabled, every other line kept.
#[must_use]
pub fn pointed(config: &str, kind: Kind, base_url: &str, secret: &str) -> String {
    let mut out = String::with_capacity(config.len());
    for Line {
        kind: section,
        key,
        text: line,
    } in sectioned(config)
    {
        let at = indent(line);
        let ours = section == Some(kind) && key;
        match line {
            _ if ours && value_of(line, "enabled").is_some() => {
                push(&mut out, &format!("{at}enabled: true"));
            }
            _ if ours && value_of(line, "base_url").is_some() => {
                push(&mut out, &format!("{at}base_url: {base_url}"));
            }
            _ if ours && value_of(line, "api_key").is_some() => {
                push(&mut out, &format!("{at}api_key: !secret {secret}"));
            }
            _ => push(&mut out, line),
        }
    }
    out
}

/// Whether the section of `kind` in `config` is held to `preset` already.
#[must_use]
pub fn holds(config: &str, kind: Kind, preset: Preset) -> bool {
    rewrite(config, kind, preset) == config
}

/// Append `line` and its newline.
fn push(out: &mut String, line: &str) {
    out.push_str(line);
    out.push('\n');
}

#[cfg(test)]
mod tests;
