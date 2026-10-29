//! "if a counter was put on ~ this turn" (Wakka, Devoted Guardian), "if a +1/+1 counter
//! was put on ~ this turn", "if one or more counters were put on ~ this turn": whether a
//! counters-put event of this turn put counters on this object (CR 122.6; an object that
//! changed zones is a new object, CR 400.7), regardless of whether anything triggered on
//! it (CR 603.1b).
//!
//! Only the source itself: whether counters were put on some other permanent "under your
//! control" would need the permanent as it was then, and "you put" would need who put
//! them, which counter events don't record.

use crate::ability::*;
use crate::oracle::patterns::ConditionPattern;
use crate::oracle::phrases::end;

fn parse(c: &str) -> Option<Condition> {
    let c = end(c);
    let r = c.strip_suffix(" on ~ this turn")?;
    let kind = if let Some(k) = r.strip_prefix("a ") {
        let k = k.strip_suffix(" was put")?;
        if k == "counter" {
            None
        } else {
            Some(k.strip_suffix(" counter")?)
        }
    } else {
        let k = r.strip_prefix("one or more ")?.strip_suffix(" were put")?;
        if k == "counters" {
            None
        } else {
            Some(k.strip_suffix(" counters")?)
        }
    };
    let kind = match kind {
        None => None,
        Some(k) => {
            let (kind, rest) = crate::oracle::costs::counter_kind(&format!("{k} counter"))
                .map(|(kind, rest)| (kind, rest.to_string()))?;
            if rest.trim() != "counter" {
                return None;
            }
            Some(kind)
        }
    };
    Some(Condition::AllTriggerConditionsThisTurn(vec![
        TriggerCond::CountersPut {
            filter: Filter::Source,
            kind,
            each: false,
        },
    ]))
}

inventory::submit! {
    ConditionPattern { name: "counters put on ~ this turn", priority: 110, parse }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses() {
        for c in [
            "a counter was put on ~ this turn",
            "a +1/+1 counter was put on ~ this turn",
            "one or more counters were put on ~ this turn",
            "one or more +1/+1 counters were put on ~ this turn",
        ] {
            assert!(parse(c).is_some(), "{c}");
        }
        for c in [
            "a counter was put on a creature this turn",
            "a counter was put on ~",
            "a counter was put on ~ this turn and you attacked",
        ] {
            assert!(parse(c).is_none(), "{c}");
        }
    }
}
