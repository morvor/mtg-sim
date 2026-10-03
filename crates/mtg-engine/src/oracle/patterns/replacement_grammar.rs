//! Replacement and prevention grammar for damage events (CR 614.1a, 614.9, 615): an
//! EVENT PATTERN (who deals the damage, to whom, how much, combat or not), a SCOPE (a
//! static ability, "this turn", "until your next turn", "the next time", "the next N
//! damage") and a REPLACEMENT (prevent all, part, or a shield of it; redirect it; change
//! the amount; or do something else instead).
//!
//! Event patterns:
//! - "[source] would deal [combat | noncombat] damage [to recipients] [this turn]"
//! - "[N or more | N or less] damage" (a condition on the event's amount)
//! - "[combat] damage would be dealt to [recipients] [this turn] [by source] [while ...]"
//! - "[recipient] would be dealt damage"
//!
//! Sources: "~", "enchanted creature", "target creature", "a [red] source [you control]",
//! "a source of your choice" (chosen as the effect is created, CR 609.7a), "a creature
//! of your choice with shadow", "sources of the color of your choice", "creatures you
//! control", "an instant or sorcery spell".
//!
//! Recipients: lists of "you", "~", "enchanted creature", "target ...", "any target",
//! "a creature you control", "permanents you control", "each creature", "each player",
//! joined by "or", "and", "and/or".
//!
//! Replacements: "prevent that damage [and ...]", "prevent half that damage, rounded up",
//! "prevent all but N of that damage", "you may prevent X of that damage", "it deals
//! double that damage instead", "it deals that much damage plus/minus N instead", "it
//! deals half that damage, rounded down, instead", "that source deals N damage instead",
//! "that damage is dealt to [target | that source's controller | ~] instead", "that
//! creature deals that damage to itself instead", "[instructions] instead" and "instead
//! [instructions]" (with "that many" the damage, "it" / "that creature" / "that player"
//! what it would be dealt to).
//!
//! One-shot effects lock in the objects and players they refer to as they're created
//! (CR 609.7b, 611.2c; `prevention::lock_def`).

use super::{EffectPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::patterns::oracle_hardening_referents::{is_no_player_referent, is_no_referent};
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

/// The variable holding "a source of your choice" (CR 609.7a).
const CHOSEN_SOURCE: Var = vars::USER + 6141;

/// Who or what damage would be dealt to.
#[derive(Clone, Debug, Default)]
pub(crate) struct To {
    pub players: Option<PlayerFilter>,
    pub objects: Option<Filter>,
}

impl To {
    fn add_player(&mut self, f: PlayerFilter) {
        self.players = Some(match self.players.take() {
            None => f,
            Some(PlayerFilter::Or(mut v)) => {
                v.push(f);
                PlayerFilter::Or(v)
            }
            Some(x) => PlayerFilter::Or(vec![x, f]),
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
    fn merge(&mut self, o: To) {
        if let Some(p) = o.players {
            self.add_player(p);
        }
        if let Some(f) = o.objects {
            self.add_object(f);
        }
    }
    fn anything() -> To {
        To {
            players: Some(PlayerFilter::Any),
            objects: Some(Filter::Any),
        }
    }
    /// The single object the damage would be dealt to, as "it" in the replacement's
    /// instructions: the source itself, or the object the event is about.
    fn it(&self) -> Sel {
        match (&self.players, &self.objects) {
            (None, Some(Filter::Source)) => Sel::This,
            _ => Sel::TriggerObject,
        }
    }
}

/// Strips `p` from the start of `s` if a word boundary follows.
pub(crate) fn word<'a>(s: &'a str, p: &str) -> Option<&'a str> {
    let r = s.strip_prefix(p)?;
    match r.chars().next() {
        None | Some(' ' | ',' | '.') => Some(r),
        _ => None,
    }
}

/// List separators between recipients.
const SEPS: [&str; 7] = [
    ", and/or ",
    ", or ",
    ", and ",
    " and/or ",
    " or ",
    " and ",
    ", ",
];

/// Whether the builder is for a one-shot effect (targets and pronouns are allowed).
pub(crate) type OneShot<'a, 'c> = Option<&'a mut Builder<'c>>;

/// One recipient. Returns it and the rest of the text.
fn to_item<'s>(s: &'s str, b: &mut OneShot) -> Option<(To, &'s str)> {
    let s = s.trim_start();
    let mut to = To::default();
    let fixed_players: [(&str, PlayerFilter); 9] = [
        ("you", PlayerFilter::You),
        ("each player", PlayerFilter::Any),
        ("a player", PlayerFilter::Any),
        ("players", PlayerFilter::Any),
        ("an opponent", PlayerFilter::Opponent),
        ("each opponent", PlayerFilter::Opponent),
        ("your opponents", PlayerFilter::Opponent),
        ("opponents", PlayerFilter::Opponent),
        ("each other player", PlayerFilter::NotYou),
    ];
    for (p, f) in fixed_players {
        if let Some(r) = word(s, p) {
            to.add_player(f);
            return Some((to, r));
        }
    }
    for p in ["~", "this creature", "him", "her"] {
        if let Some(r) = word(s, p) {
            to.add_object(Filter::Source);
            return Some((to, r));
        }
    }
    for p in [
        "enchanted creature",
        "equipped creature",
        "enchanted permanent",
        "enchanted land",
    ] {
        if let Some(r) = word(s, p) {
            // A one-shot effect means the object it's attached to as the effect is
            // created.
            to.add_object(if b.is_some() {
                Filter::In(Box::new(Sel::AttachedTo))
            } else {
                Filter::AttachedToSource
            });
            return Some((to, r));
        }
    }
    if let Some(bb) = b.as_deref_mut() {
        for p in ["that creature", "that permanent", "it"] {
            if let Some(r) = word(s, p) {
                if is_no_referent(&bb.it) {
                    return None;
                }
                to.add_object(Filter::In(Box::new(bb.it.clone())));
                return Some((to, r));
            }
        }
        if let Some(r) = word(s, "that player") {
            if matches!(bb.it_player, PlayerRef::You) || is_no_player_referent(&bb.it_player) {
                return None;
            }
            to.add_player(PlayerFilter::Ref(Box::new(bb.it_player.clone())));
            return Some((to, r));
        }
        if s.starts_with("target ")
            || s.starts_with("any target")
            || s.starts_with("another target ")
            || s.starts_with("any other target")
        {
            let (spec, r) = parse_any_target(s)?;
            if !matches!(spec.max, Value::Const(1)) {
                return None;
            }
            let what = spec.what.clone();
            let text = s[..s.len() - r.len()].trim().to_string();
            let k = bb.add_target(spec, &text);
            let obj = Filter::In(Box::new(Sel::Target(k)));
            let pl = PlayerFilter::Ref(Box::new(PlayerRef::Target(k)));
            match what {
                TargetKind::Object(_) => to.add_object(obj),
                TargetKind::Player(_) => to.add_player(pl),
                TargetKind::AnyTarget | TargetKind::ObjectOrPlayer(..) => {
                    to.add_object(obj);
                    to.add_player(pl);
                }
                _ => return None,
            }
            return Some((to, r));
        }
    }
    // Object phrases: "a creature you control", "permanents you control", "each
    // creature", "another Dinosaur you control".
    let (art, r) = if let Some(r) = word(s, "a").or_else(|| word(s, "an")) {
        (Some(false), r)
    } else if let Some(r) = word(s, "each") {
        (Some(false), r)
    } else {
        (None, s)
    };
    if r.trim_start().starts_with("target") || r.trim_start().starts_with("source") {
        return None;
    }
    let (f, plural, rest) = parse_object_phrase(r)?;
    if art.is_some() == plural {
        return None;
    }
    to.add_object(f);
    Some((to, rest))
}

/// A list of recipients. Returns it and the rest of the text.
pub(crate) fn recipients<'s>(s: &'s str, b: &mut OneShot) -> Option<(To, &'s str)> {
    let t = s.trim_start();
    // Whole phrases.
    for (p, players, objects) in [
        ("a permanent or player", PlayerFilter::Any, Filter::Any),
        ("a player or permanent", PlayerFilter::Any, Filter::Any),
        ("any permanent or player", PlayerFilter::Any, Filter::Any),
        (
            "the chosen player or a permanent they control",
            PlayerFilter::Ref(Box::new(PlayerRef::ChosenOpponent)),
            Filter::ControlledBy(PlayerRel::Chosen),
        ),
        (
            "the chosen player or a permanent that player controls",
            PlayerFilter::Ref(Box::new(PlayerRef::ChosenOpponent)),
            Filter::ControlledBy(PlayerRel::Chosen),
        ),
    ] {
        if let Some(r) = word(t, p) {
            return Some((
                To {
                    players: Some(players),
                    objects: Some(objects),
                },
                r,
            ));
        }
    }
    let (mut to, mut rest) = to_item(s, b)?;
    'outer: loop {
        for sep in SEPS {
            if let Some(r) = rest.strip_prefix(sep) {
                let saved = b.as_ref().map(|x| x.targets.len());
                if let Some((t, r2)) = to_item(r, b) {
                    to.merge(t);
                    rest = r2;
                    continue 'outer;
                }
                // "a creature, battle, or opponent": later items share the article.
                let (w, _) = split_word(r);
                let w2 = w.trim_end_matches(',');
                let item = match w2 {
                    "player" => Some(To {
                        players: Some(PlayerFilter::Any),
                        objects: None,
                    }),
                    "opponent" => Some(To {
                        players: Some(PlayerFilter::Opponent),
                        objects: None,
                    }),
                    _ => match head_noun(w2) {
                        Some(f @ (Filter::Type(_) | Filter::Permanent)) => Some(To {
                            players: None,
                            objects: Some(f),
                        }),
                        _ => None,
                    },
                };
                if let Some(t) = item {
                    to.merge(t);
                    rest = &r[w2.len()..];
                    continue 'outer;
                }
                if let (Some(bb), Some(n)) = (b.as_deref_mut(), saved) {
                    bb.targets.truncate(n);
                }
            }
        }
        break;
    }
    Some((to, rest))
}

