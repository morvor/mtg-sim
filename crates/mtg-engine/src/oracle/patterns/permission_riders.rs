//! Riders on a permission to play cards, about the spells cast or lands played with it
//! (CR 601.3, 400.7a, 400.7h, 400.7i): sentences after the permission that change its terms
//! ([`PlayTerms`], see `permissions.rs`):
//!
//! * "If you cast a spell this way, that creature enters with a finality counter on it.",
//!   "If you cast a spell this way, that artifact enters tapped.", "If you do, it enters
//!   with two +1/+1 counters on it.": a replacement effect for the permanent the spell
//!   becomes (CR 614.1c), created as the spell is cast with the permission (`on_cast`);
//!   "If you do, it enters tapped." for a land played with it too (CR 614.1c);
//! * "If you do, it gains \"...\"" (a land or a spell: `on_land`, `on_cast`), "If you do,
//!   it has affinity for artifacts.", "That spell has undaunted.": the spell (or land) gets
//!   the ability as it's cast (before its total cost is determined, CR 601.2f), and by
//!   CR 400.7a the permanent it becomes keeps it;
//! * "If you cast a spell this way, pay life equal to its mana value rather than pay its
//!   mana cost." (a required alternative cost, CR 118.9b);
//! * "Spells you cast this way cost {2} less to cast." (CR 601.2f);
//! * "Mana of any type can be spent to cast spells this way." (CR 118.14);
//! * "If you cast ~ this way and it would be put into your graveyard, exile it instead."
//!
//! Casting the same card another way (without that permission) doesn't get the rider:
//! the terms belong to the permission the player announces using (CR 601.2).

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

/// The variable that is the spell cast (or land played) while a rider's effect runs.
fn it() -> Sel {
    Sel::Var(vars::IT)
}

/// "that creature enters with a finality counter on it", "it enters tapped", "that
/// artifact enters with an additional +1/+1 counter on it", "it enters with a finality
/// counter on it and is a Vampire in addition to its other types": the replacement effect
/// for the permanent `Var(IT)` (the spell) becomes (CR 614.1c), and whether it's "enters
/// tapped".
fn enters_rider(x: &str) -> Option<(Effect, bool)> {
    let (x, ty) = match x
        .strip_suffix(" in addition to its other types")
        .and_then(|r| r.rsplit_once(" and is a "))
    {
        Some((head, ty)) => {
            let mut t = ty.chars();
            let ty: String = t.next()?.to_uppercase().chain(t).collect();
            if !crate::types::is_creature_type(&ty) {
                return None;
            }
            (head, Some(ty))
        }
        None => (x, None),
    };
    let r = [
        "that creature enters ",
        "that artifact enters ",
        "that permanent enters ",
        "that card enters ",
        "it enters ",
    ]
    .iter()
    .find_map(|p| x.strip_prefix(p))?;
    let (mut parts, tapped) = if r == "tapped" {
        (vec![Effect::EnterTapped], true)
    } else {
        let r = r.strip_prefix("with ")?;
        let (kind, n) = super::etb_replacement_grammar::counters_on(r)?;
        n.as_const()?;
        (vec![Effect::EnterWithCounters { kind, n }], false)
    };
    // "... and is a Vampire": the permanent has that type as it enters (not the spell).
    if let Some(ty) = ty {
        parts.push(Effect::OnEntry(Box::new(Effect::Modify {
            what: Sel::This,
            mods: vec![Modification::AddSubtypes(vec![ty.into()])],
            duration: Duration::Permanent,
        })));
    }
    Some((
        Effect::AddReplacement {
            def: ReplacementDef {
                event: ReplacementEvent::EntersBattlefield(Filter::In(Box::new(it()))),
                action: ReplacementAction::AsEnters(Box::new(Effect::seq(parts))),
                self_replacement: false,
                optional: false,
            },
            duration: Duration::Permanent,
            uses: Some(1),
        },
        tapped,
    ))
}

