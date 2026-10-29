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
        if let Some(s) = self.craft_value(v) {
            return s;
        }
        if let Some(s) = self.value_extra(v) {
            return s;
        }
        match v {
            Value::Const(n) => n.to_string(),
            Value::PlayerTally { var, .. } => crate::tallies::words(*var).into(),
            Value::X if self.extra.x_words.is_some() => self.extra.x_words.unwrap_or("X").into(),
            Value::X => "X".into(),
            // "for each card discarded this way": the objects the instruction before
            // affected.
            Value::Count(Filter::In(s))
                if matches!(s.as_ref(), Sel::Var(_)) && self.it_done.is_some() =>
            {
                let d = self.it_done.clone().unwrap_or_default();
                if d.contains('{') {
                    format!("the number of {d}")
                } else {
                    format!("the number of {{alt:them|{d}}}")
                }
            }
            Value::Count(f) => {
                let saved = (self.alt_and, self.plural_alts);
                (self.alt_and, self.plural_alts) = (true, true);
                let n = self.noun_det(f, Det::Plural);
                (self.alt_and, self.plural_alts) = saved;
                if Self::counts_all_permanents(f, &n) {
                    format!("the number of {n} {{opt:on the battlefield}}")
                } else {
                    format!("the number of {n}")
                }
            }
            // "the number of cards revealed this way": the cards a reveal instruction
            // revealed (a hand, the cards revealed until one was found, or the cards
            // chosen to reveal), as `value_results::this_way_sel` reads them.
            Value::CountSel(s)
                if matches!(s.as_ref(), Sel::Matching(u, _) if is_revealed_this_way(u)) =>
            {
                let Sel::Matching(_, f) = s.as_ref() else {
                    return self.gap("revealed this way");
                };
                let n = self.noun(f, Num::Many);
                format!("the number of {n} revealed this way")
            }
            // "for each of those cards that are still exiled": those of the remembered
            // cards still the same objects in exile (CR 400.7).
            Value::CountSel(s)
                if matches!(s.as_ref(), Sel::Matching(_, Filter::Custom(n))
                    if n == crate::oracle::patterns::referents_values_4::STILL_EXILED_CARD) =>
            {
                "the number of those cards that are still exiled".into()
            }
            Value::CountSel(s) if self.this_way_of(s).is_some() => {
                let n = self.noun(&Filter::In(Box::new((**s).clone())), Num::Many);
                format!("the number of {n}")
            }
            Value::CountSel(s)
                if matches!(s.as_ref(), Sel::Var(v) if *v == vars::IT)
                    && self.it_done.is_some() =>
            {
                let d = self.it_done.clone().unwrap_or_default();
                format!("the number of {d}")
            }
            // "the number of cards exiled from their hand this way" (`search_rules`).
            Value::CountSel(s)
                if matches!(s.as_ref(), Sel::Var(v) if *v == crate::search_rules::FROM_HAND) =>
            {
                let v = self.search_verb.unwrap_or("put");
                format!("the number of cards {v} from their hand this way")
            }
            Value::CountSel(s) => {
                let s = self.sel(s, Case::Obj);
                format!("the number of {s}")
            }
            Value::CountPlayers(pf) => {
                let n = self.player_filter_noun(pf, Num::Many);
                format!("the number of {n}")
            }
            // "the amassed Army's power" (CR 701.47c).
            Value::PowerOf(s) if matches!(s.as_ref(), Sel::Var(crate::kwa::kvars::AMASSED)) => {
                let p = self.sel(s, Case::Poss);
                format!("{{alt:the amassed Army's|{p}}} power")
            }
            Value::PowerOf(s) => {
                let s = self.sel(s, Case::Poss);
                // Several objects: the sum of their powers (`eval`), "their total power".
                if s == "their" {
                    return "their total power".into();
                }
                format!("{s} power")
            }
            Value::ToughnessOf(s) => {
                let s = self.sel(s, Case::Poss);
                format!("{s} toughness")
            }
            // "1 plus the mana value of the sacrificed artifact".
            Value::ManaValueOf(s) if matches!(s.as_ref(), Sel::Var(vars::SACRIFICED)) => {
                let o = self.sel(s, Case::Obj);
                let p = self.sel(s, Case::Poss);
                format!("{{alt:{p} mana value|the mana value of {o}}}")
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
            Value::AsTurnBegan(stat, who) => self.as_turn_began(*stat, who),
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
            // "where X is the amount of damage it dealt to that player".
            Value::EventAmount if self.extra.event_amount.is_some() => format!(
                "{{alt:that much|that many|that damage|{}}}",
                self.extra.event_amount.unwrap_or_default()
            ),
            // "amass Orcs X, where X is that excess damage" (CR 120.10).
            Value::EventAmount if self.excess_trigger => {
                "{alt:that much|that many|that excess damage|the excess}".into()
            }
            Value::EventAmount => "{alt:that much|that many}".into(),
            Value::Prev => "that many".into(),
            // "the amount of excess damage dealt to that creature this way": the one target
            // creature.
            Value::Var(vars::EXCESS)
                if matches!(self.targets.as_slice(), [t] if matches!(&t.what, TargetKind::Object(Filter::Type(CardType::Creature)))
                    && t.max.as_const() == Some(1)) =>
            {
                "{alt:the excess damage|the amount of excess damage dealt this way|the excess damage dealt this way|the amount of excess damage dealt to that creature this way}".into()
            }
            Value::Var(vars::EXCESS) => "{alt:the excess damage|the amount of excess damage dealt this way|the excess damage dealt this way}".into(),
            // The flips of the last coin flip (CR 705.2).
            Value::Var(crate::dice::WINS) => "the number of flips you won".into(),
            Value::Var(crate::dice::LOSSES) => "the number of flips you lost".into(),
            Value::Var(crate::dice::HEADS) => "the number of coins that come up heads".into(),
            Value::Var(crate::dice::TAILS) => "the number of coins that come up tails".into(),
            Value::Var(crate::oracle::patterns::iteration_grammar::MANA_PAID) => {
                "the total amount of mana paid this way".into()
            }
            // "the number of grace votes" (CR 701.38a).
            Value::Var(x)
                if self
                    .vote_words
                    .iter()
                    .any(|w| crate::kwa::vote::word_var(w) == *x) =>
            {
                let w = self
                    .vote_words
                    .iter()
                    .find(|w| crate::kwa::vote::word_var(w) == *x)
                    .cloned()
                    .unwrap_or_default();
                format!("the number of {w} votes")
            }
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
                // "the number of cards you've drawn this turn".
                if p == "you" {
                    "the number of cards you {alt:drew|drawn} this turn".to_string()
                } else {
                    format!("the number of cards {p} drew this turn")
                }
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
                let n = n.replace("permanent spell", "spell");
                // All players' spells together: "the number of spells cast this turn".
                if matches!(p, PlayerRef::EachPlayer) {
                    return format!("the number of {n} cast this turn");
                }
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
                    // "the number of white mana symbols in its mana cost".
                    Stat::ManaSymbols(c) if *op == AggOp::Sum => {
                        let p = match s.as_str() {
                            "it" => "its".to_string(),
                            other => super::nouns::possessive(other),
                        };
                        return format!("the number of {} mana symbols in {p} mana cost", c.word());
                    }
                    other => return self.gap(format!("Value::Aggregate {other:?}")),
                };
                let o = match op {
                    AggOp::Max => "greatest",
                    AggOp::Min => "least",
                    // "the total power of creatures you control"
                    AggOp::Sum => return format!("the total {st} of {s}"),
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
                    Among::PlaneswalkerTypes => "planeswalker types",
                    Among::BasicLandTypes => "basic land types",
                    Among::Colors => "colors",
                    // "five or more mana values among cards in your graveyard": distinct
                    // values, said with or without "different".
                    Among::ManaValues => "{opt:different} mana values",
                    Among::ManaCosts => "different mana costs",
                    Among::Powers => "different powers",
                    Among::Names => "different names",
                    Among::CounterKinds => "kinds of counters",
                    // "the greatest number of creatures you control that have a creature
                    // type in common".
                    Among::LargestCreatureTypeGroup => {
                        return format!(
                            "the greatest number of {s} that have a creature type in common"
                        );
                    }
                };
                // "for each of its colors".
                if s == "it" && matches!(among, Among::Colors) {
                    return "{alt:the number of colors among it|the number of its colors|the number of colors it is}".into();
                }
                format!("the number of {what} among {s}")
            }
            Value::OverPlayers(op, pf, v) => self.over_players_value(*op, pf, v),
            Value::Extreme(v, s, greatest) => self.extreme_value(v, s, *greatest),
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
                format!("the number of {{opt:different}} mana values among {n}")
            }
            Value::DistinctNames(f) => {
                let n = self.noun(f, Num::Many);
                format!("the number of differently named {n}")
            }
            Value::LargestSameNameGroup(f) => {
                let n = self.noun(f, Num::Many);
                format!("the number of {n} with the same name as one another")
            }
            Value::ColorsSpent => {
                format!("the number of colors of mana spent to cast {}", self.me())
            }
            Value::ManaSpent => format!("the amount of mana spent to cast {}", self.me()),
            // The number "pay any amount of life" as the permanent entered chose (see
            // `kw/pay_any_amount_of_life.rs`).
            Value::Chosen if self.extra.life_paid => "the life paid as ~it entered".into(),
            Value::Chosen => "the chosen number".into(),
            Value::TimesKicked => format!("the number of times {} was kicked", self.me()),
            Value::Speed(p) => {
                let p = self.player(p, Case::Poss);
                format!("{p} speed")
            }
            Value::TurnsTaken(p) => {
                let p = self.player(p, Case::Subj);
                format!("the number of turns {p} have taken")
            }
            // "the total number of instant and sorcery cards you own in exile and in your
            // graveyard": cards of one kind counted in two zones.
            Value::Sum(v) if super::history_values::graveyard_arrivals(v) => {
                match self.graveyard_arrivals_sum(v) {
                    Some(s) => s,
                    None => self.gap("a sum of cards put into a graveyard"),
                }
            }
            Value::Sum(v) if two_zone_count(v).is_some() => {
                let (f, zones) = two_zone_count(v).unwrap_or((Filter::Any, vec![]));
                let n = self.noun(&f, Num::Many);
                let z: Vec<&str> = zones
                    .iter()
                    .map(|z| match z {
                        ZoneKind::Exile => "you own in exile",
                        ZoneKind::Graveyard => "in your graveyard",
                        ZoneKind::Hand => "in your hand",
                        _ => "in your library",
                    })
                    .collect();
                let plain: Vec<String> = v.iter().map(|x| self.value(x)).collect();
                format!(
                    "{{alt:{}|the total number of {n} {} and {}}}",
                    plain.join(" plus "),
                    z[0],
                    z[1]
                )
            }
            Value::Sum(v) => {
                let parts: Vec<String> = v.iter().map(|x| self.value(x)).collect();
                let plain = parts.join(" plus ");
                // "the number of lands you control and land cards in your graveyard":
                // objects in different zones, never counted twice.
                let zones: Option<Vec<ZoneKind>> = v
                    .iter()
                    .map(|x| match x {
                        Value::Count(f) => Some(f.zone().unwrap_or(ZoneKind::Battlefield)),
                        Value::HandSize(_) => Some(ZoneKind::Hand),
                        _ => None,
                    })
                    .collect();
                let distinct = zones.is_some_and(|z| {
                    z.iter()
                        .enumerate()
                        .all(|(i, a)| z[..i].iter().all(|b| b != a))
                });
                let nouns: Option<Vec<String>> = parts
                    .iter()
                    .map(|p| p.strip_prefix("the number of ").map(str::to_string))
                    .collect();
                match nouns {
                    Some(n) if distinct && v.len() == 2 && !plain.contains('{') => {
                        format!("{{alt:{plain}|the number of {}}}", join_list(&n, "and"))
                    }
                    _ => plain,
                }
            }
            Value::Diff(a, b) => {
                // "If you have fewer than seven cards in hand, draw cards equal to the
                // difference": the difference of the numbers the condition compared.
                let (ka, kb) = (format!("{a:?}"), format!("{b:?}"));
                let named = self.compared.iter().any(|(x, y)| {
                    let (kx, ky) = (format!("{x:?}"), format!("{y:?}"));
                    (kx == ka && ky == kb) || (kx == kb && ky == ka)
                });
                let a = self.value(a);
                let b = self.value(b);
                if named {
                    format!("{{alt:the difference|{a} minus {b}}}")
                } else {
                    format!("{a} minus {b}")
                }
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
            Value::DivBy(v, d) => {
                let v = self.value(v);
                let d = self.value(d);
                format!("{v} divided evenly by {d}, rounded down")
            }
            Value::Min(a, b) => {
                let a = self.value(a);
                let b = self.value(b);
                format!("the lesser of {a} and {b}")
            }
            // "X can't be negative": a count clamped at zero (CR 107.1b) reads as the count.
            Value::Max(a, b) if matches!(b.as_ref(), Value::Const(0)) => self.value(a),
            Value::Max(a, b) if self.greatest_across_zones(a, b).is_some() => {
                self.greatest_across_zones(a, b).unwrap_or_default()
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
            Value::EventsThisTurn(c, t) => self.events_this_turn(c, *t),
            Value::EventsDuring(w, c, t) => self.gap(format!("EventsDuring({w:?}, {c:?}, {t:?})")),
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
            && !n.contains(" this way")
            && !n.contains("attacking")
            && !n.contains("blocking")
            && !n.contains("{opt:")
            && !n.ends_with(" ~")
    }

    pub(crate) fn amount(&mut self, v: &Value) -> (String, Option<String>) {
        // "deals damage equal to the excess": the excess damage the trigger event was
        // about (CR 120.10).
        if matches!(v, Value::EventAmount) && self.excess_trigger {
            return ("X".into(), Some(", where X is the excess".into()));
        }
        // "that much damage", "that many {E}": the same amount, worded for the noun.
        if matches!(v, Value::EventAmount) {
            return ("{alt:that much|that many}".into(), None);
        }
        // "You gain that much life": the amount of the previous instruction.
        if matches!(v, Value::Prev) {
            return ("{alt:that much|that many}".into(), None);
        }
        // "You gain 4 life for each creature tapped this way": a multiple of a number
        // remembered just before, said with that number's words.
        if let Value::Mul(k, n) = v {
            if let (Value::Const(_), Value::Var(x)) = (k.as_ref(), n.as_ref()) {
                if let Some(stored) = self
                    .stored_values
                    .iter()
                    .find(|(y, _, used)| y == x && !used)
                    .map(|(_, val, _)| val.clone())
                {
                    return self.amount(&Value::Mul(k.clone(), Box::new(stored)));
                }
            }
        }
        if let Some(x) = self.stored_x(v) {
            return (x, None);
        }
        // "Pay any amount of {E}. ~ deals that much damage": the amount paid; "Remove any
        // number of counters: It deals that much damage": the number removed.
        if matches!(v, Value::X) && (self.extra.x_paid || self.extra.x_removed) {
            return match self.extra.x_words {
                Some(w) if self.extra.x_paid => (w.into(), None),
                _ => ("{alt:that much|that many|X}".into(), None),
            };
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
        // "X plus 3 damage, where X is the number of creatures that died this way".
        if let Value::Sum(parts) = v {
            if let [a, Value::Const(k)] = parts.as_slice() {
                if *k > 0 && !Self::is_simple(a) {
                    let s = self.value(a);
                    return (format!("X plus {k}"), Some(format!(", where X is {s}")));
                }
            }
        }
        let s = self.value(v);
        ("X".into(), Some(format!(", where X is {s}")))
    }

    /// "a card" / "two cards" / "X cards, where X is ..." (the where clause is returned
    /// separately).
    /// A count said as "N [things] for each [thing]": (N's words with the noun, "for each
    /// ..."), for a value that's a number of things, or a multiple of one ("put two
    /// +1/+1 counters on ~ for each sprout vote").
    pub(crate) fn counted_for_each(&mut self, v: &Value, noun: &str) -> Option<(String, String)> {
        let (k, thing) = self.per_each(v)?;
        if thing.contains(['{', '|']) || k < 1 {
            return None;
        }
        let each = match thing.strip_prefix("times ") {
            Some(r) => format!("for each time {r}"),
            None if matches!(v, Value::Count(_))
                || matches!(v, Value::Mul(_, b) if matches!(b.as_ref(), Value::Count(_))) =>
            {
                format!("for each {thing}")
            }
            None => format!("for each {}", super::singular(&thing)),
        };
        let c = if k == 1 {
            with_article(noun)
        } else {
            format!("{} {}", number_word(k), plural(noun))
        };
        Some((c, each))
    }

    pub(crate) fn counted(&mut self, v: &Value, noun: &str) -> (String, Option<String>) {
        if let Some(x) = self.stored_x(v) {
            return (format!("{x} {}", plural(noun)), None);
        }
        // "Exile your hand, then draw that many cards": the number of objects the
        // instruction just before acted on.
        if let Value::CountSel(sel) = v {
            if let Sel::Var(x) = sel.as_ref() {
                if noun == "card" && self.this_way.last().is_some_and(|(y, _, _)| y == x) {
                    let s = self.value(v);
                    let p = plural(noun);
                    // "a card for each permanent destroyed this way".
                    let each = match s.strip_prefix("the number of ") {
                        Some(r) => format!("{} for each {r}", with_article(noun)),
                        None => format!("X {p}, where X is {s}"),
                    };
                    return (format!("{{alt:that many {p}|{each}}}"), None);
                }
            }
        }
        // "for each grace vote" (CR 701.38a).
        if let Value::Var(x) = v {
            if self
                .vote_words
                .iter()
                .any(|w| crate::kwa::vote::word_var(w) == *x)
            {
                let s = self.value(v);
                return (
                    format!("X {}", plural(noun)),
                    Some(format!(", where X is {s}")),
                );
            }
        }
        match v {
            Value::Const(1) => (with_article(noun), None),
            Value::Const(n) => (format!("{} {}", number_word(*n), plural(noun)), None),
            // The amount paid by "pay any amount of ..." ("draw that many cards").
            Value::X if self.extra.x_paid => {
                let w = self.extra.x_words.unwrap_or("that many");
                (format!("{w} {}", plural(noun)), None)
            }
            Value::X => (format!("X {}", plural(noun)), None),
            Value::EventAmount | Value::Prev | Value::Var(_) => {
                (format!("that many {}", plural(noun)), None)
            }
            Value::Custom(c) if c == "that many" => (format!("that many {}", plural(noun)), None),
            // "draw that many cards plus one".
            Value::Sum(v)
                if matches!(
                    v.as_slice(),
                    [
                        Value::EventAmount | Value::Prev | Value::Var(_),
                        Value::Const(_)
                    ]
                ) && !matches!(v[0], Value::Var(n) if n == vars::EXCESS) =>
            {
                let Value::Const(k) = v[1] else {
                    return (self.gap("that many plus"), None);
                };
                (
                    format!("that many {} plus {}", plural(noun), number_word(k)),
                    None,
                )
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
    /// "If grace gets more votes", "if condemnation gets more votes or the vote is tied"
    /// (CR 701.38a): the votes for a word compared with the most votes and the number of
    /// choices that got them.
    fn vote_condition(&mut self, c: &Condition) -> Option<String> {
        use crate::kwa::vote::{word_var, CHOICES_WITH_MOST, MOST_VOTES};
        let more = |c: &Condition, words: &[String]| -> Option<String> {
            let Condition::And(v) = c else {
                return None;
            };
            let [Condition::Compare(Value::Var(w), Cmp::Eq, Value::Var(MOST_VOTES)), Condition::Compare(Value::Var(CHOICES_WITH_MOST), Cmp::Eq, Value::Const(1))] =
                v.as_slice()
            else {
                return None;
            };
            words.iter().find(|x| word_var(x) == *w).cloned()
        };
        if self.vote_words.is_empty() {
            return None;
        }
        if let Some(w) = more(c, &self.vote_words) {
            return Some(format!("{w} gets more votes"));
        }
        if let Condition::Or(v) = c {
            if let [a, Condition::Compare(Value::Var(CHOICES_WITH_MOST), Cmp::Ge, Value::Const(2))] =
                v.as_slice()
            {
                if let Some(w) = more(a, &self.vote_words) {
                    return Some(format!("{w} gets more votes or the vote is tied"));
                }
            }
        }
        None
    }

    pub(crate) fn condition(&mut self, c: &Condition) -> String {
        self.new_clause();
        if let Some(s) = self.graveyard_order_condition(c) {
            return s;
        }
        if let Some(s) = self.compare_terms(c) {
            return s;
        }
        if let Some(v) = self.vote_condition(c) {
            return v;
        }
        if let Some(s) = self.condition_more(c) {
            return s;
        }
        match c {
            // "if it isn't being declared as an attacker" (`kw/tap_causes.rs`).
            Condition::SelMatches(Sel::TriggerObject, Filter::Custom(n))
                if n == crate::oracle::patterns::trigger_grammar_causes::NOT_TAPPED_ATTACKING =>
            {
                "it isn't being declared as an attacker".into()
            }
            // "a spell or ability an opponent controls causes ..." (`kw/tap_causes.rs`).
            Condition::Custom(n)
                if n == crate::oracle::patterns::trigger_grammar_causes::CAUSED_BY_OPPONENT =>
            {
                "a spell or ability an opponent controls caused it".into()
            }
            // "if you haven't cast the card" (`kw/delayed_conditions.rs`).
            Condition::Custom(n) if n.starts_with(crate::kw::delayed_conditions::STILL_EXILED) => {
                "{alt:you haven't cast it|it remains exiled|you haven't played it|any of those cards remain exiled}".into()
            }
            Condition::Always => self.gap("Condition::Always"),
            Condition::Never => self.gap("Condition::Never"),
            // CR 702.62b: a card is suspended if it's in exile, has suspend, and has a
            // time counter on it.
            Condition::And(v)
                if format!("{v:?}")
                    == "[SelMatches(This, InZone(Exile)), SelMatches(This, HasKeyword(Suspend)), \
                        Compare(CountersOn(This, Some(\"time\")), Gt, Const(0))]" =>
            {
                format!("{} is suspended", self.me())
            }
            // "if you have more cards in hand than each opponent": no opponent has at
            // least as many.
            Condition::Not(inner)
                if matches!(inner.as_ref(), Condition::PlayerMatches(PlayerRef::EachOpponent,
                    PlayerFilter::HandSize(Cmp::Ge, v) | PlayerFilter::Life(Cmp::Ge, v))
                    if matches!(v.as_ref(), Value::HandSize(PlayerRef::You) | Value::LifeTotal(PlayerRef::You))) =>
            {
                let Condition::PlayerMatches(_, pf) = inner.as_ref() else {
                    return self.gap("more than each opponent");
                };
                match pf {
                    PlayerFilter::HandSize(_, v)
                        if matches!(v.as_ref(), Value::HandSize(PlayerRef::You)) =>
                    {
                        "you have more cards in hand than each opponent".into()
                    }
                    PlayerFilter::Life(_, v)
                        if matches!(v.as_ref(), Value::LifeTotal(PlayerRef::You)) =>
                    {
                        "you have more life than each opponent".into()
                    }
                    _ => {
                        let inner = inner.as_ref().clone();
                        self.negated_condition(&inner)
                    }
                }
            }
            // "if no opponent has more life than that player".
            Condition::Not(inner)
                if matches!(inner.as_ref(), Condition::PlayerMatches(PlayerRef::EachOpponent, PlayerFilter::Life(Cmp::Gt, v))
                    if matches!(v.as_ref(), Value::LifeTotal(_))) =>
            {
                let Condition::PlayerMatches(_, PlayerFilter::Life(_, v)) = inner.as_ref() else {
                    return self.gap("no opponent has more life");
                };
                let Value::LifeTotal(p) = v.as_ref() else {
                    return self.gap("no opponent has more life");
                };
                let p = self.player(p, Case::Obj);
                format!("no opponent has more life than {p}")
            }
            Condition::Not(inner) => self.negated_condition(inner),
            // "If you revealed a Dragon card or controlled a Dragon as you cast this
            // spell" (`kw::revealed_or_controlled`).
            Condition::Or(v)
                if self.info.reveal_card.is_some()
                    && matches!(v.as_slice(), [Condition::CostPaid(a), Condition::CostPaid(b)]
                        if a == crate::kw::revealed_or_controlled::REVEAL
                            && b == crate::kw::revealed_or_controlled::CONTROLLED) =>
            {
                let card = self.info.reveal_card.clone().unwrap_or(Filter::Any);
                let quality = match &card {
                    Filter::And(v) => Filter::and(
                        v.iter()
                            .filter(|x| !matches!(x, Filter::Card | Filter::InZone(_) | Filter::OwnedBy(_)))
                            .cloned()
                            .collect(),
                    ),
                    other => other.clone(),
                };
                let c = self.noun_det(&card, Det::A);
                let c = c.trim_end_matches(" in your hand").to_string();
                let q = self.noun_det(&quality, Det::A);
                format!("you revealed {c} or controlled {q} as you cast ~")
            }
            // "If you cast it from your hand".
            Condition::And(v)
                if v.len() == 2
                    && matches!(v[0], Condition::WasCast)
                    && matches!(v[1], Condition::CastFrom(_)) =>
            {
                self.condition(&v[1])
            }
            // "If this spell was cast from anywhere other than your hand".
            Condition::And(v)
                if v.len() == 2
                    && matches!(v[0], Condition::WasCast)
                    && matches!(&v[1], Condition::Not(x) if matches!(x.as_ref(), Condition::CastFrom(_))) =>
            {
                let Condition::Not(x) = &v[1] else {
                    return String::new();
                };
                let z = match x.as_ref() {
                    Condition::CastFrom(z) => zone_word(*z),
                    _ => "zone",
                };
                format!("~ {{alt:was|is}} cast from anywhere other than your {z}")
            }
            // "It's your first, second, or third turn of the game".
            Condition::And(_) if crate::rule_statics::turns_taken::early_turns_n(c).is_some() => {
                let n = crate::rule_statics::turns_taken::early_turns_n(c).unwrap_or(1);
                let l = crate::rule_statics::turns_taken::ordinal_list(n);
                format!("it's your {l} turn of the game")
            }
            // CR 730.1: "it's neither day nor night" (the game has neither designation).
            Condition::And(v)
                if v.len() == 2
                    && v.iter().any(|x| matches!(x, Condition::Not(c) if matches!(c.as_ref(), Condition::IsDay)))
                    && v.iter().any(|x| matches!(x, Condition::Not(c) if matches!(c.as_ref(), Condition::IsNight))) =>
            {
                "it's neither day nor night".into()
            }
            // "If {W}{U} was spent to cast this spell": mana of each of those colors.
            Condition::And(v)
                if v.len() >= 2
                    && v.iter().all(|x| matches!(x,
                        Condition::Compare(Value::Custom(n), Cmp::Ge, Value::Const(1))
                            if n.starts_with("mana_spent_of:"))) =>
            {
                let syms: String = v
                    .iter()
                    .filter_map(|x| match x {
                        Condition::Compare(Value::Custom(n), ..) => {
                            n.strip_prefix("mana_spent_of:").map(|l| format!("{{{l}}}"))
                        }
                        _ => None,
                    })
                    .collect();
                format!("{syms} was spent to cast {}", self.me())
            }
            // "if it's at least one of the chosen colors": the spell exists (it's the
            // triggering spell) and the count about it; the first says nothing of its own.
            Condition::And(v)
                if matches!(v.as_slice(), [Condition::SelMatches(Sel::TriggerObject, Filter::Any), Condition::Compare(Value::Custom(_), _, _)]) =>
            {
                self.condition(&v[1])
            }
            Condition::And(v) => {
                // "if it's your main phase": a main phase of your turn (CR 505.1).
                if matches!(
                    v.as_slice(),
                    [Condition::YourTurn, Condition::Phase(PhaseCond::MainPhase)]
                ) {
                    return "it's your main phase".into();
                }
                if let Some(s) = self.attempt(|r| r.events_joined(v, "and")) {
                    return s;
                }
                if let Some(s) = self.attempt(|r| r.cast_from_elsewhere(v)) {
                    return s;
                }
                // "if it's the second creature spell you cast this turn".
                if let [Condition::SelMatches(Sel::TriggerSpell, k), Condition::Compare(a, cmp, b)] =
                    v.as_slice()
                {
                    if let Some(s) = self.attempt(|r| r.nth_spell_cast(a, *cmp, b, Some(k))) {
                        return s;
                    }
                }
                // "if it's the first combat phase of your turn".
                if let [Condition::YourTurn, Condition::Custom(n)] = v.as_slice() {
                    if n == crate::kw::activated_ability_kind::FIRST_COMBAT_PHASE {
                        return "it's the first combat phase of your turn".into();
                    }
                }
                let parts: Vec<String> = v.iter().map(|x| self.condition(x)).collect();
                let merged = merge_subject(&parts, "and");
                // "if you control artifacts named Crown of Empires and Scepter of Empires".
                let named = |c: &Condition| match c {
                    Condition::Exists(Filter::And(f)) => match f.as_slice() {
                        [Filter::Type(t), Filter::Named(n), Filter::ControlledBy(PlayerRel::You)] => {
                            Some((*t, n.to_string()))
                        }
                        _ => None,
                    },
                    _ => None,
                };
                if let Some(ns) = v.iter().map(named).collect::<Option<Vec<_>>>() {
                    if ns.len() > 1 && ns.iter().all(|(t, _)| *t == ns[0].0) {
                        let kind = plural(&ns[0].0.word().to_lowercase());
                        let names: Vec<String> = ns.into_iter().map(|(_, n)| n).collect();
                        return format!(
                            "{{alt:{merged}|you control {kind} named {}}}",
                            join_list(&names, "and")
                        );
                    }
                }
                merged
            }
            Condition::Or(v) => {
                if let Some(s) = self.attempt(|r| r.events_joined(v, "or")) {
                    return s;
                }
                if let Some(s) = self.attempt(|r| r.some_of_them_history(v)) {
                    return s;
                }
                if let Some(s) = self.attempt(|r| r.own_entered_or_cast(v)) {
                    return s;
                }
                let parts: Vec<String> = v.iter().map(|x| self.condition(x)).collect();
                merge_subject(&parts, "or")
            }
            // "if the Ring has tempted you four or more times this game" (CR 701.54).
            Condition::Compare(Value::Custom(n), Cmp::Ge, Value::Const(k))
                if n == crate::kwa::ring::TIMES_TEMPTED =>
            {
                format!("the Ring has tempted you {} or more times this game", number_word(*k))
            }
            Condition::Compare(a, cmp, b) => self.compare_condition(a, *cmp, b),
            Condition::Exists(f) => self.exists(f, false),
            Condition::SelNonEmpty(s) => {
                if let Some(h) = self.attempt(|r| r.some_of_them_history(std::slice::from_ref(c))) {
                    return h;
                }
                if let Some(h) = self.attempt(|r| r.acted_on_this_way(s)) {
                    return h;
                }
                let s = self.sel(s, Case::Subj);
                format!("{s} exists")
            }
            // "If its mana value was 3 or less" (judged by last known information if
            // it has left its zone, CR 608.2h).
            Condition::SelMatches(s, Filter::ManaValue(c, v)) => {
                let poss = self.sel(s, Case::Poss);
                let v = self.value(v);
                format!("{poss} mana value is {}", cmp_phrase(*c, &v))
            }
            // "if ~ entered this turn".
            // "If you controlled that permanent" (after it left the battlefield; its last
            // known information, CR 608.2h).
            Condition::SelMatches(s, Filter::ControlledBy(PlayerRel::You))
                if !matches!(s, Sel::This) =>
            {
                let subj = self.sel(s, Case::Subj);
                format!("{{alt:{subj} is under your control|you control {subj}|you controlled {subj}}}")
            }
            Condition::SelMatches(s, Filter::EnteredThisTurn) => {
                let subj = self.sel(s, Case::Subj);
                format!("{subj} entered this turn")
            }
            // "If it's blue or black": colors only, as adjectives.
            Condition::SelMatches(s, Filter::Or(v))
                if v.len() > 1 && v.iter().all(|f| matches!(f, Filter::Color(_))) =>
            {
                let words: Vec<String> = v
                    .iter()
                    .filter_map(|f| match f {
                        Filter::Color(c) => Some(c.word().to_string()),
                        _ => None,
                    })
                    .collect();
                let subj = self.sel(s, Case::Subj);
                format!("{subj} is {}", join_list(&words, "or"))
            }
            // "if that spell targets only ~", "if it has a single target".
            Condition::SelMatches(s, Filter::StackTargets(tf)) => {
                let subj = self.sel(s, Case::Subj);
                let t = self.targets_filter(tf);
                match (t.strip_prefix("that "), t.strip_prefix("with ")) {
                    (Some(r), _) => format!("{subj} {r}"),
                    (_, Some(r)) => format!("{subj} has {r}"),
                    _ => format!("{subj} {t}"),
                }
            }
            // "if ~ didn't attack this turn".
            Condition::SelMatches(s, Filter::AttackedThisTurn) => {
                let subj = self.sel(s, Case::Subj);
                format!("{subj} attacked this turn")
            }
            Condition::SelMatches(s, f) => {
                if let Some(c) = self.custom_condition_clause(s, f, false) {
                    return c;
                }
                let subj = self.sel(s, Case::Subj);
                let pred = self.is_predicate(f, false);
                format!("{subj} {pred}")
            }
            Condition::PlayerMatches(p, pf) => {
                let subj = self.some_player(p);
                let pred = self.player_predicate(pf, &subj, false);
                // "an opponent has less life than you" is said from your side: "you have
                // more life than an opponent".
                let mirrored = match pred.as_str() {
                    "has less life than you" => Some("more life"),
                    "has fewer cards in hand than you" => Some("more cards in hand"),
                    _ => None,
                };
                match mirrored {
                    Some(m) if !subj.contains(['{', '|', '}']) => {
                        format!("{{alt:{subj} {pred}|you have {m} than {subj}}}")
                    }
                    _ => format!("{subj} {pred}"),
                }
            }
            Condition::YourTurn => "it's your turn".into(),
            Condition::NotYourTurn => "it's not your turn".into(),
            Condition::CostPaid(name) => self.cost_paid(name),
            Condition::WasCast => "you cast it".into(),
            Condition::PrevHappened => "you do".into(),
            Condition::PrevAffectedAny => "a card was affected this way".into(),
            // "If this spell was cast from a graveyard" (the zone it was cast from, CR
            // 601.2a); cards casting their own card say "if you cast it from your ...".
            // "If ~ was cast from exile" (there's one exile zone, CR 406.1).
            Condition::CastFrom(ZoneKind::Exile) => {
                "{alt:you cast it from exile|~ {alt:was|is} cast from exile}".into()
            }
            Condition::CastFrom(z) => {
                let z = zone_word(*z);
                // Your own card is in your zone (CR 400.3): "from a graveyard" is "from
                // your graveyard".
                format!("{{alt:you cast it from your {z}|~ {{alt:was|is}} cast from a {z}|~ {{alt:was|is}} cast from your {z}}}")
            }
            Condition::AllTriggerConditionsThisTurn(v) => {
                let parts: Vec<String> = v.iter().map(|t| self.happened_this_turn(t)).collect();
                join_list(&parts, "and")
            }
            Condition::ChosenWord(w) => format!("{w} was chosen"),
            Condition::Phase(p) => match p {
                PhaseCond::Combat => "it's combat".into(),
                // Any player's main phase.
                PhaseCond::MainPhase => "it's a main phase".into(),
                PhaseCond::Upkeep => "it's an upkeep step".into(),
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
            "behold" if self.info.behold.is_some() => self.behold_paid().unwrap_or_default(),
            n if n.starts_with("kicker ") => {
                let m = self.me();
                format!("{m} was kicked with its {} kicker", &n["kicker ".len()..])
            }
            n if n.starts_with("alternative cost ") => {
                format!("the {} cost was paid", &n["alternative cost ".len()..])
            }
            // "If it escaped" (CR 702.138c).
            "escape" => {
                let m = self.me();
                format!(
                    "{{alt:{m} escaped|{} escape cost was paid}}",
                    nouns::possessive(&m)
                )
            }
            "sneak" | "surge" | "prowl" | "spectacle" | "mayhem" | "freerunning" | "madness"
            | "dash" | "blitz" | "evoke" | "emerge" | "plot" | "disturb" | "overload"
            | "harmonize" | "impending" | "flashback" | "awaken" | "jump-start" | "prototype"
            | "squad" | "offspring" | "bestow" => {
                let m = self.me();
                format!("{} {name} cost was paid", nouns::possessive(&m))
            }
            other => format!("the {other} cost was paid"),
        }
    }

    fn negated_condition(&mut self, c: &Condition) -> String {
        if let Condition::Compare(a, cmp, b) = c {
            if let Some(s) = self.attempt(|r| r.events_compare(a, *cmp, b, true)) {
                return s;
            }
        }
        if let Condition::Custom(n) = c {
            if let Some(s) = self.custom_condition_terms(n, true) {
                return s;
            }
        }
        match c {
            Condition::Exists(f) => self.exists(f, true),
            Condition::YourTurn => "it's not your turn".into(),
            Condition::NotYourTurn => "it's your turn".into(),
            Condition::PrevHappened => "you don't".into(),
            // "if ~ didn't attack this turn".
            Condition::SelMatches(s, Filter::AttackedThisTurn) => {
                let subj = self.sel(s, Case::Subj);
                format!("{subj} didn't attack this turn")
            }
            Condition::SelMatches(s, f) => {
                if let Some(c) = self.custom_condition_clause(s, f, true) {
                    return c;
                }
                let subj = self.sel(s, Case::Subj);
                let pred = self.is_predicate(f, true);
                format!("{subj} {pred}")
            }
            // Only the predicates worded negatively; the others would lose the negation.
            Condition::PlayerMatches(p, pf)
                if matches!(
                    pf,
                    PlayerFilter::Life(..)
                        | PlayerFilter::Opponent
                        | PlayerFilter::You
                        | PlayerFilter::Any
                ) =>
            {
                let subj = self.some_player(p);
                let pred = self.player_predicate(pf, &subj, true);
                format!("{subj} {pred}")
            }
            Condition::CostPaid(name) => {
                let s = self.cost_paid(name);
                s.replace(" was ", " wasn't ")
            }
            // "If it's not their turn" (the active player's turn, CR 102.1).
            Condition::PlayerMatches(p, PlayerFilter::Active) if !matches!(p, PlayerRef::You) => {
                let poss = self.player(p, Case::Poss);
                let poss = if poss.starts_with("{alt:") || poss == "their" {
                    "{alt:their|that player's}".to_string()
                } else {
                    poss
                };
                format!("it's not {poss} turn")
            }
            Condition::WasCast => "you didn't cast it".into(),
            // "If you didn't cast it from your hand" / "if ~ was cast from anywhere other
            // than your hand".
            Condition::CastFrom(z) => {
                let z = zone_word(*z);
                format!("{{alt:you didn't cast it from your {z}|~ {{alt:was|is}} cast from anywhere other than your {z}}}")
            }
            // "If you didn't cast it from your hand": not cast, or cast from elsewhere.
            Condition::And(v)
                if v.len() == 2
                    && matches!(v[0], Condition::WasCast)
                    && matches!(v[1], Condition::CastFrom(_)) =>
            {
                self.negated_condition(&v[1])
            }
            // "you haven't cast a spell this turn"
            Condition::Compare(Value::SpellsCastThisTurn(..), Cmp::Ge, Value::Const(1)) => {
                let s = self.condition(c);
                match s.strip_prefix("you've ") {
                    Some(r) => format!("you haven't {r}"),
                    None if s.contains(" has cast ") => {
                        s.replacen(" has cast ", " hasn't cast ", 1)
                    }
                    None => format!("it's not true that {s}"),
                }
            }
            Condition::Custom(n)
                if n == crate::oracle::patterns::grant_conditions::COMMITTED_CRIME_THIS_TURN =>
            {
                "you haven't committed a crime this turn".into()
            }
            Condition::Custom(n) if n == "you_were_the_starting_player" => {
                "you weren't the starting player".into()
            }
            other => {
                let s = self.condition(other);
                format!("it's not true that {s}")
            }
        }
    }

    /// "you control an artifact", "you control no Islands", "there are ... in your graveyard".
    pub(crate) fn exists(&mut self, f: &Filter, negated: bool) -> String {
        // "if a card is exiled with it" (CR 607.2a: exiled by an ability of this object).
        if let (Filter::And(v), false) = (f, negated) {
            let linked =
                |x: &Filter| matches!(x, Filter::In(s) if matches!(s.as_ref(), Sel::Linked));
            if v.iter().any(linked) {
                let rest = Filter::and(
                    v.iter()
                        .filter(|x| !linked(x) && !matches!(x, Filter::InZone(ZoneKind::Exile)))
                        .cloned()
                        .collect(),
                );
                let n = match rest {
                    Filter::Any | Filter::Card => "a card".to_string(),
                    r => self.noun_det(&r, Det::A),
                };
                let m = self.me();
                let there = self.exists_plain(f, negated);
                return format!("{{alt:{there}|{n} is exiled with {m}}}");
            }
        }
        self.exists_plain(f, negated)
    }

    /// [`Self::exists`] without the alternative wordings above.
    fn exists_plain(&mut self, f: &Filter, negated: bool) -> String {
        // "if a creature is attacking you".
        if let (Filter::And(v), false) = (f, negated) {
            if let [Filter::Type(CardType::Creature), Filter::Attacking, Filter::AttackingPlayer(r)] =
                v.as_slice()
            {
                let p = self.rel_object(*r);
                return format!("a creature is attacking {p}");
            }
        }
        // "if you don't control a Ring-bearer" (your Ring-bearer is a creature you
        // control, CR 701.54a).
        if matches!(f, Filter::Custom(n) if n == crate::kwa::ring::RING_BEARER) {
            return if negated {
                "you don't control a Ring-bearer".into()
            } else {
                "you control a Ring-bearer".into()
            };
        }
        // "if another opponent controls one or more nonland permanents that spell could
        // target": an opponent other than the controller of what the spell targets.
        if let (Filter::And(v), false) = (f, negated) {
            let other_than_targeted = |x: &Filter| {
                matches!(x, Filter::Not(n) if matches!(n.as_ref(), Filter::ControlledByPlayer(p)
                    if matches!(p.as_ref(), PlayerRef::ControllerOf(s) if matches!(s.as_ref(), Sel::All(t) if matches!(t, Filter::TargetOf(ts) if matches!(ts.as_ref(), Sel::TriggerSpell))))))
            };
            if v.iter().any(other_than_targeted)
                && v.iter()
                    .any(|x| matches!(x, Filter::ControlledBy(PlayerRel::Opponent)))
            {
                let rest = Filter::and(
                    v.iter()
                        .filter(|x| {
                            !other_than_targeted(x)
                                && !matches!(x, Filter::ControlledBy(PlayerRel::Opponent))
                        })
                        .cloned()
                        .collect(),
                );
                let n = self.noun_det(&rest, Det::A);
                let many = self.noun(&rest, Num::Many);
                return format!("another opponent controls {{alt:{n}|one or more {many}}}");
            }
        }
        // "if your team controls another Warrior": you or a teammate (CR 102.4).
        if let Filter::And(v) = f {
            let team = |x: &Filter| {
                matches!(x, Filter::Or(o) if matches!(o.as_slice(),
                    [Filter::ControlledBy(PlayerRel::You), Filter::ControlledBy(PlayerRel::Teammate)]))
            };
            if v.iter().filter(|x| team(x)).count() == 1 && f.zone().is_none() {
                let rest = Filter::and(v.iter().filter(|x| !team(x)).cloned().collect());
                return if negated {
                    let n = self.noun(&rest, Num::Many);
                    format!("your team controls no {n}")
                } else {
                    let n = self.noun_det(&rest, Det::A);
                    format!("your team controls {n}")
                };
            }
        }
        let (ctrl, rest) = split_controller(f);
        // "you control the creature with the greatest power or tied for the greatest
        // power": one of yours has power at least the greatest among all creatures.
        if let (Some(PlayerRel::You), Filter::And(v), false) = (ctrl, &rest, negated) {
            if let [t @ Filter::Type(_), Filter::Power(Cmp::Ge, p)] = v.as_slice() {
                if matches!(p.as_ref(), Value::GreatestPower(g) if format!("{g:?}") == format!("{t:?}"))
                {
                    let n = self.noun(t, Num::One);
                    return format!(
                        "you control the {n} with the greatest power or tied for the greatest power"
                    );
                }
            }
        }
        let zone = f.zone();
        match (ctrl, zone) {
            // "your opponents control no creatures": no opponent controls one.
            (Some(PlayerRel::Opponent), None)
            | (Some(PlayerRel::Opponent), Some(ZoneKind::Battlefield))
                if negated =>
            {
                let n = self.noun(&rest, Num::Many);
                format!("your opponents control no {n}")
            }
            (Some(r), None) | (Some(r), Some(ZoneKind::Battlefield)) => {
                let subj = self.rel_subject(r);
                let verb = if subj == "you" { "control" } else { "controls" };
                if negated {
                    // "you control no Humans" / "you don't control a Human".
                    let n = self.noun(&rest, Num::Many);
                    let dont = if subj == "you" { "don't" } else { "doesn't" };
                    format!("{subj} {{alt:{verb} no|{dont} control}} {n}")
                } else {
                    let n = self.noun_det(&rest, Det::A);
                    // "if you control one or more other creatures ...": another one.
                    if let Some(r) = n.strip_prefix("another ").filter(|r| !r.contains('{')) {
                        let many = self.noun(&rest, Num::Many);
                        if let Some(m) = many.strip_prefix("other ") {
                            return format!(
                                "{subj} {verb} {{alt:another {r}|one or more other {m}}}"
                            );
                        }
                    }
                    format!("{subj} {verb} {n}")
                }
            }
            // "there is a Mountain on the battlefield": a permanent is on the battlefield
            // (CR 110.1), so cards may say so or not.
            _ => {
                let on_bf = if zone.is_none() || zone == Some(ZoneKind::Battlefield) {
                    " {opt:on the battlefield}"
                } else {
                    ""
                };
                if negated {
                    let n = self.noun(f, Num::Many);
                    format!("there are no {n}{on_bf}")
                } else {
                    let n = self.noun_det(f, Det::A);
                    format!("there is {n}{on_bf}")
                }
            }
        }
    }

    /// "is tapped", "is a creature", "has flying".
    pub(crate) fn is_predicate(&mut self, f: &Filter, negated: bool) -> String {
        if let Filter::Custom(n) = f {
            if let Some(p) = super::custom_filters::custom_predicate(n) {
                let is = if negated { "isn't" } else { "is" };
                return format!("{is} {p}");
            }
        }
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
        // "is a creature card", "is a noncreature, nonland card": a card's types and
        // qualities make one noun phrase.
        let card_noun = |a: &&Filter| {
            matches!(
                a,
                Filter::Card
                    | Filter::PermanentCard
                    | Filter::Type(_)
                    | Filter::Subtype(_)
                    | Filter::Supertype(_)
                    | Filter::Color(_)
                    | Filter::Colorless
                    | Filter::Multicolored
                    | Filter::Monocolored
                    | Filter::Historic
                    | Filter::ChosenType
                    | Filter::Or(_)
                    | Filter::Not(_)
            )
        };
        if parts.is_empty()
            && atoms.len() > 1
            && atoms.iter().any(|a| matches!(a, Filter::Card))
            && atoms.iter().all(card_noun)
        {
            let f = Filter::And(atoms.into_iter().cloned().collect());
            let n = self.noun_det(&f, Det::A);
            return if negated {
                format!("isn't {n}")
            } else {
                format!("is {n}")
            };
        }
        // "If that card has the chosen name".
        if parts.is_empty() && matches!(atoms.as_slice(), [Filter::ChosenName]) {
            return if negated {
                "doesn't have {alt:the chosen name|that name}".into()
            } else {
                "has {alt:the chosen name|that name}".into()
            };
        }
        for a in atoms {
            let p = match a {
                Filter::Tapped => "tapped".to_string(),
                Filter::Untapped => "untapped".into(),
                Filter::Attacking => "attacking".into(),
                Filter::AttackingAlone => "attacking alone".into(),
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
                    // "if it has counters on it": one or more.
                    if k.is_none() {
                        return "has {alt:a counter|counters} on it".into();
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
                // "if it's nonbasic": a land without the supertype basic (CR 205.4c).
                Filter::Not(inner)
                    if !negated
                        && matches!(
                            inner.as_ref(),
                            Filter::Supertype(crate::types::Supertype::Basic)
                        ) =>
                {
                    return "{alt:is nonbasic|isn't basic}".into();
                }
                Filter::Not(inner) => {
                    let p = self.is_predicate(inner, !negated);
                    return p;
                }
                // "is attacking or blocking": a creature in combat (CR 506.4).
                Filter::Or(v) if v.iter().all(|x| combat_status(x).is_some()) => {
                    let w: Vec<String> = v
                        .iter()
                        .filter_map(combat_status)
                        .map(str::to_string)
                        .collect();
                    let n = self.noun_det(a, Det::A);
                    format!("{{alt:{}|{n}}}", join_list(&w, "or"))
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
        // "has more life than you", "has at least four more cards in hand than you".
        let than_you = |v: &Value| -> Option<(bool, Option<i32>)> {
            match v {
                Value::LifeTotal(PlayerRef::You) => Some((true, None)),
                Value::HandSize(PlayerRef::You) => Some((false, None)),
                Value::Sum(xs) => match xs.as_slice() {
                    [Value::LifeTotal(PlayerRef::You), Value::Const(n)] => Some((true, Some(*n))),
                    [Value::HandSize(PlayerRef::You), Value::Const(n)] => Some((false, Some(*n))),
                    _ => None,
                },
                _ => None,
            }
        };
        if !negated && !you {
            if let PlayerFilter::Life(c, v) | PlayerFilter::HandSize(c, v) = pf {
                if let Some((life, extra)) = than_you(v) {
                    let what = if life { "life" } else { "cards in hand" };
                    let is_life = matches!(pf, PlayerFilter::Life(..));
                    if is_life == life {
                        match (c, extra) {
                            (Cmp::Gt, None) => return format!("{have} more {what} than you"),
                            (Cmp::Lt, None) if life => {
                                return format!("{have} less {what} than you")
                            }
                            (Cmp::Lt, None) => return format!("{have} fewer {what} than you"),
                            (Cmp::Ge, Some(n)) => {
                                return format!(
                                    "{have} at least {} more {what} than you",
                                    number_word(n)
                                )
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        // "you have at least 7 life more than your starting life total".
        if let PlayerFilter::Life(Cmp::Ge, v) = pf {
            if let Value::Sum(xs) = v.as_ref() {
                if let [Value::StartingLife, Value::Const(n)] = xs.as_slice() {
                    let n = number_word(*n);
                    let life = format!("at least {n} life more than your starting life total");
                    return if negated {
                        format!("doesn't have {life}")
                    } else {
                        format!("{have} {life}")
                    };
                }
            }
        }
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
            PlayerFilter::HasInitiative => format!("{have} the initiative"),
            PlayerFilter::AttackRelation {
                attacking,
                other,
                past,
                planeswalkers,
            } => {
                let o = self.player_filter_object(other);
                let pw = if *planeswalkers {
                    " and/or a planeswalker you control"
                } else {
                    ""
                };
                match (attacking, past) {
                    (true, false) => format!("{{alt:is|are}} attacking {o}{pw}"),
                    (true, true) => format!("attacked {o}{pw}"),
                    (false, false) => format!("{{alt:is|are}} being attacked by {o}"),
                    (false, true) => format!("{{alt:was|were}} attacked by {o}"),
                }
            }
            PlayerFilter::MaxSpeed => format!("{have} max speed"),
            PlayerFilter::Chose(w) => format!("chose {w}"),
            PlayerFilter::ChosenPlayers => "{alt:is|are} one of the chosen players".into(),
            // "If you are one of those players" (with the highest number).
            PlayerFilter::ChoseNumber(Cmp::Eq, v)
                if you
                    && matches!(v.as_ref(), Value::Custom(n) if n == crate::choices::HIGHEST_NUMBER) =>
            {
                "are one of those players".into()
            }
            PlayerFilter::ChoseNumber(c, v) => {
                let v = self.value(v);
                format!("chose a number {}", cmp_phrase(*c, &v))
            }
            PlayerFilter::VotedLikeYou(true) => "voted for a choice you voted for".into(),
            PlayerFilter::VotedLikeYou(false) => "voted for a choice you didn't vote for".into(),
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
        if let Some(s) = self.guessed(a, cmp, b) {
            return s;
        }
        // Pack tactics: "if you attacked with creatures with total power 6 or greater
        // this combat" (the attackers' power as they were declared).
        if let (Value::Custom(n), Cmp::Ge, Value::Const(k)) = (a, cmp, b) {
            if n == crate::kw::pack_tactics::ATTACKED_WITH_TOTAL_POWER {
                return format!(
                    "{{alt:you attacked with creatures with total power {k} or greater this combat|the total power of the creatures you attacked with this combat is {k} or {{alt:greater|more}}}}"
                );
            }
        }
        // "if 4 or more damage was dealt to it this turn".
        let to_self = |t: &TriggerCond| match t {
            TriggerCond::DealsDamage {
                source: Filter::Any,
                to: DamageRecipient::Object(f),
                combat_only: false,
            } => {
                matches!(f, Filter::Source)
                    || matches!(f, Filter::In(s) if matches!(s.as_ref(), Sel::TriggerLki))
            }
            _ => false,
        };
        if let (Value::EventsThisTurn(t, Tally::Amount), Cmp::Ge, Value::Const(k)) = (a, cmp, b) {
            if to_self(t) {
                let m = self.me();
                let v = self.value(a);
                let w = number_word(*k);
                return format!(
                "{{alt:{v} is {w} or {{alt:greater|more}}|{w} or more damage was dealt to {m} this turn}}"
            );
            }
        }
        if let Some(s) = self.attempt(|r| r.events_compare(a, cmp, b, false)) {
            return s;
        }
        // "if your library has no cards in it".
        if let (Value::LibrarySize(p), Cmp::Eq, Value::Const(0)) = (a, cmp, b) {
            let p = self.player(p, Case::Poss);
            return format!(
                "{{alt:there are no cards in {p} library|{p} library has no cards in it}}"
            );
        }
        // "if no mana was spent to cast it" (none of the amount, CR 601.2h).
        if let (Value::Custom(n), Cmp::Eq, Value::Const(0)) = (a, cmp, b) {
            if n == "opus:mana spent to cast that spell" {
                return "no mana was spent to cast {alt:it|that spell}".into();
            }
        }
        // "your life total is less than or equal to half your starting life total": twice
        // the one compared with the other (exact halves, no rounding).
        if let (Value::Mul(k, x), Value::StartingLife) = (a, b) {
            let rel = match cmp {
                Cmp::Le => Some("less than or equal to"),
                Cmp::Lt => Some("less than"),
                Cmp::Ge => Some("greater than or equal to"),
                Cmp::Gt => Some("greater than"),
                _ => None,
            };
            if let (Value::Const(2), Some(rel)) = (k.as_ref(), rel) {
                let xs = self.value(x);
                let bs = self.value(b);
                return format!("{xs} is {rel} half {bs}");
            }
        }
        if let Some(s) = self.this_turn_compare(a, cmp, b) {
            return s;
        }
        // "If excess damage was dealt this way" (CR 120.10).
        if matches!(a, Value::Var(vars::EXCESS))
            && matches!(
                (cmp, b),
                (Cmp::Gt, Value::Const(0)) | (Cmp::Ge, Value::Const(1))
            )
        {
            return "excess damage was dealt this way".into();
        }
        // "if you have more life than an opponent": more than the lowest life total among
        // your opponents.
        if let (
            Value::LifeTotal(PlayerRef::You),
            Cmp::Gt,
            Value::OverPlayers(AggOp::Min, PlayerFilter::Opponent, v),
        ) = (a, cmp, b)
        {
            if matches!(v.as_ref(), Value::LifeTotal(PlayerRef::Iterated)) {
                return "you have more life than an opponent".into();
            }
        }
        // "if a graveyard has twenty or more cards in it".
        if let (Value::OverPlayers(AggOp::Max, pf, v), Cmp::Ge) = (a, cmp) {
            if let Some(s) = self.over_players_at_least(pf, v, b) {
                return s;
            }
        }
        // "an opponent controls more lands than you".
        if let (Value::CountPlayers(PlayerFilter::And(v)), Cmp::Ge, Value::Const(1)) = (a, cmp, b) {
            if let [base @ (PlayerFilter::Opponent | PlayerFilter::Any), PlayerFilter::Controls(f, c, n)] =
                v.as_slice()
            {
                let who = self.player_filter_object(base);
                let what = self.count_phrase(f, *c, n);
                return format!("{who} controls {what}");
            }
        }
        // "if a player has one or fewer cards in hand".
        if let (
            Value::CountPlayers(pf @ (PlayerFilter::HandSize(..) | PlayerFilter::Life(..))),
            Cmp::Ge,
            Value::Const(1),
        ) = (a, cmp, b)
        {
            let pred = self.player_predicate(pf, "a player", false);
            return format!("a player {pred}");
        }
        // "you have two or more opponents".
        if let (Value::CountPlayers(PlayerFilter::Opponent), Value::Const(n), Cmp::Ge) = (a, b, cmp)
        {
            return format!("you have {} or more opponents", number_word(*n));
        }
        // "a player has more life than each other player": exactly one player has the most.
        if let (Value::CountPlayers(pf), Value::Const(1), Cmp::Eq) = (a, b, cmp) {
            if let Some((stat, controls)) = self.most_of(pf) {
                return if controls {
                    format!("a player controls more {stat} than each other player")
                } else {
                    format!("a player has more {stat} than each other player")
                };
            }
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
        if let Value::Custom(name) = a {
            if let Some(s) = self.attempt(|r| r.cast_history_compare(name, cmp, b)) {
                return s;
            }
        }
        if let Some(s) = self.attempt(|r| r.nth_spell_cast(a, cmp, b, None)) {
            return s;
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
        // "if a card is exiled with it".
        if let (Value::Count(f), Cmp::Ge, Value::Const(1)) = (a, cmp, b) {
            if matches!(f, Filter::And(v) if v.iter().any(|x| matches!(x, Filter::In(s) if matches!(s.as_ref(), Sel::Linked))))
            {
                return self.exists(f, false);
            }
        }
        // "as long as two or more Equipment are attached to it".
        if let (Value::Count(Filter::And(v)), Cmp::Ge) = (a, cmp) {
            if let Some(s) = v.iter().find_map(|x| match x {
                Filter::AttachedToAnyOf(s) => Some(s.as_ref().clone()),
                _ => None,
            }) {
                let rest = Filter::and(
                    v.iter()
                        .filter(|x| !matches!(x, Filter::AttachedToAnyOf(_)))
                        .cloned()
                        .collect(),
                );
                let n = self.count_phrase(&rest, cmp, b);
                let o = self.sel(&s, Case::Obj);
                let are = if matches!(b, Value::Const(1)) {
                    "is"
                } else {
                    "are"
                };
                return format!("{n} {are} attached to {{alt:it|{o}}}");
            }
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
            // Permanents are on the battlefield (CR 110.1): "there are five or more
            // Islands on the battlefield".
            if f.zone().is_none_or(|z| z == ZoneKind::Battlefield) {
                return format!("there are {n} {{opt:on the battlefield}}");
            }
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
            let nth = ordinal_word(*n as u32);
            let full = format!("this is the {nth} time this ability has resolved this turn");
            // "If it's the second time, ...": after the first, the rest is understood.
            if *n >= 2 {
                return format!("{{alt:{full}|it's the {nth} time}}");
            }
            return full;
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
                    return format!(
                        "{{alt:there is a {rest}|{subj} has a {counters} on it|{subj} has one or more {counters} on it|{subj} has {counters} on it}}"
                    );
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
            Cmp::Le if !super::nouns::is_plain_number(&b) => {
                format!("is less than or equal to {b}")
            }
            Cmp::Le => format!("is {b} or less"),
            Cmp::Gt => format!("is greater than {b}"),
            Cmp::Ge if !super::nouns::is_plain_number(&b) => {
                format!("is greater than or equal to {b}")
            }
            Cmp::Ge => format!("is {b} or {{alt:greater|more}}"),
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
            Duration::UntilEndOfTurnOrRoll(n) => {
                format!("until end of turn, or until any player rolls a {n}, whichever comes first")
            }
            Duration::UntilYourNextTurn => "until your next turn".into(),
            Duration::UntilEndOfYourNextTurn => "until the end of your next turn".into(),
            // CR 611.2b: "until ~ leaves the battlefield" is the same duration.
            Duration::WhileSourceOnBattlefield => {
                let me = self.me();
                format!(
                    "{{alt:for as long as {me} remains on the battlefield|until {me} leaves the battlefield}}"
                )
            }
            Duration::WhileYouControlSource => format!("for as long as you control {}", self.me()),
            // "until ~ is cast from exile": once it's cast, it's no longer in exile.
            Duration::WhileCondition(Condition::Not(c))
                if matches!(c.as_ref(), Condition::Custom(n)
                    if n == crate::oracle::patterns::grant_conditions::SOURCE_CAST_FROM_EXILE) =>
            {
                format!("until {} is cast from exile", self.me())
            }
            Duration::WhileCondition(c) => {
                let c = self.condition(c);
                format!("for as long as {c}")
            }
            Duration::Permanent => String::new(),
            Duration::UntilHostLeaves | Duration::WhileFaceDown => String::new(),
            Duration::WhileAffectedHasCounter(k) => format!(
                "for as long as it has {} on it",
                with_article(&counter_name(k))
            ),
            Duration::ThisTurn => "this turn".into(),
            Duration::ThroughNextUntapStep => "during its controller's next untap step".into(),
            Duration::ThroughYourNextUntapStep => "during your next untap step".into(),
            Duration::ThroughNextUntapSteps(n) => format!(
                "during its controller's next {} untap steps",
                crate::oracle::render::number_word(*n as i32)
            ),
            Duration::UntilYourNextStep(s) => {
                let s = self.step_name(*s);
                format!("until your next {s}")
            }
            Duration::UntilEndOfCombatOnYourNextTurn => {
                "until end of combat on your next turn".into()
            }
            Duration::UntilNextStep(s) => {
                let s = self.step_name(*s);
                format!("until the next {s}")
            }
            Duration::UntilItsControllersNextStep(s) => {
                let s = self.step_name(*s);
                format!("until its controller's next {s}")
            }
            Duration::UntilEndOfYourNextStep(s) => {
                let s = self.step_name(*s);
                format!("until the end of your next {s}")
            }
            Duration::UntilEndOfThisStep => "until end of step".into(),
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

/// `[Count(F in zone A, owned by you), Count(F in zone B, owned by you)]`: (F, [A, B]).
fn two_zone_count(v: &[Value]) -> Option<(Filter, Vec<ZoneKind>)> {
    let [Value::Count(Filter::And(a)), Value::Count(Filter::And(b))] = v else {
        return None;
    };
    let split = |parts: &[Filter]| -> Option<(Vec<Filter>, ZoneKind)> {
        let z = parts.iter().find_map(|p| match p {
            Filter::InZone(z) => Some(*z),
            _ => None,
        })?;
        if !parts
            .iter()
            .any(|p| matches!(p, Filter::OwnedBy(PlayerRel::You)))
        {
            return None;
        }
        let rest: Vec<Filter> = parts
            .iter()
            .filter(|p| !matches!(p, Filter::InZone(_) | Filter::OwnedBy(_)))
            .cloned()
            .collect();
        Some((rest, z))
    };
    let (ra, za) = split(a)?;
    let (rb, zb) = split(b)?;
    if format!("{ra:?}") != format!("{rb:?}") || za == zb {
        return None;
    }
    Some((Filter::and(ra), vec![za, zb]))
}

/// The cards "revealed this way" (see `value_results::this_way_sel`): those a reveal
/// instruction left in [`vars::DUG`], the cards revealed one at a time that it didn't
/// ([`vars::REVEALED`]), and the cards revealed from a hand.
pub(crate) fn is_revealed_this_way(s: &Sel) -> bool {
    let Sel::Union(v) = s else { return false };
    match v.as_slice() {
        [Sel::Var(a), Sel::Matching(b, Filter::Not(n)), Sel::Var(c)] => {
            *a == vars::DUG
                && matches!(b.as_ref(), Sel::Var(x) if *x == vars::REVEALED)
                && matches!(n.as_ref(), Filter::In(d) if matches!(d.as_ref(), Sel::Var(x) if *x == vars::DUG))
                && *c == crate::kw::reveal_from_hand::REVEALED
        }
        _ => false,
    }
}

/// "attacking", "blocking": a combat status, said of a creature.
fn combat_status(f: &Filter) -> Option<&'static str> {
    let status = |a: &Filter| match a {
        Filter::Attacking => Some("attacking"),
        Filter::Blocking => Some("blocking"),
        _ => None,
    };
    match f {
        Filter::And(v) => match v.as_slice() {
            [Filter::Type(CardType::Creature), s] | [s, Filter::Type(CardType::Creature)] => {
                status(s)
            }
            _ => None,
        },
        other => status(other),
    }
}