/// The source of the damage: a filter, and the choices made as the effect is created
/// ("a source of your choice", "the color of your choice").
#[derive(Clone, Debug)]
pub(crate) struct Src {
    pub filter: Filter,
    pub pre: Vec<Effect>,
}

/// Colors and card types qualifying "source(s)".
fn source_qualities(s: &str) -> Option<Option<Filter>> {
    let s = s.trim();
    if s.is_empty() {
        return Some(None);
    }
    let mut alts = Vec::new();
    for alt in s.split(" and/or ").flat_map(|x| x.split(" or ")) {
        let mut all = Vec::new();
        for w in alt.split(' ').filter(|w| !w.is_empty()) {
            let f = match adjective(w) {
                Some(f @ (Filter::Color(_) | Filter::Colorless | Filter::Multicolored)) => f,
                Some(f @ Filter::Not(_)) => f,
                _ => match head_noun(w)? {
                    f @ (Filter::Type(_) | Filter::Subtype(_)) => f,
                    _ => return None,
                },
            };
            all.push(f);
        }
        alts.push(Filter::and(all));
    }
    Some(Some(if alts.len() == 1 {
        alts.pop()?
    } else {
        Filter::Or(alts)
    }))
}

/// "sources [qualities] [you control | of the chosen color | ...]" (plural) or "a
/// [qualities] source [you control]" (singular). The whole text must be understood.
fn sources_noun(s: &str, pre: &mut Vec<Effect>) -> Option<Filter> {
    let s = s.trim();
    let (plural, r) = if let Some(r) = word(s, "a").or_else(|| word(s, "an")) {
        (false, r.trim_start())
    } else if let Some(r) = word(s, "any") {
        (false, r.trim_start())
    } else {
        (true, s)
    };
    let noun = if plural { "sources" } else { "source" };
    let idx = r.find(noun)?;
    let before = &r[..idx];
    let after = &r[idx + noun.len()..];
    if !(after.is_empty() || after.starts_with(' ')) {
        return None;
    }
    let mut parts = Vec::new();
    let (other, before) = match before.trim().strip_prefix("other") {
        Some(x) if x.is_empty() || x.starts_with(' ') => (true, x),
        _ => (false, before),
    };
    if other {
        parts.push(Filter::Other);
    }
    if let Some(q) = source_qualities(before)? {
        parts.push(q);
    }
    match after.trim() {
        "" => {}
        "you control" => parts.push(Filter::ControlledBy(PlayerRel::You)),
        "you don't control" => parts.push(Filter::not(Filter::ControlledBy(PlayerRel::You))),
        "an opponent controls" | "your opponents control" => {
            parts.push(Filter::ControlledBy(PlayerRel::Opponent))
        }
        "of the chosen color" => parts.push(Filter::ChosenColor),
        "with the chosen name" => parts.push(Filter::ChosenName),
        "of the color of your choice" => {
            pre.push(Effect::Choose {
                who: PlayerRef::You,
                kind: ChoiceKind::Color,
            });
            parts.push(Filter::ChosenColor);
        }
        _ => return None,
    }
    Some(Filter::and(parts))
}