/// "it has affinity for artifacts", "that spell has undaunted", "it gains flying": a
/// keyword the spell gets as it's cast (CR 601.2f: before its total cost is determined),
/// kept by the permanent it becomes (CR 400.7a).
fn keyword_rider(x: &str) -> Option<Effect> {
    let r = [
        "it has ",
        "that spell has ",
        "it gains ",
        "that spell gains ",
    ]
    .iter()
    .find_map(|p| x.strip_prefix(p))?;
    if r.contains('"') || r.contains(" until ") {
        return None;
    }
    let mods = crate::oracle::effects::keyword_mods(r)?;
    Some(Effect::Modify {
        what: it(),
        mods,
        duration: Duration::Permanent,
    })
}

/// `f` without the requirement to be a spell (the object is the spell being cast).
fn without_spell(f: Filter) -> Filter {
    match f {
        Filter::Spell => Filter::Any,
        Filter::And(v) => Filter::and(v.into_iter().map(without_spell).collect()),
        other => other,
    }
}

/// Adds `e` (done as a spell with the qualities `f` is cast with the permission) to the
/// permission's `on_cast`.
fn add_on_cast(terms: &mut PlayTerms, f: Filter, e: Effect) {
    let f = without_spell(f);
    let e = if matches!(f, Filter::Any) {
        e
    } else {
        Effect::If {
            cond: Condition::SelMatches(it(), f),
            then: Box::new(e),
            otherwise: Box::new(Effect::Noop),
        }
    };
    terms.on_cast = Some(Box::new(match terms.on_cast.take() {
        Some(mine) => Effect::seq(vec![*mine, e]),
        None => e,
    }));
}

/// A rider sentence (lowercase) after a permission that lets `spells` be cast and `lands`
/// be played: adds it to `terms`. False if it isn't one (the terms are then unchanged).
pub(crate) fn apply(s: &str, terms: &mut PlayTerms, spells: bool, lands: bool, b: &Builder) -> bool {
    let l = end(s.trim()).trim();
    let mut t = terms.clone();
    if !apply_inner(l, &mut t, spells, lands, b) {
        return false;
    }
    *terms = t;
    true
}

