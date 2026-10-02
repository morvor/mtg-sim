//! "Exchange control of target nonland permanent you control and target permanent an
//! opponent controls that shares a card type with it." (Daring Thief), "Exchange control
//! of target artifact or creature and another target permanent that shares one of those
//! types with it." (Legerdemain): the second target must have a relationship with the
//! first (`TargetSpec::related_to`, see `target_groups.rs`), checked as the targets are
//! chosen (CR 601.2c; the first target is offered only if it has a partner) and again as
//! the spell or ability resolves (CR 608.2b): if they no longer share the type, neither is
//! a legal target and the exchange doesn't happen (CR 701.12b).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{object_ref, Builder};
use crate::oracle::phrases::end;
use crate::types::CardType;

const SHARES_A_CARD_TYPE: &str = " that shares a card type with it";
const SHARES_ONE_OF_THOSE: &str = " that shares one of those types with it";

/// The card types the words of a target phrase name ("target artifact or creature").
fn named_types(text: &str) -> Vec<CardType> {
    text.split([' ', ','])
        .filter_map(CardType::from_word)
        .fold(Vec::new(), |mut v, t| {
            if !v.contains(&t) {
                v.push(t);
            }
            v
        })
}

fn exchange_related(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("exchange control of ")?;
    let (first, second) = r.split_once(" and ")?;
    let (second, among) = if let Some(s) = second.strip_suffix(SHARES_A_CARD_TYPE) {
        (s, false)
    } else {
        (second.strip_suffix(SHARES_ONE_OF_THOSE)?, true)
    };
    if !first.starts_with("target ") {
        return None;
    }
    let before = b.targets.len();
    let (a, t1) = object_ref(first, b)?;
    let (c, t2) = object_ref(second, b)?;
    if !end(&t1).is_empty() || !end(&t2).is_empty() || b.targets.len() != before + 2 {
        return None;
    }
    let (Sel::Target(s0), Sel::Target(s1)) = (&a, &c) else {
        return None;
    };
    let grp = if among {
        let types = named_types(first);
        if types.is_empty() {
            return None;
        }
        TargetGroup::ShareCardTypeAmong(types)
    } else {
        TargetGroup::ShareCardType
    };
    let spec = &mut b.targets[*s1 as usize];
    spec.related_to = Some((*s0, grp));
    spec.text = format!(
        "{}{}",
        spec.text,
        if among {
            SHARES_ONE_OF_THOSE
        } else {
            SHARES_A_CARD_TYPE
        }
    );
    Some(Effect::ExchangeControl { a, b: c })
}

inventory::submit! { EffectPattern { name: "exchange control of target [A] and target [B] that shares a card type with it", priority: 110, parse: exchange_related } }