/// The source of damage. The whole text must be understood.
pub(crate) fn source(s: &str, b: &mut OneShot) -> Option<Src> {
    let s = s.trim();
    let mut pre = Vec::new();
    match s {
        "~" | "this creature" => {
            return Some(Src {
                filter: Filter::Source,
                pre,
            })
        }
        "enchanted creature" | "equipped creature" => {
            return Some(Src {
                filter: if b.is_some() {
                    Filter::In(Box::new(Sel::AttachedTo))
                } else {
                    Filter::AttachedToSource
                },
                pre,
            })
        }
        _ => {}
    }
    if let Some(bb) = b.as_deref_mut() {
        if matches!(s, "it" | "that creature") && !is_no_referent(&bb.it) {
            return Some(Src {
                filter: Filter::In(Box::new(bb.it.clone())),
                pre,
            });
        }
        if s.starts_with("target ") || s.starts_with("up to one target ") {
            let s2 = s.strip_prefix("up to one ").unwrap_or(s);
            let (spec, r) = parse_target(s)?;
            let _ = s2;
            if !r.trim().is_empty()
                || !matches!(spec.what, TargetKind::Object(_) | TargetKind::Spell(_))
                || !matches!(spec.max, Value::Const(1))
            {
                return None;
            }
            let k = bb.add_target(spec, s);
            return Some(Src {
                filter: Filter::In(Box::new(Sel::Target(k))),
                pre,
            });
        }
    }
    // "a [red] source of your choice", "a creature of your choice with shadow" (CR 609.7a).
    if let Some(i) = s.find(" of your choice") {
        b.as_ref()?;
        let rest = format!("{}{}", &s[..i], &s[i + " of your choice".len()..]);
        let f = sources_noun(&rest, &mut pre).or_else(|| {
            let r = rest
                .strip_prefix("a ")
                .or_else(|| rest.strip_prefix("an "))?;
            let (f, plural, tail) = parse_object_phrase(r)?;
            (!plural && tail.trim().is_empty()).then_some(f)
        })?;
        pre.push(Effect::ChooseSource {
            who: PlayerRef::You,
            filter: f.clone(),
            var: CHOSEN_SOURCE,
        });
        return Some(Src {
            filter: Filter::and(vec![Filter::In(Box::new(Sel::Var(CHOSEN_SOURCE))), f]),
            pre,
        });
    }
    if let Some(f) = sources_noun(s, &mut pre) {
        if !pre.is_empty() && b.is_none() {
            return None;
        }
        return Some(Src { filter: f, pre });
    }
    // Object phrases: "a creature you control", "creatures you control", "an instant or
    // sorcery spell", "a Zombie you control".
    let (art, r) = match word(s, "a").or_else(|| word(s, "an")) {
        Some(r) => (true, r),
        None => match word(s, "another") {
            Some(_) => (true, s),
            None => (false, s),
        },
    };
    let (f, plural, tail) = parse_object_phrase(r)?;
    if plural == art || !tail.trim().is_empty() {
        return None;
    }
    Some(Src { filter: f, pre })
}

/// "N or more damage", "N or less damage", "an amount of damage less than X": a condition
/// on the event's amount. Returns the condition and the rest after "damage".
fn amount_condition(s: &str) -> Option<(Condition, &str)> {
    let (n, r) = parse_number(s)?;
    if !matches!(n, Value::Const(_)) {
        return None;
    }
    let (cmp, r) = if let Some(r) = r.strip_prefix("or more ") {
        (Cmp::Ge, r)
    } else if let Some(r) = r.strip_prefix("or less ") {
        (Cmp::Le, r)
    } else if let Some(r) = r.strip_prefix("or greater ") {
        (Cmp::Ge, r)
    } else {
        return None;
    };
    Some((Condition::Compare(Value::EventAmount, cmp, n), r))
}

/// A parsed damage event pattern.
#[derive(Clone, Debug)]
pub(crate) struct DamageEvent {
    pub src: Src,
    pub to: To,
    /// `Some(true)`: combat damage only; `Some(false)`: noncombat damage only.
    pub combat: Option<bool>,
    pub cond: Option<Condition>,
    /// "this turn" was said.
    pub this_turn: bool,
    /// "while [condition]" (a static ability's condition).
    pub while_cond: Option<Condition>,
}

impl DamageEvent {
    pub fn event(&self) -> ReplacementEvent {
        let e = match self.combat {
            Some(false) => ReplacementEvent::NoncombatDamage {
                source: self.src.filter.clone(),
                to_players: self.to.players.clone(),
                to_objects: self.to.objects.clone(),
            },
            c => ReplacementEvent::Damage {
                source: self.src.filter.clone(),
                to_players: self.to.players.clone(),
                to_objects: self.to.objects.clone(),
                combat_only: c == Some(true),
            },
        };
        match &self.cond {
            Some(c) => ReplacementEvent::Where {
                event: Box::new(e),
                cond: c.clone(),
            },
            None => e,
        }
    }
}

/// "combat damage" / "noncombat damage" / "damage" at the start.
fn damage_kind(s: &str) -> Option<(Option<bool>, &str)> {
    let s = s.trim_start();
    if let Some(r) = word(s, "combat damage") {
        return Some((Some(true), r));
    }
    if let Some(r) = word(s, "noncombat damage") {
        return Some((Some(false), r));
    }
    word(s, "damage").map(|r| (None, r))
}

/// Strips " this turn" (or " this combat") from the start or end of `s`.
fn this_turn(s: &str) -> (bool, &str) {
    for p in [" this turn", " this combat"] {
        if let Some(r) = s.strip_suffix(p) {
            return (true, r);
        }
        if let Some(r) = s.strip_prefix(p) {
            return (true, r);
        }
    }
    (false, s)
}

/// "while [condition]" at the end of an event: a condition on the game state.
fn while_suffix<'s>(s: &'s str, ctx: &CompileContext) -> Option<(&'s str, Option<Condition>)> {
    match s.find(" while ") {
        Some(i) => {
            let c = &s[i + " while ".len()..];
            // "while it has a +1/+1 counter on it": "it" is the source.
            let c = match c.strip_prefix("it ") {
                Some(r) => format!("~ {r}"),
                None => c.to_string(),
            };
            let cond = crate::oracle::statics::parse_condition(&c, ctx)?;
            Some((&s[..i], Some(cond)))
        }
        None => Some((s, None)),
    }
}

