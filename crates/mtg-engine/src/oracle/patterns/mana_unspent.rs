//! Unspent mana (CR 500.5, 703.4q): "You don't lose unspent red mana as steps and phases
//! end." (Electro, Leyline Tyrant), "Players don't lose unspent mana as steps and phases
//! end." (Upwelling), "If you would lose unspent mana, that mana becomes colorless
//! instead." (Horizon Stone, Kruphix), and "Until end of turn, you don't lose unspent red
//! mana as steps and phases end." (The Last Agni Kai). The pool is emptied by
//! `mana_abilities::empty_pool`, which reads these player modifications.

use super::{EffectPattern, StaticPattern};
use crate::ability::*;
use crate::mana::ManaType;
use crate::mana_abilities::{KEEP_UNSPENT_MANA, UNSPENT_MANA_BECOMES};
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;
use crate::types::Color;

/// "unspent mana" / "unspent red mana" → the modification name.
fn keep_mod(what: &str) -> Option<PlayerModification> {
    let what = what.strip_suffix(" as steps and phases end")?;
    let name = if what == "unspent mana" {
        KEEP_UNSPENT_MANA.to_string()
    } else {
        let c = what.strip_prefix("unspent ")?.strip_suffix(" mana")?;
        let t = ManaType::from_color(Color::from_word(c)?);
        format!("{KEEP_UNSPENT_MANA} {t:?}")
    };
    Some(PlayerModification::Custom(name.into()))
}

/// "[players] don't lose unspent [color] mana as steps and phases end".
fn keep_static(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l);
    let (who, rest) = [
        ("you don't lose ", PlayerFilter::You),
        ("players don't lose ", PlayerFilter::Any),
        ("each player doesn't lose ", PlayerFilter::Any),
    ]
    .into_iter()
    .find_map(|(p, f)| l.strip_prefix(p).map(|r| (f, r)))?;
    let effect = keep_mod(rest)?;
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::PlayerEffect {
            affected: who,
            effect,
        })),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "mana: don't lose unspent mana", priority: 55, parse: keep_static } }

/// "if you would lose unspent mana, that mana becomes colorless instead" (a replacement
/// effect, CR 614.1a).
fn becomes_static(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l)
        .strip_prefix("if you would lose unspent mana, that mana becomes ")?
        .strip_suffix(" instead")?;
    let t = if r == "colorless" {
        ManaType::C
    } else {
        ManaType::from_color(Color::from_word(r)?)
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::PlayerEffect {
            affected: PlayerFilter::You,
            effect: PlayerModification::Custom(format!("{UNSPENT_MANA_BECOMES} {t:?}").into()),
        })),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "mana: unspent mana becomes colorless", priority: 55, parse: becomes_static } }

/// "until end of turn, you don't lose unspent red mana as steps and phases end".
fn keep_until_end_of_turn(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("until end of turn, you don't lose ")?;
    Some(Effect::AddPlayerEffect {
        who: PlayerRef::You,
        effect: keep_mod(r)?,
        duration: Duration::EndOfTurn,
    })
}

inventory::submit! { EffectPattern { name: "mana: until end of turn, don't lose unspent mana", priority: 55, parse: keep_until_end_of_turn } }
