//! Pluggable oracle-text patterns. Each file in this directory is compiled as a module
//! (see build.rs) and registers patterns with `inventory::submit!`. Patterns are tried
//! after the core patterns in `oracle/effects.rs`, `oracle/triggers.rs`, and
//! `oracle/statics.rs`, in order of ascending `priority` (then name).
//!
//! ```ignore
//! fn draw_then_discard(l: &str, b: &mut Builder) -> Option<Effect> { ... }
//! inventory::submit! { EffectPattern { name: "draw then discard", priority: 100, parse: draw_then_discard } }
//! ```

use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::CompileContext;
use std::sync::OnceLock;

include!(concat!(env!("OUT_DIR"), "/oracle_pattern_mods.rs"));

/// Parses an effect clause (lowercase, no trailing period).
pub struct EffectPattern {
    pub name: &'static str,
    pub priority: i32,
    pub parse: fn(&str, &mut Builder) -> Option<Effect>,
}
inventory::collect!(EffectPattern);

/// Parses a trigger condition (lowercase text after "when"/"whenever"/"at", with the
/// leading word included for "at"). Returns (trigger, "it" referent, "that player" referent).
pub struct TriggerPattern {
    pub name: &'static str,
    pub priority: i32,
    pub parse: fn(&str) -> Option<(TriggerCond, Sel, PlayerRef)>,
}
inventory::collect!(TriggerPattern);

/// Parses a whole static ability line (lowercase, no trailing period; original text given).
pub struct StaticPattern {
    pub name: &'static str,
    pub priority: i32,
    pub parse: fn(&str, &str, &CompileContext) -> Option<Vec<Ability>>,
}
inventory::collect!(StaticPattern);

/// Parses a condition ("you control an artifact").
pub struct ConditionPattern {
    pub name: &'static str,
    pub priority: i32,
    pub parse: fn(&str) -> Option<Condition>,
}
inventory::collect!(ConditionPattern);

/// A sentence that modifies the effect of the preceding sentence instead of adding an
/// effect of its own ("Destroy target creature. It can't be regenerated."). Receives the
/// lowercase sentence (no trailing period) and the effect parsed so far for the previous
/// sentence; returns true if it understood the sentence and updated that effect.
pub struct FollowupPattern {
    pub name: &'static str,
    pub priority: i32,
    pub apply: fn(&str, &mut Effect, &mut Builder) -> bool,
}
inventory::collect!(FollowupPattern);

/// Parses an entire ability block that the standard classifier can't handle (e.g. level
/// up bars, class levels, saga chapters, "Choose a Background"). Tried before the standard
/// classifier. Receives the original (normalized, not lowercased) block.
pub struct AbilityPattern {
    pub name: &'static str,
    pub priority: i32,
    pub parse: fn(&str, &CompileContext) -> Option<Vec<Ability>>,
}
inventory::collect!(AbilityPattern);

/// How many modes a modal header lets a player choose, and who chooses them (CR 700.2).
pub struct ModalHeader {
    pub min: Value,
    pub max: Value,
    pub allow_repeat: bool,
    pub chooser: ModeChooser,
}

/// Parses the header line of a modal spell or ability that precedes its bulleted modes
/// ("choose one or more", "choose one at random"): lowercase, without the trailing dash,
/// colon or period. The core parser handles the plain "choose one/two/one or both" forms.
pub struct ModalHeaderPattern {
    pub name: &'static str,
    pub priority: i32,
    pub parse: fn(&str, &CompileContext) -> Option<ModalHeader>,
}
inventory::collect!(ModalHeaderPattern);

pub fn modal_header_patterns() -> &'static [&'static ModalHeaderPattern] {
    static P: OnceLock<Vec<&'static ModalHeaderPattern>> = OnceLock::new();
    P.get_or_init(|| {
        sorted(inventory::iter::<ModalHeaderPattern>.into_iter(), |p| {
            (p.priority, p.name)
        })
    })
}

/// Parses one part of a cost (lowercase, e.g. "blight 1", "waterbend {2}") that the core
/// cost parser doesn't understand.
pub struct CostPattern {
    pub name: &'static str,
    pub priority: i32,
    pub parse: fn(&str) -> Option<CostPart>,
}
inventory::collect!(CostPattern);

pub fn cost_patterns() -> &'static [&'static CostPattern] {
    static P: OnceLock<Vec<&'static CostPattern>> = OnceLock::new();
    P.get_or_init(|| {
        sorted(inventory::iter::<CostPattern>.into_iter(), |p| {
            (p.priority, p.name)
        })
    })
}

/// Regroups the text's ability blocks before they're parsed, for abilities printed over
/// several lines (e.g. a leveler's level symbol, P/T box, and abilities in one striation).
/// Grouped lines are joined with newlines into one block.
pub struct BlockGroupPattern {
    pub name: &'static str,
    pub priority: i32,
    pub group: fn(Vec<String>, &CompileContext) -> Vec<String>,
}
inventory::collect!(BlockGroupPattern);

pub fn block_group_patterns() -> &'static [&'static BlockGroupPattern] {
    static P: OnceLock<Vec<&'static BlockGroupPattern>> = OnceLock::new();
    P.get_or_init(|| {
        sorted(inventory::iter::<BlockGroupPattern>.into_iter(), |p| {
            (p.priority, p.name)
        })
    })
}

fn sorted<T: 'static>(
    it: impl Iterator<Item = &'static T>,
    key: impl Fn(&T) -> (i32, &'static str),
) -> Vec<&'static T> {
    let mut v: Vec<&'static T> = it.collect();
    v.sort_by_key(|x| key(x));
    v
}

pub fn effect_patterns() -> &'static [&'static EffectPattern] {
    static P: OnceLock<Vec<&'static EffectPattern>> = OnceLock::new();
    P.get_or_init(|| {
        sorted(inventory::iter::<EffectPattern>.into_iter(), |p| {
            (p.priority, p.name)
        })
    })
}
pub fn trigger_patterns() -> &'static [&'static TriggerPattern] {
    static P: OnceLock<Vec<&'static TriggerPattern>> = OnceLock::new();
    P.get_or_init(|| {
        sorted(inventory::iter::<TriggerPattern>.into_iter(), |p| {
            (p.priority, p.name)
        })
    })
}
pub fn static_patterns() -> &'static [&'static StaticPattern] {
    static P: OnceLock<Vec<&'static StaticPattern>> = OnceLock::new();
    P.get_or_init(|| {
        sorted(inventory::iter::<StaticPattern>.into_iter(), |p| {
            (p.priority, p.name)
        })
    })
}
pub fn condition_patterns() -> &'static [&'static ConditionPattern] {
    static P: OnceLock<Vec<&'static ConditionPattern>> = OnceLock::new();
    P.get_or_init(|| {
        sorted(inventory::iter::<ConditionPattern>.into_iter(), |p| {
            (p.priority, p.name)
        })
    })
}
pub fn followup_patterns() -> &'static [&'static FollowupPattern] {
    static P: OnceLock<Vec<&'static FollowupPattern>> = OnceLock::new();
    P.get_or_init(|| {
        sorted(inventory::iter::<FollowupPattern>.into_iter(), |p| {
            (p.priority, p.name)
        })
    })
}
pub fn ability_patterns() -> &'static [&'static AbilityPattern] {
    static P: OnceLock<Vec<&'static AbilityPattern>> = OnceLock::new();
    P.get_or_init(|| {
        sorted(inventory::iter::<AbilityPattern>.into_iter(), |p| {
            (p.priority, p.name)
        })
    })
}
