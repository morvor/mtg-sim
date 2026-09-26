//! "Prevent all [combat | noncombat] damage" by and to groups of objects and players
//! (CR 615.1a, 120.2b):
//!
//! One-shot effects (until end of turn), e.g.
//! - "Prevent all damage that would be dealt this turn to creatures you control."
//! - "Prevent all damage that would be dealt this turn by creatures your opponents
//!   control."
//! - "Prevent all damage that would be dealt to you this turn by attacking creatures."
//! - "Prevent all combat damage that would be dealt this turn by creatures with power 2
//!   or less." / "... by non-Elf creatures." / "... by creatures without trample."
//! - "Prevent all damage that would be dealt to you and creatures you control this turn."
//! - "Prevent all damage that black sources and red sources would deal this turn."
//! - "Prevent all combat damage that other creatures would deal this turn."
//!
//! Static abilities, e.g.
//! - "Prevent all noncombat damage that would be dealt to other creatures you control."
//! - "Prevent all damage that would be dealt to you by sources you don't control."
//! - "Prevent all damage that would be dealt to ~ by artifact sources."
//! - "Prevent all damage that ~ would deal to red creatures."
//! - "Prevent all damage that would be dealt to ~ by creatures it's blocking."
//! - "Prevent all damage that would be dealt to ~ during your turn."
//!
//! Groups ("creatures you control") aren't locked in as the effect is created: the
//! prevention applies to whatever matches when the damage would be dealt (CR 611.2c
//! concerns only effects that change characteristics or control).

use super::{EffectPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

/// Strips `p` from the start of `s` if a word boundary follows.
fn word<'a>(s: &'a str, p: &str) -> Option<&'a str> {
    let r = s.strip_prefix(p)?;
    match r.chars().next() {
        None | Some(' ') => Some(r.trim_start()),
        _ => None,
    }
}

#[derive(Default)]
pub(super) struct Parties {
    pub(super) players: Option<PlayerFilter>,
    pub(super) objects: Option<Filter>,
}

impl Parties {
    fn add_player(&mut self, p: PlayerFilter) {
        self.players = Some(match self.players.take() {
            None => p,
            Some(x) => PlayerFilter::Or(vec![x, p]),
        });
    }
    fn add_object(&mut self, f: Filter) {
        self.objects = Some(match self.objects.take() {
            None => f,
            Some(Filter::Or(mut v)) => {
                v.push(f);
                Filter::Or(v)
            }
            Some(x) => Filter::Or(vec![x, f]),
        });
    }
}

/// "[qualities] sources [controller]": "sources you don't control", "artifact sources",
/// "non-Human sources", "black sources". Returns the filter and the rest.
fn plural_sources(s: &str) -> Option<(Filter, &str)> {
    let idx = s.find("sources")?;
    let before = s[..idx].trim();
    let after = &s[idx + "sources".len()..];
    if !(after.is_empty() || after.starts_with(' ')) {
        return None;
    }
    let mut parts = Vec::new();
    for w in before.split(' ').filter(|w| !w.is_empty()) {
        let f = match adjective(w) {
            Some(
                f @ (Filter::Color(_) | Filter::Colorless | Filter::Multicolored | Filter::Not(_)),
            ) => f,
            Some(_) => return None,
            None => match head_noun(w)? {
                f @ (Filter::Type(_) | Filter::Subtype(_)) => f,
                _ => return None,
            },
        };
        parts.push(f);
    }
    let after = after.trim_start();
    let (rel, rest) = if let Some(r) = word(after, "you control") {
        (Some(PlayerRel::You), r)
    } else if let Some(r) = word(after, "you don't control") {
        (Some(PlayerRel::NotYou), r)
    } else if let Some(r) = word(after, "your opponents control") {
        (Some(PlayerRel::Opponent), r)
    } else {
        (None, after)
    };
    if let Some(rel) = rel {
        parts.push(Filter::ControlledBy(rel));
    }
    Some((Filter::and(parts), rest))
}

/// Targets named by a one-shot clause ("by target attacking creature with flying"): their
/// slots continue from `base`. Statics can't have targets (`base` is `None`).
pub(super) struct Targets {
    pub(super) base: Option<usize>,
    pub(super) specs: Vec<(TargetSpec, String)>,
}

