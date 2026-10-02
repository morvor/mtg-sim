//! Countering spells and abilities described with qualifiers (CR 701.6, 115.4):
//!
//! - lists of stack objects: "counter target activated ability, triggered ability, or
//!   legendary spell", "counter target instant spell, sorcery spell, or triggered ability",
//!   "counter target creature or legendary spell", "counter target triggered ability or
//!   colorless spell", "counter up to four target spells and/or abilities";
//! - an ability's source (as it last existed, CR 113.7a): "from an artifact source", "from
//!   a noncreature source", "from an artifact, creature, enchantment, or land";
//! - other qualifiers: "an opponent controls", "that targets a land you control", "with
//!   power or toughness 2 or less", "cast from a graveyard", "with {X} in its mana cost",
//!   "that's the second spell cast this turn";
//! - "unless its controller pays {2}", "unless that ability's controller pays {W}",
//!   "unless its controller pays twice {X}";
//! - non-targeted: "counter all abilities your opponents control", "counter all spells
//!   your opponents control and all abilities your opponents control", "exile all other
//!   spells and counter all abilities";
//! - the permanent whose ability was countered: "If a permanent's ability is countered this
//!   way, destroy that permanent.", "... and destroy that artifact if it's on the
//!   battlefield" (see `kw/basic_effects.rs` for the source filter).

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;

/// What a stack-object description can match.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum StackKind {
    Spell,
    Ability,
    Both,
}

fn activated() -> Filter {
    Filter::Custom(crate::game_terms::ACTIVATED_ABILITY.into())
}

fn triggered() -> Filter {
    Filter::Custom(crate::game_terms::TRIGGERED_ABILITY.into())
}

/// One item of a list of stack objects: its filter, its kind (`None` for a bare adjective
/// sharing a later item's head noun: "creature" in "creature or legendary spell"), and the
/// rest.
fn stack_item(s: &str) -> Option<(Filter, Option<StackKind>, &str)> {
    let s = s.trim_start();
    let fixed: [(&str, StackKind); 12] = [
        ("spells and/or abilities", StackKind::Both),
        ("spells or abilities", StackKind::Both),
        ("spell or ability", StackKind::Both),
        ("activated or triggered abilities", StackKind::Ability),
        ("activated or triggered ability", StackKind::Ability),
        ("activated abilities", StackKind::Ability),
        ("activated ability", StackKind::Ability),
        ("triggered abilities", StackKind::Ability),
        ("triggered ability", StackKind::Ability),
        ("abilities", StackKind::Ability),
        ("ability", StackKind::Ability),
        ("spells", StackKind::Spell),
    ];
    for (p, k) in fixed {
        if let Some(r) = strip(s, p) {
            let f = if p.starts_with("activated or") || p.starts_with("abilit") {
                Filter::Or(vec![activated(), triggered()])
            } else if p.starts_with("activated") {
                activated()
            } else if p.starts_with("triggered") {
                triggered()
            } else if k == StackKind::Both {
                Filter::Any
            } else {
                Filter::Spell
            };
            return Some((f, Some(k), r));
        }
    }
    if let Some((f, _, r)) = parse_object_phrase(s) {
        if mentions_spell(&f) {
            return Some((f, Some(StackKind::Spell), r));
        }
    }
    let (w, r) = split_word(s);
    let w = w.trim_end_matches(',');
    let (f, _, tail) = parse_object_phrase(w)?;
    if !tail.trim().is_empty() || mentions_spell(&f) {
        return None;
    }
    let r = if s[w.len()..].starts_with(',') {
        &s[w.len()..]
    } else {
        r
    };
    Some((f, None, r))
}

fn mentions_spell(f: &Filter) -> bool {
    match f {
        Filter::Spell | Filter::SpellOnStack => true,
        Filter::And(v) | Filter::Or(v) => v.iter().any(mentions_spell),
        _ => false,
    }
}

