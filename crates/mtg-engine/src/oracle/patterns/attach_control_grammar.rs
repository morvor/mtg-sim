//! Permanent status grammar: attaching and unattaching (CR 701.3), read compositionally.
//!
//! ```text
//! attach     := ["you may"] "attach" attachments "to" recipient
//! attachments:= referent ("~", "it", "them", "that Equipment", "her")
//!             | target phrase ("target Equipment you control", "up to one target
//!               Equipment", "any number of target Equipment you control")
//!             | "a"/"an" noun-phrase            (one chosen as the instruction is performed)
//!             | "any number of" noun-phrase      (any number chosen as it's performed)
//!             | "all" noun-phrase                ("all Equipment you control")
//! noun-phrase:= object phrase, or two nouns sharing the qualifiers ("Auras and Equipment
//!               you control")
//! recipient  := referent | target phrase | "target permanent or player"
//!             | player ("that player", "target opponent")
//!             | "a"/"an" noun-phrase             (one it could be attached to, chosen)
//! unattach   := "unattach" referent
//! ```
//!
//! Each attachment that can legally be attached to the recipient becomes attached to it;
//! one that can't doesn't move, and one already attached to it stays (CR 701.3b, 701.3c,
//! 301.5c, 303.4). Untargeted attachments and recipients are chosen by the controller of
//! the effect as it's performed (CR 608.2c). In "attach X to it", "it" is what it meant
//! before X was named.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{object_ref, player_ref, Builder};
use crate::oracle::patterns::oracle_hardening_referents::is_no_referent;
use crate::oracle::phrases::*;

/// The permanent an "attach an Equipment you control to ..." instruction chose, which a
/// later "it" refers to ("If you do, unattach it at the beginning of the next end step").
pub const ATTACHED: Var = vars::USER + 8300;

fn word_end(rest: &str) -> bool {
    rest.is_empty() || rest.starts_with([' ', ',', '.'])
}

/// An object phrase naming one kind of object or two kinds sharing what follows the second
/// noun ("Auras and Equipment you control", "Aura or Equipment card"). Returns the
/// filter, whether it's plural, and the rest.
pub fn noun_phrase(s: &str) -> Option<(Filter, bool, String)> {
    let s = s.trim_start();
    // "auras and equipment you control": two head nouns joined by "and".
    let words: Vec<&str> = s.splitn(3, ' ').collect();
    if words.len() == 3 && words[1] == "and" && head_noun(words[0]).is_some() {
        let second = words[2];
        let (fb, plural, rest) = parse_object_phrase(second)?;
        let shared = &second[split_word(second).0.len()..second.len() - rest.len()];
        let a = format!("{}{shared}", words[0]);
        let (fa, _, tail) = parse_object_phrase(&a)?;
        if !tail.trim().is_empty() {
            return None;
        }
        return Some((Filter::Or(vec![fa, fb]), plural, rest.to_string()));
    }
    let (f, plural, rest) = parse_object_phrase(s)?;
    Some((f, plural, rest.to_string()))
}

/// The objects being attached. Returns the selection and the rest of the text.
fn attachments(s: &str, b: &mut Builder) -> Option<(Sel, String)> {
    let s = s.trim_start();
    // "that Equipment", "that Aura": what the text named before.
    for p in ["that equipment", "that aura"] {
        if let Some(r) = s.strip_prefix(p) {
            if !word_end(r) {
                continue;
            }
            let it = super::pronoun_groups::singular_it(b);
            if is_no_referent(&it) || matches!(it, Sel::This) {
                return None;
            }
            return Some((it, r.to_string()));
        }
    }
    // "an Equipment you control": one chosen as the instruction is performed.
    if let Some(r) = s.strip_prefix("an ").or_else(|| s.strip_prefix("a ")) {
        let (f, plural, rest) = noun_phrase(r)?;
        if plural || f.zone().is_some_and(|z| z != ZoneKind::Battlefield) {
            return None;
        }
        let f = super::filters_relational::resolve_referent(f, b)?;
        return Some((
            Sel::Choose {
                chooser: PlayerRef::You,
                filter: Filter::and(vec![f, Filter::InZone(ZoneKind::Battlefield)]),
                count: Value::c(1),
                up_to: false,
                store: Some(ATTACHED),
            },
            rest,
        ));
    }
    // "any number of Auras and Equipment you control" (not targeted).
    if let Some(r) = s.strip_prefix("any number of ") {
        if !r.starts_with("target ") {
            // ("Equipment" is its own plural.)
            let (f, _, rest) = noun_phrase(r)?;
            if f.zone().is_some_and(|z| z != ZoneKind::Battlefield) {
                return None;
            }
            let f = super::filters_relational::resolve_referent(f, b)?;
            let f = Filter::and(vec![f, Filter::InZone(ZoneKind::Battlefield)]);
            return Some((
                Sel::Choose {
                    chooser: PlayerRef::You,
                    filter: f,
                    // Unbounded ("any number of", as for targets).
                    count: Value::c(99),
                    up_to: true,
                    store: None,
                },
                rest,
            ));
        }
    }
    // "all Equipment attached to that creature" (Rhuk, Hexgold Nabber).
    if let Some(r) = s.strip_prefix("all ") {
        if r.contains("attached to ") {
            let saved = (b.targets.len(), b.it.clone());
            if let Some((f, rest)) = attached_objects(r, b) {
                return Some((Sel::All(f), rest));
            }
            b.targets.truncate(saved.0);
            b.it = saved.1;
        }
    }
    // "all Auras and Equipment you control".
    if let Some(r) = s.strip_prefix("all ") {
        if let Some((f, true, rest)) = noun_phrase(r) {
            if matches!(f, Filter::Or(_)) {
                let f = super::filters_relational::resolve_referent(f, b)?;
                return Some((
                    Sel::All(Filter::and(vec![f, Filter::InZone(ZoneKind::Battlefield)])),
                    rest,
                ));
            }
        }
    }
    let (sel, rest) = object_ref(s, b)?;
    // Cards elsewhere ("target Aura card in your graveyard") aren't attached by this
    // instruction (they'd have to be put onto the battlefield).
    if let Sel::Target(slot) = &sel {
        if let Some(TargetKind::Object(f)) = b.targets.get(*slot as usize).map(|t| &t.what) {
            if f.zone().is_some_and(|z| z != ZoneKind::Battlefield) {
                return None;
            }
        }
    }
    if matches!(sel, Sel::All(ref f) if f.zone().is_some_and(|z| z != ZoneKind::Battlefield)) {
        return None;
    }
    Some((sel, rest))
}