/// The event of "if [event], ..." for damage (without "if").
pub(crate) fn damage_event(
    s: &str,
    b: &mut OneShot,
    ctx: &CompileContext,
) -> Option<DamageEvent> {
    let s = s.trim();
    let (s, while_cond) = while_suffix(s, ctx)?;
    let saved = b.as_ref().map(|x| x.targets.len());
    let restore = |b: &mut OneShot| {
        if let (Some(bb), Some(n)) = (b.as_deref_mut(), saved) {
            bb.targets.truncate(n);
        }
    };
    // "[recipient] would be dealt [combat] damage"
    if let Some((r, (combat, tail))) = s
        .split_once(" would be dealt ")
        .and_then(|(r, kind)| Some((r, damage_kind(kind)?)))
    {
        let (tt, tail) = this_turn(tail);
        if tail.trim().is_empty() {
            if let Some((to, rest)) = recipients(r, b) {
                if rest.trim().is_empty() {
                    return Some(DamageEvent {
                        src: Src {
                            filter: Filter::Any,
                            pre: vec![],
                        },
                        to,
                        combat,
                        cond: None,
                        this_turn: tt,
                        while_cond,
                    });
                }
            }
            restore(b);
        }
    }
    // "[combat] damage would be dealt [this turn] to [recipients] [this turn] [by source]"
    if let Some((kind, r)) = s.split_once(" would be dealt ") {
        let (combat, tail) = damage_kind(kind)?;
        if !tail.trim().is_empty() {
            return None;
        }
        let r = r.replace("to any creature", "to each creature");
        let r = r.as_str();
        let (tt1, r) = this_turn(&format!(" {r}")).to_owned_pair();
        let r = r.trim_start().strip_prefix("to ")?.to_string();
        let (to, rest) = recipients(&r, b)?;
        let (tt2, rest) = this_turn(rest);
        let rest = rest.trim();
        let src = if rest.is_empty() {
            Src {
                filter: Filter::Any,
                pre: vec![],
            }
        } else {
            match rest.strip_prefix("by ").and_then(|x| source(x, b)) {
                Some(s) => s,
                None => {
                    restore(b);
                    return None;
                }
            }
        };
        return Some(DamageEvent {
            src,
            to,
            combat,
            cond: None,
            this_turn: tt1 || tt2,
            while_cond,
        });
    }
    // "[source] would deal [N or more] [combat] damage [to recipients] [this turn]"
    let (subj, r) = s.split_once(" would deal ")?;
    let (cond, r) = match amount_condition(r) {
        Some((c, r)) => (Some(c), r),
        None => (None, r),
    };
    let (combat, r) = damage_kind(r)?;
    let (tt1, r) = this_turn(r);
    let r = r.trim();
    let (to, tt2) = if r.is_empty() {
        (To::anything(), false)
    } else {
        let x = r.strip_prefix("to ")?;
        let (to, rest) = recipients(x, b)?;
        let (tt2, rest) = this_turn(rest);
        if !rest.trim().is_empty() {
            restore(b);
            return None;
        }
        (to, tt2)
    };
    let Some(src) = source(subj, b) else {
        restore(b);
        return None;
    };
    Some(DamageEvent {
        src,
        to,
        combat,
        cond,
        this_turn: tt1 || tt2,
        while_cond,
    })
}

trait OwnedPair {
    fn to_owned_pair(self) -> (bool, String);
}
impl OwnedPair for (bool, &str) {
    fn to_owned_pair(self) -> (bool, String) {
        (self.0, self.1.to_string())
    }
}

/// Instructions performed instead of (or after preventing) the damage: "it" is what the
/// damage would be dealt to, "that player" the player it would be dealt to, "that many"
/// the damage. No targets.
pub(crate) fn instructions(text: &str, ctx: &CompileContext, it: Sel) -> Option<Effect> {
    // "that many" / "that much" is the event's amount (as in a trigger, see
    // `triggers_referents`): parsed as X, then X is the event amount.
    let has_x = text.split(|c: char| !c.is_alphanumeric()).any(|w| w == "x");
    let t = text
        .replace("that many", "x")
        .replace("that much", "x")
        .replace("the damage prevented this way", "x");
    let uses_amount = t != text;
    if has_x && uses_amount {
        return None;
    }
    let mut b = Builder::new(ctx);
    b.in_trigger = true;
    b.it = it;
    b.it_player = PlayerRef::TriggerPlayer;
    // "that source's controller": the source of the damage.
    for p in ["that source", "the source"] {
        b.named.push((p.into(), Sel::TriggerOtherObject));
    }
    let effect = crate::oracle::effects::parse_effect_text(&format!("{t}."), &mut b)?;
    if !b.targets.is_empty() {
        return None;
    }
    if !uses_amount {
        return Some(effect);
    }
    x_to_event_amount(&effect)
}

/// Replaces X with the event's amount.
pub(crate) fn x_to_event_amount(e: &Effect) -> Option<Effect> {
    use serde_json::Value as J;
    fn walk(v: J) -> J {
        match v {
            J::String(s) if s == "X" => J::String("EventAmount".into()),
            J::Object(m) => J::Object(m.into_iter().map(|(k, v)| (k, walk(v))).collect()),
            J::Array(a) => J::Array(a.into_iter().map(walk).collect()),
            other => other,
        }
    }
    let json = serde_json::to_value(e).ok()?;
    serde_json::from_value(walk(json)).ok()
}

/// "If damage is prevented this way, [instructions].", "If damage from a red source is
/// prevented this way, ...", "You gain life equal to the damage prevented this way.",
/// "Exile cards from the top of your library equal to the damage prevented this way.":
/// the rest of a prevention effect, performed right after the damage is prevented
/// (CR 615.5), with "that much" / "the damage prevented this way" the damage prevented.
pub(crate) fn prevented_followup(l: &str, ctx: &CompileContext, it: Sel) -> Option<Effect> {
    let l = end(l.trim());
    if let Some(r) = l.strip_prefix("if damage is prevented this way, ") {
        return instructions(r, ctx, it);
    }
    if let Some(r) = l.strip_prefix("if damage from a ") {
        let (q, r) = r.split_once(" source is prevented this way, ")?;
        let f = source_qualities(q)??;
        let e = instructions(r, ctx, it)?;
        return Some(Effect::If {
            cond: Condition::SelMatches(Sel::TriggerOtherObject, f),
            then: Box::new(e),
            otherwise: Box::new(Effect::Noop),
        });
    }
    if !l.contains("prevented this way") {
        return None;
    }
    instructions(l, ctx, it)
}

/// What the restated recipient at the end of an amount change may be.
fn restated(s: &str) -> bool {
    matches!(
        s.trim(),
        "" | "to that permanent or player"
            | "to that player or permanent"
            | "to that player"
            | "to that permanent"
            | "to that creature"
            | "to you"
            | "to ~"
            | "to it"
            | "to that player or battle"
    )
}

/// "half that damage, rounded down" etc.
fn half(s: &str) -> Option<(Value, &str)> {
    let r = s.strip_prefix("half that damage, rounded ")?;
    let (up, r) = if let Some(r) = r.strip_prefix("up") {
        (true, r)
    } else {
        (false, r.strip_prefix("down")?)
    };
    let r = r.strip_prefix(',').unwrap_or(r);
    Some((Value::Div(Box::new(Value::EventAmount), 2, up), r))
}

/// The amount after "that much damage plus " / "plus X, where X is ...".
fn amount_tail(s: &str, ctx: &CompileContext) -> Option<(Value, String)> {
    let s = s.trim();
    // "plus an amount of damage equal to the number of fire counters on ~"
    if let Some(r) = s.strip_prefix("an amount of damage equal to ") {
        let mut tmp = Builder::new(ctx);
        let (v, rest) = crate::oracle::patterns::value_grammar::parse_value(r, &mut tmp)?;
        if !tmp.targets.is_empty() {
            return None;
        }
        return Some((v, rest));
    }
    let (n, r) = parse_number(s)?;
    Some((n, r.to_string()))
}