/// A description of an ability's source: "an artifact source", "a noncreature source", "an
/// artifact or enchantment source", "an artifact, creature, enchantment, or land".
fn source_description(s: &str) -> Option<(Filter, &str)> {
    let s = s.trim_start();
    let s = strip(s, "an ").or_else(|| strip(s, "a "))?;
    if let Some(i) = s.find(" source") {
        let (head, rest) = (&s[..i], &s[i + " source".len()..]);
        // "a noncreature source": adjectives without a head noun.
        let f = match parse_object_phrase(head) {
            Some((f, _, tail)) if tail.trim().is_empty() => f,
            _ => Filter::and(
                head.split(' ')
                    .map(|w| adjective(w).or_else(|| head_noun(w)))
                    .collect::<Option<Vec<_>>>()?,
            ),
        };
        return Some((f, rest));
    }
    parse_object_phrase(s).map(|(f, _, rest)| (f, rest))
}

/// A qualifier after a list of stack objects that applies to all of them.
fn stack_qualifier(s: &str) -> Option<(Filter, &str)> {
    let t = s.trim_start();
    if let Some(r) = strip(t, "from ") {
        let (f, rest) = source_description(r)?;
        return Some((Filter::AbilityFrom(Box::new(f)), rest));
    }
    for (p, rel) in [
        ("you control", PlayerRel::You),
        ("an opponent controls", PlayerRel::Opponent),
        ("your opponents control", PlayerRel::Opponent),
    ] {
        if let Some(r) = strip(t, p) {
            return Some((Filter::ControlledBy(rel), r));
        }
    }
    if let Some(r) = strip(t, "you don't control") {
        return Some((Filter::not(Filter::ControlledBy(PlayerRel::You)), r));
    }
    for (p, z) in [
        ("cast from a graveyard", ZoneKind::Graveyard),
        ("cast from exile", ZoneKind::Exile),
    ] {
        if let Some(r) = strip(t, p) {
            return Some((Filter::CastFrom(z), r));
        }
    }
    if let Some(r) = strip(t, "with {x} in its mana cost") {
        return Some((Filter::HasX, r));
    }
    if let Some(r) = strip(t, "that's the second spell cast this turn") {
        return Some((
            Filter::Custom(crate::kw::basic_effects::SECOND_SPELL_CAST_THIS_TURN.into()),
            r,
        ));
    }
    // "with power or toughness 2 or less"
    if let Some(r) = strip(t, "with power or toughness ") {
        let (n, r2) = parse_number(r)?;
        let (cmp, r3) = if let Some(x) = strip(r2, "or less") {
            (Cmp::Le, x)
        } else if let Some(x) = strip(r2, "or greater") {
            (Cmp::Ge, x)
        } else {
            return None;
        };
        return Some((
            Filter::Or(vec![
                Filter::Power(cmp, Box::new(n.clone())),
                Filter::Toughness(cmp, Box::new(n)),
            ]),
            r3,
        ));
    }
    // "that targets [only] ...", up to the end of the phrase.
    for (p, only) in [("that targets only ", true), ("that targets ", false)] {
        if let Some(r) = strip(t, p) {
            let (objects, players) = super::r115_targets::targeted_thing(r)?;
            let q = if only {
                TargetsFilter::Only { objects, players }
            } else {
                TargetsFilter::Targets { objects, players }
            };
            return Some((Filter::StackTargets(Box::new(q)), ""));
        }
    }
    None
}

/// A list of stack objects with its qualifiers: the filter, what it can match, and the
/// rest of the text.
pub(crate) fn stack_objects(s: &str) -> Option<(Filter, StackKind, &str)> {
    let mut items: Vec<(Filter, Option<StackKind>)> = Vec::new();
    let mut rest = s;
    loop {
        let (f, k, r) = stack_item(rest)?;
        // The object phrase parser may have taken the comma after the item.
        let consumed = rest[..rest.len() - r.len()].trim_end();
        let took_comma = [",", " or", " and/or"].iter().any(|x| consumed.ends_with(x));
        items.push((f, k));
        rest = r;
        let t = rest.trim_start();
        let next = [", or ", ", and/or ", ", "]
            .iter()
            .find_map(|sep| rest.strip_prefix(sep))
            .or_else(|| strip(t, "or "))
            .or_else(|| strip(t, "and/or "))
            .or_else(|| took_comma.then_some(t));
        // Not a list separator: the item's own qualifier ("spell or ability an opponent
        // controls") or the end.
        match next {
            Some(n) if stack_item(n).is_some() => rest = n,
            _ => break,
        }
    }
    let any_spell = items.iter().any(|(_, k)| *k == Some(StackKind::Spell));
    if items.iter().any(|(_, k)| k.is_none()) && !any_spell {
        return None;
    }
    let mut kinds = Vec::new();
    let fs: Vec<Filter> = items
        .into_iter()
        .map(|(f, k)| match k {
            Some(k) => {
                kinds.push(k);
                f
            }
            None => {
                kinds.push(StackKind::Spell);
                Filter::and(vec![f, Filter::Spell])
            }
        })
        .collect();
    let mut parts = vec![if fs.len() == 1 {
        fs.into_iter().next().expect("one item")
    } else {
        Filter::Or(fs)
    }];
    while let Some((q, r)) = stack_qualifier(rest) {
        // A source only for abilities.
        if matches!(q, Filter::AbilityFrom(_)) && kinds.contains(&StackKind::Spell) {
            return None;
        }
        parts.push(q);
        rest = r;
    }
    let kind = if kinds.iter().all(|k| *k == StackKind::Spell) {
        StackKind::Spell
    } else if kinds.iter().all(|k| *k == StackKind::Ability) {
        StackKind::Ability
    } else {
        StackKind::Both
    };
    Some((Filter::and(parts), kind, rest))
}