/// What the objects are attached to. `what` is the attachments (for a recipient chosen
/// among those they could be attached to).
fn recipient(s: &str, what: &Sel, b: &mut Builder) -> Option<(Sel, String)> {
    let s = s.trim_start();
    if let Some(r) = s.strip_prefix("target permanent or player") {
        let spec = TargetSpec::one(
            TargetKind::ObjectOrPlayer(Filter::Permanent, PlayerFilter::Any),
            "target permanent or player",
        );
        let slot = b.add_target(spec, "target permanent or player");
        return Some((Sel::Target(slot), r.to_string()));
    }
    // "a creature you control": one the attachments could be attached to, chosen.
    if let Some(r) = s.strip_prefix("an ").or_else(|| s.strip_prefix("a ")) {
        let (f, plural, rest) = noun_phrase(r)?;
        if plural || f.zone().is_some_and(|z| z != ZoneKind::Battlefield) {
            return None;
        }
        let f = super::filters_relational::resolve_referent(f, b)?;
        return Some((
            Sel::Choose {
                chooser: PlayerRef::You,
                filter: Filter::and(vec![
                    f,
                    Filter::InZone(ZoneKind::Battlefield),
                    Filter::CanBeAttachedBy(Box::new(what.clone())),
                ]),
                count: Value::c(1),
                up_to: false,
                store: None,
            },
            rest,
        ));
    }
    // "another creature", "another permanent it can enchant" (Fumble, Aura Graft): one
    // chosen as it's performed, other than the one the attachments are attached to.
    if let Some(r) = s.strip_prefix("another ") {
        // "another permanent of that type" (Enchantment Alteration): the type of the
        // permanent it's attached to, not a chosen type.
        if r.contains(" of that ") {
            return None;
        }
        let (f, plural, rest) = noun_phrase(r)?;
        let trimmed = rest.trim_start();
        let rest = trimmed
            .strip_prefix("it can enchant")
            .or_else(|| trimmed.strip_prefix("it could enchant"))
            .map_or(rest.clone(), str::to_string);
        if plural || f.zone().is_some_and(|z| z != ZoneKind::Battlefield) {
            return None;
        }
        let f = super::filters_relational::resolve_referent(f, b)?;
        let mut parts = vec![
            f,
            Filter::InZone(ZoneKind::Battlefield),
            Filter::not(Filter::In(Box::new(Sel::HostOf(Box::new(what.clone()))))),
        ];
        // One attachment: a permanent it can be attached to (several are each attached
        // if they can be, CR 701.3b).
        let single = match what {
            Sel::Target(slot) => b
                .targets
                .get(*slot as usize)
                .is_some_and(|t| t.max.as_const() == Some(1)),
            Sel::This | Sel::Var(_) => !matches!(b.group, Some(_)),
            _ => false,
        };
        if single {
            parts.push(Filter::CanBeAttachedBy(Box::new(what.clone())));
        }
        return Some((
            Sel::Choose {
                chooser: PlayerRef::You,
                filter: Filter::and(parts),
                count: Value::c(1),
                up_to: false,
                store: None,
            },
            rest,
        ));
    }
    // A player (a Curse is attached to a player, CR 303.4).
    for p in ["that player", "target player", "target opponent"] {
        if s.strip_prefix(p).is_some_and(word_end) {
            let (who, rest) = player_ref(s, b)?;
            return Some((Sel::Players(who), rest));
        }
    }
    let (sel, rest) = object_ref(s, b)?;
    if matches!(sel, Sel::All(_)) {
        return None;
    }
    if let Sel::Target(slot) = &sel {
        let spec = b.targets.get(*slot as usize)?;
        // One object: "up to one target creature", not "any number of target creatures".
        if spec.max.as_const() != Some(1) {
            return None;
        }
        if let TargetKind::Object(f) = &spec.what {
            if f.zone().is_some_and(|z| z != ZoneKind::Battlefield) {
                return None;
            }
        }
    }
    Some((sel, rest))
}

