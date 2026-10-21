//! Numbers players choose (usually secretly) and what the text says about them once
//! they're revealed (`choices.rs`): "each player with the highest number loses that much
//! life", "~ deals damage equal to the highest number to each player who chose that
//! number", "If they match, ..."; and other numbers implemented in code: "if two or more
//! players have lost the game" (CR 800.4), "you gain 1 life for each of the chosen colors
//! it is", "for each black mana symbol in their mana costs".

use super::*;
use crate::choices::{HIGHEST_NUMBER, LOWEST_NUMBER, NUMBERS_MATCH, OWN_NUMBER};

impl Renderer<'_> {
    /// "a number" (any number 0 or greater, capped by the engine at 1000), "a number
    /// greater than 0", "1, 2, or 3".
    pub(crate) fn number_choice(&mut self, min: i32, max: i32) -> String {
        if max == 1000 {
            return match min {
                0 => "{alt:a number|a number 0 or greater}".into(),
                n => format!(
                    "{{alt:a number greater than {}|a number {n} or greater}}",
                    n - 1
                ),
            };
        }
        let list: Vec<String> = (min..=max).map(|n| n.to_string()).collect();
        if (2..=4).contains(&list.len()) {
            let or = join_list(&list, "or");
            return format!("{{alt:a number from {min} to {max}|{or}}}");
        }
        format!("a number from {min} to {max}")
    }

    /// Values about the chosen numbers.
    pub(crate) fn chosen_number_value(&mut self, name: &str) -> Option<String> {
        Some(match name {
            HIGHEST_NUMBER => "{alt:the highest number|the highest number chosen|that much}".into(),
            LOWEST_NUMBER => "the lowest number".into(),
            OWN_NUMBER => "the number they chose".into(),
            crate::oracle::patterns::conditions_compare::PLAYERS_LOST_GAME => {
                "the number of players who have lost the game".into()
            }
            crate::choices::CHOSEN_COLORS_IT_IS => "the number of the chosen colors it is".into(),
            // CR 701.38a: the votes the player performing the instruction received.
            crate::kwa::vote::VOTES_RECEIVED if self.in_as_player => {
                "the number of votes they received".into()
            }
            crate::kwa::vote::VOTES_RECEIVED => "the number of votes you received".into(),
            _ => return None,
        })
    }

    /// A player filter about the chosen numbers, after "each player": "with the highest
    /// number", "who didn't choose the lowest number".
    pub(crate) fn chose_number_filter(&mut self, c: Cmp, v: &Value) -> Option<String> {
        let Value::Custom(n) = v else {
            return None;
        };
        Some(match (c, n.as_str()) {
            (Cmp::Eq, HIGHEST_NUMBER) => {
                "{alt:with the highest number|who chose that number|who chose the highest number}"
                    .into()
            }
            (Cmp::Ne, LOWEST_NUMBER) => "who didn't choose the lowest number".into(),
            (Cmp::Eq, LOWEST_NUMBER) => "who chose the lowest number".into(),
            _ => return None,
        })
    }

    /// Comparisons of custom numbers this module words: "two or more players have lost
    /// the game", "it's at least one of the chosen colors".
    pub(crate) fn custom_compare_named(&mut self, name: &str, cmp: Cmp, n: i32) -> Option<String> {
        match (name, cmp, n) {
            (crate::oracle::patterns::conditions_compare::PLAYERS_LOST_GAME, Cmp::Ge, n) => Some(
                format!("{} or more players have lost the game", number_word(n)),
            ),
            (crate::choices::CHOSEN_COLORS_IT_IS, Cmp::Ge, 1) => {
                Some("it's at least one of the chosen colors".into())
            }
            _ => None,
        }
    }

    /// "If they match" (the chosen numbers).
    pub(crate) fn chosen_number_condition(&mut self, name: &str) -> Option<String> {
        if name == crate::kw::delayed_conditions::SOURCE_STILL_ON_BATTLEFIELD {
            let m = self.me();
            return Some(format!("{m} is still on the battlefield"));
        }
        if name == crate::kw::control_since_your_turn::CONTROLLED_SINCE_YOUR_TURN {
            let m = self.me();
            return Some(format!(
                "you've controlled {m} continuously since the beginning of your most recent turn"
            ));
        }
        (name == NUMBERS_MATCH).then(|| "{alt:they match|those numbers match}".into())
    }

    /// Instruction sequences about chosen numbers and counted qualities: (text, used).
    pub(crate) fn chosen_number_seq(&mut self, v: &[Effect]) -> Option<(String, usize)> {
        match v {
            // "~ deals damage equal to the highest number to each player who chose that
            // number."
            [Effect::ForEachPlayer {
                who: PlayerRef::Each(PlayerFilter::ChoseNumber(c, n)),
                effect,
            }, ..] => {
                let Effect::AsPlayer {
                    who: PlayerRef::Iterated,
                    effect: inner,
                } = effect.as_ref()
                else {
                    return None;
                };
                let Effect::DealDamage {
                    source: Sel::This,
                    amount,
                    to: Sel::Players(PlayerRef::You),
                } = inner.as_ref()
                else {
                    return None;
                };
                let f = self.chose_number_filter(*c, n)?;
                let m = self.me();
                let a = self.value(amount);
                Some((
                    format!("{m} deals damage equal to {a} to each player {f}"),
                    1,
                ))
            }
            // "For each vote an opponent received, ~ deals 2 damage to that player and each
            // creature that player controls." (once for each vote, CR 701.38a).
            [Effect::ForEach {
                sel: Sel::Var(crate::kwa::vote::VOTES_CAST),
                var,
                effect,
            }, ..] => {
                let Effect::If {
                    cond: Condition::PlayerMatches(PlayerRef::Var(p), pf),
                    then,
                    otherwise,
                } = effect.as_ref()
                else {
                    return None;
                };
                if p != var || !matches!(otherwise.as_ref(), Effect::Noop) {
                    return None;
                }
                let who = match pf {
                    PlayerFilter::Opponent => "an opponent",
                    PlayerFilter::You => "you",
                    _ => return None,
                };
                self.target_vars.push((*var, "that player".into(), false));
                let t = self.effect(then);
                Some((format!("for each vote {who} received, {t}"), 1))
            }
            // "For each vote you received, draw a card."
            [Effect::Repeat {
                times: Value::Custom(n),
                effect,
            }, ..]
                if n == crate::kwa::vote::VOTES_RECEIVED && !self.in_as_player =>
            {
                let e = self.effect(effect);
                Some((format!("for each vote you received, {e}"), 1))
            }
            // "you gain 1 life for each of the chosen colors it is".
            [Effect::SetX {
                value: Value::Mul(k, x),
            }, Effect::GainLife {
                who: PlayerRef::You,
                n: Value::X,
            }, ..] => {
                let (Value::Const(k), Value::Custom(name)) = (k.as_ref(), x.as_ref()) else {
                    return None;
                };
                if name != crate::choices::CHOSEN_COLORS_IT_IS {
                    return None;
                }
                Some((
                    format!("you gain {k} life for each of the chosen colors it is"),
                    2,
                ))
            }
            _ => None,
        }
    }

    /// "Spells cost an additional "Sacrifice a Swamp" to cast for each black mana symbol in
    /// their mana costs." (a cost repeated once for each such symbol).
    pub(crate) fn additional_cost_per_symbol(&mut self, cm: &CostModifier) -> Option<String> {
        let CostChange::AdditionalCost(cost) = &cm.change else {
            return None;
        };
        let [CostPart::Repeated { cost: one, times }] = cost.parts.as_slice() else {
            return None;
        };
        let Value::Custom(n) = times else {
            return None;
        };
        let letter = n.strip_prefix("cost: mana symbols of color in the cost:")?;
        let color = crate::types::Color::from_letter(letter.chars().next()?)?;
        if cm.who != PlayerRel::Any || cost.mana.is_some() {
            return None;
        }
        let what = self.cost(one);
        let what = super::capitalize(&what);
        let c = color.word();
        Some(match &cm.applies_to {
            CostTarget::Spells(Filter::Any) => format!(
                "spells cost an additional \"{what}\" to cast for each {c} mana symbol in their mana costs"
            ),
            CostTarget::Abilities(Filter::Any) => format!(
                "activated abilities cost an additional \"{what}\" to activate for each {c} mana symbol in their activation costs"
            ),
            _ => return None,
        })
    }
}

