//! "Each creature that convoked ~ connives." (Lethal Scheme): each creature that was
//! tapped to pay for the spell with convoke (CR 702.51c) connives (CR 701.50).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;
use crate::types::CardType;
use smol_str::SmolStr;

fn convokers_connive(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if l != "each creature that convoked ~ connives" && l != "each creature that convoked it connives"
    {
        return None;
    }
    Some(super::a701_actions::keyword_action(
        KeywordAction::Connive,
        PlayerRef::You,
        Sel::All(Filter::And(vec![
            Filter::Type(CardType::Creature),
            Filter::Custom(SmolStr::new(crate::kw::convoke::CONVOKED_IT)),
        ])),
        Value::c(1),
    ))
}

inventory::submit! { EffectPattern { name: "each creature that convoked it connives", priority: 100, parse: convokers_connive } }
