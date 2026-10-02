//! Costs ([`Cost`], CR 118) and activated abilities.

use super::effects::{join_words, third_person};
use super::*;

impl Renderer<'_> {
    /// A cost as printed before the colon: "{2}, {T}, Sacrifice a creature".
    pub(crate) fn cost(&mut self, c: &Cost) -> String {
        let mut parts: Vec<String> = Vec::new();
        // Loyalty costs come alone (CR 606.4).
        for p in &c.parts {
            if let CostPart::Loyalty(n) = p {
                parts.push(if *n > 0 {
                    format!("+{n}")
                } else if *n < 0 {
                    format!("−{}", -n)
                } else {
                    "0".into()
                });
            }
        }
        if let Some(m) = &c.mana {
            parts.push(m.to_string());
        }
        for p in &c.parts {
            if matches!(p, CostPart::Tap) {
                parts.push("{T}".into());
            }
            if matches!(p, CostPart::Untap) {
                parts.push("{Q}".into());
            }
        }
        // "Remove three quest counters from ~ and sacrifice it": a later cost names the
        // object again as "it".
        let mut named_self = false;
        for p in &c.parts {
            match p {
                CostPart::Tap | CostPart::Untap | CostPart::Loyalty(_) => {}
                other => {
                    let mut s = self.cost_part(other);
                    if named_self {
                        s = s.replace(" ~ ", " ~it ");
                        if let Some(x) = s.strip_suffix(" ~") {
                            s = format!("{x} ~it");
                        }
                    }
                    named_self |= s.contains('~');
                    parts.push(capitalize(&s));
                }
            }
        }
        if parts.is_empty() {
            return "{0}".into();
        }
        parts.join(", ")
    }

    /// One non-mana cost, as an imperative ("sacrifice a creature").
    pub(crate) fn cost_part(&mut self, p: &CostPart) -> String {
        match p {
            CostPart::Tap => "{T}".into(),
            CostPart::Untap => "{Q}".into(),
            CostPart::PayLife(v) => {
                let (a, w) = self.amount(v);
                format!("pay {a} life{}", w.unwrap_or_default())
            }
            CostPart::Loyalty(n) => format!("{n}"),
            CostPart::SacrificeSelf => "sacrifice ~".into(),
            CostPart::Sacrifice { filter, count } => {
                let det = self.det_for(count);
                let f = super::effects::strip_controller(filter);
                let head = self.noun(&f, Num::One);
                self.sacrificed = head.split_whitespace().last().map(str::to_string);
                let n = self.noun_det(&f, det);
                format!("sacrifice {n}")
            }
            CostPart::DiscardSelf => "discard ~".into(),
            CostPart::Discard {
                filter,
                count,
                random,
            } => {
                let noun = if matches!(filter, Filter::Any) {
                    "card".to_string()
                } else {
                    let n = self.noun(&filter.clone().in_zone(ZoneKind::Hand), Num::One);
                    n.trim_end_matches(" in a hand").to_string()
                };
                let (c, w) = self.counted(count, &noun);
                let r = if *random { " at random" } else { "" };
                format!("discard {c}{r}{}", w.unwrap_or_default())
            }
            CostPart::DiscardHand => "discard your hand".into(),
            CostPart::ExileSelf => match self.zone {
                FunctionZone::Graveyard => "exile ~ from your graveyard".into(),
                FunctionZone::Hand => "exile ~ from your hand".into(),
                _ => "exile ~".into(),
            },
            CostPart::Exile {
                filter,
                zone,
                count,
            } => {
                let f = filter.clone().in_zone(*zone).and_owner_you();
                let det = self.det_for(count);
                let n = self.noun_det(&f, det);
                format!("exile {n}")
            }
            CostPart::ReturnToHand { filter, count } => {
                let det = self.det_for(count);
                let n = self.noun_det(filter, det);
                let owner = if matches!(count, Value::Const(1)) {
                    "its owner's"
                } else {
                    "their owners'"
                };
                format!("return {n} to {owner} hand")
            }
            CostPart::ReturnSelfToHand => "return ~ to its owner's hand".into(),
            CostPart::RemoveCounters { kind, count } => {
                if let Value::Const(n) = count {
                    if *n >= 1000 {
                        return format!("remove all {} from ~", plural(&counter_name(kind)));
                    }
                }
                // "Remove any number of storage counters from ~: Add {W} for each storage
                // counter removed this way."
                if matches!(count, Value::X) {
                    self.x_for_each =
                        Some(format!("for each {} removed this way", counter_name(kind)));
                }
                let (c, w) = self.counted(count, &counter_name(kind));
                format!("remove {c} from ~{}", w.unwrap_or_default())
            }
            CostPart::RemoveCountersFromAmong {
                kind,
                filter,
                count,
            } => {
                let noun = match kind {
                    Some(k) => counter_name(k),
                    None => "counter".into(),
                };
                let (c, w) = self.counted(count, &noun);
                let w = w.unwrap_or_default();
                // "Remove two counters from ~" (counters of any kinds).
                if matches!(filter, Filter::Source) {
                    let m = self.me();
                    return format!("remove {c} from {m}{w}");
                }
                // "Remove a counter from a creature you control": one counter comes from
                // one of them.
                if matches!(count, Value::Const(1)) {
                    let f = self.noun_det(filter, nouns::Det::A);
                    return format!("remove {c} from {f}{w}");
                }
                let f = self.noun(filter, Num::Many);
                format!("remove {c} from among {f}{w}")
            }
            CostPart::AddCounters { kind, count } => {
                let (c, w) = self.counted(count, &counter_name(kind));
                format!("put {c} on ~{}", w.unwrap_or_default())
            }
            CostPart::TapUntapped { filter, count } => {
                let f = Filter::and(vec![Filter::Untapped, filter.clone()]);
                let det = self.det_for(count);
                let n = self.noun_det(&f, det);
                format!("tap {n}")
            }
            CostPart::TapTotalPower { filter, power, .. } => {
                let f = Filter::and(vec![Filter::Untapped, filter.clone()]);
                let n = self.noun(&f, Num::Many);
                let p = self.value(power);
                format!("tap any number of {n} with total power {p} or greater")
            }
            CostPart::UntapTapped { filter, count } => {
                let f = Filter::and(vec![Filter::Tapped, filter.clone()]);
                let det = self.det_for(count);
                let n = self.noun_det(&f, det);
                format!("untap {n}")
            }
            CostPart::PayEnergy(v) => match v {
                Value::Const(n) if *n > 0 => format!("pay {}", "{E}".repeat(*n as usize)),
                other => {
                    let s = self.value(other);
                    format!("pay {s} {{E}}")
                }
            },
            CostPart::PayPlayerCounters { kind, count } if kind.as_str() == "energy" => match count
            {
                Value::Const(n) if *n > 0 => format!("pay {}", "{E}".repeat(*n as usize)),
                other => {
                    let s = self.value(other);
                    format!("pay {s} {{E}}")
                }
            },
            CostPart::PayPlayerCounters { kind, count } => {
                let (c, w) = self.counted(count, &counter_name(kind));
                format!("pay {c}{}", w.unwrap_or_default())
            }
            CostPart::Mill(v) => {
                let (c, w) = self.counted(v, "card");
                format!("mill {c}{}", w.unwrap_or_default())
            }
            CostPart::RevealFromHand { filter, count } => {
                let noun = {
                    let n = self.noun(&filter.clone().in_zone(ZoneKind::Hand), Num::One);
                    n.trim_end_matches(" in a hand")
                        .trim_end_matches(" in your hand")
                        .to_string()
                };
                let (c, w) = self.counted(count, &noun);
                format!("reveal {c} from your hand{}", w.unwrap_or_default())
            }
            CostPart::ExertSelf => "exert ~".into(),
            CostPart::CollectEvidence(n) => format!("collect evidence {n}"),
            CostPart::Forage => "forage".into(),
            CostPart::PutFromHandOnLibrary { filter, count, top } => {
                let noun = {
                    let n = self.noun(&filter.clone().in_zone(ZoneKind::Hand), Num::One);
                    n.trim_end_matches(" in a hand").to_string()
                };
                let (c, w) = self.counted(count, &noun);
                let pos = if *top {
                    "on top of"
                } else {
                    "on the bottom of"
                };
                let order = if matches!(count, Value::Const(1)) {
                    ""
                } else {
                    " in any order"
                };
                format!(
                    "put {c} from your hand {pos} your library{order}{}",
                    w.unwrap_or_default()
                )
            }
            CostPart::Effect(e) => self.effect(e),
            CostPart::PayManaCostOf(s) => {
                let s = self.sel(s, players::Case::Poss);
                format!("pay {s} mana cost")
            }
            CostPart::Repeated { cost, times } => {
                let c = self.cost(cost);
                let t = match times {
                    Value::Count(f) => {
                        let n = self.for_each_noun(f);
                        format!("for each {n}")
                    }
                    other => {
                        let v = self.value(other);
                        format!("X times, where X is {v}")
                    }
                };
                format!("{c} {t}")
            }
        }
    }

    /// A cost as a verb phrase after "may"/"unless [player]": "pay {2}", "pay 3 life",
    /// "sacrifice a creature".
    pub(crate) fn cost_as_payment(&mut self, c: &Cost) -> String {
        let mut parts: Vec<String> = Vec::new();
        if let Some(m) = &c.mana {
            parts.push(format!("pay {m}"));
        }
        for p in &c.parts {
            match p {
                CostPart::Tap => parts.push("tap ~".into()),
                CostPart::Untap => parts.push("untap ~".into()),
                // "have ~ deal 4 damage to [the player who pays]".
                CostPart::Effect(e) => match &**e {
                    Effect::DealDamage {
                        source: Sel::This,
                        amount,
                        to: Sel::Players(PlayerRef::You),
                    } => {
                        let s = match amount {
                            Value::Const(n) => format!("have ~ deal {n} damage to you"),
                            v => {
                                let v = self.value(v);
                                format!("have ~ deal damage to you equal to {v}")
                            }
                        };
                        parts.push(s);
                    }
                    _ => {
                        let s = self.cost_part(p);
                        parts.push(s);
                    }
                },
                other => {
                    let s = self.cost_part(other);
                    // "pay {1} for each card revealed this way".
                    if matches!(other, CostPart::Repeated { .. }) && s.starts_with('{') {
                        parts.push(format!("pay {s}"));
                    } else {
                        parts.push(s);
                    }
                }
            }
        }
        // "pay {2} and 2 life".
        if parts.len() > 1 && parts.iter().all(|x| x.starts_with("pay ")) {
            for x in parts.iter_mut().skip(1) {
                *x = x["pay ".len()..].to_string();
            }
        }
        join_list(&parts, "and")
    }

    /// An activated ability: "{cost}: {effect} {restrictions}".
    pub(crate) fn activated(&mut self, a: &ActivatedAbility) -> String {
        if let Some(t) = self.class_level_up(a) {
            return t;
        }
        let saved = self.zone;
        self.zone = a.zone;
        let cost = self.cost(&a.cost);
        let saved_salient = self.self_salient;
        // "Sacrifice ~: It deals 2 damage to any target."
        self.self_salient = cost.contains('~');
        let body = self.body(&a.body);
        self.self_salient = saved_salient;
        let mut restr: Vec<String> = Vec::new();
        match a.timing {
            ActivationTiming::Instant => {}
            ActivationTiming::Sorcery => restr.push("as a sorcery".into()),
            ActivationTiming::YourUpkeep => restr.push("during your upkeep".into()),
            ActivationTiming::Combat => restr.push("during combat".into()),
            ActivationTiming::BeforeBlockers => {
                restr.push("during combat before blockers are declared".into())
            }
            ActivationTiming::YourTurn => restr.push("during your turn".into()),
            ActivationTiming::OpponentsTurn => restr.push("during an opponent's turn".into()),
            ActivationTiming::CombatWindow(ct) => restr.push(self.combat_timing(&ct)),
            ActivationTiming::AsInstant => restr.push("as an instant".into()),
        }
        // CR 702.142a: a boast ability can be activated only if the creature attacked this
        // turn and only once each turn; that's what "Boast —" says.
        let boast = self.keyword_ability == Some(crate::keywords::KeywordKind::Boast);
        match a.max_per_turn {
            Some(1) if boast => {}
            None => {}
            Some(1) => restr.push("once each turn".into()),
            Some(2) => restr.push("twice each turn".into()),
            Some(n) => restr.push(format!("{} times each turn", number_word(n as i32))),
        }
        match a.max_total {
            None => {}
            Some(1) => restr.push("once".into()),
            Some(n) => restr.push(format!("{} times", number_word(n as i32))),
        }
        let solved = a.condition.as_ref().is_some_and(super::is_solved);
        // "X can't be 0." (CR 107.3a): the condition on the announced X alone.
        let x_not_zero = matches!(
            &a.condition,
            Some(Condition::Compare(Value::X, Cmp::Ge, Value::Const(1)))
        );
        match a
            .condition
            .as_ref()
            .filter(|_| !solved && !boast && !x_not_zero)
        {
            // "Activate only during your turn before attackers are declared."
            Some(Condition::YourTurn) => {
                if let ActivationTiming::CombatWindow(_) = a.timing {
                    let t = restr.remove(0);
                    restr.insert(0, format!("during your turn {t}"));
                } else {
                    restr.push("during your turn".into());
                }
            }
            Some(c) => {
                let c = self.condition(c);
                restr.push(format!("if {c}"));
            }
            None => {}
        }
        let mut s = format!("{cost}: {body}");
        for oc in &a.own_cost_changes {
            // Payment rules ("Spend only black mana on X") are written below.
            if matches!(oc.change, CostChange::Rule(_)) && oc.condition.is_none() {
                continue;
            }
            let c = self.own_cost_change(oc);
            s = join_words(&[s, c]);
        }
        if !restr.is_empty() {
            let r: Vec<String> = restr
                .iter()
                .enumerate()
                .map(|(i, r)| {
                    if i == 0 {
                        r.clone()
                    } else {
                        format!("only {r}")
                    }
                })
                .collect();
            s = join_words(&[s, format!("Activate only {}.", join_list(&r, "and"))]);
        }
        if a.any_player {
            s.push_str(" Any player may activate this ability.");
        }
        if a.only_opponents {
            s.push_str(" Only your opponents may activate this ability.");
        }
        match (a.cant_be_copied, x_not_zero) {
            (true, true) => s.push_str(" This ability can't be copied and X can't be 0."),
            (true, false) => s.push_str(" This ability can't be copied."),
            (false, true) => s.push_str(" X can't be 0."),
            (false, false) => {}
        }
        // "Spend only black mana on X." (see `payment_rules`): after each mode of a modal
        // ability, as the card says it.
        for r in crate::payment_rules::ability_rules(a) {
            let t = format!(" {}.", capitalize(&super::statics::cost_rule_text(&r)));
            if a.body.modal.is_some() && s.contains("\n•") {
                s = s
                    .lines()
                    .map(|l| {
                        if l.trim_start().starts_with('•') {
                            format!("{l}{t}")
                        } else {
                            l.to_string()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
            } else {
                s.push_str(&t);
            }
        }
        self.zone = saved;
        let _ = third_person;
        if solved {
            return format!("Solved — {s}");
        }
        s
    }
}

trait OwnerYou {
    fn and_owner_you(self) -> Filter;
}

impl OwnerYou for Filter {
    /// Cost zones are the payer's own ("Exile a card from your graveyard").
    fn and_owner_you(self) -> Filter {
        let has_owner = match &self {
            Filter::And(v) => v.iter().any(|f| matches!(f, Filter::OwnedBy(_))),
            Filter::OwnedBy(_) => true,
            _ => false,
        };
        if has_owner {
            self
        } else {
            Filter::and(vec![self, Filter::OwnedBy(PlayerRel::You)])
        }
    }
}