/// One group of damage sources or recipients: "~", "enchanted creature", "creatures you
/// control", "other creatures", "attacking creatures without flying", "artifact sources",
/// "creatures it's blocking", "creatures blocking it", or (one-shot effects) a single
/// target object, locked in as the effect is created. Returns the filter and the rest.
fn object_group<'a>(s: &'a str, t: &mut Targets) -> Option<(Filter, &'a str)> {
    let s = s.trim_start();
    if s.starts_with("target ") {
        let base = t.base?;
        let (spec, rest) = parse_target(s)?;
        if spec.min != 1
            || !matches!(spec.max, Value::Const(1))
            || !matches!(spec.what, TargetKind::Object(_))
        {
            return None;
        }
        let slot = (base + t.specs.len()) as u8;
        let text = s[..s.len() - rest.len()].trim().to_string();
        t.specs.push((spec, text));
        return Some((Filter::In(Box::new(Sel::Target(slot))), rest.trim_start()));
    }
    for (p, f) in [
        ("~", Filter::Source),
        ("enchanted creature", Filter::AttachedToSource),
        ("equipped creature", Filter::AttachedToSource),
    ] {
        if let Some(r) = word(s, p) {
            return Some((f, r));
        }
    }
    if let Some(r) = word(s, "creatures it's blocking") {
        return Some((
            Filter::and(vec![Filter::creature(), Filter::BlockedBySource]),
            r,
        ));
    }
    if let Some(r) = word(s, "creatures blocking it") {
        return Some((
            Filter::and(vec![Filter::creature(), Filter::BlockingSource]),
            r,
        ));
    }
    if let Some(x) = plural_sources(s) {
        // Only when "sources" is the head of this group (not a later group's noun).
        let head = &s[..s.find("sources")?];
        if !head.contains(" and ") && !head.contains(" by ") && !head.contains(" to ") {
            return Some(x);
        }
    }
    let (f, plural, rest) = parse_object_phrase(s)?;
    if !plural {
        return None;
    }
    Some((f, rest.trim_start()))
}

/// Recipients joined by "and": "you", "players", "you and creatures you control", "you
/// and other permanents you control", "~".
fn recipients<'a>(s: &'a str, t: &mut Targets) -> Option<(Parties, &'a str)> {
    let mut out = Parties::default();
    let mut r = s.trim_start();
    loop {
        if let Some(x) = word(r, "you") {
            out.add_player(PlayerFilter::You);
            r = x;
        } else if let Some(x) = word(r, "players") {
            out.add_player(PlayerFilter::Any);
            r = x;
        } else {
            let (f, x) = object_group(r, t)?;
            out.add_object(f);
            r = x;
        }
        match word(r, "and") {
            Some(x) => r = x,
            None => break,
        }
    }
    Some((out, r))
}

/// Sources joined by "and": "black sources and red sources", "blue creatures and black
/// creatures".
fn sources<'a>(s: &'a str, t: &mut Targets) -> Option<(Filter, &'a str)> {
    let mut alts = Vec::new();
    let mut r = s.trim_start();
    loop {
        let (f, x) = object_group(r, t)?;
        alts.push(f);
        r = x;
        match word(r, "and") {
            Some(x) => r = x,
            None => break,
        }
    }
    let f = if alts.len() == 1 {
        alts.pop()?
    } else {
        Filter::Or(alts)
    };
    Some((f, r))
}

/// Which damage a prevention clause is about.
#[derive(Clone, Copy)]
pub(super) enum Kind {
    All,
    Combat,
    Noncombat,
}

pub(super) struct Clause {
    pub(super) kind: Kind,
    pub(super) to: Option<Parties>,
    pub(super) by: Option<Filter>,
    pub(super) this_turn: bool,
    pub(super) during_your_turn: bool,
}

/// "prevent all [combat|noncombat] damage that would be dealt [this turn] [to R] [this
/// turn] [by S] [this turn] [during your turn]" / "prevent all [combat] damage [that] S
/// would deal [to R] [this turn]".
fn clause(l: &str, t: &mut Targets) -> Option<Clause> {
    damage_clause(end(l).strip_prefix("prevent all ")?, t)
}

