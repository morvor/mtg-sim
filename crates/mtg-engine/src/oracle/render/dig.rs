//! Rendering of the dig steps ([`DigStep`], `dig_steps.rs`): "you may put a land card from
//! among them into your hand", "put the rest on the bottom of your library in a random
//! order", "reveal cards from the top of your library until you reveal two land cards".

use super::*;

impl Renderer<'_> {
    /// Where cards go, as a dig's text says it ("into your hand", "onto the battlefield
    /// tapped", "on the bottom of your library in any order").
    fn dig_destination(&mut self, to: &Destination, many: bool) -> String {
        let mut d = match to.zone {
            ZoneKind::Hand => "into your hand".to_string(),
            ZoneKind::Graveyard => "into your graveyard".to_string(),
            ZoneKind::Exile => "into exile".to_string(),
            ZoneKind::Library => match to.position {
                LibraryPosition::Top if many => "on top of your library in any order".into(),
                LibraryPosition::Top => "on top of your library".into(),
                LibraryPosition::Bottom if many => {
                    "{alt:on the bottom of your library|on the bottom} in any order".into()
                }
                LibraryPosition::Bottom => "on the bottom of your library".into(),
                LibraryPosition::BottomRandom => {
                    "{alt:on the bottom of your library|on the bottom} in a random order".into()
                }
                LibraryPosition::FromTop(_) => String::new(),
                LibraryPosition::Shuffled => "into your library".into(),
            },
            _ => self.destination_phrase(to, many, true),
        };
        if to.zone == ZoneKind::Battlefield {
            d = "onto the battlefield".into();
            if to.tapped {
                d.push_str(" tapped");
            }
            if to.attacking {
                d.push_str(" and attacking");
            }
            if let Some(c) = &to.controller {
                if !matches!(c, PlayerRef::You) {
                    let p = self.player(c, super::players::Case::Poss);
                    d.push_str(&format!(" under {p} control"));
                } else {
                    // The player putting it there controls it (CR 110.2a): cards may say
                    // so.
                    d.push_str(" {opt:under your control}");
                }
            }
            for (k, n) in &to.with_counters {
                let name = counter_name(&format!("{k:?}").to_lowercase());
                let n = match n {
                    Value::Const(1) => with_article(&format!("{name} counter")),
                    Value::Const(k) => format!("{} {name} counters", number_word(*k)),
                    other => format!("{} {name} counters", self.value(other)),
                };
                let it = if many { "them" } else { "it" };
                d.push_str(&format!(" with {n} on {it}"));
            }
        }
        d
    }

    pub(crate) fn dig_step(&mut self, step: &DigStep) -> String {
        match step {
            DigStep::Take {
                from,
                filter,
                each_of,
                count,
                up_to,
                random,
                reveal,
                to,
                ..
            } => {
                // "Put that card onto the battlefield": the card a reveal found (all of
                // one card).
                if matches!(from, Sel::Var(v) if *v == vars::IT)
                    && count.is_none()
                    && matches!(filter, Filter::Any)
                    && each_of.is_empty()
                    && !*reveal
                {
                    let d = self.dig_destination(to, false);
                    return if self.in_as_player {
                        format!("{{alt:the player|that player}} puts that card {d}")
                    } else {
                        format!("put that card {d}")
                    };
                }
                let many = !matches!(count, Some(Value::Const(1)));
                let what = if !each_of.is_empty() {
                    let names: Vec<String> = each_of
                        .iter()
                        .map(|f| {
                            let n = self.card_noun(f);
                            with_article(&n)
                        })
                        .collect();
                    join_list(&names, "and/or")
                } else {
                    let noun = self.card_noun(filter);
                    match count {
                        None => format!("all {}", plural(&noun)),
                        Some(Value::Const(1)) if *random => with_article(&format!("random {noun}")),
                        Some(Value::Const(1)) => with_article(&noun),
                        Some(Value::Const(k)) if *k >= 999 => {
                            format!("any number of {}", plural(&noun))
                        }
                        Some(Value::Const(k)) if *up_to => {
                            format!("up to {} {}", number_word(*k), plural(&noun))
                        }
                        Some(Value::Const(k)) => format!("{} {}", number_word(*k), plural(&noun)),
                        Some(v) => {
                            let v = self.value(v);
                            format!("{v} {}", plural(&noun))
                        }
                    }
                };
                let may = if *up_to && each_of.is_empty() && matches!(count, Some(Value::Const(1)))
                    || (!each_of.is_empty() && *up_to)
                {
                    "you may "
                } else if matches!(count, Some(Value::Const(k)) if *k >= 999) {
                    // Any number includes none: "you may reveal any number of ...".
                    "{opt:you may} "
                } else {
                    ""
                };
                let verb = if *reveal { "reveal" } else { "put" };
                let d = self.dig_destination(to, many);
                // "You may reveal a land card" / "reveal up to one land card".
                if *reveal
                    && *up_to
                    && each_of.is_empty()
                    && !*random
                    && matches!(count, Some(Value::Const(1)))
                {
                    let noun = self.card_noun(filter);
                    let head = format!(
                        "{{alt:you may reveal {what}|reveal up to one {noun}}} from among them"
                    );
                    // Kept among them for the next instruction (`dig_steps.rs`).
                    if d.is_empty() {
                        return head;
                    }
                    return format!("{head} and put {{alt:it|the revealed card}} {d}");
                }
                if *reveal && !d.is_empty() {
                    let it = if many {
                        "{alt:them|the revealed cards}"
                    } else {
                        "{alt:it|the revealed card}"
                    };
                    format!("{may}reveal {what} from among them and put {it} {d}")
                } else if d.is_empty() {
                    format!("{may}{verb} {what} from among them")
                } else {
                    format!("{may}{verb} {what} from among them {d}")
                }
            }
            DigStep::Rest { to, .. } => {
                let d = self.dig_destination(to, true);
                // "Shuffle the rest into your library" (not put in place).
                if to.zone == ZoneKind::Library && to.position == LibraryPosition::Shuffled {
                    return if self.in_as_player {
                        format!("{{alt:the player|that player}} shuffles the rest {d}")
                    } else {
                        format!("shuffle the rest {d}")
                    };
                }
                format!("put the rest {d}")
            }
            DigStep::Until {
                who,
                filter,
                count,
                exile,
            } => {
                let p = self.possessive_for(who);
                let noun = self.card_noun(filter);
                let n = match count {
                    Value::Const(1) => with_article(&noun),
                    Value::Const(k) => format!("{} {}", number_word(*k), plural(&noun)),
                    other => {
                        let v = self.value(other);
                        format!("{v} {}", plural(&noun))
                    }
                };
                let verb = if *exile { "exile" } else { "reveal" };
                let subj = if p == "your" { "you" } else { "they" };
                let vp =
                    format!("{verb} cards from the top of {p} library until {subj} {verb} {n}");
                self.with_subject(who, &vp, false)
            }
        }
    }
}