/// What a "the player who controls the fewest" block counts, if `e` is one: (what each
/// player keeps, sacrificed or discarded).
enum Balance {
    /// Permanents matching the filter, sacrificed.
    Sacrifice(Filter),
    /// Cards in hand, discarded.
    Discard,
}

/// "Each player chooses a number of lands they control equal to the number of lands
/// controlled by the player who controls the fewest, then sacrifices the rest."
fn balance_block(e: &Effect) -> Option<Balance> {
    let Effect::ForEachPlayer {
        who: PlayerRef::EachPlayer,
        effect,
    } = e
    else {
        return None;
    };
    let Effect::Seq(v) = effect.as_ref() else {
        return None;
    };
    let [Effect::StoreValue {
        var,
        value: Value::Diff(mine, fewest),
    }, Effect::AsPlayer {
        who: PlayerRef::Iterated,
        effect: act,
    }] = v.as_slice()
    else {
        return None;
    };
    let Value::OverPlayers(AggOp::Min, PlayerFilter::Any, each) = fewest.as_ref() else {
        return None;
    };
    if format!("{mine:?}") != format!("{each:?}") {
        return None;
    }
    match (mine.as_ref(), act.as_ref()) {
        (
            Value::Count(f),
            Effect::Sacrifice {
                who: PlayerRef::You,
                filter,
                count: Value::Var(c),
            },
        ) if c == var => {
            // The count is of what they control on the battlefield of that kind.
            let Filter::And(parts) = f else {
                return None;
            };
            let kept: Vec<Filter> = parts
                .iter()
                .filter(|p| {
                    !matches!(
                        p,
                        Filter::InZone(ZoneKind::Battlefield)
                            | Filter::ControlledBy(PlayerRel::Iterated)
                    )
                })
                .cloned()
                .collect();
            let kind = Filter::and(kept);
            let same = format!("{kind:?}") == format!("{filter:?}")
                || (matches!(kind, Filter::Permanent) && matches!(filter, Filter::Permanent));
            same.then(|| Balance::Sacrifice(filter.clone()))
        }
        (
            Value::HandSize(PlayerRef::Iterated),
            Effect::Discard {
                who: PlayerRef::You,
                n: Value::Var(c),
                random: false,
                filter: Filter::Any,
            },
        ) if c == var => Some(Balance::Discard),
        _ => None,
    }
}