/// "attach [attachments] to [recipient]".
fn p_attach(l: &str, b: &mut Builder) -> Option<Effect> {
    if super::zz_probe_ps::disabled() {
        return None;
    }
    let r = end(l).strip_prefix("attach ")?;
    // "attach a Curse attached to you to one of your opponents" (Lynde, Cheerful
    // Tormentor): an opponent chosen as it's performed.
    if let Some(a) = r.strip_suffix(" to one of your opponents") {
        let saved = (b.targets.len(), b.it.clone());
        let parsed = attachments(a, b).filter(|(w, rest)| {
            end(rest).is_empty() && matches!(w, Sel::Choose { .. })
        });
        let Some((what, _)) = parsed else {
            b.targets.truncate(saved.0);
            b.it = saved.1;
            return None;
        };
        if let Sel::Choose { store: Some(v), .. } = &what {
            b.it = Sel::Var(*v);
        }
        return Some(Effect::seq(vec![
            Effect::Choose {
                who: PlayerRef::You,
                kind: ChoiceKind::Opponent,
            },
            crate::kw::attach_choice::attach_chosen(
                what,
                Sel::Players(PlayerRef::ChosenOpponent),
            ),
        ]));
    }
    // The attachment phrase may itself contain " to " ("target Aura attached to a
    // creature you control to target creature you control"): try each split.
    for (i, _) in r.match_indices(" to ") {
        let (a, t) = (&r[..i], &r[i + 4..]);
        let saved = (
            b.targets.len(),
            b.it.clone(),
            b.it_player.clone(),
            b.group.clone(),
        );
        let parsed = (|| {
            let (what, rest) = attachments(a, b)?;
            if !end(&rest).is_empty() {
                return None;
            }
            // "attach up to one target Equipment you control to it": "it" is what it was
            // before the Equipment was named.
            let after_what = b.it.clone();
            b.it = saved.1.clone();
            let n = b.targets.len();
            let (to, rest) = recipient(t, &what, b)?;
            // "Return ~ ..., then attach it to that creature": a pronoun that came out as
            // the attachment itself isn't understood (it can't be attached to itself).
            if !end(&rest).is_empty() || format!("{to:?}") == format!("{what:?}") {
                return None;
            }
            if let Sel::Choose { store, .. } = &what {
                // Both chosen ("attach an Aura you control to a creature"): not read.
                if matches!(to, Sel::Choose { .. }) {
                    return None;
                }
                if let Some(v) = store {
                    b.it = Sel::Var(*v);
                }
                // Chosen among those that could be attached to it (CR 701.3a).
                return Some(crate::kw::attach_choice::attach_chosen(what, to));
            }
            if b.targets.len() == n && matches!(to, Sel::This | Sel::Choose { .. }) {
                b.it = after_what;
            }
            Some(Effect::Attach { what, to })
        })();
        if parsed.is_some() {
            return parsed;
        }
        b.targets.truncate(saved.0);
        (b.it, b.it_player, b.group) = (saved.1, saved.2, saved.3);
    }
    None
}

inventory::submit! { EffectPattern { name: "attach grammar: attach [attachments] to [recipient]", priority: 110, parse: p_attach } }

/// "unattach enchanted Equipment", "unattach it", "unattach target Equipment".
fn p_unattach(l: &str, b: &mut Builder) -> Option<Effect> {
    if super::zz_probe_ps::disabled() {
        return None;
    }
    let r = end(l).strip_prefix("unattach ")?;
    if r.starts_with("all ") {
        return None;
    }
    let (what, rest) = object_ref(r, b)?;
    if !end(&rest).is_empty() || matches!(what, Sel::All(_)) {
        return None;
    }
    Some(Effect::Unattach { what })
}

// ---------------------------------------------------------------------------
// "attached to [object]" qualifiers
// ---------------------------------------------------------------------------

/// "attached to a creature [you control]", "attached to a land", "attached to ~",
/// "attached to enchanted creature", "attached to creatures": attached to that object or
/// to one of those objects (CR 301.5, 303.4). Pronouns ("attached to it", "attached to
/// that creature") and targets are read by the parsers that know what they refer to
/// (`value_grammar`, the zone-move grammar, [`p_destroy_exile_attached`]).
fn attached_to_suffix<'a>(t: &'a str, _so_far: &Filter) -> Option<(Filter, &'a str)> {
    if super::zz_probe_ps::disabled() {
        return None;
    }
    let r = t
        .strip_prefix("attached to ")
        .or_else(|| t.strip_prefix("that's attached to "))
        .or_else(|| t.strip_prefix("that are attached to "))?;
    // "a Curse attached to you": attached to the source's controller (CR 303.4).
    if let Some(rest) = r.strip_prefix("you").filter(|x| word_end(x)) {
        return Some((
            Filter::AttachedToAnyOf(Box::new(Sel::Players(PlayerRef::You))),
            rest,
        ));
    }
    let named = ["a ", "an ", "~", "enchanted ", "equipped "];
    let (sel, rest) = if named.iter().any(|p| r.starts_with(p)) {
        super::filters_relational::object_in(r)?
    } else {
        // "attached to creatures": any of them.
        let (f, plural, rest) = parse_object_phrase(r)?;
        if !plural {
            return None;
        }
        (Sel::All(f), rest)
    };
    if super::filters_relational::mentions_referent(&sel)
        || matches!(sel, Sel::Choose { .. } | Sel::Players(_))
    {
        return None;
    }
    // Only permanents: "attached to a creature card" isn't something on the battlefield.
    if let Sel::All(f) = &sel {
        if f.zone().is_some_and(|z| z != ZoneKind::Battlefield) {
            return None;
        }
    }
    Some((Filter::AttachedToAnyOf(Box::new(sel)), rest))
}

