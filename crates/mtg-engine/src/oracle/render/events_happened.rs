//! Whether, or how often, something happened in a stretch of the game, as a condition:
//! comparisons of [`Value::EventsThisTurn`] and [`Value::EventsDuring`] with a number
//! ("if ~ attacked or blocked this turn", "if you didn't play a land this turn", "if three
//! or more cards left your graveyard this turn", "if no permanents left the battlefield
//! this turn", "if an opponent lost life last turn", "if it attacked during your last
//! turn"), and of the objects those events were about ([`Sel::ThisTurn`]: "if you attacked
//! with three or more Merfolk this turn").
//!
//! The events are those a triggered ability with the same trigger condition would trigger
//! on (CR 603.1b), so each clause words the trigger event in the past tense.

use super::nouns::Det;
use super::players::Case;
use super::*;

/// How many of the events the condition asks for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Quant {
    /// One or more.
    Some,
    /// None.
    No,
    /// At least this many (2 or more).
    AtLeast(i32),
    /// Exactly one: the event being looked at is the first ("if it's the first time").
    First,
}

impl Quant {
    /// The quantity a comparison of a count with a constant asks for, if it's one of these.
    pub(crate) fn of(cmp: Cmp, n: i32, negated: bool) -> Option<Quant> {
        let q = match (cmp, n) {
            (Cmp::Ge, 1) | (Cmp::Gt, 0) => Quant::Some,
            (Cmp::Ge, n) if n >= 2 => Quant::AtLeast(n),
            (Cmp::Gt, n) if n >= 1 => Quant::AtLeast(n + 1),
            (Cmp::Eq, 0) | (Cmp::Lt, 1) | (Cmp::Le, 0) => Quant::No,
            (Cmp::Eq, 1) => Quant::First,
            _ => return None,
        };
        Some(match (q, negated) {
            (q, false) => q,
            (Quant::Some, true) => Quant::No,
            (Quant::No, true) => Quant::Some,
            _ => return None,
        })
    }
}

/// A past event about one definite subject: "~ attacked" / "~ didn't attack".
struct Def {
    subj: String,
    /// "attacked", "was dealt damage".
    past: String,
    /// "didn't", "wasn't", "weren't".
    neg_aux: &'static str,
    /// "attack", "dealt damage".
    neg_rest: String,
    /// The predicate says how many (or how much): "lost 4 or more life", "cast three or
    /// more spells".
    counted: bool,
}

impl Def {
    fn new(
        subj: String,
        past: impl Into<String>,
        neg_aux: &'static str,
        neg_rest: impl Into<String>,
    ) -> Def {
        Def {
            subj,
            past: past.into(),
            neg_aux,
            neg_rest: neg_rest.into(),
            counted: false,
        }
    }

    fn counted(mut self) -> Def {
        self.counted = true;
        self
    }

    fn counted_if(self, yes: bool) -> Def {
        if yes {
            self.counted()
        } else {
            self
        }
    }

    /// The clause for `q`, without the stretch of time.
    fn say(&self, q: Quant) -> Option<String> {
        match q {
            Quant::Some => Some(format!("{} {}", perfect(&self.subj), self.past)),
            Quant::AtLeast(_) if self.counted => {
                Some(format!("{} {}", perfect(&self.subj), self.past))
            }
            Quant::No => Some(format!("{} {} {}", self.subj, self.neg_aux, self.neg_rest)),
            _ => None,
        }
    }
}

/// "you've" for "you" before a past participle that's also the past tense ("you've
/// played a land", "you've cast a spell"): cards say either.
fn perfect(subj: &str) -> String {
    if subj == "you" {
        "you've".into()
    } else {
        subj.to_string()
    }
}

/// What a past event is about.
enum Said {
    /// One definite subject.
    Def(Def),
    /// A clause already worded for the quantity, without the stretch of time.
    Clause(String),
    /// A complete clause, with the stretch of time in it.
    Full(String),
}

/// The player whose objects the filter is limited to ("you control"), and the rest.
fn controller_of(f: &Filter) -> (Option<PlayerRel>, Filter) {
    match f {
        Filter::And(v) => {
            let r = v.iter().find_map(|x| match x {
                Filter::ControlledBy(r) => Some(*r),
                _ => None,
            });
            let rest = v
                .iter()
                .filter(|x| !matches!(x, Filter::ControlledBy(_)))
                .cloned()
                .collect();
            (r, Filter::and(rest))
        }
        Filter::ControlledBy(r) => (Some(*r), Filter::Any),
        f => (None, f.clone()),
    }
}

/// The owner the filter is limited to ("you own"), and the rest.
fn owner_of(f: &Filter) -> (Option<PlayerRel>, Filter) {
    match f {
        Filter::And(v) => {
            let r = v.iter().find_map(|x| match x {
                Filter::OwnedBy(r) => Some(*r),
                _ => None,
            });
            let rest = v
                .iter()
                .filter(|x| !matches!(x, Filter::OwnedBy(_)))
                .cloned()
                .collect();
            (r, Filter::and(rest))
        }
        Filter::OwnedBy(r) => (Some(*r), Filter::Any),
        f => (None, f.clone()),
    }
}

