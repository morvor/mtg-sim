//! "Whenever you cast an instant or sorcery spell from your hand during an opponent's turn,
//! exile that card with three time counters on it instead of putting it into your
//! graveyard as it resolves. Then if the exiled card doesn't have suspend, it gains
//! suspend." (Gandalf of the Secret Fire).
//!
//! The triggered ability gives the spell a replacement effect of where it goes as it
//! resolves (CR 608.2n, 614.1a): it's exiled with the time counters instead of being put
//! into its owner's graveyard. If it's countered instead, it goes to the graveyard as
//! usual. Then, if the card in exile doesn't have suspend, it gains suspend (CR 702.62a:
//! suspend's triggered abilities function in exile; the card is suspended, CR 702.62b).
//! The ability is a marker the spell has ([`marker`]); see `kw/plot.rs` for the same
//! mechanism without counters (Lilah, Undefeated Slickshot).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::*;
use crate::types::*;
use smol_str::SmolStr;

pub struct SuspendAsItResolves;

/// Prefix of the marker `StaticEffect::Custom`, followed by "[kind]:[n]" and, for "Then if
/// the exiled card doesn't have suspend, it gains suspend.", [`GAINS_SUSPEND`].
const PREFIX: &str = "resolve exile with counters:";
const GAINS_SUSPEND: &str = ":gains suspend";

/// The marker ability given to a spell: "exile this card with `n` `kind` counters on it
/// instead of putting it into your graveyard as it resolves[. Then if the exiled card
/// doesn't have suspend, it gains suspend]".
pub fn marker(kind: &str, n: u32, gains_suspend: bool) -> Modification {
    let name = format!(
        "{PREFIX}{kind}:{n}{}",
        if gains_suspend { GAINS_SUSPEND } else { "" }
    );
    let mut s = StaticAbility::new(StaticEffect::Custom(SmolStr::new(&name)));
    s.zone = FunctionZone::Stack;
    Modification::AddAbility(AbilityDef::new(AbilityKind::Static(s), name))
}

/// The parameters of a marker's name: (counter kind, number, gains suspend).
pub fn parse_marker(name: &str) -> Option<(CounterKind, u32, bool)> {
    let r = name.strip_prefix(PREFIX)?;
    let (r, suspend) = match r.strip_suffix(GAINS_SUSPEND) {
        Some(r) => (r, true),
        None => (r, false),
    };
    let (kind, n) = r.rsplit_once(':')?;
    Some((kind.into(), n.parse().ok()?, suspend))
}

/// The marker abilities the spell has.
fn markers(g: &Game, spell: ObjectId) -> Vec<(CounterKind, u32, bool)> {
    g.obj(spell)
        .chars
        .abilities
        .iter()
        .filter_map(|a| match &a.kind {
            AbilityKind::Static(s) => match &s.effect {
                StaticEffect::Custom(n) => parse_marker(n),
                _ => None,
            },
            _ => None,
        })
        .collect()
}

impl KeywordRules for SuspendAsItResolves {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn global_resolved_destination(
        &self,
        g: &Game,
        spell: ObjectId,
    ) -> Option<(Zone, LibraryPosition)> {
        (!markers(g, spell).is_empty()).then_some((Zone::Exile, LibraryPosition::Top))
    }

    /// The time counters it's exiled with, then suspend.
    fn global_after_spell_resolved(&self, g: &mut Game, spell: ObjectId, new: ObjectId) {
        if !g.is_live(new) || g.obj(new).zone != Zone::Exile {
            return;
        }
        let controller = g.obj(spell).controller;
        for (kind, n, suspend) in markers(g, spell) {
            g.put_counters(
                Entity::Object(new),
                &kind,
                n,
                crate::event_causes::CounterPut {
                    source: Some(spell),
                    by: Some(controller),
                    origin: crate::events::CounterOrigin::Effect,
                },
            );
            if suspend {
                let mut ctx = Ctx::new(None, controller);
                ctx.set_var(vars::IT, vec![Entity::Object(new)]);
                let e = crate::oracle::patterns::r702_062_gains_suspend::gains_suspend(Sel::Var(
                    vars::IT,
                ));
                g.exec(&e, &mut ctx);
            }
        }
    }
}

inventory::submit! { KeywordRegistration(&SuspendAsItResolves) }