inventory::submit! { super::FilterSuffixPattern { name: "attach grammar: attached to [object]", priority: 100, parse: attached_to_suffix } }

/// "creature with another Aura attached to it" (Daybreak Coronet's enchant ability): an
/// Aura other than the source is attached to it.
fn with_another_aura_suffix<'a>(t: &'a str, _so_far: &Filter) -> Option<(Filter, &'a str)> {
    if super::zz_probe_ps::disabled() {
        return None;
    }
    let r = t.strip_prefix("with another aura attached to it")?;
    word_end(r).then(|| {
        (
            Filter::Custom(crate::kw::attach_choice::ANOTHER_AURA_ATTACHED.into()),
            r,
        )
    })
}

inventory::submit! { super::FilterSuffixPattern { name: "attach grammar: with another Aura attached to it", priority: 100, parse: with_another_aura_suffix } }

inventory::submit! { EffectPattern { name: "attach grammar: unattach [object]", priority: 101, parse: p_unattach } }

/// Whether a filter says what its objects are attached to (something the text refers
/// to: not a pronoun without an antecedent).
fn names_host(f: &Filter) -> bool {
    match f {
        Filter::AttachedToAnyOf(sel) | Filter::AttachedTo(sel) => {
            !matches!(**sel, Sel::None) && !is_no_referent(sel)
        }
        Filter::And(v) => v.iter().any(names_host),
        _ => false,
    }
}

/// An object phrase that says what its objects are attached to ("Equipment attached to
/// that creature", "Auras and Equipment attached to target creature", "permanents
/// attached to creatures"), read with the builder so "it" and "that creature" mean what
/// they refer to. Returns the filter (on the battlefield) and the rest.
pub(crate) fn attached_objects(s: &str, b: &mut Builder) -> Option<(Filter, String)> {
    if !s.contains("attached to ") {
        return None;
    }
    let (f, rest) = super::value_grammar::objects(s, b)?;
    let f = super::filters_relational::resolve_referent(f, b)?;
    if !names_host(&f) || f.zone().is_some_and(|z| z != ZoneKind::Battlefield) {
        return None;
    }
    Some((
        Filter::and(vec![f, Filter::InZone(ZoneKind::Battlefield)]),
        rest,
    ))
}

/// The objects a destroy/exile instruction names when they're described by what they're
/// attached to: "all Equipment attached to that creature", "up to one Equipment attached
/// to that creature" (chosen as it's performed), "up to one target Equipment attached to
/// that creature", "target creature and all Equipment attached to it", "all creatures and
/// all permanents attached to creatures".
fn attached_selection(s: &str, b: &mut Builder) -> Option<(Sel, String)> {
    let s = s.trim_start();
    // "[object] and all [objects attached to it]": both. "Any number of target creatures
    // and all Auras attached to them": attached to any of them.
    if let Some((head, group)) = s.split_once(" and all ") {
        if !head.contains("attached to ") || head.starts_with("all ") {
            let (first, rest) = object_ref(head, b)?;
            if !end(&rest).is_empty() || matches!(first, Sel::Choose { .. }) {
                return None;
            }
            let group = match group.strip_suffix(" attached to them") {
                Some(g) if matches!(first, Sel::Target(_)) => format!("{g} attached to it"),
                _ => group.to_string(),
            };
            if matches!(first, Sel::Target(_)) {
                b.it = first.clone();
            }
            let (f, rest) = attached_objects(&group, b)?;
            return Some((Sel::Union(vec![first, Sel::All(f)]), rest));
        }
    }
    if let Some(r) = s.strip_prefix("all ") {
        let (f, rest) = attached_objects(r, b)?;
        return Some((Sel::All(f), rest));
    }
    let (min, max, r) = if let Some(r) = s.strip_prefix("up to one ") {
        (0, 1, r)
    } else {
        (1, 1, s)
    };
    if let Some(r) = r.strip_prefix("target ") {
        let (f, rest) = attached_objects(r, b)?;
        let text = format!(
            "{}target {}",
            if min == 0 { "up to one " } else { "" },
            &r[..r.len() - rest.len()].trim_end()
        );
        let mut spec = TargetSpec::object(f, text.clone());
        spec.min = Value::c(min);
        spec.max = Value::c(max);
        // "Exile up to one target Equipment attached to that creature. If that creature
        // would die this turn, ...": "that creature" is still the creature.
        let it = b.it.clone();
        let slot = b.add_target(spec, &text);
        b.it = it;
        return Some((Sel::Target(slot), rest));
    }
    if min == 0 {
        let (f, rest) = attached_objects(r, b)?;
        return Some((
            Sel::Choose {
                chooser: PlayerRef::You,
                filter: f,
                count: Value::c(1),
                up_to: true,
                store: None,
            },
            rest,
        ));
    }
    None
}