fn apply_inner(l: &str, terms: &mut PlayTerms, spells: bool, lands: bool, b: &Builder) -> bool {
    // "Spells you cast this way cost {2} less to cast."
    if let Some(r) = l
        .strip_prefix("spells you cast this way cost {")
        .and_then(|r| r.strip_suffix("} less to cast"))
    {
        let Ok(n) = r.parse::<u32>() else {
            return false;
        };
        if !spells || n == 0 {
            return false;
        }
        terms.cost_reduction += n;
        return true;
    }
    // "You may cast Equipment spells this way without paying their mana costs.", "you may
    // play cards this way without paying their mana costs".
    if let Some(r) = l
        .strip_prefix("you may cast ")
        .or_else(|| l.strip_prefix("you may play "))
        .and_then(|r| r.strip_suffix(" this way without paying their mana costs"))
    {
        if !spells || terms.free_option.is_some() || terms.alt_cost.is_some() {
            return false;
        }
        let f = if r == "cards" || r == "spells" {
            Filter::Any
        } else {
            let Some(card) = r.strip_suffix(" spells") else {
                return false;
            };
            let phrase = format!("{card} cards");
            let Some((f, true, rest)) = crate::oracle::phrases::parse_object_phrase(&phrase)
            else {
                return false;
            };
            if !end(rest).trim().is_empty() {
                return false;
            }
            f
        };
        terms.free_option = Some(f);
        return true;
    }
    // "If that spell would be put into a graveyard, put it on the bottom of its owner's
    // library instead."
    if matches!(
        l,
        "if that spell would be put into a graveyard, put it on the bottom of its owner's library instead"
            | "if a spell cast this way would be put into a graveyard, put it on the bottom of its owner's library instead"
    ) {
        if !spells || terms.graveyard_instead.is_some() || terms.exile_instead {
            return false;
        }
        let mut to = Destination::zone(ZoneKind::Library);
        to.position = LibraryPosition::Bottom;
        terms.graveyard_instead = Some(to);
        return true;
    }
    // "Mana of any type can be spent to cast spells this way."
    if matches!(
        l,
        "mana of any type can be spent to cast spells this way"
            | "mana of any type can be spent to cast a spell this way"
            | "mana of any type can be spent to cast that spell"
    ) {
        if !spells || terms.spend_any_type {
            return false;
        }
        terms.spend_any_type = true;
        return true;
    }
    // "If you cast ~ this way and it would be put into your graveyard, exile it instead."
    if matches!(
        l,
        "if you cast ~ this way and it would be put into your graveyard, exile it instead"
            | "if you cast ~ this way and it would be put into a graveyard, exile it instead"
    ) {
        if !spells || terms.exile_instead {
            return false;
        }
        terms.exile_instead = true;
        return true;
    }
    // "That spell has undaunted." (the spell cast with the permission).
    if l.starts_with("that spell has ") {
        let Some(e) = keyword_rider(l) else {
            return false;
        };
        if !spells {
            return false;
        }
        add_on_cast(terms, Filter::Any, e);
        return true;
    }
    // "If you cast a [kind] spell this way, X", "If you do, X".
    let (kind, x, either) = if let Some(x) = l.strip_prefix("if you do, ") {
        (Filter::Any, x, true)
    } else if let Some((k, x)) = l
        .strip_prefix("if you cast ")
        .and_then(|r| r.split_once(" this way, "))
    {
        let Some(f) = super::played_this_way::spell_kind(k) else {
            return false;
        };
        (f, x, false)
    } else {
        return false;
    };
    // "pay life equal to its mana value rather than pay its mana cost" (CR 118.9b).
    if x == "pay life equal to its mana value rather than pay its mana cost" && !either {
        if !spells || terms.alt_cost.is_some() || !matches!(kind, Filter::Any) {
            return false;
        }
        terms.alt_cost = Some(Cost {
            mana: None,
            parts: vec![CostPart::PayLife(Value::ManaValueOf(Box::new(Sel::This)))],
        });
        return true;
    }
    if !spells {
        return false;
    }
    if let Some((e, tapped)) = enters_rider(x) {
        // "If you do, it enters tapped": a land played with it too.
        if either && lands {
            if !tapped {
                return false;
            }
            terms.lands_enter_tapped = true;
        }
        add_on_cast(terms, kind, e);
        return true;
    }
    if let Some(e) = keyword_rider(x) {
        if either && lands {
            terms.on_land = Some(Box::new(e.clone()));
        }
        add_on_cast(terms, kind, e);
        return true;
    }
    // "it gains \"...\"" and other effects on the spell (and a land played with it).
    let Some(e) = super::played_this_way::spell_effect(x, b, it()) else {
        return false;
    };
    if either && lands {
        terms.on_land = Some(Box::new(e.clone()));
    }
    add_on_cast(terms, kind, e);
    true
}

/// "When you do, X" after a static permission: "When you play a card this way, X" (or
/// "When you cast a spell this way, X" for a permission to cast spells), see
/// `played_this_way`.
pub(crate) fn when_you_do(s: &str, spells: bool, lands: bool) -> String {
    let t = s.trim();
    let lower = t.to_lowercase();
    if lower.starts_with("when you do, ") {
        let x = &t["when you do, ".len()..];
        if lands && spells {
            return format!("When you play a card this way, {x}");
        }
        if spells {
            return format!("When you cast a spell this way, {x}");
        }
    }
    t.to_string()
}

/// After an effect that gives a permission ("{T}: Until end of turn, you may cast creature
/// spells from your graveyard by foraging in addition to paying their other costs. If you
/// cast a spell this way, ...", "You may cast an artifact spell from among them this turn.
/// If you do, it has affinity for artifacts."): the rider becomes part of its terms.
fn f_rider(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l.trim()).trim().to_lowercase();
    // "If ~ was cast from a graveyard, you may play cards this way without paying their
    // mana costs.": the permission with that term if the condition holds as the effect
    // resolves.
    if let Some((c, r)) = l
        .strip_prefix("if ")
        .and_then(|r| r.split_once(", "))
        .filter(|(_, r)| r.starts_with("you may "))
    {
        let Some(cond) = crate::oracle::statics::parse_condition(c, b.ctx) else {
            return false;
        };
        let target = match prev {
            Effect::Seq(v) => match v.last_mut() {
                Some(x) => x,
                None => return false,
            },
            other => other,
        };
        let mut copy = target.clone();
        let lands = last_allows_lands(&copy);
        let Some(terms) = terms_of_last(&mut copy) else {
            return false;
        };
        let lands = lands && !terms.spells_only;
        if !apply(r, terms, true, lands, b) {
            return false;
        }
        let original = std::mem::take(target);
        *target = Effect::If {
            cond,
            then: Box::new(copy),
            otherwise: Box::new(original),
        };
        return true;
    }
    let mut copy = prev.clone();
    let lands = last_allows_lands(&copy);
    let Some(terms) = terms_of_last(&mut copy) else {
        return false;
    };
    let lands = lands && !terms.spells_only;
    if !apply(&l, terms, true, lands, b) {
        return false;
    }
    *prev = copy;
    true
}