/// The replacement for a damage event (see the module docs).
pub(crate) fn damage_action(
    s: &str,
    ev: &DamageEvent,
    b: &mut OneShot,
    ctx: &CompileContext,
) -> Option<(ReplacementAction, bool)> {
    let s = end(s.trim());
    let it = ev.to.it();
    // "you may prevent X of that damage, where X is ..."
    if let Some(r) = s.strip_prefix("you may ") {
        let (a, _) = damage_action(r, ev, b, ctx)?;
        return Some((a, true));
    }
    if s == "prevent that damage" {
        return Some((ReplacementAction::Prevent, false));
    }
    if let Some(r) = s
        .strip_prefix("prevent that damage and ")
        .or_else(|| s.strip_prefix("prevent that damage, "))
    {
        let e = instructions(r, ctx, it)?;
        return Some((ReplacementAction::PreventAndThen(None, Box::new(e)), false));
    }
    if let Some(r) = s.strip_prefix("prevent ") {
        if let Some((v, r)) = half(r) {
            return r.trim().is_empty().then_some((ReplacementAction::PreventPortion(v), false));
        }
        if let Some(r) = r.strip_prefix("all but ") {
            let (n, r) = parse_number(r)?;
            if r.trim() != "of that damage" {
                return None;
            }
            let v = Value::Max(
                Box::new(Value::c(0)),
                Box::new(Value::Diff(Box::new(Value::EventAmount), Box::new(n))),
            );
            return Some((ReplacementAction::PreventPortion(v), false));
        }
        // "prevent X of that damage, where X is the number of Clerics you control"
        if let Some(r) = r.strip_prefix("x of that damage, where x is ") {
            let mut tmp = Builder::new(ctx);
            let (v, rest) = crate::oracle::patterns::value_grammar::parse_value(r, &mut tmp)?;
            if !rest.trim().is_empty() {
                return None;
            }
            return Some((ReplacementAction::PreventPortion(v), false));
        }
        return None;
    }
    // Redirection (CR 614.9).
    if let Some(r) = s
        .strip_prefix("that damage is dealt to ")
        .and_then(|r| r.strip_suffix(" instead"))
    {
        let sel = redirect_to(r, b)?;
        return Some((ReplacementAction::Redirect(sel), false));
    }
    if let Some(r) = s.strip_prefix("you may have that damage dealt to ") {
        let sel = redirect_to(r.strip_suffix(" instead")?, b)?;
        return Some((ReplacementAction::Redirect(sel), true));
    }
    for p in [
        "it deals that damage to its controller instead",
        "that spell deals that damage to its controller instead",
        "that source deals that damage to its controller instead",
    ] {
        if s == p {
            return Some((
                ReplacementAction::Redirect(Sel::Players(PlayerRef::ControllerOf(Box::new(
                    Sel::TriggerOtherObject,
                )))),
                false,
            ));
        }
    }
    if matches!(
        s,
        "that creature deals that damage to itself instead"
            | "it deals that damage to itself instead"
    ) {
        return Some((ReplacementAction::Redirect(Sel::TriggerOtherObject), false));
    }
    // Amount changes: "it deals double that damage [to ...] instead", "instead it deals
    // that much damage plus N".
    let amount = s
        .strip_prefix("instead it deals ")
        .or_else(|| s.strip_prefix("instead that source deals "))
        .map(|r| (r, false))
        .or_else(|| {
            s.strip_prefix("it deals ")
                .or_else(|| s.strip_prefix("that source deals "))
                .and_then(|r| r.strip_suffix(" instead"))
                .map(|r| (r, true))
        });
    if let Some((r, _)) = amount {
        if let Some(tail) = r
            .strip_prefix("double that damage")
            .or_else(|| r.strip_prefix("twice that much damage"))
        {
            return restated(tail).then_some((ReplacementAction::Multiply(2), false));
        }
        if let Some(tail) = r.strip_prefix("triple that damage") {
            return restated(tail).then_some((ReplacementAction::Multiply(3), false));
        }
        if let Some((v, tail)) = half(r) {
            // Deals half: the rest is subtracted.
            if !restated(tail) {
                return None;
            }
            let rest = Value::Diff(Box::new(Value::EventAmount), Box::new(v));
            return Some((ReplacementAction::Subtract(rest), false));
        }
        if let Some(x) = r.strip_prefix("that much damage plus ") {
            // "plus x, where x is ~'s power"
            if let Some(w) = x.strip_prefix("x, where x is ") {
                let mut tmp = Builder::new(ctx);
                let (v, rest) =
                    crate::oracle::patterns::value_grammar::parse_value(w, &mut tmp)?;
                return restated(&rest).then_some((ReplacementAction::Add(v), false));
            }
            let (v, tail) = amount_tail(x, ctx)?;
            return restated(&tail).then_some((ReplacementAction::Add(v), false));
        }
        if let Some(x) = r.strip_prefix("that much damage minus ") {
            let (v, tail) = amount_tail(x, ctx)?;
            return restated(&tail).then_some((ReplacementAction::Subtract(v), false));
        }
        // "that source deals 3 damage to that permanent or player instead": the amount
        // becomes N (only with "N or more damage" in the event).
        if let Some((n, tail)) = parse_number(r) {
            let tail = tail.strip_prefix("damage")?;
            if matches!(n, Value::Const(_))
                && restated(tail)
                && matches!(ev.cond, Some(Condition::Compare(_, Cmp::Ge, _)))
            {
                let v = Value::Diff(Box::new(Value::EventAmount), Box::new(n));
                return Some((ReplacementAction::Subtract(v), false));
            }
        }
        return None;
    }
    // "put that many -1/-1 counters on that creature instead", "instead that player mills
    // that many cards".
    let inner = s
        .strip_prefix("instead ")
        .or_else(|| s.strip_suffix(" instead"))?;
    let e = instructions(inner, ctx, it)?;
    Some((ReplacementAction::Instead(Box::new(e)), false))
}

/// The new recipient of redirected damage: "~", "you", "target creature", "any target",
/// "that source's controller", "the source's controller".
fn redirect_to(s: &str, b: &mut OneShot) -> Option<Sel> {
    let s = s.trim();
    match s {
        "~" | "this creature" => return Some(Sel::This),
        "you" => return Some(Sel::Players(PlayerRef::You)),
        "that source's controller" | "the source's controller" | "that spell's controller" => {
            return Some(Sel::Players(PlayerRef::ControllerOf(Box::new(
                Sel::TriggerOtherObject,
            ))))
        }
        _ => {}
    }
    let bb = b.as_deref_mut()?;
    let (spec, r) = parse_any_target(s)?;
    if !r.trim().is_empty() || spec.fixed_min() != Some(1) || !matches!(spec.max, Value::Const(1))
    {
        return None;
    }
    let k = bb.add_target(spec, s);
    Some(Sel::Target(k))
}