/// "destroy [objects attached to ...]", "exile [objects attached to ...]" (see
/// [`attached_selection`]).
fn p_destroy_exile_attached(l: &str, b: &mut Builder) -> Option<Effect> {
    if super::zz_probe_ps::disabled() {
        return None;
    }
    let l = end(l);
    let (exile, r) = if let Some(r) = l.strip_prefix("destroy ") {
        (false, r)
    } else {
        (true, l.strip_prefix("exile ")?)
    };
    let saved = (b.targets.len(), b.it.clone());
    let parsed = attached_selection(r, b).filter(|(_, rest)| end(rest).is_empty());
    let Some((what, _)) = parsed else {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        return None;
    };
    Some(if exile {
        Effect::Exile {
            what,
            face_down: false,
            link: false,
        }
    } else {
        Effect::Destroy {
            what,
            no_regen: false,
        }
    })
}

inventory::submit! { EffectPattern { name: "attach grammar: destroy/exile [objects attached to ...]", priority: 110, parse: p_destroy_exile_attached } }

// ---------------------------------------------------------------------------
// Conditions about what's attached to a permanent
// ---------------------------------------------------------------------------

/// The permanent a condition says things are attached to: "~", "equipped creature",
/// "enchanted creature" (the one the source is attached to).
fn host(s: &str) -> Option<Sel> {
    match s {
        "~" => Some(Sel::This),
        "equipped creature" | "enchanted creature" | "enchanted permanent" => Some(Sel::AttachedTo),
        _ => None,
    }
}

/// "two or more Equipment are attached to ~", "another Aura is attached to enchanted
/// creature", "an Aura is attached to ~": counted as the condition is checked ("another":
/// other than the source).
fn attached_count_condition(c: &str) -> Option<Condition> {
    if super::zz_probe_ps::disabled() {
        return None;
    }
    let c = end(c);
    let (objs, to) = c
        .split_once(" are attached to ")
        .or_else(|| c.split_once(" is attached to "))?;
    let host = host(to)?;
    let (n, objs) = if let Some(r) = objs.strip_prefix("another ") {
        (Value::c(1), format!("other {r}"))
    } else if let Some(r) = objs.strip_prefix("an ").or_else(|| objs.strip_prefix("a ")) {
        (Value::c(1), r.to_string())
    } else {
        let (n, r) = parse_number(objs)?;
        let r = r.trim_start().strip_prefix("or more ")?;
        (n, r.to_string())
    };
    let (f, _, rest) = parse_object_phrase(&objs)?;
    if !rest.trim().is_empty() || f.zone().is_some() {
        return None;
    }
    Some(Condition::Compare(
        Value::Count(Filter::and(vec![
            f,
            Filter::AttachedToAnyOf(Box::new(host)),
        ])),
        Cmp::Ge,
        n,
    ))
}

inventory::submit! { super::ConditionPattern { name: "attach grammar: N or more [objects] are attached to [permanent]", priority: 100, parse: attached_count_condition } }

/// "[subject] has [keywords] as long as two or more Equipment are attached to it" (Balan,
/// Wandering Knight; Brass Knuckles), "As long as another Aura is attached to enchanted
/// creature, it has first strike and lifelink" (Face of Divinity): "it" is the subject
/// ("~", "equipped creature", "enchanted creature"). Read as the static ability with the
/// subject named in place of "it".
fn attached_it_static(block: &str, ctx: &crate::oracle::CompileContext) -> Option<Vec<Ability>> {
    if super::zz_probe_ps::disabled() || block.contains('\n') || block.contains('"') {
        return None;
    }
    let lower = block.to_lowercase();
    let l = end(&lower);
    let subjects = ["~", "equipped creature", "enchanted creature"];
    let text = if let Some((head, cond)) = l.split_once(" as long as ") {
        let subject = subjects
            .iter()
            .find(|s| head.starts_with(&format!("{s} ")))?;
        let cond = cond
            .strip_suffix(" attached to it")
            .filter(|c| c.ends_with(" are") || c.ends_with(" is"))?;
        format!("as long as {cond} attached to {subject}, {head}")
    } else {
        let r = l.strip_prefix("as long as ")?;
        let (cond, rest) = r.split_once(", ")?;
        let subject = subjects
            .iter()
            .find(|s| cond.ends_with(&format!(" attached to {s}")))?;
        let rest = rest.strip_prefix("it ")?;
        format!("as long as {cond}, {subject} {rest}")
    };
    let v = crate::oracle::statics::parse_static(&text, ctx)?;
    if v.is_empty()
        || !v
            .iter()
            .all(|a| matches!(&a.kind, AbilityKind::Static(s) if s.condition.is_some()))
    {
        return None;
    }
    Some(
        v.into_iter()
            .map(|a| AbilityDef::with_link(a.kind.clone(), block, a.link))
            .collect(),
    )
}

inventory::submit! { super::AbilityPattern { name: "attach grammar: as long as [objects] are attached to it", priority: 49, parse: attached_it_static } }

// ---------------------------------------------------------------------------
// Entering attached: "return ~ from your graveyard to the battlefield attached to that
// creature", "put an Aura card from your hand onto the battlefield attached to ~"
// ---------------------------------------------------------------------------

