//! Combat damage assignment rules (CR 510.1), as rules rather than abilities:
//!
//! * "For each creature you control with no abilities, you may have that creature assign
//!   its combat damage as though it weren't blocked." (Ruxa, Patient Professor; Zilortha;
//!   with "As long as ~ is attacking, " Siege Behemoth): its controller chooses for each
//!   such creature just before it assigns its combat damage (Siege Behemoth ruling);
//! * "Enchanted creature's controller may have it assign its combat damage as though it
//!   weren't blocked." (Indomitable Might);
//! * "X target blocked creatures assign their combat damage this turn as though they
//!   weren't blocked." (Outmaneuver) — no choice;
//! * "You may have ~ assign its combat damage this turn as though it weren't blocked."
//!   (Gurzigost) — chosen as the ability resolves (Gurzigost ruling);
//! * "~ assigns no combat damage this combat." (Master of Cruelties; CR 510.1a);
//! * "~ can block an additional creature each combat for each Equipment attached to ~."
//!   (Kemba's Legion; counted as blockers are declared).
//!
//! See `Restriction::AssignAsThoughUnblocked` and `kw::assign_as_though_unblocked`.

use super::{EffectPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::effects::{object_ref, Builder};
use crate::oracle::phrases::{end, parse_object_phrase};
use crate::oracle::CompileContext;

fn static_ability(r: Restriction, cond: Option<Condition>, text: &str) -> Ability {
    let mut s = StaticAbility::new(StaticEffect::Restriction(r));
    s.condition = cond;
    AbilityDef::new(AbilityKind::Static(s), text)
}

/// Whether a filter is about objects the source's controller controls ("creature you
/// control"): "you may have that creature ..." is then the creature's controller's choice.
fn yours(f: &Filter) -> bool {
    match f {
        Filter::ControlledBy(PlayerRel::You) => true,
        Filter::And(v) => v.iter().any(yours),
        _ => false,
    }
}

/// "[as long as ~ is attacking, ]for each [creature you control ...], you may have that
/// creature assign its combat damage as though it weren't blocked", "enchanted creature's
/// controller may have it assign its combat damage as though it weren't blocked".
fn may_assign_unblocked(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l.trim());
    if let Some(attached) = l.strip_suffix(
        "'s controller may have it assign its combat damage as though it weren't blocked",
    ) {
        if !matches!(attached, "enchanted creature" | "equipped creature") {
            return None;
        }
        return Some(vec![static_ability(
            Restriction::AssignAsThoughUnblocked {
                what: Filter::AttachedToSource,
                may: true,
            },
            None,
            text,
        )]);
    }
    let (cond, rest) = match l.strip_prefix("as long as ") {
        Some(r) => {
            let (c, rest) = r.split_once(", ")?;
            let (cond, _) =
                super::statics_conditions::parse_static_condition(c, Some(&Sel::This), ctx)?;
            (Some(cond), rest)
        }
        None => (None, l),
    };
    let noun = rest.strip_prefix("for each ")?.strip_suffix(
        ", you may have that creature assign its combat damage as though it weren't blocked",
    )?;
    let (f, plural, tail) = parse_object_phrase(noun)?;
    if plural || !end(tail).is_empty() || !yours(&f) {
        return None;
    }
    Some(vec![static_ability(
        Restriction::AssignAsThoughUnblocked { what: f, may: true },
        cond,
        text,
    )])
}

inventory::submit! { StaticPattern { name: "combat damage rules: may assign combat damage as though unblocked", priority: 110, parse: may_assign_unblocked } }

/// "~ can block an additional creature each combat for each Equipment attached to ~".
fn extra_blocks_counted(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let each =
        end(l.trim()).strip_prefix("~ can block an additional creature each combat for each ")?;
    let n = super::statics::parse_for_each(each, Some(&Sel::This))?;
    Some(vec![static_ability(
        Restriction::ExtraBlocksCounted {
            blocker: Filter::Source,
            n,
        },
        None,
        text,
    )])
}

inventory::submit! { StaticPattern { name: "combat damage rules: an additional block for each", priority: 110, parse: extra_blocks_counted } }

/// The objects of a resolving effect's subject: specific objects, or the source when the
/// text names it.
fn subject(s: &str, b: &mut Builder) -> Option<Filter> {
    let s = s.trim();
    let (what, rest) = object_ref(s, b)?;
    if !end(&rest).is_empty()
        || matches!(what, Sel::All(_) | Sel::Players(_) | Sel::None)
        || (matches!(what, Sel::This) && !(s.starts_with('~') || s == "it"))
        || super::oracle_hardening_referents::is_no_referent(&what)
    {
        return None;
    }
    Some(Filter::In(Box::new(what)))
}

/// "X target blocked creatures assign their combat damage this turn as though they
/// weren't blocked", "(you may) have ~ assign its combat damage this turn as though it
/// weren't blocked", "~ assigns no combat damage this combat".
fn assign_effects(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    if let Some(s) = l
        .strip_suffix(" assigns no combat damage this combat")
        .or_else(|| l.strip_suffix(" assign no combat damage this combat"))
    {
        let f = subject(s, b)?;
        return Some(Effect::AddRestriction {
            restriction: Restriction::AssignsNoCombatDamage(f),
            duration: Duration::EndOfCombat,
        });
    }
    let (may, l) = match l.strip_prefix("you may ") {
        Some(r) => (true, r),
        None => (false, l),
    };
    let (s, have) = if let Some(s) = l
        .strip_suffix(" assign their combat damage this turn as though they weren't blocked")
        .or_else(|| {
            l.strip_suffix(" assigns its combat damage this turn as though it weren't blocked")
        }) {
        (s, false)
    } else if let Some(s) = l.strip_prefix("have ").and_then(|r| {
        r.strip_suffix(" assign its combat damage this turn as though it weren't blocked")
    }) {
        (s, true)
    } else {
        return None;
    };
    // "you may have ~ ...": chosen as the effect resolves (Gurzigost ruling).
    if may && !have {
        return None;
    }
    let f = subject(s, b)?;
    let e = Effect::AddRestriction {
        restriction: Restriction::AssignAsThoughUnblocked {
            what: f,
            may: false,
        },
        duration: Duration::EndOfTurn,
    };
    Some(if may {
        Effect::May {
            who: PlayerRef::You,
            effect: Box::new(e),
        }
    } else {
        e
    })
}

inventory::submit! { EffectPattern { name: "combat damage rules: assign as though unblocked this turn / no combat damage this combat", priority: 90, parse: assign_effects } }
