//! "As ~ enters, exile [cards] from your graveyard." (CR 614.1c, 614.12): a replacement
//! effect that moves other cards as the permanent enters. The cards are exiled with the
//! entering object, linked to its abilities that refer to them ("the exiled card", CR
//! 607.2b); the permanent keeps them as its own (see `perform_move`).
//!
//! * "exile an instant or sorcery card from your graveyard" (Living Lore), "exile a
//!   creature card from a graveyard" (Dermotaxi), "exile any number of creature cards
//!   from your graveyard" (Sutured Ghoul), "exile up to X creature cards from your
//!   graveyard. It enters with three +1/+1 counters on it for each creature card exiled
//!   this way." (Mimeoplasm, Revered One; X is the X paid to cast it, CR 107.3m);
//! * "you may put two cards your opponents own from exile into their owners' graveyards.
//!   If you do, ~ enters with four +1/+1 counters on it." (Ulamog's Despoiler): an
//!   optional cost paid as it enters.
//!
//! The permanent itself isn't among the cards it can exile, even entering from a
//! graveyard (Sutured Ghoul ruling).

use super::AbilityPattern;
use crate::ability::*;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

/// The variable holding the cards exiled as it entered.
const EXILED: Var = vars::USER + 4243;

fn etb(action: ReplacementAction, text: &str) -> Ability {
    AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(
            ReplacementDef {
                event: ReplacementEvent::EntersBattlefield(Filter::Source),
                action,
                self_replacement: false,
                optional: false,
            },
        ))),
        text,
    )
}

/// "an instant or sorcery card from your graveyard", "any number of creature cards from
/// your graveyard", "up to X creature cards from your graveyard": the choice.
fn chosen_cards(r: &str) -> Option<Sel> {
    // (For "any number", the number of cards there are: filled in below.)
    let (count, up_to, r) = if let Some(r) = r.strip_prefix("any number of ") {
        (Value::Const(-1), true, r)
    } else if let Some(r) = r.strip_prefix("up to ") {
        let (n, r) = parse_number(r)?;
        (n, true, r)
    } else {
        let (n, r) = parse_number(r)?;
        // Exactly X ("If you can't, ...") isn't read here.
        if matches!(n, Value::X) {
            return None;
        }
        (n, false, r)
    };
    let (f, _, tail) = parse_object_phrase(r)?;
    let owner = match end(tail) {
        "from your graveyard" => Filter::OwnedBy(PlayerRel::You),
        "from a graveyard" => Filter::OwnedBy(PlayerRel::Any),
        "" if f.zone() == Some(ZoneKind::Graveyard) => Filter::Any,
        _ => return None,
    };
    if !matches!(f.zone(), None | Some(ZoneKind::Graveyard)) {
        return None;
    }
    let filter = Filter::and(vec![
        f,
        Filter::InZone(ZoneKind::Graveyard),
        owner,
        Filter::Other,
    ]);
    let count = match count {
        Value::Const(-1) => Value::Count(filter.clone()),
        n => n,
    };
    Some(Sel::Choose {
        chooser: PlayerRef::You,
        filter,
        count,
        up_to,
        store: Some(EXILED),
    })
}

/// "It enters with three +1/+1 counters on it for each creature card exiled this way."
fn counters_for_each_exiled(s: &str) -> Option<Effect> {
    let r = ["it enters with ", "~ enters with "]
        .iter()
        .find_map(|p| s.strip_prefix(p))?;
    let (n, r) = parse_number(r)?;
    let (kind, r) = crate::oracle::costs::counter_kind(r)?;
    let r = strip(r, "counters")
        .or_else(|| strip(r, "counter"))?
        .strip_prefix("on it for each ")?;
    let what = r.strip_suffix(" exiled this way")?;
    let (_, _, tail) = parse_object_phrase(what)?;
    if !end(tail).is_empty() {
        return None;
    }
    Some(Effect::EnterWithCounters {
        kind,
        n: Value::Mul(
            Box::new(n),
            Box::new(Value::CountSel(Box::new(Sel::Var(EXILED)))),
        ),
    })
}

fn as_enters_exile(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if !ctx.is_permanent() || block.contains('\n') {
        return None;
    }
    let text = crate::oracle::strip_ability_word(block.trim());
    let lower = text.to_lowercase();
    let l = end(&lower);
    let r = l.strip_prefix("as ~ enters, ")?;
    let sentences: Vec<&str> = r.split(". ").collect();
    let body = if let Some(cards) = sentences[0].strip_prefix("exile ") {
        let what = chosen_cards(cards)?;
        let exile = Effect::Exile {
            what,
            face_down: false,
            link: true,
        };
        match sentences.as_slice() {
            [_] => exile,
            [_, more] => Effect::seq(vec![exile, counters_for_each_exiled(more)?]),
            _ => return None,
        }
    } else if let Some(cost) = sentences[0].strip_prefix("you may ") {
        // "you may put two cards your opponents own from exile into their owners'
        // graveyards. If you do, ~ enters with four +1/+1 counters on it."
        let [_, then] = sentences.as_slice() else {
            return None;
        };
        let part = crate::oracle::costs::parse_cost_part(cost)?;
        let then = then.strip_prefix("if you do, ")?;
        let (kind, n) = enters_with(then)?;
        Effect::seq(vec![
            Effect::PayOptional {
                who: PlayerRef::You,
                cost: Cost::free().with(part),
                then: Box::new(Effect::Noop),
                otherwise: Box::new(Effect::Noop),
            },
            Effect::If {
                cond: Condition::PrevHappened,
                then: Box::new(Effect::EnterWithCounters { kind, n }),
                otherwise: Box::new(Effect::Noop),
            },
        ])
    } else {
        return None;
    };
    Some(vec![etb(ReplacementAction::AsEnters(Box::new(body)), block)])
}

/// "~ enters with four +1/+1 counters on it".
fn enters_with(s: &str) -> Option<(crate::types::CounterKind, Value)> {
    let r = ["~ enters with ", "it enters with "]
        .iter()
        .find_map(|p| s.strip_prefix(p))?;
    let (n, r) = parse_number(r)?;
    let (kind, r) = crate::oracle::costs::counter_kind(r)?;
    let r = strip(r, "counters").or_else(|| strip(r, "counter"))?;
    (end(r) == "on it").then_some((kind, n))
}

inventory::submit! { AbilityPattern { name: "as ~ enters, exile cards from your graveyard (linked)", priority: 45, parse: as_enters_exile } }