/// Splits "if [event], [action]" at the comma that ends the event, trying each.
fn split_if<'s>(s: &'s str) -> Vec<(&'s str, &'s str)> {
    s.match_indices(", ")
        .map(|(i, _)| (&s[..i], &s[i + 2..]))
        .collect()
}

/// "If [damage event], [replacement]." as a static ability.
fn s_if_damage(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let st = static_if_damage(l, ctx)?;
    Some(vec![AbilityDef::new(AbilityKind::Static(st), text)])
}

fn static_if_damage(l: &str, ctx: &CompileContext) -> Option<StaticAbility> {
    let l = end(l.trim());
    // "If damage would be dealt to ~, prevent that damage. [The rest of the effect.]"
    if let Some((first, second)) = l.split_once(". ") {
        let mut st = static_if_damage(first, ctx)?;
        let StaticEffect::Replacement(def) = &mut st.effect else {
            return None;
        };
        if !matches!(def.action, ReplacementAction::Prevent) {
            return None;
        }
        let it = match &def.event {
            ReplacementEvent::Damage {
                to_players: None,
                to_objects: Some(Filter::Source),
                ..
            } => Sel::This,
            _ => Sel::TriggerObject,
        };
        let e = prevented_followup(second, ctx, it)?;
        def.action = ReplacementAction::PreventAndThen(None, Box::new(e));
        return Some(st);
    }
    let r = l.strip_prefix("if ")?;
    for (ev, act) in split_if(r) {
        let mut none: OneShot = None;
        let Some(dev) = damage_event(ev, &mut none, ctx) else {
            continue;
        };
        if dev.this_turn || !dev.src.pre.is_empty() {
            return None;
        }
        let Some((action, optional)) = damage_action(act, &dev, &mut none, ctx) else {
            continue;
        };
        let def = ReplacementDef {
            event: dev.event(),
            action,
            self_replacement: false,
            optional,
        };
        let mut st = StaticAbility::new(StaticEffect::Replacement(def));
        st.condition = dev.while_cond.clone();
        return Some(st);
    }
    None
}

inventory::submit! { StaticPattern { name: "replacement grammar: if [damage event], [replacement]", priority: 150, parse: s_if_damage } }

/// A one-shot effect's duration prefix: "until end of turn, ", "until your next turn, ",
/// "this turn, ".
pub(crate) fn duration_prefix(l: &str) -> (Option<Duration>, &str) {
    for (p, d) in [
        ("until end of turn, ", Duration::EndOfTurn),
        ("this turn, ", Duration::EndOfTurn),
        ("until your next turn, ", Duration::UntilYourNextTurn),
    ] {
        if let Some(r) = l.strip_prefix(p) {
            return (Some(d), r);
        }
    }
    (None, l)
}

/// The effect creating a replacement: the choices first, then the effect.
fn with_pre(pre: Vec<Effect>, e: Effect) -> Effect {
    if pre.is_empty() {
        e
    } else {
        let mut v = pre;
        v.push(e);
        Effect::seq(v)
    }
}

/// Runs a parse, restoring the builder's targets if it fails.
pub(crate) fn attempt<T>(b: &mut Builder, f: impl FnOnce(&mut Builder) -> Option<T>) -> Option<T> {
    let saved = b.targets.len();
    let r = f(b);
    if r.is_none() {
        b.targets.truncate(saved);
    }
    r
}

/// "[Until end of turn, | Until your next turn, ] if [damage event] [this turn],
/// [replacement]" as a one-shot effect.
fn p_if_damage(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let (dur, r) = duration_prefix(l);
    let r = r.strip_prefix("if ")?;
    let ctx = b.ctx;
    for (ev, act) in split_if(r) {
        let got = attempt(b, |b| {
            let mut ob: OneShot = Some(b);
            let dev = damage_event(ev, &mut ob, ctx)?;
            if dev.while_cond.is_some() {
                return None;
            }
            let duration = match (&dur, dev.this_turn) {
                (Some(d), false) => d.clone(),
                (None, true) => Duration::EndOfTurn,
                _ => return None,
            };
            let (action, optional) = damage_action(act, &dev, &mut ob, ctx)?;
            Some(with_pre(
                dev.src.pre.clone(),
                Effect::AddReplacement {
                    def: ReplacementDef {
                        event: dev.event(),
                        action,
                        self_replacement: false,
                        optional,
                    },
                    duration,
                    uses: None,
                },
            ))
        });
        if got.is_some() {
            return got;
        }
    }
    None
}

inventory::submit! { EffectPattern { name: "replacement grammar: [this turn] if [damage event], [replacement]", priority: 150, parse: p_if_damage } }

/// "The next time [damage event] this turn, [replacement]" (one application, CR 615.7)
/// and "Each time [damage event] this turn, [replacement]".
fn p_next_time(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let (uses, r) = if let Some(r) = l.strip_prefix("the next time ") {
        (Some(1), r)
    } else {
        (None, l.strip_prefix("each time ")?)
    };
    let ctx = b.ctx;
    for (ev, act) in split_if(r) {
        let got = attempt(b, |b| {
            let mut ob: OneShot = Some(b);
            let dev = damage_event(ev, &mut ob, ctx)?;
            if !dev.this_turn || dev.while_cond.is_some() {
                return None;
            }
            let (action, optional) = damage_action(act, &dev, &mut ob, ctx)?;
            // A shield of a portion isn't printed with "the next time" except "prevent
            // half that damage", which applies to that one damage event.
            Some(with_pre(
                dev.src.pre.clone(),
                Effect::AddReplacement {
                    def: ReplacementDef {
                        event: dev.event(),
                        action,
                        self_replacement: false,
                        optional,
                    },
                    duration: if ev.ends_with("this combat") {
                        Duration::EndOfCombat
                    } else {
                        Duration::EndOfTurn
                    },
                    uses,
                },
            ))
        });
        if got.is_some() {
            return got;
        }
    }
    None
}

inventory::submit! { EffectPattern { name: "replacement grammar: the next time [damage event] this turn, [replacement]", priority: 150, parse: p_next_time } }

/// "[prevent] the next N [combat] damage that would be dealt [this turn] to [recipients]
/// [this turn] [by source]" and "the next N damage that [source] would deal to
/// [recipients] this turn": the damage event and the amount.
fn next_n_damage(r: &str, b: &mut Builder) -> Option<(Value, DamageEvent)> {
    let ctx = b.ctx;
    let (n, r) = parse_number(r)?;
    let r = r.trim_start();
    let (combat, r) = damage_kind(r)?;
    let r = r.trim_start();
    let kind = match combat {
        Some(true) => "combat damage",
        Some(false) => "noncombat damage",
        None => "damage",
    };
    let text = if let Some(x) = r.strip_prefix("that would be dealt by ") {
        // "that would be dealt by ~ this turn"
        let (tt, src) = this_turn(x);
        format!("{src} would deal {kind}{}", if tt { " this turn" } else { "" })
    } else if let Some(x) = r.strip_prefix("that would be dealt ") {
        format!("{kind} would be dealt {x}")
    } else {
        let x = r.strip_prefix("that ")?;
        let (subj, rest) = x.split_once(" would deal ")?;
        if combat.is_some() {
            return None;
        }
        format!("{subj} would deal damage {rest}")
    };
    let mut ob: OneShot = Some(b);
    let dev = damage_event(&text, &mut ob, ctx)?;
    Some((n, dev))
}