/// "target [stack objects]", "up to one target ...", "up to four target ...".
pub(crate) fn stack_target(s: &str) -> Option<(TargetSpec, &str)> {
    let s = s.trim_start();
    let (min, max, r) = if let Some(r) = strip(s, "up to ") {
        let (n, r) = parse_number(r)?;
        (0, n, r)
    } else {
        (1, Value::Const(1), s)
    };
    let r = strip(r, "target ")?;
    let (f, kind, rest) = stack_objects(r)?;
    let what = match kind {
        StackKind::Spell => TargetKind::Spell(f),
        StackKind::Ability => TargetKind::Ability(f),
        StackKind::Both => TargetKind::SpellOrAbility(f),
    };
    let spec = TargetSpec {
        what,
        min,
        max,
        distinct_from: vec![],
        divide: None,
        chosen_by_opponent: false,
        text: s[..s.len() - rest.len()].trim().to_string(),
        condition: None,
        together: None,
    };
    Some((spec, rest))
}

/// The cost in "unless [its controller] pays [cost]": "{2}", "twice {X}".
fn unless_cost(s: &str) -> Option<Cost> {
    let s = end(s);
    if let Some(m) = s.strip_prefix("twice ") {
        if m.starts_with('{') && !m.contains(' ') {
            return crate::oracle::keywords::parse_keyword_cost(&format!("{m}{m}"));
        }
        return None;
    }
    crate::oracle::keywords::parse_keyword_cost(s)
}

/// "counter [target stack objects] [unless its controller pays X]".
fn counter_described(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("counter ")?;
    if r.starts_with("all ") {
        return counter_all(r);
    }
    let (spec, rest) = stack_target(r)?;
    let text = spec.text.clone();
    let rest = rest.trim();
    let unless = [
        "unless its controller pays ",
        "unless that ability's controller pays ",
        "unless that spell's controller pays ",
    ]
    .iter()
    .find_map(|p| rest.strip_prefix(p));
    if unless.is_none() && !end(rest).is_empty() {
        return None;
    }
    let cost = match unless {
        Some(c) => Some(unless_cost(c)?),
        None => None,
    };
    let slot = b.add_target(spec, &text);
    let what = Sel::Target(slot);
    let counter = Effect::CounterSpell { what: what.clone() };
    Some(match cost {
        Some(cost) => Effect::PayOptional {
            who: PlayerRef::ControllerOf(Box::new(what)),
            cost,
            then: Box::new(Effect::Noop),
            otherwise: Box::new(counter),
        },
        None => counter,
    })
}

/// "counter all abilities [your opponents control]", "counter all spells your opponents
/// control and all abilities your opponents control".
fn counter_all(r: &str) -> Option<Effect> {
    let mut alts = Vec::new();
    for part in r.split(" and ") {
        let p = part.trim().strip_prefix("all ")?;
        let (f, _, rest) = stack_objects(p)?;
        if !end(rest).is_empty() {
            return None;
        }
        alts.push(f);
    }
    let f = if alts.len() == 1 {
        alts.pop().expect("one")
    } else {
        Filter::Or(alts)
    };
    Some(Effect::CounterSpell {
        what: Sel::All(Filter::and(vec![f, Filter::InZone(ZoneKind::Stack)])),
    })
}

