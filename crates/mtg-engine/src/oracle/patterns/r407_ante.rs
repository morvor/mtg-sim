//! Oracle patterns for ante cards (CR 407): the reminder "Remove this card from your deck
//! before playing if you're not playing for ante." (CR 407.3), "ante the top card of your
//! library", "each player [may] ante[s] the top card of their library", and Darkpact's
//! "You own target card in the ante. Exchange that card with the top card of your
//! library."

use super::{AbilityPattern, EffectPattern};
use crate::ability::*;
use crate::ante::{ANTE_CARD, ANTE_TOP, EXCHANGE_WITH_TOP, GAIN_OWNERSHIP};
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

/// "Remove this card from your deck before playing if you're not playing for ante." (on
/// permanents and on instants and sorceries alike).
fn ante_reminder(block: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let text = block.trim();
    let l = text.to_lowercase().replace('’', "'");
    if end(&l) != "remove ~ from your deck before playing if you're not playing for ante" {
        return None;
    }
    let mut s = StaticAbility::new(StaticEffect::Custom(ANTE_CARD.into()));
    // It matters for deck construction and for bringing the card into the game.
    s.zone = FunctionZone::Anywhere;
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { AbilityPattern { name: "r407 ante card reminder", priority: 100, parse: ante_reminder } }

/// "ante the top card of your library", "each player antes the top card of their
/// library", "each player may ante the top card of their library", and a list ending with
/// one of those ("discard your hand, ante the top card of your library").
fn ante_top(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if let Some((first, last)) = l.rsplit_once(", ") {
        if !last.contains("ante the top card") {
            return None;
        }
        let a = crate::oracle::effects::parse_clause(first, b)?;
        let c = ante_top(last, b)?;
        return Some(Effect::seq(vec![a, c]));
    }
    let ante = Effect::Custom(ANTE_TOP.into());
    match l {
        "ante the top card of your library" => Some(ante),
        "each player antes the top card of their library" => Some(Effect::ForEachPlayer {
            who: PlayerRef::EachPlayer,
            effect: Box::new(ante),
        }),
        "each player may ante the top card of their library" => Some(Effect::ForEachPlayer {
            who: PlayerRef::EachPlayer,
            effect: Box::new(Effect::May {
                who: PlayerRef::Iterated,
                effect: Box::new(ante),
            }),
        }),
        _ => None,
    }
}

inventory::submit! { EffectPattern { name: "r407 ante the top card", priority: 100, parse: ante_top } }

/// Darkpact: "you own target card in the ante" (CR 407.3: only ante cards change a
/// card's owner).
fn own_target_in_ante(l: &str, b: &mut Builder) -> Option<Effect> {
    if end(l) != "you own target card in the ante" {
        return None;
    }
    let spec = TargetSpec::object(
        Filter::and(vec![Filter::Card, Filter::InZone(ZoneKind::Ante)]),
        "target card in the ante",
    );
    let slot = b.add_target(spec, "target card in the ante");
    Some(Effect::Seq(vec![
        Effect::Store {
            var: vars::IT,
            sel: Sel::Target(slot),
        },
        Effect::Custom(GAIN_OWNERSHIP.into()),
    ]))
}

inventory::submit! { EffectPattern { name: "r407 own target card in the ante", priority: 100, parse: own_target_in_ante } }

/// Darkpact: "exchange that card with the top card of your library".
fn exchange_with_top(l: &str, b: &mut Builder) -> Option<Effect> {
    if end(l) != "exchange that card with the top card of your library" {
        return None;
    }
    let Sel::Target(slot) = b.it else {
        return None;
    };
    Some(Effect::Seq(vec![
        Effect::Store {
            var: vars::IT,
            sel: Sel::Target(slot),
        },
        Effect::Custom(EXCHANGE_WITH_TOP.into()),
    ]))
}

inventory::submit! { EffectPattern { name: "r407 exchange with the top card", priority: 100, parse: exchange_with_top } }