/// "Prevent the next N damage that would be dealt to [recipients] this turn [by ...]": a
/// prevention shield (CR 615.7) shared by all the recipients.
fn p_prevent_next(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let r = l.strip_prefix("prevent the next ")?;
    attempt(b, |b| {
        let (n, dev) = next_n_damage(r, b)?;
        if !dev.this_turn || dev.while_cond.is_some() {
            return None;
        }
        // One shield for one recipient; "each creature and each player" would need a
        // shield for each.
        let single = matches!(
            (&dev.to.players, &dev.to.objects),
            (Some(PlayerFilter::You), None)
                | (None, Some(Filter::Source))
                | (None, Some(Filter::In(_)))
                | (Some(PlayerFilter::Ref(_)), Some(Filter::In(_)))
                | (Some(PlayerFilter::Ref(_)), None)
        ) || !matches!(dev.src.filter, Filter::Any)
            || dev.to.players.as_ref().is_some_and(|p| matches!(p, PlayerFilter::Or(_)))
            || dev.to.objects.as_ref().is_some_and(|o| matches!(o, Filter::Or(_)));
        if !single && !matches!((&dev.to.players, &dev.to.objects), (Some(PlayerFilter::You), Some(_))) {
            return None;
        }
        Some(with_pre(
            dev.src.pre.clone(),
            Effect::AddReplacement {
                def: ReplacementDef {
                    event: dev.event(),
                    action: ReplacementAction::PreventAmount(n),
                    self_replacement: false,
                    optional: false,
                },
                duration: Duration::EndOfTurn,
                uses: None,
            },
        ))
    })
}

inventory::submit! { EffectPattern { name: "replacement grammar: prevent the next N damage", priority: 150, parse: p_prevent_next } }

/// "The next N damage that would be dealt to [recipients] this turn is dealt to [new
/// recipient] instead" (a redirection shield, CR 614.9).
fn p_redirect_next(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let r = l.strip_prefix("the next ")?;
    let (head, to) = r.rsplit_once(" is dealt to ")?;
    let to = to.strip_suffix(" instead")?;
    attempt(b, |b| {
        let (n, dev) = next_n_damage(head, b)?;
        if !dev.this_turn || dev.while_cond.is_some() || dev.cond.is_some() {
            return None;
        }
        let mut ob: OneShot = Some(b);
        let sel = redirect_to(to, &mut ob)?;
        Some(with_pre(
            dev.src.pre.clone(),
            Effect::AddReplacement {
                def: ReplacementDef {
                    event: dev.event(),
                    action: ReplacementAction::RedirectNext(sel, n),
                    self_replacement: false,
                    optional: false,
                },
                duration: Duration::EndOfTurn,
                uses: None,
            },
        ))
    })
}

inventory::submit! { EffectPattern { name: "replacement grammar: the next N damage is dealt to X instead", priority: 150, parse: p_redirect_next } }

/// "[Until your next turn, ] prevent all [combat] damage that would be dealt [to ...]
/// [this turn] [by ...]" / "prevent all damage [that] [source] would deal [to ...] [this
/// turn]" as a one-shot effect or (without a duration) a static ability.
fn prevent_all(
    l: &str,
    b: &mut OneShot,
    ctx: &CompileContext,
) -> Option<(DamageEvent, Option<Duration>)> {
    let (dur, r) = duration_prefix(end(l.trim()));
    let r = r.strip_prefix("prevent all ")?;
    let (combat, r) = damage_kind(r)?;
    let r = r.trim_start();
    let kind = match combat {
        Some(true) => "combat damage",
        Some(false) => "noncombat damage",
        None => "damage",
    };
    let (text, for_as_long) = {
        let (r, fal) = match r.strip_suffix(" for as long as ~ remains on the battlefield") {
            Some(x) => (x, true),
            None => (r, false),
        };
        let t = if let Some(x) = r.strip_prefix("that would be dealt ") {
            // "that would be dealt to and dealt by X" is handled elsewhere.
            if x.starts_with("to and ") {
                return None;
            }
            match x.strip_prefix("by ") {
                // "that would be dealt by [source] [this turn]"
                Some(src) => {
                    let (tt, src) = this_turn(src);
                    format!(
                        "{src} would deal {kind}{}",
                        if tt { " this turn" } else { "" }
                    )
                }
                None => format!("{kind} would be dealt {x}"),
            }
        } else {
            let x = r.strip_prefix("that ").unwrap_or(r);
            let (subj, rest) = x.split_once(" would deal")?;
            format!("{subj} would deal {kind}{rest}")
        };
        (t, fal)
    };
    let dev = damage_event(&text, b, ctx)?;
    let dur = match (dur, dev.this_turn, for_as_long) {
        (Some(d), false, false) => Some(d),
        (None, true, false) => Some(Duration::EndOfTurn),
        (None, false, true) => Some(Duration::WhileSourceOnBattlefield),
        (None, false, false) => None,
        _ => return None,
    };
    Some((dev, dur))
}

fn p_prevent_all(l: &str, b: &mut Builder) -> Option<Effect> {
    attempt(b, |b| {
        let ctx = b.ctx;
        let mut ob: OneShot = Some(b);
        let (dev, dur) = prevent_all(l, &mut ob, ctx)?;
        if dev.while_cond.is_some() {
            return None;
        }
        Some(with_pre(
            dev.src.pre.clone(),
            Effect::AddReplacement {
                def: ReplacementDef {
                    event: dev.event(),
                    action: ReplacementAction::Prevent,
                    self_replacement: false,
                    optional: false,
                },
                duration: dur?,
                uses: None,
            },
        ))
    })
}

inventory::submit! { EffectPattern { name: "replacement grammar: [until ...] prevent all damage", priority: 150, parse: p_prevent_all } }

fn s_prevent_all(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let mut none: OneShot = None;
    let (dev, dur) = prevent_all(l, &mut none, ctx)?;
    if dur.is_some() || !dev.src.pre.is_empty() {
        return None;
    }
    let def = ReplacementDef {
        event: dev.event(),
        action: ReplacementAction::Prevent,
        self_replacement: false,
        optional: false,
    };
    let mut st = StaticAbility::new(StaticEffect::Replacement(def));
    st.condition = dev.while_cond;
    Some(vec![AbilityDef::new(AbilityKind::Static(st), text)])
}

