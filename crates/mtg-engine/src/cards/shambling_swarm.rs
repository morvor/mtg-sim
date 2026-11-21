//! Shambling Swarm: "When ~ dies, distribute three -1/-1 counters among one, two, or three
//! target creatures. For each -1/-1 counter you put on a creature this way, remove a -1/-1
//! counter from that creature at the beginning of the next end step." The counters removed
//! needn't be the same ones (ruling); a creature that left the battlefield is a new object
//! that loses none (CR 400.7).

use super::{map_effect, parse, ManualAbility};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};

const DISTRIBUTE: &str = "card:Shambling Swarm:distribute the counters and remove as many at the next end step";
const TEXT: &str = "When ~ dies, distribute three -1/-1 counters among one, two, or three target creatures. For each -1/-1 counter you put on a creature this way, remove a -1/-1 counter from that creature at the beginning of the next end step.";
const M1: &str = "-1/-1";

inventory::submit! { ManualAbility {
    card: "Shambling Swarm",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "When ~ dies, distribute three -1/-1 counters among one, two, or three target creatures.")
            .iter()
            .map(|a| map_effect(a, TEXT, |_| Effect::Custom(DISTRIBUTE.into())))
            .collect()
    },
    reason: "removes at the next end step as many -1/-1 counters as it distributed to each creature: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != DISTRIBUTE {
            return false;
        }
        let targets = ctx.targets.first().cloned().unwrap_or_default();
        let div = ctx.divided.first().cloned().unwrap_or_default();
        let mut removals = Vec::new();
        for (i, t) in targets.into_iter().enumerate() {
            let n = div.get(i).copied().unwrap_or(0);
            if n == 0 {
                continue;
            }
            let put = g.put_counters(t, M1, n, crate::event_causes::CounterPut::of(ctx));
            if put > 0 {
                let var = vars::USER + 4400 + i as Var;
                ctx.set_var(var, vec![t]);
                removals.push(Effect::RemoveCounters {
                    what: Sel::Var(var),
                    kind: Some(M1.into()),
                    n: Value::Const(put as i32),
                });
            }
        }
        if !removals.is_empty() {
            let delayed = Effect::DelayedTrigger {
                trigger: TriggerCond::BeginningOf {
                    step: TriggerStep::End,
                    whose: PlayerRel::Any,
                },
                body: Box::new(Body::effect(Effect::Seq(removals))),
                once: true,
            };
            g.exec(&delayed, ctx);
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