/// Whether the last permission the effect gives lets lands be played.
fn last_allows_lands(e: &Effect) -> bool {
    match e {
        Effect::AddPlayerEffect {
            effect: PlayerModification::PlayPermission(pp),
            ..
        } => pp.lands,
        Effect::Seq(v) => v.last().is_some_and(last_allows_lands),
        Effect::If { then, .. } => last_allows_lands(then),
        Effect::WithPlayTerms { effect, .. } => last_allows_lands(effect),
        _ => true,
    }
}

/// The terms of the last permission the effect gives (a permission given without terms
/// gets them).
fn terms_of_last(e: &mut Effect) -> Option<&mut PlayTerms> {
    if matches!(e, Effect::GrantPlayPermission { .. }) {
        let grant = std::mem::take(e);
        *e = Effect::WithPlayTerms {
            terms: PlayTerms::default(),
            effect: Box::new(grant),
        };
    }
    let mine = matches!(e, Effect::Seq(_))
        || matches!(e, Effect::If { otherwise, .. } if matches!(**otherwise, Effect::Noop));
    if !mine {
        return super::permission_grammar::last_terms(e);
    }
    match e {
        Effect::Seq(v) => v.last_mut().and_then(terms_of_last),
        Effect::If { then, .. } => terms_of_last(then),
        _ => None,
    }
}

/// "At the beginning of your next upkeep, you may cast that card without paying its mana
/// cost. If that spell would be put into your graveyard, exile it instead." (Power Pack):
/// the delayed triggered ability's spell (CR 603.7, 614.1a).
fn f_delayed_cast_exiled(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let l = end(l.trim()).trim().to_lowercase();
    if !matches!(
        l.as_str(),
        "if that spell would be put into your graveyard, exile it instead"
            | "if that spell would be put into a graveyard, exile it instead"
    ) {
        return false;
    }
    fn last_delayed(e: &mut Effect) -> Option<&mut Effect> {
        if matches!(e, Effect::DelayedTrigger { .. }) {
            return Some(e);
        }
        match e {
            Effect::Seq(v) => v.last_mut().and_then(last_delayed),
            _ => None,
        }
    }
    fn ends_casting(e: &Effect) -> bool {
        match e {
            Effect::CastCard { .. } => true,
            Effect::Seq(v) => v.last().is_some_and(ends_casting),
            _ => false,
        }
    }
    let Some(Effect::DelayedTrigger { body, .. }) = last_delayed(prev) else {
        return false;
    };
    if !ends_casting(&body.effect) {
        return false;
    }
    let replacement = Effect::AddReplacement {
        def: ReplacementDef {
            event: ReplacementEvent::ZoneChange {
                filter: Filter::In(Box::new(it())),
                // (Only the spell: "it" is still the card if none was cast.)
                from: Some(ZoneKind::Stack),
                to: Some(ZoneKind::Graveyard),
            },
            action: ReplacementAction::MoveInstead(Destination::zone(ZoneKind::Exile)),
            self_replacement: false,
            optional: false,
        },
        duration: Duration::Permanent,
        uses: None,
    };
    let old = std::mem::take(&mut body.effect);
    body.effect = Effect::seq(vec![old, replacement]);
    true
}

inventory::submit! { FollowupPattern { name: "permission riders: the spell a delayed trigger casts is exiled instead", priority: 40, apply: f_delayed_cast_exiled } }

inventory::submit! { FollowupPattern { name: "permission riders: a rider on the spells cast with an effect's permission", priority: 96, apply: f_rider } }