impl Renderer<'_> {
    /// "Each player chooses a number of lands they control equal to the number of lands
    /// controlled by the player who controls the fewest, then sacrifices the rest. Players
    /// sacrifice creatures and discard cards the same way."
    pub(crate) fn balance_seq(&mut self, v: &[Effect]) -> Option<(String, usize)> {
        let first = balance_block(v.first()?)?;
        let Balance::Sacrifice(f) = first else {
            return None;
        };
        let n = self.noun(&f, Num::Many);
        let mut out = format!(
            "each player chooses a number of {n} they control equal to the number of {n} controlled by the player who controls the fewest, then sacrifices the rest"
        );
        let mut same: Vec<String> = Vec::new();
        let mut used = 1;
        for e in &v[1..] {
            match balance_block(e) {
                Some(Balance::Sacrifice(g)) => {
                    let m = self.noun(&g, Num::Many);
                    same.push(format!("sacrifice {m}"));
                }
                Some(Balance::Discard) => same.push("discard cards".into()),
                None => break,
            }
            used += 1;
        }
        if !same.is_empty() {
            let one = same.len() == 1;
            let s = join_list(&same, "and");
            // "Each player discards cards the same way." / "Players sacrifice creatures and
            // discard cards the same way."
            let subj = if one {
                format!(
                    "{{alt:each player {}|players {s}}}",
                    effects::third_person(&s)
                )
            } else {
                format!("players {s}")
            };
            out.push_str(&format!(". {subj} the same way"));
        }
        Some((out, used))
    }
}
