//! General Jarkeld: "{T}: Choose two target blocked attacking creatures. If each of those
//! creatures could be blocked by all creatures that the other is blocked by, each creature
//! that's blocking exactly one of those attacking creatures stops blocking it and is
//! blocking the other attacking creature. Activate only during the declare blockers step."
//! The switched blockers don't become blockers again (no "blocks" triggers), and abilities
//! that already triggered still resolve (rulings).

use super::{map_effect, parse, ManualAbility};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::ObjectId;

const SWAP: &str = "card:General Jarkeld:switch the blockers of the two attackers";
const TEXT: &str = "{T}: Choose two target blocked attacking creatures. If each of those creatures could be blocked by all creatures that the other is blocked by, each creature that's blocking exactly one of those attacking creatures stops blocking it and is blocking the other attacking creature. Activate only during the declare blockers step.";

inventory::submit! { ManualAbility {
    card: "General Jarkeld",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "{T}: Tap two target blocked attacking creatures. Activate only during the declare blockers step.")
            .iter()
            .map(|a| map_effect(a, TEXT, |_| Effect::Custom(SWAP.into())))
            .collect()
    },
    reason: "switches the blockers of two blocked attackers if each could be blocked by the other's: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != SWAP {
            return false;
        }
        let picked: Vec<ObjectId> = ctx
            .targets
            .iter()
            .flatten()
            .filter_map(|e| e.object())
            .collect();
        let [a, b] = picked[..] else {
            return true;
        };
        let Some(cs) = g.combat.as_ref() else {
            return true;
        };
        let (ba, bb) = (cs.blockers_of(a), cs.blockers_of(b));
        // "Could be blocked by": as though the blockers were untapped and not blocking.
        let mut probe = g.clone();
        for x in ba.iter().chain(bb.iter()) {
            probe.objects[x.0 as usize].tapped = false;
        }
        if !bb.iter().all(|x| probe.can_block(*x, a)) || !ba.iter().all(|x| probe.can_block(*x, b)) {
            return true;
        }
        let only_a: Vec<ObjectId> = ba.iter().copied().filter(|x| !bb.contains(x)).collect();
        let only_b: Vec<ObjectId> = bb.iter().copied().filter(|x| !ba.contains(x)).collect();
        let Some(cs) = g.combat.as_mut() else {
            return true;
        };
        let swap = |v: &mut Vec<ObjectId>, from: ObjectId, to: ObjectId| {
            for x in v.iter_mut() {
                if *x == from {
                    *x = to;
                }
            }
        };
        for bl in cs.blockers.iter_mut() {
            if only_a.contains(&bl.id) {
                swap(&mut bl.blocking, a, b);
            } else if only_b.contains(&bl.id) {
                swap(&mut bl.blocking, b, a);
            }
        }
        for at in cs.attackers.iter_mut() {
            if at.id == a {
                at.blockers.retain(|x| !only_a.contains(x));
                at.blockers.extend(only_b.iter().copied());
            } else if at.id == b {
                at.blockers.retain(|x| !only_b.contains(x));
                at.blockers.extend(only_a.iter().copied());
            }
        }
        g.dirty = true;
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
