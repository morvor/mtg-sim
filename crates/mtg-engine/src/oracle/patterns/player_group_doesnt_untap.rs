//! "Creatures target player controls don't untap during that player's next untap step."
//! (Misstep), "Creatures and lands target opponent controls don't untap during their next
//! untap step." (Exhaustion), "During that player's next untap step, creatures they control
//! don't untap." (Imaginary Threats): a rule-modifying effect on a group of permanents a
//! player controls, not locked to the permanents there as it resolves (CR 611.2c applies
//! only to effects that modify characteristics or controllers): the permanents that player
//! controls during that untap step don't untap, including ones that entered or became
//! tapped after the effect began (CR 502.3). It lasts until that player's next untap step
//! has passed.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{bind_target_player, Builder};
use crate::oracle::phrases::{end, parse_object_phrase};

fn player_group_doesnt_untap(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let filter = if let Some(subject) = [
        " don't untap during that player's next untap step",
        " don't untap during their next untap step",
    ]
    .iter()
    .find_map(|s| l.strip_suffix(s))
    {
        // "[permanents] target player controls": the player is that target.
        let (f, plural, tail) = parse_object_phrase(subject)?;
        if !plural || crate::oracle::phrases::target_player_controls(tail).is_none() {
            return None;
        }
        let (f, rest) = bind_target_player(f, tail, b);
        if !end(&rest).is_empty() {
            return None;
        }
        f
    } else {
        // "During that player's next untap step, [permanents] they control don't untap."
        let r = l.strip_prefix("during that player's next untap step, ")?;
        let subject = r.strip_suffix(" they control don't untap")?;
        let PlayerRef::Target(slot) = b.it_player else {
            return None;
        };
        let (f, plural, tail) = parse_object_phrase(subject)?;
        if !plural || !end(tail).is_empty() {
            return None;
        }
        Filter::and(vec![f, Filter::ControlledBy(PlayerRel::Target(slot))])
    };
    Some(Effect::AddRestriction {
        restriction: Restriction::DoesntUntap(filter),
        duration: Duration::ThroughNextUntapStep,
    })
}

inventory::submit! {
    EffectPattern {
        name: "[permanents] target player controls don't untap during that player's next untap step",
        priority: 100,
        parse: player_group_doesnt_untap,
    }
}
