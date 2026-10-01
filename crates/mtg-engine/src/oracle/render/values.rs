//! Numbers ([`Value`]), conditions ([`Condition`]) and durations ([`Duration`]).

use super::nouns::{cmp_phrase, Det};
use super::players::Case;
use super::*;

impl Renderer<'_> {
    /// Whether a value is printed as a plain number or X (rather than "X, where X is ...").
    pub(crate) fn is_simple(v: &Value) -> bool {
        matches!(v, Value::Const(_) | Value::X)
    }

    /// A number as a noun phrase ("3", "X", "the number of creatures you control").
    pub(crate) fn value(&mut self, v: &Value) -> String {
        match v {
            Value::Const(n) => n.to_string(),
            Value::X => "X".into(),
            Value::Count(f) => {
                let n = self.noun_det(f, Det::Plural);
                format!("the number of {n}")
            }
            Value::CountSel(s) => {
                let s = self.sel(s, Case::Obj);
                format!("the number of {s}")
            }
            Value::CountPlayers(pf) => {
                let n = self.player_filter_noun(pf, Num::Many);
                format!("the number of {n}")
            }
            Value::PowerOf(s) => {
                let s = self.sel(s, Case::Poss);
                format!("{s} power")
            }
            Value::ToughnessOf(s) => {
                let s = self.sel(s, Case::Poss);
                format!("{s} toughness")
            }
            Value::ManaValueOf(s) => {
                let s = self.sel(s, Case::Poss);
                format!("{s} mana value")
            }
            Value::LoyaltyOf(s) => {
                let s = self.sel(s, Case::Poss);
                format!("{s} loyalty")
            }
            Value::CountersOn(s, k) => {
                let s = self.sel(s, Case::Obj);
                match k {
                    Some(k) => format!("the number of {k} counters on {s}"),
                    None => format!("the number of counters on {s}"),
                }
            }
            Value::PlayerCounters(p, k) => {
                let p = self.player(p, Case::Subj);
                let has = if p == "you" { "have" } else { "has" };
                format!("the number of {k} counters {p} {has}")
            }
            Value::LifeTotal(p) => {
                let p = self.player(p, Case::Poss);
                format!("{p} life total")
            }
            Value::StartingLife => "your starting life total".into(),
            Value::HandSize(p) => {
                let p = self.player(p, Case::Poss);
                format!("the number of cards in {p} hand")
            }
            Value::LibrarySize(p) => {
                let p = self.player(p, Case::Poss);
                format!("the number of cards in {p} library")
            }
            Value::GraveyardSize(p) => {
                let p = self.player(p, Case::Poss);
                format!("the number of cards in {p} graveyard")
            }
            Value::CardsInGraveyard(p, f) => {
                let p = self.player(p, Case::Poss);
                let n = self.noun(f, Num::Many);
                let n = if n.ends_with("card") || n.ends_with("cards") {
                    n
                } else {
                    format!("{n} cards")
                };
                format!("the number of {n} in {p} graveyard")
            }
            Value::EventAmount => "that much".into(),
            Value::Prev => "that many".into(),
            Value::Var(vars::EXCESS) => "the excess damage".into(),
            Value::Var(_) => "that many".into(),
            Value::Devotion(cs) => {
                let w: Vec<String> = cs.iter().map(|c| c.word().to_string()).collect();
                format!("your devotion to {}", join_list(&w, "and"))
            }
            Value::Domain => "the number of basic land types among lands you control".into(),
            Value::StormCount => "the number of spells cast before it this turn".into(),
            Value::CardsDrawnThisTurn(p) => {
                let p = self.player(p, Case::Subj);
                format!("the number of cards {p} drew this turn")
            }
            Value::LifeGainedThisTurn(p) => {
                let p = self.player(p, Case::Subj);
                format!("the amount of life {p} gained this turn")
            }
            Value::LifeLostThisTurn(p) => {
                let p = self.player(p, Case::Subj);
                format!("the amount of life {p} lost this turn")
            }
            Value::CreaturesDiedThisTurn => "the number of creatures that died this turn".into(),
            Value::PermanentsEnteredThisTurn(p, f) => {
                let n = self.noun(f, Num::Many);
                let p = self.player(p, Case::Poss);
                format!(
                    "the number of {n} that entered the battlefield under {p} control this turn"
                )
            }
            Value::SpellsCastThisTurn(p, f) => {
                let n = self.noun(f, Num::Many);
                let n = if n.contains("spell") {
                    n
                } else {
                    format!("{n} spells")
                };
                let p = self.player(p, Case::Subj);
                let have = if p == "you" {
                    "you've"
                } else {
                    "that player has"
                };
                format!("the number of {n} {have} cast this turn")
            }
            Value::TimesResolvedThisTurn => {
                "the number of times this ability has resolved this turn".into()
            }
            Value::CardTypesAmong(f) => {
                let n = self.noun(f, Num::Many);
                format!("the number of card types among {n}")
            }
            Value::ColorPairsAmong(f) => {
                let n = self.noun(f, Num::Many);
                format!("the number of color pairs among {n}")
            }
            Value::ColorsAmong(f) => {
                let n = self.noun(f, Num::Many);
                format!("the number of colors among {n}")
            }
            Value::ClassLevel => "~'s level".into(),
            Value::XOf(s) => {
                let s = self.sel(s, Case::Poss);
                format!("the value of X for {s}")
            }
            Value::GreatestPower(f) => {
                let n = self.noun(f, Num::Many);
                format!("the greatest power among {n}")
            }
            Value::GreatestManaValue(f) => {
                let n = self.noun(f, Num::Many);
                format!("the greatest mana value among {n}")
            }
            Value::BasePowerOf(s) => {
                let s = self.sel(s, Case::Poss);
                format!("{s} base power")
            }
            Value::DistinctNames(f) => {
                let n = self.noun(f, Num::Many);
                format!("the number of differently named {n}")
            }
            Value::ColorsSpent => "the number of colors of mana spent to cast ~".into(),
            Value::ManaSpent => "the amount of mana spent to cast ~".into(),
            Value::Chosen => "the chosen number".into(),
            Value::TimesKicked => "the number of times ~ was kicked".into(),
            Value::Speed(p) => {
                let p = self.player(p, Case::Poss);
                format!("{p} speed")
            }
            Value::Sum(v) => {
                let parts: Vec<String> = v.iter().map(|x| self.value(x)).collect();
                parts.join(" plus ")
            }
            Value::Diff(a, b) => {
                let a = self.value(a);
                let b = self.value(b);
                format!("{a} minus {b}")
            }
            Value::Mul(a, b) => match (a.as_ref(), b.as_ref()) {
                (Value::Const(2), x) | (x, Value::Const(2)) => {
                    let x = self.value(x);
                    format!("twice {x}")
                }
                (Value::Const(3), x) | (x, Value::Const(3)) => {
                    let x = self.value(x);
                    format!("three times {x}")
                }
                (a, b) => {
                    let a = self.value(a);
                    let b = self.value(b);
                    format!("{a} times {b}")
                }
            },
            Value::Div(v, 2, up) => {
                let v = self.value(v);
                let r = if *up { "up" } else { "down" };
                format!("half {v}, rounded {r}")
            }
            Value::Div(v, n, up) => {
                let v = self.value(v);
                let r = if *up { "up" } else { "down" };
                format!("{v} divided by {n}, rounded {r}")
            }
            Value::Min(a, b) => {
                let a = self.value(a);
                let b = self.value(b);
                format!("the lesser of {a} and {b}")
            }
            Value::Max(a, b) => {
                let a = self.value(a);
                let b = self.value(b);
                format!("the greater of {a} and {b}")
            }
            Value::If(c, a, b) => {
                let c = self.condition(c);
                let a = self.value(a);
                let b = self.value(b);
                format!("{a} if {c}, otherwise {b}")
            }
            Value::Custom(name) => self.custom_value(name),
        }
    }

    /// An amount before a noun: "3", "X", or "X" with a "where X is" clause to add.
    /// Returns (amount, where-clause).
    pub(crate) fn amount(&mut self, v: &Value) -> (String, Option<String>) {
        if Self::is_simple(v) {
            return (self.value(v), None);
        }
        let s = self.value(v);
        ("X".into(), Some(format!(", where X is {s}")))
    }

    /// "a card" / "two cards" / "X cards, where X is ..." (the where clause is returned
    /// separately).
    pub(crate) fn counted(&mut self, v: &Value, noun: &str) -> (String, Option<String>) {
        match v {
            Value::Const(1) => (with_article(noun), None),
            Value::Const(n) => (format!("{} {}", number_word(*n), plural(noun)), None),
            Value::X => (format!("X {}", plural(noun)), None),
            other => {
                let s = self.value(other);
                (
                    format!("X {}", plural(noun)),
                    Some(format!(", where X is {s}")),
                )
            }
        }
    }

    /// A condition as a clause ("you control an artifact").
    pub(crate) fn condition(&mut self, c: &Condition) -> String {
        match c {
            Condition::Always => self.gap("Condition::Always"),
            Condition::Never => self.gap("Condition::Never"),
            Condition::Not(inner) => self.negated_condition(inner),
            Condition::And(v) => {
                let parts: Vec<String> = v.iter().map(|x| self.condition(x)).collect();
                join_list(&parts, "and")
            }
            Condition::Or(v) => {
                let parts: Vec<String> = v.iter().map(|x| self.condition(x)).collect();
                join_list(&parts, "or")
            }
            Condition::Compare(a, cmp, b) => self.compare_condition(a, *cmp, b),
            Condition::Exists(f) => self.exists(f, false),
            Condition::SelNonEmpty(s) => {
                let s = self.sel(s, Case::Subj);
                format!("{s} exists")
            }
            Condition::SelMatches(s, f) => {
                let subj = self.sel(s, Case::Subj);
                let pred = self.is_predicate(f, false);
                format!("{subj} {pred}")
            }
            Condition::PlayerMatches(p, pf) => {
                let subj = self.player(p, Case::Subj);
                let pred = self.player_predicate(pf, &subj, false);
                format!("{subj} {pred}")
            }
            Condition::YourTurn => "it's your turn".into(),
            Condition::NotYourTurn => "it's not your turn".into(),
            Condition::CostPaid(name) => self.cost_paid(name),
            Condition::WasCast => "you cast it".into(),
            Condition::PrevHappened => "you do".into(),
            Condition::PrevAffectedAny => "a card was affected this way".into(),
            Condition::CastFrom(z) => format!("you cast it from your {}", zone_word(*z)),
            Condition::AllTriggerConditionsThisTurn(_) => {
                self.gap("Condition::AllTriggerConditionsThisTurn")
            }
            Condition::ChosenWord(w) => format!("{w} was chosen"),
            Condition::Phase(p) => match p {
                PhaseCond::Combat => "it's combat".into(),
                PhaseCond::MainPhase => "it's your main phase".into(),
                PhaseCond::Upkeep => "it's your upkeep".into(),
                PhaseCond::DeclareAttackers => "it's the declare attackers step".into(),
                PhaseCond::EndStep => "it's the end step".into(),
            },
            Condition::CitysBlessing => "you have the city's blessing".into(),
            Condition::IsMonarch => "you're the monarch".into(),
            Condition::HasInitiative => "you have the initiative".into(),
            Condition::IsDay => "it's day".into(),
            Condition::IsNight => "it's night".into(),
            Condition::MaxSpeed => "you have max speed".into(),
            Condition::CombatTiming(ct) => self.combat_timing(ct),
            Condition::Chose(w) => format!("{w} was chosen"),
            Condition::Custom(name) => self.custom_condition(name),
        }
    }

    fn cost_paid(&self, name: &str) -> String {
        match name {
            "kicked" | "kicker" => "~ was kicked".into(),
            "bargained" | "bargain" => "~ was bargained".into(),
            "gift" => "the gift was promised".into(),
            other => format!("the {other} cost was paid"),
        }
    }

    fn negated_condition(&mut self, c: &Condition) -> String {
        match c {
            Condition::Exists(f) => self.exists(f, true),
            Condition::YourTurn => "it's not your turn".into(),
            Condition::NotYourTurn => "it's your turn".into(),
            Condition::PrevHappened => "you don't".into(),
            Condition::SelMatches(s, f) => {
                let subj = self.sel(s, Case::Subj);
                let pred = self.is_predicate(f, true);
                format!("{subj} {pred}")
            }
            Condition::PlayerMatches(p, pf) => {
                let subj = self.player(p, Case::Subj);
                let pred = self.player_predicate(pf, &subj, true);
                format!("{subj} {pred}")
            }
            Condition::CostPaid(name) => {
                let s = self.cost_paid(name);
                s.replace(" was ", " wasn't ")
            }
            Condition::WasCast => "you didn't cast it".into(),
            other => {
                let s = self.condition(other);
                format!("it's not true that {s}")
            }
        }
    }

    /// "you control an artifact", "you control no Islands", "there are ... in your graveyard".
    pub(crate) fn exists(&mut self, f: &Filter, negated: bool) -> String {
        let (ctrl, rest) = split_controller(f);
        let zone = f.zone();
        match (ctrl, zone) {
            (Some(r), None) | (Some(r), Some(ZoneKind::Battlefield)) => {
                let subj = self.rel_subject(r);
                let verb = if subj == "you" { "control" } else { "controls" };
                if negated {
                    let n = self.noun(&rest, Num::Many);
                    format!("{subj} {verb} no {n}")
                } else {
                    let n = self.noun_det(&rest, Det::A);
                    format!("{subj} {verb} {n}")
                }
            }
            _ => {
                if negated {
                    let n = self.noun(f, Num::Many);
                    format!("there are no {n}")
                } else {
                    let n = self.noun_det(f, Det::A);
                    format!("there is {n}")
                }
            }
        }
    }

    /// "is tapped", "is a creature", "has flying".
    pub(crate) fn is_predicate(&mut self, f: &Filter, negated: bool) -> String {
        let atoms: Vec<&Filter> = match f {
            Filter::And(v) => v.iter().collect(),
            other => vec![other],
        };
        let mut parts = Vec::new();
        for a in atoms {
            let p = match a {
                Filter::Tapped => "tapped".to_string(),
                Filter::Untapped => "untapped".into(),
                Filter::Attacking => "attacking".into(),
                Filter::Blocking => "blocking".into(),
                Filter::Blocked => "blocked".into(),
                Filter::Unblocked => "unblocked".into(),
                Filter::FaceDown => "face down".into(),
                Filter::Modified => "modified".into(),
                Filter::Enchanted => "enchanted".into(),
                Filter::Equipped => "equipped".into(),
                Filter::Attached => "attached to a permanent".into(),
                Filter::Token => "a token".into(),
                Filter::Prepared => "prepared".into(),
                Filter::InZone(z) => match z {
                    ZoneKind::Battlefield => "on the battlefield".into(),
                    ZoneKind::Exile => "in exile".into(),
                    z => format!("in a {}", zone_word(*z)),
                },
                Filter::ControlledBy(PlayerRel::You) => "under your control".into(),
                Filter::HasKeyword(k) => {
                    let w = self.keyword_kind_word(*k);
                    if negated {
                        return format!("doesn't have {w}");
                    }
                    return format!("has {w}");
                }
                Filter::HasCounter(k) => {
                    let c = match k {
                        Some(k) => counter_name(k),
                        None => "counter".into(),
                    };
                    if negated {
                        return format!("has no {} on it", plural(&c));
                    }
                    return format!("has {} on it", with_article(&c));
                }
                Filter::Power(c, v) => {
                    let v = self.value(v);
                    if negated {
                        return format!("doesn't have power {}", cmp_phrase(*c, &v));
                    }
                    return format!("has power {}", cmp_phrase(*c, &v));
                }
                Filter::Not(inner) => {
                    let p = self.is_predicate(inner, !negated);
                    return p;
                }
                other => {
                    let n = self.noun_det(other, Det::A);
                    n
                }
            };
            parts.push(p);
        }
        let joined = join_list(&parts, "and");
        if negated {
            format!("isn't {joined}")
        } else {
            format!("is {joined}")
        }
    }

    fn player_predicate(&mut self, pf: &PlayerFilter, subj: &str, negated: bool) -> String {
        let you = subj == "you";
        let (have, has_no) = if you {
            ("have", "have")
        } else {
            ("has", "has")
        };
        match pf {
            PlayerFilter::Life(c, v) => {
                let v = self.value(v);
                let life = match c {
                    Cmp::Le => format!("{v} or less life"),
                    Cmp::Ge => format!("{v} or more life"),
                    Cmp::Lt => format!("less than {v} life"),
                    Cmp::Gt => format!("more than {v} life"),
                    Cmp::Eq => format!("exactly {v} life"),
                    Cmp::Ne => format!("other than {v} life"),
                };
                if negated {
                    format!("doesn't have {life}")
                } else {
                    format!("{have} {life}")
                }
            }
            PlayerFilter::HandSize(c, v) => {
                let v = self.value(v);
                let n = match (c, v.as_str()) {
                    (Cmp::Eq, "0") => "no cards".to_string(),
                    (Cmp::Le, x) => format!("{x} or fewer cards"),
                    (Cmp::Ge, x) => format!("{x} or more cards"),
                    (Cmp::Lt, x) => format!("fewer than {x} cards"),
                    (Cmp::Gt, x) => format!("more than {x} cards"),
                    (_, x) => format!("exactly {x} cards"),
                };
                format!("{has_no} {n} in hand")
            }
            PlayerFilter::GraveyardSize(c, v) => {
                let v = self.value(v);
                let n = match c {
                    Cmp::Ge => format!("{v} or more cards"),
                    Cmp::Le => format!("{v} or fewer cards"),
                    _ => format!("{} cards", cmp_phrase(*c, &v)),
                };
                let poss = if you { "your" } else { "their" };
                format!("{have} {n} in {poss} graveyard")
            }
            PlayerFilter::Controls(f, c, v) => {
                let n = self.count_phrase(f, *c, v);
                let verb = if you { "control" } else { "controls" };
                format!("{verb} {n}")
            }
            PlayerFilter::Monarch => "is the monarch".into(),
            PlayerFilter::Poisoned => "is poisoned".into(),
            PlayerFilter::MaxSpeed => format!("{have} max speed"),
            PlayerFilter::DealtDamageThisTurn => "was dealt damage this turn".into(),
            PlayerFilter::Counters(k, c, v) => {
                let v = self.value(v);
                let n = match c {
                    Cmp::Ge => format!("{v} or more"),
                    _ => cmp_phrase(*c, &v),
                };
                format!("{have} {n} {k} counters")
            }
            other => {
                let n = self.player_filter_noun(other, Num::One);
                let n = with_article(&n);
                if negated {
                    format!("isn't {n}")
                } else {
                    format!("is {n}")
                }
            }
        }
    }

    fn compare_condition(&mut self, a: &Value, cmp: Cmp, b: &Value) -> String {
        // "you control three or more creatures".
        if let Value::Count(f) = a {
            if let Some((r, rest)) = split_controller(f).0.map(|r| (r, split_controller(f).1)) {
                if f.zone().is_none_or(|z| z == ZoneKind::Battlefield) {
                    let subj = self.rel_subject(r);
                    let verb = if subj == "you" { "control" } else { "controls" };
                    let n = self.count_phrase(&rest, cmp, b);
                    return format!("{subj} {verb} {n}");
                }
            }
            let n = self.count_phrase(f, cmp, b);
            return format!("there are {n}");
        }
        if let Value::LifeTotal(p) = a {
            let subj = self.player(p, Case::Subj);
            let pred =
                self.player_predicate(&PlayerFilter::Life(cmp, Box::new(b.clone())), &subj, false);
            return format!("{subj} {pred}");
        }
        if let Value::HandSize(p) = a {
            let subj = self.player(p, Case::Subj);
            let pred = self.player_predicate(
                &PlayerFilter::HandSize(cmp, Box::new(b.clone())),
                &subj,
                false,
            );
            return format!("{subj} {pred}");
        }
        if let Value::GraveyardSize(p) = a {
            let subj = self.player(p, Case::Subj);
            let pred = self.player_predicate(
                &PlayerFilter::GraveyardSize(cmp, Box::new(b.clone())),
                &subj,
                false,
            );
            return format!("{subj} {pred}");
        }
        let a = self.value(a);
        let b = self.value(b);
        let rel = match cmp {
            Cmp::Eq => format!("is {b}"),
            Cmp::Ne => format!("isn't {b}"),
            Cmp::Lt => format!("is less than {b}"),
            Cmp::Le => format!("is {b} or less"),
            Cmp::Gt => format!("is greater than {b}"),
            Cmp::Ge => format!("is {b} or greater"),
        };
        format!("{a} {rel}")
    }

    pub(crate) fn combat_timing(&mut self, ct: &CombatTiming) -> String {
        let point = match ct.point {
            CombatPoint::Combat => "combat",
            CombatPoint::AttackersDeclared => "attackers are declared",
            CombatPoint::BlockersDeclared => "blockers are declared",
            CombatPoint::CombatDamageStep => "the combat damage step",
            CombatPoint::EndOfCombatStep => "the end of combat step",
        };
        let ba = if ct.after { "after" } else { "before" };
        if ct.during_combat {
            format!("during combat {ba} {point}")
        } else {
            format!("{ba} {point}")
        }
    }

    /// A duration as a phrase appended to the effect ("until end of turn"); empty for
    /// indefinite effects.
    pub(crate) fn duration(&mut self, d: &Duration) -> String {
        match d {
            Duration::EndOfTurn => "until end of turn".into(),
            Duration::EndOfCombat => "until end of combat".into(),
            Duration::UntilYourNextTurn => "until your next turn".into(),
            Duration::UntilEndOfYourNextTurn => "until the end of your next turn".into(),
            Duration::WhileSourceOnBattlefield => {
                "for as long as ~ remains on the battlefield".into()
            }
            Duration::WhileYouControlSource => "for as long as you control ~".into(),
            Duration::WhileCondition(c) => {
                let c = self.condition(c);
                format!("for as long as {c}")
            }
            Duration::Permanent => String::new(),
            Duration::UntilHostLeaves => String::new(),
            Duration::ThisTurn => "this turn".into(),
            Duration::ThroughNextUntapStep => "during its controller's next untap step".into(),
            Duration::UntilYourNextStep(s) => {
                let s = self.step_name(*s);
                format!("until your next {s}")
            }
            Duration::UntilPlaneswalk { away_from_plane } => {
                if *away_from_plane {
                    "until a player planeswalks away from a plane".into()
                } else {
                    "until a player planeswalks".into()
                }
            }
        }
    }

    pub(crate) fn step_name(&self, s: TriggerStep) -> &'static str {
        match s {
            TriggerStep::Untap => "untap step",
            TriggerStep::Upkeep => "upkeep",
            TriggerStep::Draw => "draw step",
            TriggerStep::PrecombatMain => "precombat main phase",
            TriggerStep::BeginningOfCombat => "combat",
            TriggerStep::DeclareAttackers => "declare attackers step",
            TriggerStep::DeclareBlockers => "declare blockers step",
            TriggerStep::CombatDamage => "combat damage step",
            TriggerStep::EndOfCombat => "end of combat step",
            TriggerStep::PostcombatMain => "postcombat main phase",
            TriggerStep::End => "end step",
            TriggerStep::Cleanup => "cleanup step",
            TriggerStep::Turn => "turn",
        }
    }
}

/// Splits "… you control" off a filter: (controller, the rest).
pub(crate) fn split_controller(f: &Filter) -> (Option<PlayerRel>, Filter) {
    match f {
        Filter::ControlledBy(r) => (Some(*r), Filter::Any),
        Filter::And(v) => {
            let mut ctrl = None;
            let mut rest = Vec::new();
            for x in v {
                match x {
                    Filter::ControlledBy(r) if ctrl.is_none() => ctrl = Some(*r),
                    other => rest.push(other.clone()),
                }
            }
            (ctrl, Filter::and(rest))
        }
        other => (None, other.clone()),
    }
}
