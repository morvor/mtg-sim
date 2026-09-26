//! Redirection of damage to or from groups (CR 614.9):
//!
//! Static abilities:
//! - "All damage that would be dealt to you is dealt to ~ instead." (Empyrial Archangel)
//! - "All damage that would be dealt to you is dealt to enchanted creature instead."
//!   (Pariah), "... equipped creature instead." (Pariah's Shield)
//! - "All damage that would be dealt to you and other permanents you control is dealt to
//!   ~ instead." (Ancient Adamantoise)
//! - "All damage that would be dealt to enchanted creature is dealt to its controller
//!   instead." (Treacherous Link)
//! - "As long as ~ is untapped, all damage that would be dealt to you by unblocked
//!   creatures is dealt to ~ instead." (Veteran Bodyguard)
//!
//! One-shot effects (this turn):
//! - "All damage that would be dealt to target creature this turn is dealt to you
//!   instead." (Sivvi's Valor)
//! - "{T}: All combat damage that would be dealt to you by unblocked creatures this turn
//!   is dealt to ~ instead." (Kjeldoran Royal Guard)
//! - "All combat damage that would be dealt to you this turn is dealt to target attacking
//!   creature instead." (Turn the Tables)
//!
//! The recipient of a static redirection is determined as the damage would be dealt; a
//! one-shot effect's recipient is locked in as the effect is created
//! (`prevention::lock_def`). Damage isn't redirected to something that's no longer a
//! creature, planeswalker, or battle on the battlefield (CR 614.9, see
//! `Game::valid_damage_recipient`).

use super::replacements_prevent_groups::{damage_clause, def, groups_only, Clause, Targets};
use super::{EffectPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

/// The single object "its" can refer to in "is dealt to its controller instead": the
/// enchanted creature the damage would be dealt to, or the clause's one target.
fn its(c: &Clause, t: &Targets) -> Option<Sel> {
    let to_attached = matches!(
        c.to.as_ref().map(|p| (&p.players, &p.objects)),
        Some((None, Some(Filter::AttachedToSource)))
    );
    if to_attached && t.specs.is_empty() {
        return Some(Sel::AttachedTo);
    }
    if t.specs.len() == 1 {
        return Some(Sel::Target(t.base? as u8));
    }
    None
}

/// "~", "enchanted creature", "you", "its controller", "that spell's controller",
/// "target [attacking] creature" (one-shot effects).
fn destination(s: &str, c: &Clause, t: &mut Targets) -> Option<Sel> {
    let s = s.trim();
    Some(match s {
        "~" | "this creature" => Sel::This,
        "enchanted creature" | "equipped creature" => Sel::AttachedTo,
        "you" => Sel::Players(PlayerRef::You),
        "its controller" | "that spell's controller" | "that creature's controller" => {
            Sel::Players(PlayerRef::ControllerOf(Box::new(its(c, t)?)))
        }
        _ => {
            let base = t.base?;
            let (spec, rest) = parse_target(s)?;
            if !rest.trim().is_empty()
                || spec.min != 1
                || !matches!(spec.max, Value::Const(1))
                || !matches!(spec.what, TargetKind::Object(_))
            {
                return None;
            }
            let slot = (base + t.specs.len()) as u8;
            t.specs.push((spec, s.to_string()));
            Sel::Target(slot)
        }
    })
}

/// "all [combat] damage that would be dealt ... is dealt to D instead".
fn redirect(l: &str, t: &mut Targets) -> Option<(Clause, ReplacementAction)> {
    let r = end(l).strip_prefix("all ")?;
    let (left, right) = r.split_once(" is dealt to ")?;
    let c = damage_clause(left, t)?;
    let d = destination(right.strip_suffix(" instead")?, &c, t)?;
    Some((c, ReplacementAction::Redirect(d)))
}

fn s_redirect(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let mut t = Targets {
        base: None,
        specs: vec![],
    };
    let (c, action) = redirect(l, &mut t)?;
    if c.this_turn || c.during_your_turn {
        return None;
    }
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(def(
            &c, action,
        )))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "replacements: all damage that would be dealt to X is dealt to Y instead", priority: 70, parse: s_redirect } }

fn p_redirect(l: &str, b: &mut Builder) -> Option<Effect> {
    let mut t = Targets {
        base: Some(b.targets.len()),
        specs: vec![],
    };
    let (c, action) = redirect(l, &mut t)?;
    if !c.this_turn || c.during_your_turn || !groups_only(&c) {
        return None;
    }
    for (spec, text) in t.specs {
        b.add_target(spec, &text);
    }
    Some(Effect::AddReplacement {
        def: def(&c, action),
        duration: Duration::EndOfTurn,
        uses: None,
    })
}

inventory::submit! { EffectPattern { name: "replacements: all damage that would be dealt this turn is dealt to Y instead", priority: 70, parse: p_redirect } }