/// What a permanent put onto the battlefield is attached to: a referent ("that creature",
/// "~", "it", "enchanted creature"), a target object or player, or an object chosen as it
/// enters ("a creature you control").
fn entry_recipient(s: &str, b: &mut Builder) -> Option<Sel> {
    let s = s.trim();
    if let Some(r) = s.strip_prefix("a ").or_else(|| s.strip_prefix("an ")) {
        let (f, plural, rest) = parse_object_phrase(r)?;
        if plural || !end(rest).is_empty() || f.zone().is_some_and(|z| z != ZoneKind::Battlefield)
        {
            return None;
        }
        let f = super::filters_relational::resolve_referent(f, b)?;
        return Some(Sel::Choose {
            chooser: PlayerRef::You,
            filter: Filter::and(vec![f, Filter::InZone(ZoneKind::Battlefield)]),
            count: Value::c(1),
            up_to: false,
            store: None,
        });
    }
    for p in ["target player", "target opponent"] {
        if s.strip_prefix(p).is_some_and(word_end) {
            let (who, rest) = player_ref(s, b)?;
            if !end(&rest).is_empty() {
                return None;
            }
            return match who {
                PlayerRef::Target(slot) => Some(Sel::Target(slot)),
                _ => None,
            };
        }
    }
    let (sel, rest) = object_ref(s, b)?;
    if !end(&rest).is_empty()
        || is_no_referent(&sel)
        || matches!(
            sel,
            Sel::None | Sel::All(_) | Sel::Players(_) | Sel::Choose { .. }
        )
    {
        return None;
    }
    if let Sel::Target(slot) = &sel {
        let spec = b.targets.get(*slot as usize)?;
        if spec.max.as_const() != Some(1) {
            return None;
        }
        if let TargetKind::Object(f) = &spec.what {
            if f.zone().is_some_and(|z| z != ZoneKind::Battlefield) {
                return None;
            }
        }
    }
    Some(sel)
}

/// Sets what the one battlefield destination in `e` enters attached to. `None` unless
/// exactly one instruction puts something onto the battlefield.
fn with_entry_attachment(e: Effect, to: &Sel) -> Option<Effect> {
    use serde_json::Value as J;
    fn walk(v: &mut J, to: &J, n: &mut usize) {
        match v {
            J::Object(m) => {
                if m.get("zone").and_then(|z| z.as_str()) == Some("Battlefield")
                    && m.get("attached_to").is_some_and(|a| a.is_null())
                {
                    m.insert("attached_to".into(), to.clone());
                    *n += 1;
                }
                for x in m.values_mut() {
                    walk(x, to, n);
                }
            }
            J::Array(a) => a.iter_mut().for_each(|x| walk(x, to, n)),
            _ => {}
        }
    }
    let mut json = serde_json::to_value(&e).ok()?;
    let to = serde_json::to_value(to).ok()?;
    let mut n = 0;
    walk(&mut json, &to, &mut n);
    (n == 1).then(|| serde_json::from_value(json).ok()).flatten()
}

/// What the one `Move` onto the battlefield in `e` moves.
fn moved_onto_battlefield(e: &Effect) -> Option<Sel> {
    use serde_json::Value as J;
    fn walk(v: &J, out: &mut Vec<J>) {
        match v {
            J::Object(m) => {
                if let Some(J::Object(mv)) = m.get("Move") {
                    let to_battlefield = mv
                        .get("to")
                        .and_then(|t| t.get("zone"))
                        .and_then(|z| z.as_str())
                        == Some("Battlefield");
                    if let (true, Some(w)) = (to_battlefield, mv.get("what")) {
                        out.push(w.clone());
                    }
                }
                m.values().for_each(|x| walk(x, out));
            }
            J::Array(a) => a.iter().for_each(|x| walk(x, out)),
            _ => {}
        }
    }
    let mut out = Vec::new();
    walk(&serde_json::to_value(e).ok()?, &mut out);
    match out.as_slice() {
        [w] => serde_json::from_value(w.clone()).ok(),
        _ => None,
    }
}

/// "[put/return objects onto/to the battlefield] attached to [recipient][, then ...]": the
/// objects enter attached to it — an Aura that can't legally enchant it, or one attached
/// to something undefined, stays where it is; an Equipment that can't equip it enters
/// unattached (CR 301.5e, 303.4f–i).
fn p_enter_attached(l: &str, b: &mut Builder) -> Option<Effect> {
    if super::zz_probe_ps::disabled() {
        return None;
    }
    let l = end(l);
    if l.matches(" attached to ").count() != 1 {
        return None;
    }
    let (head, after) = l.split_once(" attached to ")?;
    if !(head.ends_with(" the battlefield") || head.ends_with(" under your control")) {
        return None;
    }
    // The recipient runs to the next instruction (", then shuffle").
    let (recipient, tail) = match after.find(", ") {
        Some(i) => (&after[..i], &after[i..]),
        None => (after, ""),
    };
    let saved = (
        b.targets.len(),
        b.it.clone(),
        b.it_player.clone(),
        b.group.clone(),
    );
    let parsed = (|| {
        // A referent means what it meant before the objects were named ("Return up to
        // two target Aura cards ... attached to that creature"); a target is the text's
        // next one.
        let targeted = recipient.starts_with("target ");
        let pre = if targeted {
            None
        } else {
            Some(entry_recipient(recipient, b)?)
        };
        let e = crate::oracle::effects::parse_clause(&format!("{head}{tail}"), b)?;
        let mut to = match pre {
            Some(to) => to,
            None => {
                let it = b.it.clone();
                let to = entry_recipient(recipient, b)?;
                b.it = it;
                to
            }
        };
        // "attached to a creature you control": one it can legally be attached to
        // (Nomad Mythmaker's ruling).
        if let Sel::Choose { filter, .. } = &mut to {
            if let Some(what) = moved_onto_battlefield(&e) {
                *filter = Filter::and(vec![
                    filter.clone(),
                    Filter::CanBeAttachedBy(Box::new(what)),
                ]);
            }
        }
        with_entry_attachment(e, &to)
    })();
    if parsed.is_none() {
        b.targets.truncate(saved.0);
        (b.it, b.it_player, b.group) = (saved.1, saved.2, saved.3);
    }
    parsed
}