impl Renderer<'_> {
    /// "this turn", "last turn", "during your last turn", "since your last upkeep".
    fn window_phrase(&mut self, w: Option<&HistoryWindow>) -> String {
        let Some(w) = w else {
            return "this turn".into();
        };
        match w {
            HistoryWindow::ThisCombat => "this combat".into(),
            HistoryWindow::LastTurn => "last turn".into(),
            HistoryWindow::PlayersLastTurn(p) => {
                let poss = self.player(p, Case::Poss);
                format!("during {poss} last turn")
            }
            HistoryWindow::SinceLastTurn(p) => {
                let poss = self.player(p, Case::Poss);
                format!("since {poss} last turn")
            }
            HistoryWindow::SinceLastUpkeep(p) => {
                let poss = self.player(p, Case::Poss);
                format!("since {poss} last upkeep")
            }
        }
    }

    /// The subject a filter names when it's one particular object: "~", "it", "enchanted
    /// creature".
    fn definite(&mut self, f: &Filter) -> Option<String> {
        match f {
            Filter::Source => Some(self.me()),
            Filter::In(s) if !matches!(s.as_ref(), Sel::All(_)) => Some(self.sel(s, Case::Subj)),
            _ => None,
        }
    }

    /// The object of a verb when it's one particular object, else "a [noun]".
    fn object_noun(&mut self, f: &Filter) -> String {
        match f {
            Filter::Source => self.me(),
            Filter::In(s) if !matches!(s.as_ref(), Sel::All(_)) => self.sel(s, Case::Obj),
            f => self.noun_det(f, Det::A),
        }
    }

    /// A quantified subject and its verb: "a creature card was put", "one or more cards
    /// were put", "three or more cards were put", "no permanents were put".
    fn quantified(&mut self, f: &Filter, q: Quant, sing: &str, plur: &str) -> Option<String> {
        Some(match q {
            // "one or more cards left your graveyard", "a creature card left your
            // graveyard".
            Quant::Some if matches!(f, Filter::Card | Filter::Any) => {
                let many = self.noun(f, Num::Many);
                format!("one or more {many} {plur}")
            }
            Quant::Some => {
                let a = self.noun_det(f, Det::A);
                format!("{a} {sing}")
            }
            Quant::AtLeast(n) => {
                let many = self.noun(f, Num::Many);
                format!("{} or more {many} {plur}", number_word(n))
            }
            Quant::No => {
                let many = self.noun(f, Num::Many);
                format!("no {many} {plur}")
            }
            Quant::First => return None,
        })
    }

    /// A player subject: "you", "an opponent", "that player"; `iter` stands for the
    /// player a comparison over players is about ("if an opponent lost life").
    fn past_player(&mut self, r: PlayerRel, iter: Option<&str>) -> String {
        match (r, iter) {
            (PlayerRel::Iterated, Some(s)) => s.to_string(),
            (r, _) => self.rel_subject(r),
        }
    }

    /// "was"/"were" for a player subject.
    fn be_past(subj: &str, neg: bool) -> &'static str {
        match (subj == "you", neg) {
            (true, false) => "were",
            (true, true) => "weren't",
            (false, false) => "was",
            (false, true) => "wasn't",
        }
    }

    /// "life", "4 or more life".
    fn amount_of(q: Quant, what: &str) -> Option<String> {
        match q {
            Quant::Some | Quant::No => Some(what.to_string()),
            Quant::AtLeast(n) => Some(format!("{n} or more {what}")),
            Quant::First => None,
        }
    }

    /// A trigger event in the past, for `q` events. `amount`: the events are tallied by
    /// their amounts (life, damage), so `q` is about the amount.
    fn said(
        &mut self,
        c: &TriggerCond,
        q: Quant,
        amount: bool,
        iter: Option<&str>,
    ) -> Option<Said> {
        // A player subject with a simple past: "you gained life" / "you didn't gain life".
        let player_event = |r: &mut Self, who: PlayerRel, past: &str, neg_rest: &str| -> Said {
            let subj = r.past_player(who, iter);
            Said::Def(Def::new(subj, past, "didn't", neg_rest))
        };
        // The same, with the amount in the predicate ("lost 4 or more life").
        let counted_event = |r: &mut Self, who: PlayerRel, past: &str, neg_rest: &str| -> Said {
            let subj = r.past_player(who, iter);
            Said::Def(Def::new(subj, past, "didn't", neg_rest).counted())
        };
        Some(match c {
            TriggerCond::Attacks(f) if !amount => {
                if let Some(s) = self.definite(f) {
                    Said::Def(Def::new(s, "attacked", "didn't", "attack"))
                } else {
                    // "you attacked with a Hero", "a Pirate attacked".
                    let (r, rest) = controller_of(f);
                    match r {
                        Some(r) => {
                            let who = self.past_player(r, iter);
                            let n = self.quantified_object(&rest, q)?;
                            Said::Clause(format!("{who} attacked with {n}"))
                        }
                        None => Said::Clause(self.quantified(f, q, "attacked", "attacked")?),
                    }
                }
            }
            TriggerCond::Blocks(f) if !amount => {
                let s = self.definite(f)?;
                Said::Def(Def::new(s, "blocked", "didn't", "block"))
            }
            TriggerCond::BecomesBlocked(f) if !amount => {
                let s = self.definite(f)?;
                Said::Def(Def::new(s, "was blocked", "wasn't", "blocked"))
            }
            TriggerCond::EntersBattlefield(f) if !amount => {
                if let Some(s) = self.definite(f) {
                    return Some(Said::Def(Def::new(
                        s,
                        "entered {opt:the battlefield}",
                        "didn't",
                        "enter {opt:the battlefield}",
                    )));
                }
                // "a creature entered the battlefield under an opponent's control".
                let (r, rest) = controller_of(f);
                match r {
                    Some(r) => {
                        let poss = self.rel_possessive(r, Num::One);
                        let s = self.quantified(&rest, q, "entered", "entered")?;
                        let main = format!("{s} the battlefield under {poss} control");
                        // "an opponent had a green creature enter the battlefield under
                        // their control".
                        if q == Quant::Some && r != PlayerRel::You {
                            let who = self.rel_subject(r);
                            let a = self.noun_det(&rest, Det::A);
                            Said::Clause(format!(
                                "{{alt:{main}|{who} had {a} enter the battlefield under their control}}"
                            ))
                        } else {
                            Said::Clause(main)
                        }
                    }
                    None => {
                        let s = self.quantified(f, q, "entered", "entered")?;
                        Said::Clause(format!("{s} the battlefield"))
                    }
                }
            }
            // "~ came under your control": it entered under your control or its control
            // changed to you.
            TriggerCond::AnyOf(v)
                if !amount
                    && v.len() == 2
                    && matches!(
                        (&v[0], &v[1]),
                        (TriggerCond::EntersBattlefield(a), TriggerCond::ControlChanged(b))
                            if format!("{a:?}") == format!("{b:?}")
                    ) =>
            {
                let TriggerCond::EntersBattlefield(f) = &v[0] else {
                    return None;
                };
                let (r, rest) = controller_of(f);
                let s = self.definite(&rest)?;
                let poss = self.rel_possessive(r?, Num::One);
                Said::Def(Def::new(
                    s,
                    format!("came under {poss} control"),
                    "didn't",
                    format!("come under {poss} control"),
                ))
            }
            TriggerCond::LeavesBattlefield(f) if !amount => {
                if let Some(s) = self.definite(f) {
                    return Some(Said::Def(Def::new(
                        s,
                        "left the battlefield",
                        "didn't",
                        "leave the battlefield",
                    )));
                }
                let (r, rest) = controller_of(f);
                let s = self.quantified(&rest, q, "left", "left")?;
                match r {
                    Some(r) => {
                        let poss = self.rel_possessive(r, Num::One);
                        Said::Clause(format!("{s} the battlefield under {poss} control"))
                    }
                    None => Said::Clause(format!("{s} the battlefield")),
                }
            }
            TriggerCond::ZoneChange { filter, from, to } if !amount => {
                Said::Clause(self.zone_change_past(filter, *from, *to, q, iter)?)
            }
            // "three or more cards were put into your graveyard from anywhere other than
            // the battlefield".
            TriggerCond::AnyOf(v) if !amount && self.from_anywhere_but(v).is_some() => {
                let (filter, to, but) = self.from_anywhere_but(v)?;
                let s = self.zone_change_past(&filter, None, Some(to), q, iter)?;
                let z = zone_word(but);
                let z = if z == "battlefield" {
                    "the battlefield".to_string()
                } else {
                    format!("your {z}")
                };
                Said::Clause(format!("{s} other than {z}"))
            }
            // "your team gained life": you and/or your teammates (CR 102.4).
            TriggerCond::AnyOf(v)
                if matches!(
                    v.as_slice(),
                    [
                        TriggerCond::GainsLife {
                            who: PlayerRel::You
                        },
                        TriggerCond::GainsLife {
                            who: PlayerRel::Teammate
                        }
                    ] | [
                        TriggerCond::LosesLife {
                            who: PlayerRel::You
                        },
                        TriggerCond::LosesLife {
                            who: PlayerRel::Teammate
                        }
                    ]
                ) =>
            {
                let Said::Def(mut d) = self.said(&v[0], q, amount, iter)? else {
                    return None;
                };
                d.subj = "your team".into();
                Said::Def(d)
            }
            TriggerCond::GainsLife { who } => {
                let a = Self::amount_of(q, "life")?;
                counted_event(self, *who, &format!("gained {a}"), &format!("gain {a}"))
            }
            TriggerCond::LosesLife { who } => {
                let a = Self::amount_of(q, "life")?;
                counted_event(self, *who, &format!("lost {a}"), &format!("lose {a}"))
            }
            // "you've cast a spell from a graveyard", "an opponent cast three or more
            // spells".
            TriggerCond::CastSpell { who, filter } if !amount => {
                let atoms: Vec<Filter> = match filter {
                    Filter::And(v) => v.clone(),
                    f => vec![f.clone()],
                };
                let from = atoms.iter().find_map(|x| match x {
                    Filter::CastFrom(z) => Some(*z),
                    _ => None,
                });
                let rest = Filter::and(
                    atoms
                        .into_iter()
                        .filter(|x| !matches!(x, Filter::CastFrom(_) | Filter::Spell))
                        .collect(),
                );
                let noun = if matches!(rest, Filter::Any) {
                    Filter::Spell
                } else {
                    Filter::and(vec![rest, Filter::Spell])
                };
                let n = self.quantified_object(&noun, q)?;
                let from = match from {
                    None => String::new(),
                    Some(ZoneKind::Exile) => " from exile".into(),
                    Some(ZoneKind::Graveyard) => " from a graveyard".into(),
                    Some(z) => format!(" from your {}", zone_word(z)),
                };
                let subj = self.past_player(*who, iter);
                match q {
                    Quant::No => Said::Clause(format!("{subj} didn't cast a spell{from}")),
                    _ => Said::Def(
                        Def::new(
                            subj,
                            format!("cast {n}{from}"),
                            "didn't",
                            format!("cast {n}{from}"),
                        )
                        .counted(),
                    ),
                }
            }
            TriggerCond::LandPlayed {
                who,
                filter: Filter::Any,
            } if !amount => player_event(self, *who, "played a land", "play a land"),
            TriggerCond::Searched(who) if !amount => {
                let subj = self.past_player(*who, iter);
                let poss = if subj == "you" { "your" } else { "their" };
                Said::Def(Def::new(
                    subj,
                    format!("searched {poss} library"),
                    "didn't",
                    format!("search {poss} library"),
                ))
            }
            TriggerCond::Cycled { who, filter } if !amount => {
                let subj = self.past_player(*who, iter);
                let f = if matches!(filter, Filter::Any) {
                    Filter::Card
                } else {
                    filter.clone()
                };
                let n = self.quantified_object(&f, q)?;
                Said::Clause(format!("{subj} cycled {n}"))
            }
            TriggerCond::PlayerAction { name, who } if !amount => {
                let (past, bare) = match name.as_str() {
                    "scry" => ("scried", "scry"),
                    "surveil" => ("surveilled", "surveil"),
                    _ => return None,
                };
                player_event(self, *who, past, bare)
            }
            // "you've activated an ability of a card in a graveyard".
            TriggerCond::AbilityActivated {
                who,
                source,
                include_mana: true,
            } if !amount && !matches!(source, Filter::Any) => {
                let n = self.noun_det(source, Det::A);
                player_event(
                    self,
                    *who,
                    &format!("activated an ability of {n}"),
                    &format!("activate an ability of {n}"),
                )
            }
            // "you've activated a loyalty ability" (CR 606.3).
            TriggerCond::Where { trigger, cond }
                if !amount
                    && matches!(
                        trigger.as_ref(),
                        TriggerCond::AbilityActivated {
                            source: Filter::Any,
                            include_mana: false,
                            ..
                        }
                    )
                    && matches!(cond, Condition::SelMatches(Sel::TriggerSpell, Filter::Custom(n)) if n == "loyalty ability") =>
            {
                let TriggerCond::AbilityActivated { who, .. } = trigger.as_ref() else {
                    return None;
                };
                player_event(
                    self,
                    *who,
                    "activated a loyalty ability",
                    "activate a loyalty ability",
                )
            }
            TriggerCond::IsDealtDamage {
                filter,
                combat_only,
            } => {
                let s = self.definite(filter)?;
                let combat = if *combat_only { "combat " } else { "" };
                let d = Self::amount_of(
                    if amount { q } else { Quant::Some },
                    &format!("{combat}damage"),
                )?;
                Said::Def(
                    Def::new(s, format!("was dealt {d}"), "wasn't", format!("dealt {d}"))
                        .counted_if(amount),
                )
            }
            TriggerCond::DealtExcessDamage {
                filter,
                noncombat_only: false,
            } if !amount => {
                if let Some(s) = self.definite(filter) {
                    return Some(Said::Def(Def::new(
                        s,
                        "was dealt excess damage",
                        "wasn't",
                        "dealt excess damage",
                    )));
                }
                // "a creature or planeswalker an opponent controlled".
                let (r, rest) = controller_of(filter);
                match r {
                    None => Said::Clause(self.quantified(
                        filter,
                        q,
                        "was dealt excess damage",
                        "were dealt excess damage",
                    )?),
                    // The controller as the damage was dealt.
                    Some(r) if q == Quant::Some => {
                        let a = self.noun_det(&rest, Det::A);
                        let who = self.rel_subject(r);
                        Said::Clause(format!("{a} {who} controlled was dealt excess damage"))
                    }
                    Some(_) => return None,
                }
            }
            TriggerCond::Noncombat(inner) => match inner.as_ref() {
                TriggerCond::DealsDamage {
                    source: Filter::Any,
                    to: DamageRecipient::Player(r),
                    ..
                } => self.player_dealt(*r, None, "noncombat ", q, amount, iter)?,
                _ => return None,
            },
            TriggerCond::DealsDamage {
                source,
                to,
                combat_only,
            } => {
                let combat = if *combat_only { "combat " } else { "" };
                if let Some(s) = self.definite(source) {
                    let d = Self::amount_of(
                        if amount { q } else { Quant::Some },
                        &format!("{combat}damage"),
                    )?;
                    let to = match to {
                        DamageRecipient::Any => String::new(),
                        DamageRecipient::Player(r) => {
                            let o = self.rel_object(*r);
                            format!(" to {o}")
                        }
                        DamageRecipient::Object(f) => {
                            let o = self.object_noun(f);
                            format!(" to {o}")
                        }
                        DamageRecipient::PlayerOrPlaneswalker(_) => return None,
                    };
                    return Some(Said::Def(
                        Def::new(
                            s,
                            format!("dealt {d}{to}"),
                            "didn't",
                            format!("deal {d}{to}"),
                        )
                        .counted_if(amount),
                    ));
                }
                match to {
                    DamageRecipient::Player(r) => {
                        let by = if matches!(source, Filter::Any) {
                            None
                        } else {
                            Some(self.noun_det(source, Det::A))
                        };
                        self.player_dealt(*r, by, combat, q, amount, iter)?
                    }
                    _ => return None,
                }
            }
            // "if it's the first time +1/+1 counters have been put on that permanent this
            // turn".
            TriggerCond::CountersPut {
                filter,
                kind,
                each: false,
            } if !amount && q == Quant::First => {
                let s = self.object_noun(filter);
                let s = if s == "it" || s == "~it" {
                    "{alt:it|that permanent|that creature}".to_string()
                } else {
                    s
                };
                let k = match kind {
                    Some(k) => format!("{}s", counter_name(k)),
                    None => "counters".into(),
                };
                Said::Full(format!(
                    "it's the first time {k} have been put on {s} this turn"
                ))
            }
            // "that player attacked you".
            TriggerCond::PlayerAttacked {
                attacker,
                defender: PlayerFilter::You,
                with,
                min: 1,
            } if !amount && matches!(with, Filter::Type(CardType::Creature)) => {
                let subj = self.past_player(*attacker, iter);
                Said::Def(Def::new(subj, "attacked you", "didn't", "attack you"))
            }
            // "you've played a land or cast a spell this turn from anywhere other than
            // your hand", "you didn't play a card from exile".
            TriggerCond::AnyOf(v) if !amount && v.len() == 2 => {
                if let Some(s) = self.played_from(&v[0], &v[1], q) {
                    return Some(s);
                }
                self.any_of(v, q, iter)?
            }
            TriggerCond::AnyOf(v) if !amount => self.any_of(v, q, iter)?,
            _ => return None,
        })
    }

    /// "a Hero", "three or more Merfolk", "two or more cards" as the object of a verb.
    fn quantified_object(&mut self, f: &Filter, q: Quant) -> Option<String> {
        Some(match q {
            Quant::Some => self.noun_det(f, Det::A),
            Quant::AtLeast(n) => {
                let many = self.noun(f, Num::Many);
                format!("{} or more {many}", number_word(n))
            }
            Quant::No => {
                let many = self.noun(f, Num::Many);
                format!("no {many}")
            }
            Quant::First => return None,
        })
    }

    /// "an opponent was dealt noncombat damage", "a player was dealt combat damage by a
    /// Zombie", "you haven't been dealt combat damage", "a player was dealt 6 or more
    /// combat damage".
    fn player_dealt(
        &mut self,
        r: PlayerRel,
        by: Option<String>,
        combat: &str,
        q: Quant,
        amount: bool,
        iter: Option<&str>,
    ) -> Option<Said> {
        let subj = self.past_player(r, iter);
        let d = Self::amount_of(
            if amount { q } else { Quant::Some },
            &format!("{combat}damage"),
        )?;
        let by = by.map(|b| format!(" by {b}")).unwrap_or_default();
        let have = if subj == "you" { "have" } else { "has" };
        let neg = Self::be_past(&subj, true);
        let pos = Self::be_past(&subj, false);
        Some(match q {
            Quant::Some | Quant::AtLeast(_) if subj == "you" => {
                Said::Clause(format!("{{alt:you were|you've been}} dealt {d}{by}"))
            }
            Quant::Some | Quant::AtLeast(_) => {
                Said::Clause(format!("{subj} {{alt:{pos}|{have} been}} dealt {d}{by}"))
            }
            Quant::No => Said::Clause(format!("{subj} {{alt:{neg}|{have}n't been}} dealt {d}{by}")),
            Quant::First => return None,
        })
    }

    /// Cards put into a zone, or leaving one: "a creature card was put into your graveyard
    /// from anywhere", "one or more cards left your graveyard", "a permanent was put into
    /// your hand from the battlefield", "one or more cards were put into exile", "an
    /// opponent had three or more cards put into their graveyard from anywhere".
    fn zone_change_past(
        &mut self,
        filter: &Filter,
        from: Option<ZoneKind>,
        to: Option<ZoneKind>,
        q: Quant,
        iter: Option<&str>,
    ) -> Option<String> {
        let (owner, rest) = owner_of(filter);
        let poss = match owner {
            Some(PlayerRel::Iterated) => Some("their".to_string()),
            Some(r) => Some(self.rel_possessive(r, Num::One)),
            None => None,
        };
        // A zone a card is put into or leaves: its owner's (CR 400.3).
        let zone = |z: ZoneKind, poss: &Option<String>| -> Option<String> {
            Some(match z {
                ZoneKind::Exile => "exile".to_string(),
                ZoneKind::Battlefield => "the battlefield".to_string(),
                ZoneKind::Graveyard | ZoneKind::Hand | ZoneKind::Library => {
                    let p = poss.as_deref().unwrap_or("a");
                    format!("{p} {}", zone_word(z))
                }
                _ => return None,
            })
        };
        match (from, to) {
            (from, Some(to)) => {
                let into = zone(to, &poss)?;
                let from = match from {
                    None => " from anywhere".to_string(),
                    Some(z) => format!(" from {}", zone(z, &poss)?),
                };
                // "an opponent had three or more cards put into their graveyard".
                if let (Some(PlayerRel::Iterated), Some(who)) = (owner, iter) {
                    let n = self.quantified_object(&rest, q)?;
                    return Some(format!("{who} had {n} put into {into}{from}"));
                }
                let s = self.quantified(&rest, q, "was put", "were put")?;
                // "were put into exile": no origin is said for a card exiled from
                // anywhere.
                let from = if to == ZoneKind::Exile && from == " from anywhere" {
                    String::new()
                } else {
                    from
                };
                Some(format!("{s} into {into}{from}"))
            }
            (Some(from), None) => {
                let z = zone(from, &poss)?;
                let s = self.quantified(&rest, q, "left", "left")?;
                Some(format!("{s} {z}"))
            }
            (None, None) => None,
        }
    }

    /// Cards of one kind put into one zone from every zone but one: (the cards, the zone,
    /// the zone left out).
    fn from_anywhere_but(&self, v: &[TriggerCond]) -> Option<(Filter, ZoneKind, ZoneKind)> {
        let mut filter: Option<&Filter> = None;
        let mut to = None;
        let mut froms = Vec::new();
        for c in v {
            let TriggerCond::ZoneChange {
                filter: f,
                from: Some(z),
                to: Some(t),
            } = c
            else {
                return None;
            };
            if filter.is_some_and(|g| format!("{g:?}") != format!("{f:?}"))
                || to.is_some_and(|x| x != *t)
            {
                return None;
            }
            filter = Some(f);
            to = Some(*t);
            froms.push(*z);
        }
        let to = to?;
        let all = [
            ZoneKind::Library,
            ZoneKind::Hand,
            ZoneKind::Battlefield,
            ZoneKind::Graveyard,
            ZoneKind::Stack,
            ZoneKind::Exile,
            ZoneKind::Command,
        ];
        let missing: Vec<ZoneKind> = all
            .iter()
            .filter(|z| **z != to && !froms.contains(z))
            .copied()
            .collect();
        match missing.as_slice() {
            [m] if froms.len() == all.len() - 2 => Some((filter?.clone(), to, *m)),
            _ => None,
        }
    }

    /// Playing a land or casting a spell from somewhere, together: "played a land or cast a
    /// spell from anywhere other than your hand", "play a card from exile" (playing a card
    /// is playing it as a land or casting it as a spell, CR 305.1, 601.1a).
    fn played_from(&mut self, a: &TriggerCond, b: &TriggerCond, q: Quant) -> Option<Said> {
        let (land, cast) = match (a, b) {
            (l @ TriggerCond::LandPlayed { .. }, c @ TriggerCond::CastSpell { .. })
            | (c @ TriggerCond::CastSpell { .. }, l @ TriggerCond::LandPlayed { .. }) => (l, c),
            _ => return None,
        };
        let (
            TriggerCond::LandPlayed {
                who: w1,
                filter: lf,
            },
            TriggerCond::CastSpell {
                who: w2,
                filter: cf,
            },
        ) = (land, cast)
        else {
            return None;
        };
        if w1 != w2 {
            return None;
        }
        let came = |f: &Filter| -> Option<(ZoneKind, bool)> {
            let atoms: Vec<&Filter> = match f {
                Filter::And(v) => v.iter().collect(),
                f => vec![f],
            };
            let mut out = None;
            for x in atoms {
                match x {
                    Filter::Type(CardType::Land) | Filter::Spell | Filter::Card => {}
                    Filter::Custom(n) => {
                        out = Some((zone_named(n.strip_prefix("came from:")?)?, true))
                    }
                    Filter::Not(n) => match n.as_ref() {
                        Filter::Custom(n) => {
                            out = Some((zone_named(n.strip_prefix("came from:")?)?, false))
                        }
                        Filter::CastFrom(z) => out = Some((*z, false)),
                        _ => return None,
                    },
                    Filter::CastFrom(z) => out = Some((*z, true)),
                    _ => return None,
                }
            }
            out
        };
        let (z1, in1) = came(lf)?;
        let (z2, in2) = came(cf)?;
        if z1 != z2 || in1 != in2 {
            return None;
        }
        let subj = self.rel_subject(*w1);
        let z = match z1 {
            ZoneKind::Exile => "exile".to_string(),
            z => format!("your {}", zone_word(z)),
        };
        let from = if in1 {
            format!("from {z}")
        } else {
            format!("from anywhere other than {z}")
        };
        Some(match (q, in1) {
            // "you've played a land or cast a spell this turn from anywhere other than
            // your hand".
            (Quant::Some, false) => Said::Full(format!(
                "{subj} {{alt:played a land or cast a spell this turn {from}|played a land or cast a spell {from} this turn}}"
            )),
            (Quant::Some, true) => Said::Clause(format!("{subj} played a card {from}")),
            (Quant::No, true) => Said::Clause(format!("{subj} didn't play a card {from}")),
            _ => return None,
        })
    }

    /// Events of several kinds about one subject: "~ attacked or blocked", "you've scried
    /// or surveilled", "~ didn't attack or come under your control".
    fn any_of(&mut self, v: &[TriggerCond], q: Quant, iter: Option<&str>) -> Option<Said> {
        let mut defs = Vec::new();
        for c in v {
            match self.said(c, q, false, iter)? {
                Said::Def(d) => defs.push(d),
                _ => return None,
            }
        }
        // The subject is said once: "~ attacked or blocked". (Saying it again would
        // call it "it", or a target "it" after "target creature".)
        let keys: Vec<String> = v.iter().map(subject_key).collect::<Option<_>>()?;
        if keys.iter().any(|k| *k != keys[0]) {
            return None;
        }
        let subj = defs.first()?.subj.clone();
        Some(match q {
            Quant::Some => {
                let parts: Vec<String> = defs.iter().map(|d| d.past.clone()).collect();
                Said::Clause(format!("{} {}", perfect(&subj), join_list(&parts, "or")))
            }
            Quant::No => {
                let aux = defs[0].neg_aux;
                let parts: Vec<String> = defs
                    .iter()
                    .map(|d| {
                        if d.neg_aux == aux {
                            d.neg_rest.clone()
                        } else {
                            format!("{} {}", d.neg_aux, d.neg_rest)
                        }
                    })
                    .collect();
                Said::Clause(format!("{subj} {aux} {}", join_list(&parts, "or")))
            }
            _ => return None,
        })
    }

    /// The clause for `q` events matching `c` in the window, if it can be said.
    pub(crate) fn happened(
        &mut self,
        w: Option<&HistoryWindow>,
        c: &TriggerCond,
        q: Quant,
        amount: bool,
        iter: Option<&str>,
    ) -> Option<String> {
        let s = match self.said(c, q, amount, iter)? {
            Said::Def(d) => d.say(q)?,
            Said::Clause(s) => s,
            Said::Full(s) => return w.is_none().then_some(s),
        };
        // "if it attacked during its controller's last turn": the controller of the
        // object the events are about.
        if let (Some(HistoryWindow::PlayersLastTurn(PlayerRef::ControllerOf(x))), Some(k)) =
            (w, subject_key(c))
        {
            if k == format!("{:?}", Filter::In(x.clone())) {
                return Some(format!("{s} during its controller's last turn"));
            }
        }
        let when = self.window_phrase(w);
        Some(format!("{s} {when}"))
    }

    /// A condition that something happened this turn, without "this turn", for "during
    /// any turn [it happened]" ("During any turn you attacked with a token, you may play
    /// that card").
    pub(crate) fn during_any_turn(&mut self, c: &Condition) -> Option<String> {
        let Condition::Compare(a, cmp, Value::Const(n)) = c else {
            return None;
        };
        let q = Quant::of(*cmp, *n, false)?;
        if q != Quant::Some {
            return None;
        }
        let s = match a {
            Value::EventsThisTurn(t, Tally::Events) => {
                self.happened_peek(None, t, q, false, None)?
            }
            Value::CountSel(s) => match s.as_ref() {
                Sel::ThisTurn(t) => self.event_objects_compare(None, t, q)?,
                _ => return None,
            },
            _ => return None,
        };
        s.strip_suffix(" this turn").map(str::to_string)
    }

    /// A comparison of a count of events with a number, as a clause: "~ attacked this
    /// turn", "you didn't play a land this turn", "an opponent lost life last turn".
    pub(crate) fn events_compare(
        &mut self,
        a: &Value,
        cmp: Cmp,
        b: &Value,
        negated: bool,
    ) -> Option<String> {
        let Value::Const(n) = b else {
            return None;
        };
        let q = Quant::of(cmp, *n, negated)?;
        match a {
            Value::EventsThisTurn(c, t @ (Tally::Events | Tally::Amount)) => {
                self.happened_peek(None, c, q, *t == Tally::Amount, None)
            }
            Value::EventsDuring(w, c, t @ (Tally::Events | Tally::Amount)) => {
                self.happened_peek(Some(w), c, q, *t == Tally::Amount, None)
            }
            // "if a player lost 4 or more life this turn", "if an opponent cast three or
            // more spells this turn": the most any one of them did.
            Value::OverPlayers(
                AggOp::Max,
                pf @ (PlayerFilter::Any | PlayerFilter::Opponent),
                v,
            ) if !negated && matches!(q, Quant::Some | Quant::AtLeast(_)) => {
                let who = match pf {
                    PlayerFilter::Opponent => "an opponent",
                    _ => "a player",
                };
                match v.as_ref() {
                    Value::EventsThisTurn(c, t @ (Tally::Events | Tally::Amount)) => {
                        self.happened_peek(None, c, q, *t == Tally::Amount, Some(who))
                    }
                    Value::EventsDuring(w, c, t @ (Tally::Events | Tally::Amount)) => {
                        self.happened_peek(Some(w), c, q, *t == Tally::Amount, Some(who))
                    }
                    Value::SpellsCastThisTurn(PlayerRef::Iterated, f) => {
                        let f = if matches!(f, Filter::Any) {
                            Filter::Spell
                        } else {
                            f.clone()
                        };
                        let n = self.quantified_object(&f, q)?;
                        Some(format!("{who} cast {n} this turn"))
                    }
                    Value::LifeLostThisTurn(PlayerRef::Iterated) => {
                        let a = Self::amount_of(q, "life")?;
                        Some(format!("{who} lost {a} this turn"))
                    }
                    _ => None,
                }
            }
            // "if a source you controlled dealt 5 or more damage this turn": the most damage
            // any one of them dealt.
            Value::Extreme(m, s, true) if !negated => {
                let (Value::EventsThisTurn(inner, Tally::Amount), Sel::ThisTurn(outer)) =
                    (m.as_ref(), s.as_ref())
                else {
                    return None;
                };
                let (
                    TriggerCond::DealsDamage {
                        source: Filter::In(v),
                        to: DamageRecipient::Any,
                        combat_only: c1,
                    },
                    TriggerCond::DealsDamage {
                        source,
                        to: DamageRecipient::Any,
                        combat_only: c2,
                    },
                ) = (inner.as_ref(), outer.as_ref())
                else {
                    return None;
                };
                if !matches!(v.as_ref(), Sel::Var(x) if *x == vars::TESTED) || c1 != c2 {
                    return None;
                }
                let (r, rest) = controller_of(source);
                let noun = if matches!(rest, Filter::Any) {
                    "a source".to_string()
                } else {
                    self.noun_det(&rest, Det::A)
                };
                let whose = match r {
                    Some(r) => {
                        let w = self.rel_subject(r);
                        format!(" {w} controlled")
                    }
                    None => String::new(),
                };
                let combat = if *c1 { "combat " } else { "" };
                let d = Self::amount_of(q, &format!("{combat}damage"))?;
                Some(format!("{noun}{whose} dealt {d} this turn"))
            }
            // "if you attacked with three or more Merfolk this turn", "if you've been
            // dealt damage by two or more creatures this turn": the objects the events
            // were about.
            Value::CountSel(s) => match s.as_ref() {
                Sel::ThisTurn(c) => self.event_objects_compare(None, c, q),
                Sel::EventsDuring(w, c) => self.event_objects_compare(Some(w), c, q),
                _ => None,
            },
            _ => None,
        }
    }

    /// Several events about one subject, all of which happened: "you gained and lost life
    /// this turn".
    pub(crate) fn events_joined(&mut self, v: &[Condition], conj: &str) -> Option<String> {
        let mut conds = Vec::new();
        let mut window: Option<Option<&HistoryWindow>> = None;
        for c in v {
            let (w, t) = match c {
                Condition::Compare(
                    Value::EventsThisTurn(t, Tally::Events),
                    cmp,
                    Value::Const(n),
                ) if Quant::of(*cmp, *n, false) == Some(Quant::Some) => (None, t.as_ref()),
                Condition::Compare(
                    Value::EventsDuring(w, t, Tally::Events),
                    cmp,
                    Value::Const(n),
                ) if Quant::of(*cmp, *n, false) == Some(Quant::Some) => (Some(w), t.as_ref()),
                _ => return None,
            };
            if window.is_some_and(|x| format!("{x:?}") != format!("{w:?}")) {
                return None;
            }
            window = Some(w);
            conds.push(t);
        }
        let window = window?;
        if conds.len() < 2 {
            return None;
        }
        // "if a Pirate and a Vehicle attacked this combat".
        if conj == "and" {
            let attackers: Option<Vec<&Filter>> = conds
                .iter()
                .map(|c| match c {
                    TriggerCond::Attacks(f)
                        if !matches!(f, Filter::Source | Filter::In(_))
                            && controller_of(f).0.is_none() =>
                    {
                        Some(f)
                    }
                    _ => None,
                })
                .collect();
            if let Some(fs) = attackers {
                let ns: Vec<String> = fs.iter().map(|f| self.noun_det(f, Det::A)).collect();
                let when = self.window_phrase(window);
                return Some(format!("{} attacked {when}", join_list(&ns, "and")));
            }
        }
        let keys: Vec<String> = conds
            .iter()
            .map(|c| subject_key(c))
            .collect::<Option<_>>()?;
        if keys.iter().any(|k| *k != keys[0]) {
            return None;
        }
        let saved_i = self.introduced.clone();
        let saved_g = self.gaps.len();
        let mut defs = Vec::new();
        for c in &conds {
            match self.said(c, Quant::Some, false, None) {
                Some(Said::Def(d)) => defs.push(d),
                _ => {
                    self.introduced = saved_i;
                    self.gaps.truncate(saved_g);
                    return None;
                }
            }
        }
        let subj = perfect(&defs[0].subj);
        // "gained and lost life": the verbs joined when what follows them is the same.
        let split: Vec<(String, String)> = defs
            .iter()
            .map(|d| match d.past.split_once(' ') {
                Some((a, b)) => (a.to_string(), b.to_string()),
                None => (d.past.clone(), String::new()),
            })
            .collect();
        let s = if split.iter().all(|(_, r)| *r == split[0].1) && !split[0].1.is_empty() {
            let verbs: Vec<String> = split.iter().map(|(v, _)| v.clone()).collect();
            format!("{subj} {} {}", join_list(&verbs, conj), split[0].1)
        } else {
            let parts: Vec<String> = defs.iter().map(|d| d.past.clone()).collect();
            format!("{subj} {}", join_list(&parts, conj))
        };
        let when = self.window_phrase(window);
        Some(format!("{s} {when}"))
    }

    /// [`Self::happened`] that leaves no trace (target mentions, gaps) when it can't say
    /// the clause.
    fn happened_peek(
        &mut self,
        w: Option<&HistoryWindow>,
        c: &TriggerCond,
        q: Quant,
        amount: bool,
        iter: Option<&str>,
    ) -> Option<String> {
        let saved_i = self.introduced.clone();
        let saved_g = self.gaps.len();
        let r = self.happened(w, c, q, amount, iter);
        if r.is_none() {
            self.introduced = saved_i;
            self.gaps.truncate(saved_g);
        }
        r
    }

    /// A count of the objects of the events: "you attacked with a Hero this turn", "you've
    /// been dealt damage by two or more creatures this turn".
    fn event_objects_compare(
        &mut self,
        w: Option<&HistoryWindow>,
        c: &TriggerCond,
        q: Quant,
    ) -> Option<String> {
        let when = self.window_phrase(w);
        match c {
            TriggerCond::Attacks(f) => {
                let (r, rest) = controller_of(f);
                let who = self.rel_subject(r?);
                let n = self.quantified_object(&rest, q)?;
                match q {
                    Quant::No => {
                        let many = self.noun(&rest, Num::Many);
                        Some(format!("{who} didn't attack with any {many} {when}"))
                    }
                    _ => Some(format!("{who} attacked with {n} {when}")),
                }
            }
            TriggerCond::DealsDamage {
                source,
                to: DamageRecipient::Player(r),
                combat_only,
            } if !matches!(source, Filter::Any) && q != Quant::No => {
                let n = self.quantified_object(source, q)?;
                let combat = if *combat_only { "combat " } else { "" };
                let subj = self.rel_subject(*r);
                if subj == "you" {
                    return Some(format!(
                        "{{alt:you were|you've been}} dealt {combat}damage by {n} {when}"
                    ));
                }
                let pos = Self::be_past(&subj, false);
                Some(format!(
                    "{subj} {{alt:{pos}|has been}} dealt {combat}damage by {n} {when}"
                ))
            }
            _ => None,
        }
    }
}