inventory::submit! { StaticPattern { name: "replacement grammar: static prevent all damage", priority: 150, parse: s_prevent_all } }

/// "All [combat] damage that would be dealt [this turn] to [recipients] [this turn] [by
/// source] is dealt to [new recipient] instead" (CR 614.9).
fn redirect_all(
    l: &str,
    b: &mut OneShot,
    ctx: &CompileContext,
) -> Option<(DamageEvent, Option<Duration>, Sel)> {
    let (dur, r) = duration_prefix(end(l.trim()));
    let r = r.strip_prefix("all ")?;
    let (head, to) = r.rsplit_once(" is dealt to ")?;
    let to = to.trim_end().strip_suffix(" instead")?;
    let (combat, h) = damage_kind(head)?;
    let h = h.trim_start();
    let kind = match combat {
        Some(true) => "combat damage",
        Some(false) => "noncombat damage",
        None => "damage",
    };
    let text = if let Some(x) = h
        .strip_prefix("that would be dealt this turn by ")
        .or_else(|| h.strip_prefix("that would be dealt by "))
    {
        let (tt, src) = this_turn(x);
        let tt = tt || h.contains("dealt this turn by");
        format!("{src} would deal {kind}{}", if tt { " this turn" } else { "" })
    } else if let Some(x) = h.strip_prefix("that would be dealt ") {
        format!("{kind} would be dealt {x}")
    } else {
        let x = h.strip_prefix("that ")?;
        let (subj, rest) = x.split_once(" would deal")?;
        format!("{subj} would deal {kind}{rest}")
    };
    let dev = damage_event(&text, b, ctx)?;
    let sel = match to.trim() {
        "the chosen creature" | "the chosen permanent" => {
            let bb = b.as_deref_mut()?;
            let (_, sel) = bb
                .named
                .iter()
                .find(|(n, _)| n == to.trim())
                .cloned()
                .or_else(|| {
                    bb.chosen_creature
                        .as_ref()
                        .map(|(k, t)| (t.clone(), Sel::Target(*k)))
                })?;
            sel
        }
        "it" | "that creature" if b.as_ref().is_some_and(|x| !is_no_referent(&x.it)) => {
            b.as_ref()?.it.clone()
        }
        t => redirect_to(t, b)?,
    };
    let dur = match (dur, dev.this_turn) {
        (Some(d), false) => Some(d),
        (None, true) => Some(Duration::EndOfTurn),
        (None, false) => None,
        _ => return None,
    };
    Some((dev, dur, sel))
}

fn p_redirect_all(l: &str, b: &mut Builder) -> Option<Effect> {
    attempt(b, |b| {
        let ctx = b.ctx;
        let mut ob: OneShot = Some(b);
        let (dev, dur, sel) = redirect_all(l, &mut ob, ctx)?;
        if dev.while_cond.is_some() {
            return None;
        }
        Some(with_pre(
            dev.src.pre.clone(),
            Effect::AddReplacement {
                def: ReplacementDef {
                    event: dev.event(),
                    action: ReplacementAction::Redirect(sel),
                    self_replacement: false,
                    optional: false,
                },
                duration: dur?,
                uses: None,
            },
        ))
    })
}

inventory::submit! { EffectPattern { name: "replacement grammar: all damage is dealt to X instead", priority: 150, parse: p_redirect_all } }

/// The last prevention replacement an effect creates.
fn last_prevention(e: &mut Effect) -> Option<&mut ReplacementDef> {
    match e {
        Effect::Seq(v) => v.iter_mut().rev().find_map(last_prevention),
        Effect::AddReplacement { def, .. }
            if matches!(
                def.event,
                ReplacementEvent::Damage { .. } | ReplacementEvent::NoncombatDamage { .. }
            ) && matches!(def.action, ReplacementAction::Prevent) =>
        {
            Some(def)
        }
        _ => None,
    }
}

/// "If damage is prevented this way, [instructions]." after a one-shot prevention effect
/// (CR 615.5). The instructions can't have targets of their own.
fn f_prevented_this_way(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let ctx = b.ctx;
    let Some(def) = last_prevention(prev) else {
        return false;
    };
    let Some(e) = prevented_followup(l, ctx, Sel::TriggerObject) else {
        return false;
    };
    def.action = ReplacementAction::PreventAndThen(None, Box::new(e));
    true
}

inventory::submit! { super::FollowupPattern { name: "replacement grammar: if damage is prevented this way, ...", priority: 150, apply: f_prevented_this_way } }

/// "Combat damage can't be prevented." / "Combat damage that would be dealt by creatures
/// you control can't be prevented." (CR 615.12).
fn s_combat_cant_prevent(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l.trim());
    let r = l.strip_suffix(" can't be prevented")?;
    let f = if r == "combat damage" {
        Filter::Any
    } else {
        let src = r.strip_prefix("combat damage that would be dealt by ")?;
        let mut none: OneShot = None;
        let s = source(src, &mut none)?;
        if !s.pre.is_empty() {
            return None;
        }
        s.filter
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Restriction(
            Restriction::CombatDamageCantBePrevented(f),
        ))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "replacement grammar: combat damage can't be prevented", priority: 150, parse: s_combat_cant_prevent } }

/// "Double all damage that [source] would deal [this turn]." (CR 701.10g).
fn double_all(l: &str, b: &mut OneShot, ctx: &CompileContext) -> Option<DamageEvent> {
    let r = end(l.trim()).strip_prefix("double all damage that ")?;
    let (subj, rest) = r.split_once(" would deal")?;
    damage_event(&format!("{subj} would deal damage{rest}"), b, ctx)
}

fn s_double_all(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let mut none: OneShot = None;
    let dev = double_all(l, &mut none, ctx)?;
    if dev.this_turn || !dev.src.pre.is_empty() {
        return None;
    }
    let def = ReplacementDef {
        event: dev.event(),
        action: ReplacementAction::Multiply(2),
        self_replacement: false,
        optional: false,
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(def))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "replacement grammar: double all damage", priority: 150, parse: s_double_all } }

fn p_double_all(l: &str, b: &mut Builder) -> Option<Effect> {
    attempt(b, |b| {
        let ctx = b.ctx;
        let mut ob: OneShot = Some(b);
        let dev = double_all(l, &mut ob, ctx)?;
        if !dev.this_turn {
            return None;
        }
        Some(with_pre(
            dev.src.pre.clone(),
            Effect::AddReplacement {
                def: ReplacementDef {
                    event: dev.event(),
                    action: ReplacementAction::Multiply(2),
                    self_replacement: false,
                    optional: false,
                },
                duration: Duration::EndOfTurn,
                uses: None,
            },
        ))
    })
}

inventory::submit! { EffectPattern { name: "replacement grammar: double all damage this turn", priority: 150, parse: p_double_all } }