inventory::submit! { EffectPattern { name: "attach grammar: [put onto the battlefield] attached to [recipient]", priority: 110, parse: p_enter_attached } }

/// "unattach it at the beginning of the next end step" (Unexpected Request), where "it"
/// is an object an earlier instruction chose and stored ("you may attach an Equipment you
/// control to that creature. If you do, ..."): a delayed triggered ability (CR 603.7)
/// that keeps referring to that object.
fn p_delayed_on_stored(l: &str, b: &mut Builder) -> Option<Effect> {
    if super::zz_probe_ps::disabled() {
        return None;
    }
    let l = end(l);
    let (trigger, inner) = super::triggers_delayed::split_delay(l)?;
    if !matches!(b.it, Sel::Var(_)) || !inner.starts_with("unattach ") {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone());
    let e = crate::oracle::effects::parse_clause(inner, b);
    let ok = matches!(&e, Some(Effect::Unattach { what }) if format!("{what:?}") == format!("{:?}", saved.1));
    if !ok || b.targets.len() != saved.0 {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        return None;
    }
    Some(Effect::DelayedTrigger {
        trigger,
        body: Box::new(Body::effect(e?)),
        once: true,
    })
}

inventory::submit! { EffectPattern { name: "attach grammar: unattach it at the beginning of the next end step", priority: 110, parse: p_delayed_on_stored } }

/// "That player attaches ~ to a land of their choice." (Steam Vines): the player named
/// performs the instruction and chooses the recipient.
fn p_player_attaches(l: &str, b: &mut Builder) -> Option<Effect> {
    if super::zz_probe_ps::disabled() {
        return None;
    }
    let l = end(l);
    let (subject, r) = l.split_once(" attaches ")?;
    let r = r.strip_suffix(" of their choice")?;
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let parsed = (|| {
        let (who, rest) = player_ref(subject, b)?;
        if !end(&rest).is_empty() || matches!(who, PlayerRef::You) {
            return None;
        }
        match p_attach(&format!("attach {r}"), b)? {
            Effect::Attach {
                what,
                to:
                    Sel::Choose {
                        filter,
                        count,
                        up_to,
                        store,
                        ..
                    },
            } => Some(Effect::Attach {
                what,
                to: Sel::Choose {
                    chooser: who,
                    filter,
                    count,
                    up_to,
                    store,
                },
            }),
            _ => None,
        }
    })();
    if parsed.is_none() {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1, saved.2);
    }
    parsed
}

inventory::submit! { EffectPattern { name: "attach grammar: [player] attaches [object] to [object] of their choice", priority: 110, parse: p_player_attaches } }

/// The Equipment unattached by "unattach an Equipment from a creature you control", and
/// the creature it was attached to ("If you do, tap that creature").
const UNATTACHED: Var = vars::USER + 8330;
const UNATTACHED_FROM: Var = vars::USER + 8331;

/// "unattach an Equipment from a creature you control" (Akiri, Fearless Voyager): one
/// chosen as it's performed among those attached to such a permanent; "that creature" is
/// the permanent it was attached to.
fn p_unattach_from(l: &str, b: &mut Builder) -> Option<Effect> {
    if super::zz_probe_ps::disabled() {
        return None;
    }
    let r = end(l).strip_prefix("unattach ")?;
    let r = r.strip_prefix("an ").or_else(|| r.strip_prefix("a "))?;
    let (what, from) = r.split_once(" from ")?;
    let (fa, plural_a, rest_a) = parse_object_phrase(what)?;
    let from = from.strip_prefix("a ").or_else(|| from.strip_prefix("an "))?;
    let (fh, plural_h, rest_h) = parse_object_phrase(from)?;
    if plural_a || plural_h || !end(rest_a).is_empty() || !end(rest_h).is_empty() {
        return None;
    }
    if fa.zone().is_some() || fh.zone().is_some() {
        return None;
    }
    let hosts = Sel::All(Filter::and(vec![fh, Filter::InZone(ZoneKind::Battlefield)]));
    b.it = Sel::Var(UNATTACHED_FROM);
    Some(Effect::Seq(vec![
        Effect::Store {
            var: UNATTACHED,
            sel: Sel::Choose {
                chooser: PlayerRef::You,
                filter: Filter::and(vec![
                    fa,
                    Filter::InZone(ZoneKind::Battlefield),
                    Filter::AttachedToAnyOf(Box::new(hosts)),
                ]),
                count: Value::c(1),
                up_to: false,
                store: None,
            },
        },
        Effect::Store {
            var: UNATTACHED_FROM,
            sel: Sel::HostOf(Box::new(Sel::Var(UNATTACHED))),
        },
        Effect::Unattach {
            what: Sel::Var(UNATTACHED),
        },
    ]))
}

