//! Copy grammar V (cards4-referents-statics, family `copies`).
//!
//! - "Enchanted creature is a copy of the chosen creature." (Metamorphic Alteration),
//!   "Equipped creature is a copy of the last chosen card." (Paleontologist's Pick-Axe //
//!   Dinosaur Headdress): a static ability's copy effect on the permanent the source is
//!   attached to (CR 707.2, 613.2a), copying the object the source's linked choice noted
//!   ([`Modification::CopyOf`], `static_copy.rs`).

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

/// Whether the card notes a choice of one object for its other abilities: "As ~ enters,
/// choose a creature." or "..., choose an exiled creature card used to craft ~."
fn card_notes_one_choice(what: &str) -> bool {
    let raw = crate::oracle::raw_text().to_lowercase();
    match what {
        "the chosen creature" => raw.contains(" enters, choose a creature.")
            || raw.contains(" enters, choose another creature"),
        "the last chosen card" => raw.contains(", choose an exiled creature card"),
        _ => false,
    }
}

fn attached_is_copy_of_chosen(l: &str, orig: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l);
    let rest = l
        .strip_prefix("enchanted creature is a copy of ")
        .or_else(|| l.strip_prefix("equipped creature is a copy of "))?;
    if !ctx.is_permanent() || !card_notes_one_choice(rest) {
        return None;
    }
    let effect = StaticEffect::Continuous {
        affected: Filter::AttachedToSource,
        mods: vec![Modification::CopyOf(Box::new(Sel::LinkedNoted))],
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(effect)),
        orig,
    )])
}

inventory::submit! { StaticPattern { name: "copies 5: enchanted/equipped creature is a copy of the chosen object", priority: 100, parse: attached_is_copy_of_chosen } }

/// The card chosen to be copied.
const COPIED_CARD: Var = vars::USER + 51_201;

/// "Whenever one or more cards leave your graveyard, put a +1/+1 counter on ~, then you
/// may have ~ become a copy of an artifact or creature card from among those cards until
/// end of turn." (Spirit of Resilience): one of the cards of the triggering batch (CR
/// 603.2c, 603.10a), chosen as the ability resolves; a card that has left the zone it went
/// to is copied as it last existed there (CR 707.2, 608.2h).
fn become_copy_of_one_of_those_cards(l: &str, b: &mut crate::oracle::effects::Builder) -> Option<Effect> {
    let l = end(l);
    let (may, r) = match l.strip_prefix("you may have ~ become a copy of ") {
        Some(r) => (true, r),
        None => (false, l.strip_prefix("~ becomes a copy of ")?),
    };
    let (noun, r) = r.split_once(" from among those cards")?;
    let duration = match r.trim() {
        "until end of turn" => Duration::EndOfTurn,
        "" => Duration::Permanent,
        _ => return None,
    };
    let raw = crate::oracle::raw_text().to_lowercase();
    if !raw.contains("whenever one or more cards leave your graveyard,") || !b.in_trigger {
        return None;
    }
    let noun = noun.strip_prefix("an ").or_else(|| noun.strip_prefix("a "))?;
    let (f, false, tail) = crate::oracle::phrases::parse_object_phrase(noun)? else {
        return None;
    };
    if !end(tail).is_empty() {
        return None;
    }
    let choose = Effect::Store {
        var: COPIED_CARD,
        sel: Sel::Choose {
            chooser: PlayerRef::You,
            filter: Filter::and(vec![Filter::In(Box::new(Sel::TriggerObjects)), f]),
            count: Value::c(1),
            up_to: false,
            store: None,
        },
    };
    let copy = Effect::BecomeCopy {
        what: Sel::This,
        of: Sel::Var(COPIED_CARD),
        duration,
    };
    let e = Effect::seq(vec![choose, copy]);
    Some(if may {
        Effect::May {
            who: PlayerRef::You,
            effect: Box::new(e),
        }
    } else {
        e
    })
}

inventory::submit! { super::EffectPattern { name: "copies 5: become a copy of [a card] from among those cards", priority: 100, parse: become_copy_of_one_of_those_cards } }
