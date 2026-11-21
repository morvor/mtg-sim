//! Choosing what gets counters (CR 122.1, 614.17): "each player may put two +1/+1 counters on
//! a creature they control" (Agitator Ant) offers only the objects that can have counters of
//! that kind put on them. Agitator Ant's ruling: "If a creature can't have +1/+1 counters put
//! on it, its controller can't choose it just to have it become goaded." So does a choice the
//! compiled text makes first and stores for the instruction right after it, which puts the
//! counters on what was chosen ("each player chooses a nonland permanent and puts a doom
//! counter on it", Eye of Doom; "Each player puts a vow counter on a creature they control and
//! sacrifices the rest", Promise of Loyalty: [`narrowed_stored_choices`]), and bolster's own
//! choice (`kwa/bolster_adapt.rs`). Register `counter-choice-excludes-what-cant-have-them`.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::ObjectId;

/// `Filter::Custom` prefix: the object can have counters of the kind that follows put on it
/// ([`crate::counter_rules::counters_prevented`] doesn't stop them).
const CAN_HAVE: &str = "counters:can have ";

/// The choice of `what` to put `kind` counters on, narrowed to the objects that can have
/// them; any other selection is returned as it is.
pub fn chosen_for_counters(what: &Sel, kind: &str) -> Sel {
    match what {
        Sel::Choose {
            chooser,
            filter,
            count,
            up_to,
            store,
        } => Sel::Choose {
            chooser: chooser.clone(),
            filter: Filter::And(vec![filter.clone(), can_have(kind)]),
            count: count.clone(),
            up_to: *up_to,
            store: *store,
        },
        other => other.clone(),
    }
}

/// The filter of the objects that can have `kind` counters put on them.
pub fn can_have(kind: &str) -> Filter {
    Filter::Custom(format!("{CAN_HAVE}{kind}").into())
}

/// `v`, a sequence of instructions, with each choice that an instruction makes and stores for
/// the next one, which puts counters on what was chosen, narrowed to the objects that can
/// have those counters: a stored choice ("each player chooses a nonland permanent and puts a
/// doom counter on it", Eye of Doom) and the permanents kept by a "keep one and sacrifice
/// the rest" ("Each player puts a vow counter on a creature they control and sacrifices the
/// rest", Promise of Loyalty). `None` if there's none.
pub fn narrowed_stored_choices(v: &[Effect]) -> Option<Vec<Effect>> {
    let mut out: Option<Vec<Effect>> = None;
    for (i, pair) in v.windows(2).enumerate() {
        let Effect::AddCounters {
            what: Sel::Var(var),
            kind,
            ..
        } = &pair[1]
        else {
            continue;
        };
        let narrowed = match &pair[0] {
            Effect::Store { var: stored, sel }
                if stored == var && matches!(sel, Sel::Choose { .. }) =>
            {
                Effect::Store {
                    var: *stored,
                    sel: chosen_for_counters(sel, kind),
                }
            }
            Effect::KeepAndSacrificeRest {
                who,
                among,
                keep,
                up_to,
            } if *var == crate::apnap::KEPT => Effect::KeepAndSacrificeRest {
                who: who.clone(),
                among: among.clone(),
                keep: keep
                    .iter()
                    .map(|f| Filter::And(vec![f.clone(), can_have(kind)]))
                    .collect(),
                up_to: *up_to,
            },
            _ => continue,
        };
        out.get_or_insert_with(|| v.to_vec())[i] = narrowed;
    }
    out
}

struct CounterChoice;

impl KeywordRules for CounterChoice {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, _ctx: &Ctx) -> Option<bool> {
        let kind = name.strip_prefix(CAN_HAVE)?;
        Some(!crate::counter_rules::counters_prevented(g, id, kind))
    }
}

inventory::submit! { KeywordRegistration(&CounterChoice) }
