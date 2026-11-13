//! "Distribute three +1/+1 counters among one, two, or three target creatures. For each
//! +1/+1 counter you put on a creature this way, remove a +1/+1 counter from that
//! creature at the beginning of the next cleanup step." (Bounty of the Hunt): the counters
//! the distribution actually put on each object (after replacement effects, CR 614.1) are
//! recorded as it's performed ([`PUT_THIS_WAY`], each object once per counter), and the
//! delayed triggered ability (CR 603.7) removes one counter from each such object per
//! entry. An object that has left the battlefield since is a new object it doesn't find
//! (CR 400.7).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;

/// The objects a distribution of counters put counters on, once per counter put.
pub const PUT_THIS_WAY: Var = vars::USER + 14_530;

/// [`Effect::Custom`] prefix: `remove one per counter put:<var>:<kind>` removes, for each
/// entry of the variable, one counter of the kind from that object.
pub const REMOVE_PER_PUT: &str = "remove one per counter put:";

/// The effect removing one counter of `kind` for each entry of `var`.
pub fn remove_per_put(var: Var, kind: &str) -> Effect {
    Effect::Custom(format!("{REMOVE_PER_PUT}{var}:{kind}").into())
}

/// The variable and kind of a [`remove_per_put`] name.
pub fn parse_name(name: &str) -> Option<(Var, &str)> {
    let (v, k) = name.strip_prefix(REMOVE_PER_PUT)?.split_once(':')?;
    Some((v.parse().ok()?, k))
}

/// Records that `put` counters were put on `to` by a distribution.
pub fn record(ctx: &mut Ctx, to: Entity, put: u32) {
    let mut v = ctx.vars.get(&PUT_THIS_WAY).cloned().unwrap_or_default();
    v.extend(std::iter::repeat_n(to, put as usize));
    ctx.set_var(PUT_THIS_WAY, v);
}

pub struct CountersPutThisWay;

impl KeywordRules for CountersPutThisWay {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        let Some((var, kind)) = parse_name(name) else {
            return false;
        };
        let entries = ctx.vars.get(&var).cloned().unwrap_or_default();
        for e in entries {
            if e.object().is_some_and(|o| g.is_live(o)) {
                g.remove_counters(e, kind, 1);
            }
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&CountersPutThisWay) }
