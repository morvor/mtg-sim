//! Amounts from this turn's events ([`Value::EventsThisTurn`], [`Sel::ThisTurn`]),
//! rendered the way the value grammar reads them (`oracle/patterns/value_results.rs`).

use super::players::Case;
use super::*;

/// A filter without one of its conjuncts (and whether it had it).
fn without(f: &Filter, part: &Filter) -> (Filter, bool) {
    let same = |a: &Filter| format!("{a:?}") == format!("{part:?}");
    match f {
        Filter::And(v) if v.iter().any(same) => {
            let rest: Vec<Filter> = v.iter().filter(|x| !same(x)).cloned().collect();
            (Filter::and(rest), true)
        }
        f if same(f) => (Filter::Any, true),
        f => (f.clone(), false),
    }
}

impl Renderer<'_> {
    /// "you've", "your opponents have", "that player has".
    fn rel_perfect(&mut self, r: PlayerRel) -> String {
        match r {
            PlayerRel::You => "you've".into(),
            PlayerRel::Opponent => "{alt:your opponents have|an opponent has}".into(),
            PlayerRel::Any => "a player has".into(),
            PlayerRel::TriggerPlayer | PlayerRel::Iterated => {
                "{alt:that player has|they've}".into()
            }
            other => {
                let p = self.rel_subject(other);
                // A player named before ("Target player draws two cards, then ... the
                // number of cards they've drawn this turn").
                if p == "that player" {
                    return "{alt:that player has|they've}".into();
                }
                format!("{p} has")
            }
        }
    }

    fn many(&mut self, f: &Filter, default: &str) -> String {
        if matches!(f, Filter::Any) {
            default.to_string()
        } else {
            self.noun(f, Num::Many)
        }
    }

    pub(crate) fn events_this_turn(&mut self, c: &TriggerCond, t: Tally) -> String {
        match (c, t) {
            (TriggerCond::Dies(f), Tally::Events) => {
                let (f, yours) = without(f, &Filter::ControlledBy(PlayerRel::You));
                let (f, owned) = without(&f, &Filter::OwnedBy(PlayerRel::You));
                let n = self.many(&f, "permanents");
                if owned {
                    format!(
                        "the number of {n} put into your graveyard from the battlefield this turn"
                    )
                } else if yours {
                    format!("the number of {n} that died under your control this turn")
                } else {
                    format!(
                        "the number of {n} {{alt:that died|that were put into graveyards from the battlefield}} this turn"
                    )
                }
            }
            (TriggerCond::Attacks(f), Tally::Events) => {
                let (f, yours) = without(f, &Filter::ControlledBy(PlayerRel::You));
                if let Filter::In(s) = &f {
                    let s = self.sel(s, Case::Subj);
                    return format!("the number of times {s} has attacked this turn");
                }
                let n = self.many(&f, "creatures");
                if yours {
                    format!("the number of {n} you attacked with this turn")
                } else {
                    format!("the number of {n} that attacked this turn")
                }
            }
            (TriggerCond::CastSpell { who, filter }, Tally::Events) => {
                let (f, other) = without(filter, &Filter::Other);
                let (f, _) = without(&f, &Filter::Spell);
                let n = if matches!(f, Filter::Any) {
                    "spells".to_string()
                } else {
                    let n = self.noun(&f, Num::Many);
                    if n.contains("spell") {
                        n
                    } else {
                        format!("{n} spells")
                    }
                };
                let other = if other { "other " } else { "" };
                if matches!(who, PlayerRel::Any) {
                    return format!("the number of {other}{n} cast this turn");
                }
                let p = self.rel_perfect(*who);
                format!("the number of {other}{n} {p} cast this turn")
            }
            (TriggerCond::Draws { who }, Tally::Events) => {
                let p = self.rel_perfect(*who);
                // "they've drawn" said as one phrase in each alternative.
                let p = match p
                    .strip_prefix("{alt:")
                    .and_then(|r| r.strip_suffix('}'))
                    .and_then(|r| r.split_once('|'))
                {
                    Some((a, b)) if !a.contains('{') && !b.contains('{') => {
                        return format!(
                            "the number of cards {{alt:{a} drawn|{b} drawn}} this turn"
                        );
                    }
                    _ => p,
                };
                format!("the number of cards {p} drawn this turn")
            }
            (TriggerCond::Discards { who, filter }, Tally::Events) => {
                let n = self.many(filter, "cards");
                let p = self.rel_perfect(*who);
                format!("the number of {n} {p} {{alt:discarded|cycled or discarded}} this turn")
            }
            (TriggerCond::YouSacrifice(f), Tally::Events) => {
                let n = self.many(f, "permanents");
                format!("the number of {n} you've sacrificed this turn")
            }
            (TriggerCond::Sacrificed(f), Tally::Events) => {
                let n = self.many(f, "permanents");
                format!("the number of {n} sacrificed this turn")
            }
            (TriggerCond::TokenCreated(f), Tally::Events) => {
                let (f, _) = without(f, &Filter::ControlledBy(PlayerRel::You));
                let n = self.many(&f, "tokens");
                format!("the number of {n} you created this turn")
            }
            (TriggerCond::GainsLife { who }, Tally::Amount) => {
                let p = self.rel_subject(*who);
                format!("the amount of life {p} {{alt:gained|'ve gained|have gained}} this turn")
            }
            (TriggerCond::LosesLife { who }, Tally::Amount) => match who {
                PlayerRel::Opponent => {
                    "the {opt:total} amount of life your opponents {alt:lost|have lost} this turn"
                        .into()
                }
                PlayerRel::Any => "the total life lost by all players this turn".into(),
                other => {
                    let p = self.rel_subject(*other);
                    format!(
                        "the {{opt:amount of}} life {p} {{alt:lost|'ve lost|have lost}} this turn"
                    )
                }
            },
            (TriggerCond::PlayerDealtDamage { who, combat_only }, Tally::Amount) => {
                let p = match who {
                    PlayerRel::Opponent => "your opponents".to_string(),
                    r => self.rel_object(*r),
                };
                let combat = if *combat_only { "combat " } else { "" };
                format!(
                    "the {{opt:amount of}} {combat}damage {{opt:already}} dealt to {p} this turn"
                )
            }
            (TriggerCond::DealsDamage { source, to, .. }, Tally::Amount) => {
                let to = match to {
                    DamageRecipient::Player(r) => self.rel_object(*r),
                    DamageRecipient::Object(Filter::Source) => self.me(),
                    DamageRecipient::Object(Filter::In(s)) => self.sel(s, Case::Obj),
                    _ => return self.gap(format!("EventsThisTurn({c:?})")),
                };
                if matches!(source, Filter::Any) {
                    return format!(
                        "the {{opt:amount of}} damage {{opt:already}} dealt to {to} this turn"
                    );
                }
                let s = self.noun(source, Num::Many);
                format!(
                    "the {{opt:amount of}} damage dealt to {to} {{opt:so far}} this turn by {s}"
                )
            }
            // "the number of cards that were put into target player's graveyard from their
            // library this turn" (cards go to their owner's graveyard, CR 404.2).
            (
                TriggerCond::ZoneChange {
                    filter,
                    from: Some(from),
                    to: Some(ZoneKind::Graveyard),
                },
                Tally::Events,
            ) => match self.put_into_graveyard_from(filter, &[*from]) {
                Some(s) => s,
                None => self.gap(format!("EventsThisTurn({c:?})")),
            },
            // "the number of your opponents who were dealt combat damage by ~ or a Dragon
            // this turn".
            (
                TriggerCond::DealsDamage {
                    source,
                    to: DamageRecipient::Player(who),
                    combat_only,
                },
                Tally::Players,
            ) if !matches!(source, Filter::Any) => {
                let combat = if *combat_only { "combat " } else { "" };
                let by = self.noun_det(source, super::nouns::Det::A);
                let n = match who {
                    PlayerRel::Opponent => "{alt:opponents|your opponents}",
                    _ => "players",
                };
                format!(
                    "the number of {n} {{alt:who|that}} {{alt:was|were}} dealt {combat}damage by {by} this turn"
                )
            }
            (cond, Tally::Players) => {
                let (who, verb) = match cond {
                    TriggerCond::LosesLife { who } => (*who, "lost life"),
                    TriggerCond::GainsLife { who } => (*who, "gained life"),
                    TriggerCond::Discards { who, .. } => (*who, "discarded a card"),
                    TriggerCond::Draws { who } => (*who, "drew a card"),
                    TriggerCond::PlayerDealtDamage {
                        who,
                        combat_only: true,
                    } => (*who, "{alt:was|were} dealt combat damage"),
                    TriggerCond::PlayerDealtDamage { who, .. } => {
                        (*who, "{alt:was|were} dealt damage")
                    }
                    _ => return self.gap(format!("EventsThisTurn({c:?})")),
                };
                let n = match who {
                    PlayerRel::Opponent => "{alt:opponents|your opponents}",
                    _ => "players",
                };
                format!("the number of {n} {{alt:who|that}} {verb} this turn")
            }
            (TriggerCond::CountersPutBy { kind, .. }, Tally::Amount) => {
                let k = kind.as_ref().map(|k| k.to_string()).unwrap_or_default();
                format!(
                    "the number of {k} counters you've put on creatures under your control this turn"
                )
            }
            _ => self.gap(format!("EventsThisTurn({c:?})")),
        }
    }
}