/// "Exile all other spells and counter all abilities." (Summary Dismissal).
fn exile_spells_counter_abilities(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("exile all ")?;
    let (spells, abilities) = r.split_once(" and counter all ")?;
    let other = match spells {
        "other spells" => true,
        "spells" => false,
        _ => return None,
    };
    let (af, kind, rest) = stack_objects(abilities)?;
    if !end(rest).is_empty() || kind != StackKind::Ability {
        return None;
    }
    let mut sf = vec![Filter::Spell, Filter::InZone(ZoneKind::Stack)];
    if other {
        sf.push(Filter::Other);
    }
    Some(Effect::seq(vec![
        Effect::Exile {
            what: Sel::All(Filter::and(sf)),
            face_down: false,
            link: false,
        },
        Effect::CounterSpell {
            what: Sel::All(Filter::and(vec![af, Filter::InZone(ZoneKind::Stack)])),
        },
    ]))
}

/// The permanent that is the source of the ability in target slot `slot` (not a new
/// object it became, CR 400.7).
fn source_of_slot(slot: u8) -> Sel {
    Sel::All(Filter::and(vec![
        Filter::Permanent,
        Filter::Custom(crate::kw::basic_effects::source_of_slot(slot).into()),
    ]))
}

const COUNTERED_SOURCE: Var = vars::USER + 3301;

/// The single counter-target slot of a counter effect.
fn counter_slot(e: &Effect) -> Option<u8> {
    match e {
        Effect::CounterSpell {
            what: Sel::Target(n),
        } => Some(*n),
        Effect::PayOptional { otherwise, .. } => counter_slot(otherwise),
        _ => None,
    }
}

/// Wraps a counter effect so that the permanent whose ability it counters is remembered
/// before it's countered, and `then` is performed on it if the ability was countered.
fn with_countered_source(prev: &mut Effect, slot: u8, then: Effect) {
    let counter = std::mem::take(prev);
    *prev = Effect::seq(vec![
        Effect::Store {
            var: COUNTERED_SOURCE,
            sel: source_of_slot(slot),
        },
        counter,
        Effect::If {
            cond: Condition::PrevHappened,
            then: Box::new(then),
            otherwise: Box::new(Effect::Noop),
        },
    ]);
}

/// "If a permanent's ability is countered this way, destroy that permanent." after a
/// counter effect (Green Slime, Teferi's Response).
fn destroy_countered_source(s: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if end(s) != "if a permanent's ability is countered this way, destroy that permanent" {
        return false;
    }
    let Some(slot) = counter_slot(prev) else {
        return false;
    };
    if !matches!(
        b.targets.get(slot as usize).map(|t| &t.what),
        Some(TargetKind::Ability(_) | TargetKind::SpellOrAbility(_))
    ) {
        return false;
    }
    with_countered_source(
        prev,
        slot,
        Effect::Destroy {
            what: Sel::Var(COUNTERED_SOURCE),
            no_regen: false,
        },
    );
    true
}

/// "That permanent's activated abilities can't be activated this turn." after countering
/// an ability (Interdict): the permanent that was its source (CR 602.5a: mana abilities
/// included).
fn countered_source_cant_activate(s: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let s = end(s);
    let (dur, r) = crate::oracle::effects::duration_suffix(s);
    if r != "that permanent's activated abilities can't be activated"
        || !matches!(dur, Duration::EndOfTurn)
    {
        return false;
    }
    let Some(slot) = counter_slot(prev) else {
        return false;
    };
    if !matches!(
        b.targets.get(slot as usize).map(|t| &t.what),
        Some(TargetKind::Ability(_))
    ) {
        return false;
    }
    let counter = std::mem::take(prev);
    *prev = Effect::seq(vec![
        Effect::Store {
            var: COUNTERED_SOURCE,
            sel: source_of_slot(slot),
        },
        counter,
        Effect::AddRestriction {
            restriction: Restriction::CantActivate {
                who: PlayerFilter::Any,
                sources: Filter::In(Box::new(Sel::Var(COUNTERED_SOURCE))),
                include_mana: true,
            },
            duration: dur,
        },
    ]);
    true
}

