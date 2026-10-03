//! "You may exile a creature card from your graveyard. If you do, create a token that's a
//! copy of that card, ..." (God-Pharaoh's Gift): a card chosen as the effect happens (not
//! targeted, CR 115.10) is exiled; "that card" then refers to it in exile (CR 400.7,
//! 608.2c). Other pronouns keep their antecedents ("... deals 2 damage to that
//! creature's controller", "Otherwise, sacrifice it").

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;

/// The card exiled.
const EXILED_CARD: Var = vars::USER + 2626;

fn exile_a_graveyard_card(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("exile ")?;
    let r = r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?;
    let (f, plural, tail) = parse_object_phrase(r)?;
    if plural || !end(tail).is_empty() || f.zone() != Some(ZoneKind::Graveyard) {
        return None;
    }
    b.named
        .push(("that card".to_string(), Sel::Var(EXILED_CARD)));
    // With no earlier antecedent, "it" is the exiled card too ("Exile a creature card from
    // your graveyard. Create a token that's a copy of it.", Mordor on the March).
    // A Saga's chapter ability doesn't call the Saga "it" ("I — Exile a permanent card from
    // your graveyard. You gain life equal to its mana value.", The Aesir Escape Valhalla).
    let saga = b.ctx.type_line.subtypes.iter().any(|s| s.as_str() == "Saga");
    if matches!(b.it, Sel::None)
        || super::oracle_hardening_referents::is_no_referent(&b.it)
        || (saga && matches!(b.it, Sel::This))
    {
        b.it = Sel::Var(EXILED_CARD);
    }
    Some(Effect::Seq(vec![
        Effect::Exile {
            what: Sel::Choose {
                chooser: PlayerRef::You,
                filter: f,
                count: Value::c(1),
                up_to: false,
                store: None,
            },
            face_down: false,
            link: false,
        },
        Effect::Store {
            var: EXILED_CARD,
            sel: Sel::Var(vars::IT),
        },
    ]))
}

inventory::submit! { EffectPattern { name: "exile a [card] from your graveyard (chosen)", priority: 100, parse: exile_a_graveyard_card } }
