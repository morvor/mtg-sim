//! "~ gets +1/+0 until end of turn and deals 1 damage to you." (Firedrinker Satyr): a pump
//! and a damage instruction sharing their subject, read as the two sentences "~ gets +1/+0
//! until end of turn. ~ deals 1 damage to you."

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_sentence, Builder};
use crate::oracle::phrases::end;

fn pump_and_deals(s: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(s);
    let r = l.strip_prefix("~ gets ")?;
    let (pump, dmg) = r.split_once(" until end of turn and deals ")?;
    let first = parse_sentence(&format!("~ gets {pump} until end of turn"), b)?;
    let second = parse_sentence(&format!("~ deals {dmg}"), b)?;
    if !matches!(first, Effect::Modify { .. }) || !matches!(second, Effect::DealDamage { .. }) {
        return None;
    }
    Some(Effect::seq(vec![first, second]))
}

inventory::submit! { EffectPattern { name: "~ gets +N/+N until end of turn and deals N damage to ...", priority: 100, parse: pump_and_deals } }