/// Whether `v` sums the cards of one kind put into a graveyard from several zones this turn
/// (see [`Renderer::graveyard_arrivals_sum`]).
pub(crate) fn graveyard_arrivals(v: &[Value]) -> bool {
    let parts: Option<Vec<(String, ZoneKind)>> = v
        .iter()
        .map(|x| match x {
            Value::EventsThisTurn(c, Tally::Events) => match c.as_ref() {
                TriggerCond::ZoneChange {
                    filter: f @ Filter::And(_),
                    from: Some(z),
                    to: Some(ZoneKind::Graveyard),
                } => Some((format!("{f:?}"), *z)),
                _ => None,
            },
            _ => None,
        })
        .collect();
    parts.is_some_and(|p| p.len() >= 2 && p.iter().all(|(f, _)| *f == p[0].0))
}

impl Renderer<'_> {
    /// "the number of cards that were put into your graveyard from your hand or library
    /// this turn": the cards of `filter` (owned by one player) put into that player's
    /// graveyard from any of `from`.
    fn put_into_graveyard_from(&mut self, filter: &Filter, from: &[ZoneKind]) -> Option<String> {
        let Filter::And(v) = filter else {
            return None;
        };
        let owner = v.iter().find_map(|x| match x {
            Filter::OwnedBy(r) => Some(*r),
            _ => None,
        })?;
        let rest: Vec<Filter> = v
            .iter()
            .filter(|x| !matches!(x, Filter::OwnedBy(_)))
            .cloned()
            .collect();
        let n = self.many(&Filter::and(rest), "cards");
        let p = self.rel_possessive(owner, Num::One);
        let pp = if p == "your" { "your" } else { "their" };
        let zones: Vec<&str> = from
            .iter()
            .map(|z| match z {
                ZoneKind::Hand => Some("hand"),
                ZoneKind::Library => Some("library"),
                ZoneKind::Battlefield => Some("battlefield"),
                ZoneKind::Exile => Some("exile"),
                ZoneKind::Stack => Some("stack"),
                _ => None,
            })
            .collect::<Option<Vec<_>>>()?;
        let z = join_list(
            &zones.iter().map(|z| z.to_string()).collect::<Vec<_>>(),
            "or",
        );
        let from = match zones.as_slice() {
            ["battlefield"] => "the battlefield".to_string(),
            _ => format!("{pp} {z}"),
        };
        Some(format!(
            "the number of {n} {{alt:that were|}} put into {p} graveyard from {from} this turn"
        ))
    }

    /// The sum of the cards put into one graveyard from several zones this turn, as one
    /// number: "from your hand or library".
    pub(crate) fn graveyard_arrivals_sum(&mut self, v: &[Value]) -> Option<String> {
        let mut filter = None;
        let mut zones = Vec::new();
        for x in v {
            let Value::EventsThisTurn(c, Tally::Events) = x else {
                return None;
            };
            let TriggerCond::ZoneChange {
                filter: f,
                from: Some(z),
                to: Some(ZoneKind::Graveyard),
            } = c.as_ref()
            else {
                return None;
            };
            if filter
                .as_ref()
                .is_some_and(|g: &Filter| format!("{g:?}") != format!("{f:?}"))
            {
                return None;
            }
            filter = Some(f.clone());
            zones.push(*z);
        }
        if zones.len() < 2 {
            return None;
        }
        self.put_into_graveyard_from(&filter?, &zones)
    }
}
