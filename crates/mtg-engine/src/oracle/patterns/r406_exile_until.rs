//! Exiling cards from the top of a library until a card with some quality is exiled:
//! "Exile cards from the top of your library until you exile a nonland card.", "Target
//! opponent exiles cards from the top of their library until they exile a nonland card."
//! Every card is exiled (CR 406); "that card" is the last one (`vars::IT`). Followed by
//! what may be done with it: "You may cast that card without paying its mana cost.",
//! "Until end of turn, you may cast that card without paying its mana cost." (Stolen
//! Goods, Nicol Bolas, God-Pharaoh): a permission its controller may use this turn.

use super::card_flow_search::card_filter;
use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

fn exile_until(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (who, desc) = if let Some(r) =
        l.strip_prefix("exile cards from the top of your library until you exile ")
    {
        (PlayerRef::You, r)
    } else if let Some(r) = l.strip_prefix(
        "target opponent exiles cards from the top of their library until they exile ",
    ) {
        let slot = b.add_target(
            TargetSpec::player(PlayerFilter::Opponent, "target opponent"),
            "target opponent",
        );
        b.it_player = PlayerRef::Target(slot);
        (PlayerRef::Target(slot), r)
    } else {
        return None;
    };
    let desc = desc.strip_prefix("a ").or_else(|| desc.strip_prefix("an "))?;
    let filter = card_filter(desc, b)?;
    b.it = Sel::Var(vars::IT);
    let exile = Destination::zone(ZoneKind::Exile);
    Some(Effect::RevealUntil {
        who,
        filter: Filter::And(vec![Filter::Card, filter]),
        found_to: exile.clone(),
        rest_to: exile,
    })
}

inventory::submit! { EffectPattern { name: "r406 exile cards until you exile a [card]", priority: 90, parse: exile_until } }

/// "Until end of turn, you may cast that card without paying its mana cost." after
/// exiling it: its controller may cast it this turn (CR 601.2, 118.9).
fn until_end_of_turn_cast_it_free(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if l != "until end of turn, you may cast that card without paying its mana cost" {
        return None;
    }
    if !matches!(b.it, Sel::Var(vars::IT)) {
        return None;
    }
    Some(Effect::GrantPlayPermission {
        who: PlayerRef::You,
        what: Sel::Var(vars::IT),
        duration: Duration::EndOfTurn,
        free: true,
    })
}

inventory::submit! { EffectPattern { name: "r406 until end of turn, you may cast that card for free", priority: 80, parse: until_end_of_turn_cast_it_free } }