/// "[combat|noncombat] damage that would be dealt [this turn] [to R] [by S] [this turn]"
/// or "[combat] damage [that] S would deal [to R] [this turn]" (after "prevent all" or
/// "all").
pub(super) fn damage_clause(r: &str, t: &mut Targets) -> Option<Clause> {
    let (kind, r) = if let Some(x) = r.strip_prefix("combat damage ") {
        (Kind::Combat, x)
    } else if let Some(x) = r.strip_prefix("noncombat damage ") {
        (Kind::Noncombat, x)
    } else {
        (Kind::All, r.strip_prefix("damage ")?)
    };
    let mut c = Clause {
        kind,
        to: None,
        by: None,
        this_turn: false,
        during_your_turn: false,
    };
    let mut r = if let Some(x) = r.strip_prefix("that would be dealt") {
        x
    } else {
        // "[that] S would deal ..."
        let x = r.strip_prefix("that ").unwrap_or(r);
        let (by, x) = sources(x, t)?;
        c.by = Some(by);
        word(x.trim_start(), "would deal")?
    };
    loop {
        r = r.trim_start();
        if r.is_empty() {
            break;
        }
        if let Some(x) = word(r, "this turn") {
            if c.this_turn {
                return None;
            }
            c.this_turn = true;
            r = x;
        } else if let Some(x) = word(r, "during your turn") {
            if c.during_your_turn {
                return None;
            }
            c.during_your_turn = true;
            r = x;
        } else if let Some(x) = word(r, "to") {
            if c.to.is_some() {
                return None;
            }
            let (to, x) = recipients(x, t)?;
            c.to = Some(to);
            r = x;
        } else if let Some(x) = word(r, "by") {
            if c.by.is_some() {
                return None;
            }
            let (by, x) = sources(x, t)?;
            c.by = Some(by);
            r = x;
        } else {
            return None;
        }
    }
    Some(c)
}

pub(super) fn def(c: &Clause, action: ReplacementAction) -> ReplacementDef {
    let source = c.by.clone().unwrap_or(Filter::Any);
    let (to_players, to_objects) = match &c.to {
        Some(p) => (p.players.clone(), p.objects.clone()),
        None => (Some(PlayerFilter::Any), Some(Filter::Any)),
    };
    let event = match c.kind {
        Kind::Noncombat => ReplacementEvent::NoncombatDamage {
            source,
            to_players,
            to_objects,
        },
        k => ReplacementEvent::Damage {
            source,
            to_players,
            to_objects,
            combat_only: matches!(k, Kind::Combat),
        },
    };
    ReplacementDef {
        event,
        action,
        self_replacement: false,
        optional: false,
    }
}

/// Whether a one-shot clause can be created as is: "enchanted creature" would have to be
/// locked in as the effect is created (it's the object the Aura enchants then), so it's
/// left to other patterns. ("~" is the effect's source object, which a later new object
/// isn't, CR 400.7; targets are locked by `prevention::lock_def`.)
pub(super) fn groups_only(c: &Clause) -> bool {
    fn attached(f: &Filter) -> bool {
        match f {
            Filter::AttachedToSource => true,
            Filter::And(v) | Filter::Or(v) => v.iter().any(attached),
            _ => false,
        }
    }
    !c.by.as_ref().is_some_and(attached)
        && !c
            .to
            .as_ref()
            .and_then(|t| t.objects.as_ref())
            .is_some_and(attached)
}

fn p_prevent_groups(l: &str, b: &mut Builder) -> Option<Effect> {
    let mut t = Targets {
        base: Some(b.targets.len()),
        specs: vec![],
    };
    let c = clause(l, &mut t)?;
    if !c.this_turn || c.during_your_turn || !groups_only(&c) {
        return None;
    }
    // Without any group, "prevent all combat damage that would be dealt this turn" is the
    // core Fog pattern.
    if c.to.is_none() && c.by.is_none() {
        return None;
    }
    for (spec, text) in t.specs {
        b.add_target(spec, &text);
    }
    Some(Effect::AddReplacement {
        def: def(&c, ReplacementAction::Prevent),
        duration: Duration::EndOfTurn,
        uses: None,
    })
}

inventory::submit! { EffectPattern { name: "replacements: prevent all damage to/by groups this turn", priority: 70, parse: p_prevent_groups } }

fn s_prevent_groups(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l);
    let (lead, l) = match l.strip_prefix("during your turn, ") {
        Some(x) => (true, x),
        None => (false, l),
    };
    let mut t = Targets {
        base: None,
        specs: vec![],
    };
    let c = clause(l, &mut t)?;
    if c.this_turn || (lead && c.during_your_turn) {
        return None;
    }
    if c.to.is_none() && c.by.is_none() {
        return None;
    }
    let mut s = StaticAbility::new(StaticEffect::Replacement(def(&c, ReplacementAction::Prevent)));
    if lead || c.during_your_turn {
        s.condition = Some(Condition::YourTurn);
    }
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "replacements: static prevent all damage to/by groups", priority: 70, parse: s_prevent_groups } }
