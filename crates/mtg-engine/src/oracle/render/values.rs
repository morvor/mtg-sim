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
                let saved = self.alt_and;
                self.alt_and = true;
                let n = self.noun_det(f, Det::Plural);
                self.alt_and = saved;
                if Self::counts_all_permanents(f, &n) {
                    format!("the number of {n} {{opt:on the battlefield}}")
                } else {
                    format!("the number of {n}")
                }
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
            Value::Var(_) if self.stored_x(v).is_some() => "X".into(),
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
                let n = if matches!(f, Filter::Any | Filter::Spell) {
                    "spells".to_string()
                } else {
                    self.noun(f, Num::Many)
                };
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
            Value::ClassLevel => format!("{}'s level", self.me()),
            Value::XOf(s) => {
                let s = self.sel(s, Case::Poss);
                format!("the value of X for {s}")
            }
            Value::Aggregate(op, stat, s) => {
                let s = match s.as_ref() {
                    Sel::All(f) => self.noun(f, Num::Many),
                    other => self.sel(other, Case::Obj),
                };
                let st = match stat {
                    Stat::Power => "power".to_string(),
                    Stat::Toughness => "toughness".to_string(),
                    Stat::PowerOrToughness => "power and/or toughness".to_string(),
                    Stat::ManaValue => "mana value".to_string(),
                    Stat::Counters(Some(k)) if *op == AggOp::Sum => {
                        return format!("the number of {k} counters among {s}");
                    }
                    Stat::Counters(None) if *op == AggOp::Sum => {
                        return format!("the number of counters among {s}");
                    }
                    other => return self.gap(format!("Value::Aggregate {other:?}")),
                };
                let o = match op {
                    AggOp::Max => "greatest",
                    AggOp::Min => "least",
                    AggOp::Sum => "total",
                };
                format!("the {o} {st} among {s}")
            }
            Value::DistinctAmong(among, s) => {
                let s = match s.as_ref() {
                    Sel::All(f) => self.noun(f, Num::Many),
                    other => self.sel(other, Case::Obj),
                };
                let what = match among {
                    Among::CardTypes => "card types",
                    Among::PermanentTypes => "permanent types",
                    Among::CreatureTypes => "creature types",
                    Among::BasicLandTypes => "basic land types",
                    Among::Colors => "colors",
                    Among::ManaValues => "different mana values",
                    Among::ManaCosts => "different mana costs",
                    Among::Powers => "different powers",
                    Among::Names => "different names",
                    Among::CounterKinds => "kinds of counters",
                    Among::LargestCreatureTypeGroup => {
                        return self.gap("Among::LargestCreatureTypeGroup");
                    }
                };
                format!("the number of {what} among {s}")
            }
            Value::OverPlayers(..) => self.gap("Value::OverPlayers"),
            Value::Extreme(..) => self.gap("Value::Extreme"),
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
            Value::SpellsCastThisTurnManaValue(p, f) => {
                let n = if matches!(f, Filter::Any | Filter::Spell) {
                    "spells".to_string()
                } else {
                    let n = self.noun(f, Num::Many);
                    if n.contains("spell") {
                        n
                    } else {
                        format!("{n} spells")
                    }
                };
                let p = self.player(p, Case::Subj);
                let have = if p == "you" {
                    "you've"
                } else {
                    "that player has"
                };
                format!("the total mana value of {n} {have} cast this turn")
            }
            Value::ManaValuesAmong(f) => {
                let n = self.noun_det(f, Det::Plural);
                format!("the number of different mana values among {n}")
            }
            Value::DistinctNames(f) => {
                let n = self.noun(f, Num::Many);
                format!("the number of differently named {n}")
            }
            Value::ColorsSpent => {
                format!("the number of colors of mana spent to cast {}", self.me())
            }
            Value::ManaSpent => format!("the amount of mana spent to cast {}", self.me()),
            Value::Chosen => "the chosen number".into(),
            Value::TimesKicked => format!("the number of times {} was kicked", self.me()),
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
                (Value::Const(1), x) | (x, Value::Const(1)) => self.value(x),
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
            // "X can't be negative": a count clamped at zero (CR 107.1b) reads as the count.
            Value::Max(a, b) if matches!(b.as_ref(), Value::Const(0)) => self.value(a),
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
    /// Whether a count of `f` (rendered `n`) is of all permanents of a kind, with no
    /// controller: cards say "for each Goblin" or "for each Goblin on the battlefield".
    pub(crate) fn counts_all_permanents(f: &Filter, n: &str) -> bool {
        split_controller(f).0.is_none()
            && f.zone().is_none_or(|z| z == ZoneKind::Battlefield)
            && !n.contains(" you ")
            && !n.contains("among")
            && !n.contains("attacking")
            && !n.contains("blocking")
            && !n.contains("{opt:")
            && !n.ends_with(" ~")
    }

    pub(crate) fn amount(&mut self, v: &Value) -> (String, Option<String>) {
        if matches!(v, Value::EventAmount) {
            return ("that much".into(), None);
        }
        if matches!(v, Value::Prev) {
            return ("that many".into(), None);
        }
        if let Some(x) = self.stored_x(v) {
            return (x, None);
        }
        // A number stored earlier: "that many", or the X the text already defined.
        if matches!(v, Value::Var(n) if *n != vars::EXCESS) {
            return ("{alt:that many|X}".into(), None);
        }
        if Self::is_simple(v) {
            return (self.value(v), None);
        }
        if let Some(s) = x_arithmetic(v) {
            return (s, None);
        }
        let s = self.value(v);
        ("X".into(), Some(format!(", where X is {s}")))
    }

    /// "a card" / "two cards" / "X cards, where X is ..." (the where clause is returned
    /// separately).
    pub(crate) fn counted(&mut self, v: &Value, noun: &str) -> (String, Option<String>) {
        if let Some(x) = self.stored_x(v) {
            return (format!("{x} {}", plural(noun)), None);
        }
        match v {
            Value::Const(1) => (with_article(noun), None),
            Value::Const(n) => (format!("{} {}", number_word(*n), plural(noun)), None),
            Value::X => (format!("X {}", plural(noun)), None),
            Value::EventAmount | Value::Prev | Value::Var(_) => {
                (format!("that many {}", plural(noun)), None)
            }
            other if x_arithmetic(other).is_some() => (
                format!(
                    "{} {}",
                    x_arithmetic(other).unwrap_or_default(),
                    plural(noun)
                ),
                None,
            ),
            other => {
                let s = self.value(other);
                (
                    format!("X {}", plural(noun)),
                    Some(format!(", where X is {s}")),
                )
            }
        }
    }

    /// The subject of [`Condition::PlayerMatches`], which holds if any of the players
    /// matches: "an opponent has 10 or less life", not "each opponent".
    pub(crate) fn some_player(&mut self, p: &PlayerRef) -> String {
        match p {
            PlayerRef::EachOpponent => "an opponent".into(),
            PlayerRef::EachPlayer => "a player".into(),
            PlayerRef::EachOtherPlayer => "another player".into(),
            other => self.player(other, Case::Subj),
        }
    }

    /// A condition as a clause ("you control an artifact").
    pub(crate) fn condition(&mut self, c: &Condition) -> String {
        self.new_clause();
        match c {
            Condition::Always => self.gap("Condition::Always"),
            Condition::Never => self.gap("Condition::Never"),
            Condition::Not(inner) => self.negated_condition(inner),
            // "If you cast it from your hand".
            Condition::And(v)
                if v.len() == 2
                    && matches!(v[0], Condition::WasCast)
                    && matches!(v[1], Condition::CastFrom(_)) =>
            {
                self.condition(&v[1])
            }
            Condition::And(v) => {
                let parts: Vec<String> = v.iter().map(|x| self.condition(x)).collect();
                merge_subject(&parts, "and")
            }
            Condition::Or(v) => {
                let parts: Vec<String> = v.iter().map(|x| self.condition(x)).collect();
                merge_subject(&parts, "or")
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
                let subj = self.some_player(p);
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

    fn cost_paid(&mut self, name: &str) -> String {
        match name {
            "kicked" | "kicker" => format!("{} was kicked", self.me()),
            "bargained" | "bargain" => format!("{} was bargained", self.me()),
            "gift" => "the gift was promised".into(),
            "teamwork" | "web-slinging" | "warp" => {
                let m = self.me();
                format!("{m} was cast using {name}")
            }
            "collect evidence" => "evidence was collected".into(),
            n if n.starts_with("kicker ") => {
                let m = self.me();
                format!("{m} was kicked with its {} kicker", &n["kicker ".len()..])
            }
            n if n.starts_with("alternative cost ") => {
                format!("the {} cost was paid", &n["alternative cost ".len()..])
            }
            "sneak" | "surge" | "prowl" | "spectacle" | "mayhem" | "freerunning" | "madness"
            | "dash" | "blitz" | "evoke" | "escape" | "emerge" | "plot" | "disturb"
            | "overload" | "harmonize" | "impending" | "flashback" | "awaken" | "jump-start"
            | "prototype" | "squad" | "offspring" | "bestow" => {
                let m = self.me();
                format!("{} {name} cost was paid", nouns::possessive(&m))
            }
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
                let subj = self.some_player(p);
                let pred = self.player_predicate(pf, &subj, true);
                format!("{subj} {pred}")
            }
            Condition::CostPaid(name) => {
                let s = self.cost_paid(name);
                s.replace(" was ", " wasn't ")
            }
            Condition::WasCast => "you didn't cast it".into(),
            // "you haven't cast a spell this turn"
            Condition::Compare(Value::SpellsCastThisTurn(..), Cmp::Ge, Value::Const(1)) => {
                let s = self.condition(c);
                match s.strip_prefix("you've ") {
                    Some(r) => format!("you haven't {r}"),
                    None => format!("it's not true that {s}"),
                }
            }
            Condition::Custom(n)
                if n == crate::oracle::patterns::grant_conditions::COMMITTED_CRIME_THIS_TURN =>
            {
                "you haven't committed a crime this turn".into()
            }
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
        let mut atoms: Vec<&Filter> = match f {
            Filter::And(v) => v.iter().collect(),
            other => vec![other],
        };
        // "is still in its zone" (not an object that has moved since, CR 400.7) is what
        // "is in your graveyard" says in the present tense.
        atoms.retain(|a| !matches!(a, Filter::Custom(n) if n == crate::zones::STILL_THERE));
        let mut parts = Vec::new();
        // "in your graveyard": the zone and its owner.
        let owned_zone = atoms.iter().find_map(|a| match a {
            Filter::InZone(z) if *z != ZoneKind::Battlefield && *z != ZoneKind::Exile => Some(*z),
            _ => None,
        });
        if let Some(z) = owned_zone {
            if atoms
                .iter()
                .any(|a| matches!(a, Filter::OwnedBy(PlayerRel::You)))
            {
                atoms.retain(|a| !matches!(a, Filter::InZone(_) | Filter::OwnedBy(PlayerRel::You)));
                parts.push(format!("in your {}", zone_word(z)));
            }
        }
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
                Filter::Targets(f) => {
                    let n = self.noun_det(f, Det::A);
                    if negated {
                        return format!("doesn't target {n}");
                    }
                    return format!("targets {n}");
                }
                Filter::Supertype(s) => nouns::supertype_word(*s).to_string(),
                Filter::Color(c) => c.word().to_string(),
                Filter::Colorless => "colorless".into(),
                Filter::Multicolored => "multicolored".into(),
                Filter::Monocolored => "monocolored".into(),
                Filter::Historic => "historic".into(),
                Filter::Custom(n) => {
                    let (adj, s) = self.custom_filter_quality(n);
                    if adj {
                        s
                    } else {
                        let n = self.noun_det(a, Det::A);
                        n
                    }
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
            PlayerFilter::LessThanHalfStartingLife => {
                let p = if you { "your" } else { "their" };
                format!("{have} less than half {p} starting life total")
            }
            PlayerFilter::DealtDamageThisTurn => "was dealt damage this turn".into(),
            // CR 122.1f: "poisoned" means having one or more poison counters.
            PlayerFilter::Counters(k, Cmp::Ge, v)
                if k.as_str() == "poison" && matches!(v.as_ref(), Value::Const(1)) =>
            {
                "is poisoned".into()
            }
            PlayerFilter::Counters(k, c, v) => {
                let v = self.value(v);
                if v == "1" && matches!(c, Cmp::Ge) {
                    return format!("{have} {}", with_article(&counter_name(k)));
                }
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
        if let Some(s) = self.this_turn_compare(a, cmp, b) {
            return s;
        }
        // "you have two or more opponents".
        if let (Value::CountPlayers(PlayerFilter::Opponent), Value::Const(n), Cmp::Ge) = (a, b, cmp)
        {
            return format!("you have {} or more opponents", number_word(*n));
        }
        // "creatures you control have total power 8 or greater".
        if let (Value::PowerOf(s), Value::Const(n)) = (a, b) {
            if let Sel::All(f) = s.as_ref() {
                let noun = self.noun_det(f, Det::Plural);
                return format!(
                    "{noun} have total power {}",
                    cmp_phrase(cmp, &n.to_string())
                );
            }
        }
        if let (Value::Custom(name), Value::Const(n)) = (a, b) {
            if let Some(s) = self.custom_compare(name, cmp, *n) {
                return s;
            }
        }
        if let (Value::CreaturesDiedThisTurn, Value::Const(n)) = (a, b) {
            return match (cmp, n) {
                (Cmp::Gt, 0) | (Cmp::Ge, 1) => "a creature died this turn".into(),
                (Cmp::Eq, 0) => "no creatures died this turn".into(),
                (Cmp::Ge, n) => format!("{} or more creatures died this turn", number_word(*n)),
                _ => {
                    let v = number_word(*n);
                    format!(
                        "the number of creatures that died this turn is {}",
                        cmp_phrase(cmp, &v)
                    )
                }
            };
        }
        // "you control three or more creatures".
        if let Value::Count(f) = a {
            if let Some((r, rest)) = split_controller(f).0.map(|r| (r, split_controller(f).1)) {
                if f.zone().is_none_or(|z| z == ZoneKind::Battlefield) {
                    // A count of permanents your opponents control is over all of them.
                    let subj = if r == PlayerRel::Opponent {
                        "your opponents".to_string()
                    } else {
                        self.rel_subject(r)
                    };
                    let verb = if subj == "you" || subj == "your opponents" {
                        "control"
                    } else {
                        "controls"
                    };
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
        // "if this is the second time this ability has resolved this turn".
        if let (Value::TimesResolvedThisTurn, Cmp::Eq, Value::Const(n)) = (a, cmp, b) {
            return format!(
                "this is the {} time this ability has resolved this turn",
                ordinal_word(*n as u32)
            );
        }
        let a_value = a;
        let a = self.value(a);
        // "there are seven or more cards in your graveyard".
        if let (Some(rest), Value::Const(n)) = (a.strip_prefix("the number of "), b) {
            let w = number_word(*n);
            if matches!((cmp, n), (Cmp::Ge, 1) | (Cmp::Gt, 0)) {
                // "if there's a counter on it" / "if it has a counter on it".
                if let (Value::CountersOn(sel, _), Some((counters, _))) =
                    (a_value, rest.split_once(" on "))
                {
                    let subj = self.sel(sel, Case::Subj);
                    return format!("{{alt:there is a {rest}|{subj} has a {counters} on it}}");
                }
                return format!("there is a {rest}");
            }
            let q = match cmp {
                Cmp::Ge => Some(format!("{w} or more")),
                Cmp::Gt if *n == 0 => None,
                Cmp::Gt => Some(format!("more than {w}")),
                Cmp::Le => Some(format!("{w} or fewer")),
                Cmp::Lt => Some(format!("fewer than {w}")),
                Cmp::Eq if *n == 0 => Some("no".to_string()),
                Cmp::Eq => Some(format!("exactly {w}")),
                Cmp::Ne => None,
            };
            if let Some(q) = q {
                // "there are three or more oil counters on ~" / "it has three or more oil
                // counters on it".
                if let (Value::CountersOn(sel, Some(_)), Some((counters, _))) =
                    (a_value, rest.split_once(" on "))
                {
                    let subj = self.sel(sel, Case::Subj);
                    return format!("{{alt:there are {q} {rest}|{subj} has {q} {counters} on it}}");
                }
                return format!("there are {q} {rest}");
            }
        }
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

    /// "you gained life this turn", "you've drawn two or more cards this turn".
    fn this_turn_compare(&mut self, a: &Value, cmp: Cmp, b: &Value) -> Option<String> {
        let Value::Const(n) = b else { return None };
        let min = match cmp {
            Cmp::Ge => *n,
            Cmp::Gt => n + 1,
            _ => return None,
        };
        // These values look at one player; a group ("each opponent") counts only its
        // first player, which no wording says.
        let you = |r: &mut Self, p: &PlayerRef| -> String {
            if is_player_group(p) {
                return r.gap("a one-player count of a group of players");
            }
            r.player(p, Case::Subj)
        };
        Some(match a {
            Value::LifeGainedThisTurn(p) => {
                let w = you(self, p);
                let have = if w == "you" {
                    "you've".to_string()
                } else {
                    format!("{w} has")
                };
                let did = if w == "you" {
                    "you".to_string()
                } else {
                    w.clone()
                };
                if min <= 1 {
                    format!("{did} gained life this turn")
                } else {
                    format!("{have} gained {min} or more life this turn")
                }
            }
            Value::LifeLostThisTurn(p) => {
                let w = you(self, p);
                if min <= 1 {
                    format!("{w} lost life this turn")
                } else {
                    let have = if w == "you" {
                        "you've".to_string()
                    } else {
                        format!("{w} has")
                    };
                    format!("{have} lost {min} or more life this turn")
                }
            }
            Value::CardsDrawnThisTurn(p) => {
                let w = you(self, p);
                let have = if w == "you" {
                    "you've".to_string()
                } else {
                    format!("{w} has")
                };
                let c = if min <= 1 {
                    "a card".to_string()
                } else {
                    format!("{} or more cards", number_word(min))
                };
                format!("{have} drawn {c} this turn")
            }
            Value::SpellsCastThisTurn(p, f) => {
                // Spells the players cast, together: "an opponent has cast a spell".
                let w = match (is_player_group(p), min) {
                    (true, 1) => self.some_player(p),
                    (true, _) => self.gap("spells several players cast, together"),
                    (false, _) => you(self, p),
                };
                let have = if w == "you" {
                    "you've".to_string()
                } else {
                    format!("{w} has")
                };
                let noun = if matches!(f, Filter::Any | Filter::Spell) {
                    if min <= 1 {
                        "spell".to_string()
                    } else {
                        "spells".to_string()
                    }
                } else {
                    self.noun(f, if min <= 1 { Num::One } else { Num::Many })
                };
                let noun = if noun.contains("spell") {
                    noun
                } else if min <= 1 {
                    format!("{noun} spell")
                } else {
                    format!("{noun} spells")
                };
                let noun = noun.replace("permanent spell", "spell");
                let c = if min <= 1 {
                    with_article(&noun)
                } else {
                    format!("{} or more {noun}", number_word(min))
                };
                format!("{have} cast {c} this turn")
            }
            Value::PermanentsEnteredThisTurn(p, f) => {
                let poss = self.player(p, Case::Poss);
                let noun = self.noun(f, if min <= 1 { Num::One } else { Num::Many });
                let c = if min <= 1 {
                    with_article(&noun)
                } else {
                    format!("{} or more {noun}", number_word(min))
                };
                let v = if min <= 1 { "entered" } else { "entered" };
                format!("{c} {v} the battlefield under {poss} control this turn")
            }
            _ => return None,
        })
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
                format!("for as long as {} remains on the battlefield", self.me())
            }
            Duration::WhileYouControlSource => format!("for as long as you control {}", self.me()),
            Duration::WhileCondition(c) => {
                let c = self.condition(c);
                format!("for as long as {c}")
            }
            Duration::Permanent => String::new(),
            Duration::UntilHostLeaves => String::new(),
            Duration::ThisTurn => "this turn".into(),
            Duration::ThroughNextUntapStep => "during its controller's next untap step".into(),
            Duration::ThroughYourNextUntapStep => "during your next untap step".into(),
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
            TriggerStep::PrecombatMain => "first main phase",
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

/// "you control a Plains or you control an Island" → "you control a Plains or an Island".
fn merge_subject(parts: &[String], conj: &str) -> String {
    for prefix in ["you control ", "you have ", "there are ", "there is "] {
        if parts.len() > 1 && parts.iter().all(|p| p.starts_with(prefix)) {
            let rest: Vec<String> = parts
                .iter()
                .map(|p| p[prefix.len()..].to_string())
                .collect();
            return format!("{prefix}{}", join_list(&rest, conj));
        }
    }
    join_list(parts, conj)
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

/// A reference to several players ("each opponent").
fn is_player_group(p: &PlayerRef) -> bool {
    matches!(
        p,
        PlayerRef::EachOpponent | PlayerRef::EachPlayer | PlayerRef::EachOtherPlayer
    )
}

/// "twice X", "three times X", "X plus 3": arithmetic on X written in place.
fn x_arithmetic(v: &Value) -> Option<String> {
    match v {
        Value::Mul(a, b) => match (a.as_ref(), b.as_ref()) {
            (Value::Const(2), Value::X) => Some("twice X".into()),
            (Value::Const(3), Value::X) => Some("three times X".into()),
            _ => None,
        },
        Value::Sum(v) => match v.as_slice() {
            [Value::X, Value::Const(n)] => Some(format!("X plus {n}")),
            _ => None,
        },
        _ => None,
    }
}