inventory::submit! { EffectPattern { name: "attach grammar: unattach a [attachment] from a [permanent]", priority: 110, parse: p_unattach_from } }

/// "When you lose control of that Equipment, unattach it." (Ogre Geargrabber) after
/// gaining control of an object: a delayed triggered ability (CR 603.7) that triggers the
/// next time its controller loses control of that object.
fn p_when_you_lose_control(l: &str, b: &mut Builder) -> Option<Effect> {
    if super::zz_probe_ps::disabled() {
        return None;
    }
    let r = end(l).strip_prefix("when you lose control of ")?;
    let (obj, inner) = r.split_once(", ")?;
    if !obj.starts_with("that ") || obj.contains(" this turn") {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone());
    let parsed = (|| {
        // "that Equipment": the object the text named (a target).
        let noun = obj.strip_prefix("that ")?;
        let (f, plural, tail) = parse_object_phrase(noun)?;
        if plural || !end(tail).is_empty() {
            return None;
        }
        let what = super::pronoun_groups::singular_it(b);
        let Sel::Target(slot) = what else {
            return None;
        };
        // The target is that kind of object.
        let names = |t: &Filter| format!("{t:?}").contains(&format!("{f:?}"));
        if !matches!(&b.targets.get(slot as usize)?.what, TargetKind::Object(t) if names(t)) {
            return None;
        }
        let effect = crate::oracle::effects::parse_sentence(inner, b)?;
        if b.targets.len() != saved.0 {
            return None;
        }
        let trigger = TriggerCond::LoseControl(Filter::In(Box::new(what)));
        // What the trigger and its effect refer to is captured as it's created.
        let (mut stores, effect) = super::triggers_delayed::capture(&effect)?;
        let (s2, trigger_json) = {
            let wrapped = Effect::DelayedTrigger {
                trigger,
                body: Box::new(Body::effect(Effect::Noop)),
                once: true,
            };
            super::triggers_delayed::capture(&wrapped)?
        };
        for s in s2 {
            if !stores
                .iter()
                .any(|x| format!("{x:?}") == format!("{s:?}"))
            {
                stores.push(s);
            }
        }
        let Effect::DelayedTrigger { trigger, .. } = trigger_json else {
            return None;
        };
        stores.push(Effect::DelayedTrigger {
            trigger,
            body: Box::new(Body::effect(effect)),
            once: true,
        });
        Some(Effect::seq(stores))
    })();
    if parsed.is_none() {
        b.targets.truncate(saved.0);
        b.it = saved.1;
    }
    parsed
}

inventory::submit! { EffectPattern { name: "attach grammar: when you lose control of that [object], [effect] (delayed)", priority: 110, parse: p_when_you_lose_control } }

/// "If an Equipment is put onto the battlefield this way, you may attach it to a creature
/// you control." (Armored Skyhunter) after putting a card from among the looked-at cards
/// onto the battlefield: only an Equipment that was, attached to a creature chosen among
/// those it can equip.
fn f_if_put_this_way_attach(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if super::zz_probe_ps::disabled() {
        return false;
    }
    let l = end(l);
    let Some(r) = l
        .strip_prefix("if an ")
        .or_else(|| l.strip_prefix("if a "))
    else {
        return false;
    };
    let Some((noun, instr)) = r.split_once(" is put onto the battlefield this way, ") else {
        return false;
    };
    let Some((f, false, tail)) = parse_object_phrase(noun) else {
        return false;
    };
    if !end(tail).is_empty() || f.zone().is_some() {
        return false;
    }
    // After a look at the top cards that put some of them onto the battlefield.
    let took = matches!(&*prev, Effect::Dig { take_to, take, .. }
        if take_to.zone == ZoneKind::Battlefield && !matches!(take, Value::Const(0)));
    if !took {
        return false;
    }
    let (may, instr) = match instr.strip_prefix("you may ") {
        Some(x) => (true, x),
        None => (false, instr),
    };
    let Some(r) = instr.strip_prefix("attach it to ") else {
        return false;
    };
    let put = Sel::All(Filter::and(vec![
        f.clone(),
        Filter::In(Box::new(Sel::Var(vars::IT))),
        Filter::InZone(ZoneKind::Battlefield),
    ]));
    let saved = (b.targets.len(), b.it.clone());
    let Some((to, rest)) = recipient(r, &put, b) else {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        return false;
    };
    if !end(&rest).is_empty() || b.targets.len() != saved.0 || !matches!(to, Sel::Choose { .. }) {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        return false;
    }
    let attach = Effect::Attach { what: put, to };
    let then = if may {
        Effect::May {
            who: PlayerRef::You,
            effect: Box::new(attach),
        }
    } else {
        attach
    };
    let old = std::mem::take(prev);
    *prev = Effect::Seq(vec![
        old,
        Effect::If {
            cond: Condition::Exists(Filter::and(vec![
                f,
                Filter::In(Box::new(Sel::Var(vars::IT))),
                Filter::InZone(ZoneKind::Battlefield),
            ])),
            then: Box::new(then),
            otherwise: Box::new(Effect::Noop),
        },
    ]);
    true
}

inventory::submit! { super::FollowupPattern { name: "attach grammar: if an Equipment is put onto the battlefield this way, attach it", priority: 95, apply: f_if_put_this_way_attach } }