/// The zone a `came from:` name ends with.
fn zone_named(s: &str) -> Option<ZoneKind> {
    Some(match s {
        "Hand" => ZoneKind::Hand,
        "Graveyard" => ZoneKind::Graveyard,
        "Exile" => ZoneKind::Exile,
        "Library" => ZoneKind::Library,
        "Command" => ZoneKind::Command,
        _ => return None,
    })
}

/// What the subject of a past event is, for telling whether several events have the same
/// subject.
fn subject_key(c: &TriggerCond) -> Option<String> {
    Some(match c {
        TriggerCond::Attacks(f)
        | TriggerCond::Blocks(f)
        | TriggerCond::BecomesBlocked(f)
        | TriggerCond::EntersBattlefield(f)
        | TriggerCond::LeavesBattlefield(f)
        | TriggerCond::IsDealtDamage { filter: f, .. }
        | TriggerCond::DealtExcessDamage { filter: f, .. }
        | TriggerCond::DealsDamage { source: f, .. } => format!("{f:?}"),
        TriggerCond::AnyOf(v) => match v.as_slice() {
            [TriggerCond::EntersBattlefield(f), TriggerCond::ControlChanged(_)] => {
                format!("{:?}", controller_of(f).1)
            }
            _ => return None,
        },
        TriggerCond::GainsLife { who }
        | TriggerCond::LosesLife { who }
        | TriggerCond::LandPlayed { who, .. }
        | TriggerCond::Searched(who)
        | TriggerCond::PlayerAction { who, .. }
        | TriggerCond::AbilityActivated { who, .. }
        | TriggerCond::CastSpell { who, .. } => format!("player {who:?}"),
        _ => return None,
    })
}
