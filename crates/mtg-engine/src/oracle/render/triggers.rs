//! Triggered abilities ([`TriggeredAbility`], [`TriggerCond`], CR 603).

use super::effects::lower_first;
use super::nouns::Det;
use super::*;

/// A trigger event as (subject, verb phrase) so that conditions on the same subject can
/// be joined ("When ~ enters or dies").
struct Ev {
    subj: String,
    vp: String,
}

impl Ev {
    fn new(subj: impl Into<String>, vp: impl Into<String>) -> Ev {
        Ev {
            subj: subj.into(),
            vp: vp.into(),
        }
    }
    fn text(&self) -> String {
        if self.subj.is_empty() {
            self.vp.clone()
        } else {
            format!("{} {}", self.subj, self.vp)
        }
    }
}

impl Renderer<'_> {
    pub(crate) fn triggered(&mut self, t: &TriggeredAbility) -> String {
        // Saga chapters (CR 714.2b): "I — effect".
        if let TriggerCond::Custom(name) = &t.trigger {
            if let Some(nums) = chapter_numbers(name) {
                let body = self.body(&t.body);
                let romans: Vec<String> = nums.iter().map(|n| roman(*n)).collect();
                return format!("{} — {body}", romans.join(", "));
            }
        }
        let trig = self.trigger_text(&t.trigger);
        let saved_salient = self.self_salient;
        self.self_salient = trig.contains('~');
        let mut s = trig;
        if let Some(c) = &t.intervening_if {
            let c = self.condition(c);
            s.push_str(&format!(", if {c}"));
        }
        let body = self.body(&t.body);
        self.self_salient = saved_salient;
        s = format!("{s}, {}", lower_first(&body));
        if t.once_per_turn {
            s.push_str(" This ability triggers only once each turn.");
        }
        if t.do_once_per_turn {
            s.push_str(" Do this only once each turn.");
        }
        if t.cant_be_countered {
            s.push_str(" This ability can't be countered.");
        }
        capitalize(&s)
    }

    /// "When ~ enters", "At the beginning of your upkeep", "Whenever you cast a spell".
    pub(crate) fn trigger_text(&mut self, t: &TriggerCond) -> String {
        match t {
            TriggerCond::BeginningOf { .. } => {
                let e = self.trigger_event(t, Det::A);
                format!("at {}", e.text())
            }
            TriggerCond::ThisTurn(inner) => {
                let s = self.trigger_text(inner);
                format!("{s} this turn")
            }
            TriggerCond::UntilYourNextTurn(inner) => {
                let s = self.trigger_text(inner);
                format!("until your next turn, {}", lower_first(&s))
            }
            TriggerCond::Custom(name) if self.custom_trigger_is_complete(name) => {
                self.custom_trigger(name)
            }
            other => {
                let e = self.trigger_event(other, Det::A);
                format!("whenever {}", e.text())
            }
        }
    }

    /// Event with a determiner for its object ("a creature" / "one or more creatures").
    fn trigger_event(&mut self, t: &TriggerCond, det: Det) -> Ev {
        let obj = |r: &mut Self, f: &Filter| -> String { r.noun_det(f, det.clone()) };
        match t {
            TriggerCond::EntersBattlefield(f) => Ev::new(obj(self, f), "enters"),
            TriggerCond::LeavesBattlefield(f) => Ev::new(obj(self, f), "leaves the battlefield"),
            TriggerCond::Dies(f) => Ev::new(obj(self, f), "dies"),
            TriggerCond::ZoneChange { filter, from, to } => {
                let o = obj(self, filter);
                let vp = match (from, to) {
                    (Some(ZoneKind::Battlefield), Some(ZoneKind::Graveyard)) => "dies".to_string(),
                    (_, Some(ZoneKind::Exile)) => {
                        let f = from
                            .map(|z| format!(" from {}", self.zone_from(z, filter)))
                            .unwrap_or_default();
                        format!("is put into exile{f}")
                    }
                    (Some(fz), Some(tz)) => {
                        let tz_s = self.zone_into(*tz, filter);
                        let fz_s = self.zone_from(*fz, filter);
                        format!("is put into {tz_s} from {fz_s}")
                    }
                    (None, Some(tz)) => {
                        let tz_s = self.zone_into(*tz, filter);
                        format!("is put into {tz_s} from anywhere")
                    }
                    (Some(fz), None) => {
                        let fz_s = self.zone_from(*fz, filter);
                        format!("leaves {fz_s}")
                    }
                    (None, None) => "changes zones".into(),
                };
                Ev::new(o, vp)
            }
            TriggerCond::CastSpell { who, filter } => {
                let w = self.rel_subject(*who);
                let s = self.spell_noun(filter, det.clone());
                Ev::new(w, format!("cast {s}"))
            }
            TriggerCond::NthSpellCast { who, n } => {
                let w = self.rel_subject(*who);
                let p = if w == "you" { "your" } else { "their" };
                Ev::new(w, format!("cast {p} {} spell each turn", ordinal_word(*n)))
            }
            TriggerCond::AbilityActivated {
                who,
                source,
                include_mana,
            } => {
                let w = self.rel_subject(*who);
                let s = self.noun_det(source, Det::A);
                let kind = if *include_mana {
                    "an ability"
                } else {
                    "an ability that isn't a mana ability"
                };
                let vp = if matches!(source, Filter::Source) {
                    format!("activate {kind} of ~")
                } else {
                    format!("activate {kind} of {s}")
                };
                Ev::new(w, vp)
            }
            TriggerCond::Attacks(f) => Ev::new(obj(self, f), "attacks"),
            TriggerCond::PlayerAttacks(r) => Ev::new(self.rel_subject(*r), "attack"),
            TriggerCond::AttacksUnblocked(f) => Ev::new(obj(self, f), "attacks and isn't blocked"),
            TriggerCond::Blocks(f) => Ev::new(obj(self, f), "blocks"),
            TriggerCond::BecomesBlocked(f) => Ev::new(obj(self, f), "becomes blocked"),
            TriggerCond::BlocksOrBecomesBlocked(f) => {
                Ev::new(obj(self, f), "blocks or becomes blocked")
            }
            TriggerCond::AttacksAlone(f) => Ev::new(obj(self, f), "attacks alone"),
            TriggerCond::AttacksPlayerAlone(f) => Ev::new(obj(self, f), "attacks a player alone"),
            TriggerCond::AttacksRecipient {
                attacker,
                recipient,
            } => {
                let r = self.recipient(recipient);
                Ev::new(obj(self, attacker), format!("attacks {r}"))
            }
            TriggerCond::IsAttacked(rec) => {
                let r = self.recipient(rec);
                Ev::new(r, "is attacked")
            }
            TriggerCond::PlayerAttacksWith { who, filter, min } => {
                let w = self.rel_subject(*who);
                let n = match min {
                    0 | 1 => self.noun_det(filter, Det::OneOrMore),
                    m => {
                        let n = self.noun(filter, Num::Many);
                        format!("{} or more {n}", number_word(*m as i32))
                    }
                };
                Ev::new(w, format!("attack with {n}"))
            }
            TriggerCond::PlayerAttacksPlayer { attacker, defender } => {
                let a = self.rel_subject(*attacker);
                let d = self.rel_object(*defender);
                Ev::new(a, format!("attack {d}"))
            }
            TriggerCond::BlocksCreature { blocker, attacker } => {
                let a = self.noun_det(attacker, Det::A);
                Ev::new(obj(self, blocker), format!("blocks {a}"))
            }
            TriggerCond::BlockedByCreature { attacker, blocker } => {
                let b = self.noun_det(blocker, Det::A);
                Ev::new(obj(self, attacker), format!("becomes blocked by {b}"))
            }
            TriggerCond::BlockedByN { attacker, n } => {
                let o = obj(self, attacker);
                Ev::new(
                    o,
                    format!(
                        "becomes blocked by {} or more creatures",
                        number_word(*n as i32)
                    ),
                )
            }
            TriggerCond::DealsDamage {
                source,
                to,
                combat_only,
            } => {
                let c = if *combat_only {
                    "combat damage"
                } else {
                    "damage"
                };
                let to_s = match to {
                    DamageRecipient::Any => String::new(),
                    other => format!(" to {}", self.recipient(other)),
                };
                Ev::new(obj(self, source), format!("deals {c}{to_s}"))
            }
            TriggerCond::IsDealtDamage {
                filter,
                combat_only,
            } => {
                let c = if *combat_only {
                    "combat damage"
                } else {
                    "damage"
                };
                Ev::new(obj(self, filter), format!("is dealt {c}"))
            }
            TriggerCond::DealtExcessDamage {
                filter,
                noncombat_only,
            } => {
                let c = if *noncombat_only {
                    "excess noncombat damage"
                } else {
                    "excess damage"
                };
                Ev::new(obj(self, filter), format!("is dealt {c}"))
            }
            TriggerCond::PlayerDealtDamage { who, combat_only } => {
                let c = if *combat_only {
                    "combat damage"
                } else {
                    "damage"
                };
                Ev::new(self.rel_subject(*who), format!("is dealt {c}"))
            }
            TriggerCond::BeginningOf { step, whose } => {
                let s = self.step_name(*step);
                let text = match (step, whose) {
                    (TriggerStep::BeginningOfCombat, PlayerRel::You) => {
                        "the beginning of combat on your turn".to_string()
                    }
                    (TriggerStep::BeginningOfCombat, PlayerRel::Any) => {
                        "the beginning of each combat".into()
                    }
                    (TriggerStep::BeginningOfCombat, r) => {
                        let p = self.rel_possessive(*r, Num::One);
                        format!("the beginning of combat on {p} turn")
                    }
                    (TriggerStep::EndOfCombat, PlayerRel::Any) => "end of combat".into(),
                    (TriggerStep::Turn, PlayerRel::You) => "the beginning of your turn".into(),
                    (TriggerStep::Turn, PlayerRel::Any) => "the beginning of each turn".into(),
                    (_, PlayerRel::You) => format!("the beginning of your {s}"),
                    (_, PlayerRel::Any) => format!("the beginning of each {s}"),
                    (_, PlayerRel::Opponent) => format!("the beginning of each opponent's {s}"),
                    (_, r) => {
                        let p = self.rel_possessive(*r, Num::One);
                        format!("the beginning of {p} {s}")
                    }
                };
                Ev::new("", text)
            }
            TriggerCond::Draws { who } => Ev::new(self.rel_subject(*who), "draw a card"),
            TriggerCond::Discards { who, filter } => {
                let n = if matches!(filter, Filter::Any) {
                    "a card".to_string()
                } else {
                    let n = self.noun(&filter.clone().in_zone(ZoneKind::Hand), Num::One);
                    with_article(n.trim_end_matches(" in a hand"))
                };
                Ev::new(self.rel_subject(*who), format!("discard {n}"))
            }
            TriggerCond::GainsLife { who } => Ev::new(self.rel_subject(*who), "gain life"),
            TriggerCond::LosesLife { who } => Ev::new(self.rel_subject(*who), "lose life"),
            TriggerCond::CountersPut { filter, kind } => {
                let k = match kind {
                    Some(k) => counter_name(k),
                    None => "counter".into(),
                };
                let o = self.noun_det(filter, Det::A);
                Ev::new(
                    format!("one or more {}", plural(&k)),
                    format!("are put on {o}"),
                )
            }
            TriggerCond::CountersRemoved { filter, kind } => {
                let k = match kind {
                    Some(k) => counter_name(k),
                    None => "counter".into(),
                };
                let o = self.noun_det(filter, Det::A);
                Ev::new(with_article(&k), format!("is removed from {o}"))
            }
            TriggerCond::CounterThreshold { filter, kind, n } => {
                let o = self.noun_det(filter, Det::A);
                Ev::new(
                    format!("the {} {}", ordinal_word(*n), counter_name(kind)),
                    format!("is put on {o}"),
                )
            }
            TriggerCond::BecomesTapped(f) => Ev::new(obj(self, f), "becomes tapped"),
            TriggerCond::BecomesUntapped(f) => Ev::new(obj(self, f), "becomes untapped"),
            TriggerCond::TappedForManaOfType { filter, mana } => {
                let m = mana_symbol(*mana);
                Ev::new(obj(self, filter), format!("is tapped for {m}"))
            }
            TriggerCond::BecomesTarget { filter, by } => {
                let by_s = match by {
                    PlayerRel::Any => String::new(),
                    PlayerRel::Opponent => " an opponent controls".into(),
                    PlayerRel::You => " you control".into(),
                    r => format!(" {}", self.controls_phrase(*r, Num::One)),
                };
                Ev::new(
                    obj(self, filter),
                    format!("becomes the target of a spell or ability{by_s}"),
                )
            }
            TriggerCond::Sacrificed(f) => Ev::new(obj(self, f), "is sacrificed"),
            TriggerCond::TokenCreated(f) => {
                let o = self.noun_det(f, Det::A);
                Ev::new("you", format!("create {o}"))
            }
            TriggerCond::LandPlayed { who, filter } => {
                let w = self.rel_subject(*who);
                let n = self.noun_det(filter, Det::A);
                Ev::new(w, format!("play {n}"))
            }
            TriggerCond::Cycled { who, filter } => {
                let w = self.rel_subject(*who);
                let n = if matches!(filter, Filter::Source) {
                    "~".to_string()
                } else {
                    self.noun_det(filter, Det::A)
                };
                Ev::new(w, format!("cycle {n}"))
            }
            TriggerCond::Searched(r) => {
                let w = self.rel_subject(*r);
                let p = if w == "you" { "your" } else { "their" };
                Ev::new(w, format!("search {p} library"))
            }
            TriggerCond::TurnedFaceUp(f) => Ev::new(obj(self, f), "is turned face up"),
            TriggerCond::Transforms(f) => Ev::new(obj(self, f), "transforms"),
            TriggerCond::YouSacrifice(f) => {
                let n = self.noun_det(f, det.clone());
                Ev::new("you", format!("sacrifice {n}"))
            }
            TriggerCond::State(c) => {
                let c = self.condition(c);
                Ev::new("", c)
            }
            TriggerCond::ControlChanged(f) => {
                let n = self.noun_det(f, Det::A);
                Ev::new("you", format!("gain control of {n}"))
            }
            TriggerCond::PlayerLoses => Ev::new("a player", "loses the game"),
            TriggerCond::RollDie(r) => Ev::new(self.rel_subject(*r), "roll a die"),
            TriggerCond::FlipCoin(r) => Ev::new(self.rel_subject(*r), "flip a coin"),
            TriggerCond::Mills(r) => Ev::new(self.rel_subject(*r), "mill one or more cards"),
            TriggerCond::CommitCrime(r) => Ev::new(self.rel_subject(*r), "commit a crime"),
            TriggerCond::DayNightChanges => Ev::new("", "day becomes night or night becomes day"),
            TriggerCond::Expend { who, n } => {
                Ev::new(self.rel_subject(*who), format!("expend {n}"))
            }
            TriggerCond::PhasesOut(f) => Ev::new(obj(self, f), "phases out"),
            TriggerCond::BecomesUnattached(f) => {
                Ev::new(obj(self, f), "becomes unattached from a permanent")
            }
            TriggerCond::LoseControl(f) => {
                let n = self.noun_det(f, Det::A);
                Ev::new("you", format!("lose control of {n}"))
            }
            TriggerCond::SpellCountered(f) => {
                let n = self.spell_noun(f, det.clone());
                Ev::new(n, "is countered")
            }
            TriggerCond::AbilityResolved {
                source,
                final_chapter,
            } => {
                if *final_chapter {
                    let n = self.noun_det(source, Det::A);
                    Ev::new(format!("the final chapter ability of {n}"), "resolves")
                } else {
                    let n = self.noun_det(source, Det::A);
                    Ev::new(format!("an ability of {n}"), "resolves")
                }
            }
            TriggerCond::AbilityTriggered { cause, source } => {
                let c = self.trigger_event(cause, Det::A);
                let s = self.noun_det(source, Det::A);
                Ev::new(
                    format!("{} causes a triggered ability of {s}", c.text()),
                    "to trigger",
                )
            }
            TriggerCond::AnyOf(v) => {
                let evs: Vec<Ev> = v
                    .iter()
                    .map(|x| self.trigger_event(x, det.clone()))
                    .collect();
                if evs.iter().all(|e| e.subj == evs[0].subj) && !evs[0].subj.is_empty() {
                    let vps: Vec<String> = evs.iter().map(|e| e.vp.clone()).collect();
                    Ev::new(evs[0].subj.clone(), join_list(&vps, "or"))
                } else {
                    let ts: Vec<String> = evs.iter().map(|e| e.text()).collect();
                    Ev::new("", join_list(&ts, "or"))
                }
            }
            TriggerCond::Where { trigger, cond } => {
                let e = self.trigger_event(trigger, det);
                let c = self.condition(cond);
                Ev::new(e.subj, format!("{}, if {c}", e.vp))
            }
            TriggerCond::FirstTimeEachTurn(inner) => {
                let e = self.trigger_event(inner, det);
                Ev::new(e.subj, format!("{} for the first time each turn", e.vp))
            }
            TriggerCond::Batched { trigger, per } => {
                let _ = per;
                let d = match det {
                    Det::A => Det::OneOrMore,
                    other => other,
                };
                let e = self.trigger_event(trigger, d);
                Ev::new(e.subj, plural_verb(&e.vp))
            }
            TriggerCond::SpellCopied { who, filter } => {
                let w = self.rel_subject(*who);
                let s = self.spell_noun(filter, det.clone());
                Ev::new(w, format!("copy {s}"))
            }
            TriggerCond::PlayerAction { name, who } => {
                let w = self.rel_subject(*who);
                Ev::new(w, name.to_string())
            }
            TriggerCond::PlayerAttacked {
                attacker,
                defender,
                with,
                min,
            } => {
                let a = self.rel_subject(*attacker);
                let d = self.player_filter_object(defender);
                let w = match (with, min) {
                    (Filter::Any, _) | (_, 0) => String::new(),
                    (f, 1) => format!(" with {}", self.noun_det(f, Det::OneOrMore)),
                    (f, m) => {
                        let n = self.noun(f, Num::Many);
                        format!(" with {} or more {n}", number_word(*m as i32))
                    }
                };
                Ev::new(a, format!("attack {d}{w}"))
            }
            TriggerCond::AttachChanged {
                attached,
                obj: o,
                other,
            } => {
                let os = self.noun_det(o, det.clone());
                let other_s = self.noun_det(other, Det::A);
                let vp = if *attached {
                    format!("becomes attached to {other_s}")
                } else {
                    format!("becomes unattached from {other_s}")
                };
                Ev::new(os, vp)
            }
            TriggerCond::Phases { phased_in, filter } => {
                let vp = if *phased_in {
                    "phases in"
                } else {
                    "phases out"
                };
                Ev::new(obj(self, filter), vp)
            }
            TriggerCond::ThisTurn(inner) => {
                let e = self.trigger_event(inner, det);
                Ev::new(e.subj, format!("{} this turn", e.vp))
            }
            TriggerCond::UntilYourNextTurn(inner) => self.trigger_event(inner, det),
            TriggerCond::Noncombat(inner) => {
                let e = self.trigger_event(inner, det);
                Ev::new(e.subj, e.vp.replacen("damage", "noncombat damage", 1))
            }
            TriggerCond::TappedForMana { who, filter } => {
                if matches!(who, PlayerRel::Any) {
                    Ev::new(obj(self, filter), "is tapped for mana")
                } else {
                    let w = self.rel_subject(*who);
                    let n = self.noun_det(filter, Det::A);
                    Ev::new(w, format!("tap {n} for mana"))
                }
            }
            TriggerCond::Custom(name) => {
                let s = self.custom_trigger(name);
                Ev::new("", s)
            }
        }
    }

    /// "a spell", "an instant or sorcery spell", "one or more creature spells".
    pub(crate) fn spell_noun(&mut self, f: &Filter, det: Det) -> String {
        let has_spell = filter_mentions_spell(f);
        let f2 = if has_spell {
            f.clone()
        } else {
            Filter::and(vec![f.clone(), Filter::Spell])
        };
        self.noun_det(&f2, det)
    }

    fn recipient(&mut self, r: &DamageRecipient) -> String {
        match r {
            DamageRecipient::Any => "any target".into(),
            DamageRecipient::Player(PlayerRel::Any) => "a player".into(),
            DamageRecipient::Player(PlayerRel::Opponent) => "an opponent".into(),
            DamageRecipient::Player(r) => self.rel_object(*r),
            DamageRecipient::Object(f) => self.noun_det(f, Det::A),
            DamageRecipient::PlayerOrPlaneswalker(r) => match r {
                PlayerRel::You => "you or a planeswalker you control".into(),
                PlayerRel::Opponent => "an opponent or a planeswalker an opponent controls".into(),
                PlayerRel::Any => "a player or planeswalker".into(),
                other => {
                    let p = self.rel_object(*other);
                    format!("{p} or a planeswalker {p} controls")
                }
            },
        }
    }

    fn zone_owner(f: &Filter) -> Option<PlayerRel> {
        match f {
            Filter::OwnedBy(r) => Some(*r),
            Filter::And(v) => v.iter().find_map(Self::zone_owner),
            _ => None,
        }
    }

    /// "a graveyard", "your graveyard", "an opponent's graveyard".
    fn zone_into(&mut self, z: ZoneKind, f: &Filter) -> String {
        let zw = zone_word(z);
        match Self::zone_owner(f) {
            Some(r) => {
                let p = self.rel_possessive(r, Num::One);
                format!("{p} {zw}")
            }
            None => with_article(zw),
        }
    }

    fn zone_from(&mut self, z: ZoneKind, f: &Filter) -> String {
        match z {
            ZoneKind::Battlefield => "the battlefield".into(),
            ZoneKind::Exile => "exile".into(),
            ZoneKind::Stack => "the stack".into(),
            other => self.zone_into(other, f),
        }
    }
}

fn filter_mentions_spell(f: &Filter) -> bool {
    match f {
        Filter::Spell | Filter::SpellOnStack => true,
        Filter::And(v) | Filter::Or(v) => v.iter().any(filter_mentions_spell),
        _ => false,
    }
}

/// Saga chapter numbers in a custom trigger name ("chapter:2").
pub(crate) fn chapter_numbers(name: &str) -> Option<Vec<u32>> {
    let rest = name.strip_prefix("chapter:")?;
    rest.split(',').map(|n| n.trim().parse().ok()).collect()
}

/// Plural verb agreement for "one or more creatures [enter]": the comparison ignores
/// agreement; this only keeps the rendering readable.
fn plural_verb(vp: &str) -> String {
    let (v, rest) = match vp.split_once(' ') {
        Some((v, r)) => (v, format!(" {r}")),
        None => (vp, String::new()),
    };
    let base = match v {
        "is" => "are".to_string(),
        "dies" => "die".into(),
        "becomes" => "become".into(),
        "deals" => "deal".into(),
        "attacks" => "attack".into(),
        "blocks" => "block".into(),
        "enters" => "enter".into(),
        "leaves" => "leave".into(),
        other => other.to_string(),
    };
    format!("{base}{rest}")
}
