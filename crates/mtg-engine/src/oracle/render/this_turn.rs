//! Whether a trigger event has happened this turn ([`Condition::AllTriggerConditionsThisTurn`],
//! CR 603.1b): "if another creature died this turn", "as long as you've put one or more
//! +1/+1 counters on a creature this turn".

use super::nouns::Det;
use super::*;

/// "under your control" for the objects of a filter controlled by you, and the filter
/// without it.
fn under_your_control(f: &Filter) -> (Filter, &'static str) {
    match f {
        Filter::And(v)
            if v.iter()
                .any(|x| matches!(x, Filter::ControlledBy(PlayerRel::You))) =>
        {
            (
                Filter::and(
                    v.iter()
                        .filter(|x| !matches!(x, Filter::ControlledBy(PlayerRel::You)))
                        .cloned()
                        .collect(),
                ),
                " under your control",
            )
        }
        other => (other.clone(), ""),
    }
}

/// The past tense of a verb phrase's verb ("dies" → "died", "is put" → "was put").
fn past(vp: &str) -> Option<String> {
    let (verb, rest) = match vp.split_once(' ') {
        Some((v, r)) => (v, format!(" {r}")),
        None => (vp, String::new()),
    };
    let v = match verb {
        "is" => "was",
        "are" => "were",
        "put" | "puts" => "put",
        "dies" => "died",
        "enters" => "entered",
        "leaves" => "left",
        "attacks" => "attacked",
        "blocks" => "blocked",
        "casts" | "cast" => "cast",
        "gains" | "gain" => "gained",
        "loses" | "lose" => "lost",
        "draws" | "draw" => "drew",
        "deals" => "dealt",
        "discards" | "discard" => "discarded",
        "sacrifices" | "sacrifice" => "sacrificed",
        _ => return None,
    };
    Some(format!("{v}{rest}"))
}

impl Renderer<'_> {
    /// "another creature died", "you put a counter on a creature" + "this turn".
    pub(crate) fn happened_this_turn(&mut self, t: &TriggerCond) -> String {
        let s = match t {
            // "if a non-Skeleton creature died under your control this turn".
            TriggerCond::Dies(f) => {
                let (f, under) = under_your_control(f);
                let n = self.noun_det(&f, Det::A);
                format!("{n} {{alt:died|was put into a graveyard from the battlefield}}{under}")
            }
            // "if a +1/+1 counter was put on a permanent under your control this turn".
            // "if a counter was put on ~ this turn": one or more were.
            TriggerCond::CountersPut { filter, kind, .. } => {
                let (f, under) = under_your_control(filter);
                let k = match kind {
                    Some(k) => counter_name(k),
                    None => "counter".into(),
                };
                let n = self.noun_det(&f, Det::A);
                format!("{} was put on {n}{under}", with_article(&k))
            }
            // "If a noncreature permanent under your control was destroyed this turn".
            TriggerCond::DestroyedBy {
                filter,
                by: by @ (PlayerRel::Any | PlayerRel::Opponent),
            } => {
                let (f, under) = under_your_control(filter);
                let n = self.noun_det(&f, Det::A);
                if *by == PlayerRel::Opponent {
                    format!("{n}{under} was destroyed this turn by a spell or ability an opponent {{alt:controls|controlled}}")
                } else {
                    format!("{n}{under} was destroyed")
                }
            }
            other => {
                let (subj, vp) = self.trigger_event_parts(other);
                let Some(vp) = past(&vp) else {
                    return self.gap("a trigger event in the past tense");
                };
                // "countered by a spell or ability an opponent controlled".
                let vp = match vp.strip_suffix(" controls") {
                    Some(h) => format!("{h} {{alt:controls|controlled}}"),
                    None => vp,
                };
                if subj.is_empty() {
                    vp
                } else if subj == "you" && vp.starts_with("put ") {
                    // "you've put one or more +1/+1 counters on a creature".
                    format!("{{opt:you}} {vp}")
                } else {
                    format!("{subj} {vp}")
                }
            }
        };
        if s.contains("this turn") {
            s
        } else {
            format!("{s} this turn")
        }
    }
}
