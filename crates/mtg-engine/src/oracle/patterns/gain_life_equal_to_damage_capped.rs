//! "[~ deals X damage to any target.] You gain life equal to the damage dealt, but not
//! more life than the player's life total before the damage was dealt, the planeswalker's
//! loyalty before the damage was dealt, or the creature's toughness." (Drain Life), and
//! "... but not more than the amount of {B} spent on X, the player's life total ..."
//! (Soul Burn): life equal to the damage the previous instruction actually dealt (after
//! prevention, CR 120.4b), capped by what the target had to lose — read before the damage
//! for a player's life total or a planeswalker's loyalty, and the creature's toughness
//! whatever damage it already had (Drain Life ruling) — and by the black mana spent on an
//! X only black and/or red mana could pay (see `payment_rules.rs`). A target of another
//! kind (a battle) has no cap.

use super::FollowupPattern;
use crate::ability::*;
use crate::mana::ManaType;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;
use crate::types::CardType;

/// The cap read before the damage.
const CAP: Var = vars::USER + 5113;

const TAIL: &str = "the player's life total before the damage was dealt, the planeswalker's loyalty before the damage was dealt, or the creature's toughness";

fn gain_life_capped(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let Some(rest) = end(l).strip_prefix("you gain life equal to the damage dealt, but not more ")
    else {
        return false;
    };
    let black_on_x = match rest.strip_prefix("life than ") {
        Some(r) if r == TAIL => false,
        _ => match rest.strip_prefix("than the amount of {b} spent on x, ") {
            Some(r) if r == TAIL => true,
            _ => return false,
        },
    };
    // The previous instruction deals damage to the spell's one target.
    let Effect::DealDamage {
        to: Sel::Target(0), ..
    } = prev
    else {
        return false;
    };
    let target = || Box::new(Sel::Target(0));
    let is = |t: CardType| Condition::SelMatches(Sel::Target(0), Filter::Type(t));
    // A number no life total, loyalty or toughness reaches: no cap.
    let unbounded = Value::c(i32::MAX / 2);
    let cap = Value::If(
        Box::new(Condition::PlayerMatches(PlayerRef::Target(0), PlayerFilter::Any)),
        Box::new(Value::LifeTotal(PlayerRef::Target(0))),
        Box::new(Value::If(
            Box::new(is(CardType::Planeswalker)),
            Box::new(Value::LoyaltyOf(target())),
            Box::new(Value::If(
                Box::new(is(CardType::Creature)),
                Box::new(Value::ToughnessOf(target())),
                Box::new(unbounded),
            )),
        )),
    );
    let mut gained = Value::Min(Box::new(Value::Prev), Box::new(Value::Var(CAP)));
    if black_on_x {
        gained = Value::Min(
            Box::new(gained),
            Box::new(crate::payment_rules::mana_spent_on_x_value(ManaType::B)),
        );
    }
    *prev = Effect::seq(vec![
        Effect::StoreValue {
            var: CAP,
            value: cap,
        },
        std::mem::take(prev),
        Effect::GainLife {
            who: PlayerRef::You,
            n: gained,
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "you gain life equal to the damage dealt, but not more than [the target's life, loyalty, toughness]", priority: 60, apply: gain_life_capped } }

/// The words of the life gain this pattern compiles (`n`: its amount), for the Oracle round
/// trip: "gain life equal to the damage dealt, but not more ...".
pub(crate) fn capped_text(n: &Value) -> Option<String> {
    let capped = |v: &Value| {
        matches!(v, Value::Min(a, b)
            if matches!(**a, Value::Prev) && matches!(**b, Value::Var(CAP)))
    };
    match n {
        v if capped(v) => Some(format!("gain life equal to the damage dealt, but not more life than {TAIL}")),
        Value::Min(a, b)
            if capped(a)
                && format!("{b:?}")
                    == format!("{:?}", crate::payment_rules::mana_spent_on_x_value(ManaType::B)) =>
        {
            Some(format!(
                "gain life equal to the damage dealt, but not more than the amount of {{B}} spent on X, {TAIL}"
            ))
        }
        _ => None,
    }
}
