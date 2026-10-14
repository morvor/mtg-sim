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

/// After an effect that gives a permission ("{T}: Until end of turn, you may cast creature
/// spells from your graveyard by foraging in addition to paying their other costs. If you
/// cast a spell this way, ...", "You may cast an artifact spell from among them this turn.
/// If you do, it has affinity for artifacts."): the rider becomes part of its terms.
fn f_rider(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l.trim()).trim().to_lowercase();
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

inventory::submit! { FollowupPattern { name: "permission riders: a rider on the spells cast with an effect's permission", priority: 96, apply: f_rider } }