/// "counter target activated ability from an artifact source and destroy that artifact if
/// it's on the battlefield" (Ouphe Vandals).
fn counter_and_destroy_source(l: &str, b: &mut Builder) -> Option<Effect> {
    let (head, tail) = end(l).split_once(" and destroy that ")?;
    let (noun, cond) = tail.split_once(' ')?;
    if cond != "if it's on the battlefield" || !matches!(noun, "artifact" | "permanent") {
        return None;
    }
    let mut e = counter_described(head, b)?;
    let slot = counter_slot(&e)?;
    if !matches!(
        b.targets.get(slot as usize).map(|t| &t.what),
        Some(TargetKind::Ability(_))
    ) {
        return None;
    }
    with_countered_source(
        &mut e,
        slot,
        Effect::Destroy {
            what: Sel::Var(COUNTERED_SOURCE),
            no_regen: false,
        },
    );
    Some(e)
}

inventory::submit! { EffectPattern { name: "counter: described stack objects", priority: 40, parse: counter_described } }
inventory::submit! { EffectPattern { name: "counter: and destroy its source", priority: 40, parse: counter_and_destroy_source } }
inventory::submit! { EffectPattern { name: "counter: exile spells and counter abilities", priority: 40, parse: exile_spells_counter_abilities } }
inventory::submit! { FollowupPattern { name: "counter: the countered ability's permanent can't activate", priority: 100, apply: countered_source_cant_activate } }
inventory::submit! { FollowupPattern { name: "counter: destroy the countered ability's permanent", priority: 100, apply: destroy_countered_source } }

#[cfg(test)]
mod tests {
    use super::*;

    fn target(s: &str) -> TargetSpec {
        let (spec, rest) = stack_target(s).unwrap_or_else(|| panic!("{s}"));
        assert_eq!(end(rest), "", "{s}");
        spec
    }

    #[test]
    fn stack_object_lists() {
        assert!(matches!(
            target("target activated ability, triggered ability, or legendary spell").what,
            TargetKind::SpellOrAbility(Filter::Or(ref v)) if v.len() == 3
        ));
        assert!(matches!(
            target("target instant spell, sorcery spell, activated ability, or triggered ability").what,
            TargetKind::SpellOrAbility(Filter::Or(ref v)) if v.len() == 4
        ));
        assert!(matches!(
            target("target creature or legendary spell").what,
            TargetKind::Spell(Filter::Or(ref v)) if v.len() == 2
        ));
        assert!(matches!(
            target("target activated ability from an artifact source").what,
            TargetKind::Ability(Filter::And(ref v)) if matches!(v[1], Filter::AbilityFrom(_))
        ));
        assert!(matches!(
            target("target activated ability from an artifact, creature, enchantment, or land").what,
            TargetKind::Ability(Filter::And(ref v)) if matches!(v[1], Filter::AbilityFrom(_))
        ));
        assert!(matches!(
            target("target activated or triggered ability from an artifact or enchantment source").what,
            TargetKind::Ability(_)
        ));
        assert!(matches!(
            target("target spell or ability an opponent controls that targets a land you control").what,
            TargetKind::SpellOrAbility(Filter::And(ref v))
                if v.iter().any(|f| matches!(f, Filter::StackTargets(_)))
        ));
        let t = target("up to four target spells and/or abilities");
        assert!(matches!(t.what, TargetKind::SpellOrAbility(_)));
        assert_eq!(t.min, 0);
        assert!(matches!(target("target spell cast from a graveyard").what, TargetKind::Spell(_)));
        assert!(matches!(
            target("target creature spell with power or toughness 2 or less").what,
            TargetKind::Spell(_)
        ));
        assert!(matches!(
            target("target triggered ability or colorless spell").what,
            TargetKind::SpellOrAbility(_)
        ));
        assert!(matches!(
            target("target instant spell, sorcery spell, or triggered ability").what,
            TargetKind::SpellOrAbility(Filter::Or(ref v)) if v.len() == 3
        ));
        assert!(matches!(
            target("target activated or triggered ability from a noncreature source").what,
            TargetKind::Ability(_)
        ));
        // A source belongs to abilities only.
        assert!(stack_target("target spell from an artifact source").is_none());
    }
}
