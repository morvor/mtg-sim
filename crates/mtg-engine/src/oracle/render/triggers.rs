//! Triggered abilities ([`TriggeredAbility`], [`TriggerCond`], CR 603).

use super::effects::lower_first;
use super::nouns::Det;
use super::players::Case;
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
    /// The players of a trigger event that triggers once for a whole batch of events
    /// ("one or more opponents"), or of one event ("an opponent").
    fn batch_players(&mut self, who: PlayerRel) -> String {
        match who {
            PlayerRel::Any if self.batch_once => "one or more players".into(),
            PlayerRel::Opponent if self.batch_once => "one or more opponents".into(),
            other => self.rel_subject(other),
        }
    }

    pub(crate) fn triggered(&mut self, t: &TriggeredAbility) -> String {
        // CR 702.178a: "Max speed — [triggered ability]": it triggers only while you have
        // max speed (the object has the ability only then).
        if let TriggerCond::Where {
            trigger,
            cond: Condition::MaxSpeed,
        } = &t.trigger
        {
            let mut inner = t.clone();
            inner.trigger = trigger.as_ref().clone();
            let (i, c) = self.two_ways(|r| r.triggered(&inner), |r| r.triggered_plain(t));
            return format!("{{alt:Max speed — {i}|{c}}}");
        }
        let pushed = t
            .intervening_if
            .as_ref()
            .and_then(super::compared_values)
            .map(|p| self.compared.push(p))
            .is_some();
        let s = self.triggered_plain(t);
        if pushed {
            self.compared.pop();
        }
        s
    }

    fn triggered_plain(&mut self, t: &TriggeredAbility) -> String {
        // Saga chapters (CR 714.2b): "I — effect".
        if let TriggerCond::Custom(name) = &t.trigger {
            if let Some(nums) = chapter_numbers(name) {
                let body = self.body(&t.body);
                let romans: Vec<String> = nums.iter().map(|n| roman(*n)).collect();
                return format!("{} — {body}", romans.join(", "));
            }
        }
        // A Case's "To solve — [condition]" (CR 719.3a): at your end step, if the
        // condition is met and it isn't solved, it becomes solved.
        if matches!(&t.body.effect, Effect::Custom(n) if n == "case: becomes solved") {
            let cond = match &t.intervening_if {
                Some(Condition::And(v)) => {
                    let rest: Vec<Condition> = v
                        .iter()
                        .filter(|c| !matches!(c, Condition::Not(x) if is_solved(x)))
                        .cloned()
                        .collect();
                    if rest.len() == 1 {
                        Some(rest[0].clone())
                    } else {
                        Some(Condition::And(rest))
                    }
                }
                other => other.clone(),
            };
            let c = match &cond {
                Some(c) => self.condition(c),
                None => self.gap("to solve without condition"),
            };
            return format!("To solve — {}", capitalize(&c));
        }
        // "Solved — [ability]" (CR 719.3b).
        if t.intervening_if.as_ref().is_some_and(is_solved) {
            let mut t2 = t.clone();
            t2.intervening_if = None;
            let s = self.triggered(&t2);
            return format!("Solved — {s}");
        }
        let saved_zone = self.zone;
        self.zone = t.zone;
        let saved_attached_left = std::mem::replace(
            &mut self.attached_left,
            matches!(
                &t.trigger,
                TriggerCond::Dies(Filter::AttachedToSource)
                    | TriggerCond::LeavesBattlefield(Filter::AttachedToSource)
                    | TriggerCond::ZoneChange {
                        filter: Filter::AttachedToSource,
                        ..
                    }
            ),
        );
        self.self_salient = false;
        // "this creature deals 2 damage to that spell's controller".
        fn targeted(t: &TriggerCond) -> bool {
            match t {
                TriggerCond::BecomesTarget { .. } => true,
                TriggerCond::Where { trigger, .. } | TriggerCond::Batched { trigger, .. } => {
                    targeted(trigger)
                }
                _ => false,
            }
        }
        if targeted(&t.trigger) {
            self.trigger_player = Some("that spell's controller");
        }
        let trig = self.trigger_text(&t.trigger);
        self.trigger_names_opponent =
            trig.contains("an opponent controls") || trig.contains("your opponents control");
        let saved_salient = self.self_salient;
        // "You may exert ~ as it attacks. When you do, it gets ...": the object was just
        // named.
        // When the trigger condition also names another object ("Whenever ~ becomes
        // blocked by a creature", "Whenever ~ or another creature enters"), a later "it"
        // could be that object, so the object itself isn't "it" there.
        let saved_other = self.other_salient;
        let saved_named = std::mem::replace(&mut self.self_named_in_clause, false);
        self.other_salient = names_another_object(&trig);
        self.self_salient = (trig.contains('~') && !self.other_salient)
            || matches!(&t.trigger, TriggerCond::Custom(n) if n == crate::kw::exert::EXERTED);
        // The event is about the object itself ("Whenever ~ attacks"): the triggering
        // object is the object itself.
        let saved_is_self = std::mem::replace(&mut self.trigger_is_self, self.self_salient);
        let saved_self_other = std::mem::replace(
            &mut self.trigger_self_and_other,
            trig.contains('~') && self.other_salient,
        );
        let mut s = trig;
        if let Some(c) = &t.intervening_if {
            let c = self.condition(c);
            s.push_str(&format!(", if {c}"));
        }
        let body = self.body(&t.body);
        self.self_salient = saved_salient;
        self.other_salient = saved_other;
        self.self_named_in_clause = saved_named;
        self.trigger_is_self = saved_is_self;
        self.trigger_self_and_other = saved_self_other;
        self.zone = saved_zone;
        self.attached_left = saved_attached_left;
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
            // "At the beginning of the upkeep of enchanted creature's controller", "At the
            // beginning of enchanted player's upkeep": each upkeep whose active player
            // controls the object (or is the player) this is attached to.
            TriggerCond::Where { trigger, cond }
                if matches!(
                    cond,
                    Condition::PlayerMatches(PlayerRef::ControllerOf(s), PlayerFilter::Active)
                        if matches!(s.as_ref(), Sel::AttachedTo)
                ) && matches!(
                    trigger.as_ref(),
                    TriggerCond::BeginningOf {
                        whose: PlayerRel::Any,
                        ..
                    }
                ) && self.info.enchant.is_some() =>
            {
                let TriggerCond::BeginningOf { step, .. } = trigger.as_ref() else {
                    return String::new();
                };
                let step = self.step_name(*step);
                let e = self.info.enchant.clone().unwrap_or_default();
                if e == "player" {
                    format!("at the beginning of enchanted player's {step}")
                } else {
                    format!("at the beginning of the {step} of enchanted {e}'s controller")
                }
            }
            TriggerCond::Where { trigger, .. } | TriggerCond::FirstTimeEachTurn(trigger)
                if matches!(trigger.as_ref(), TriggerCond::BeginningOf { .. }) =>
            {
                let e = self.trigger_event(t, Det::A);
                format!("at {}", e.text())
            }
            TriggerCond::BeginningOf { .. } => {
                let e = self.trigger_event(t, Det::A);
                format!("at {}", e.text())
            }
            // "At the beginning of your upkeep or whenever ...": the first event's word.
            TriggerCond::AnyOf(v)
                if v.first()
                    .is_some_and(|x| matches!(x, TriggerCond::BeginningOf { .. })) =>
            {
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

    /// A trigger event as (subject, verb phrase in the present tense).
    pub(crate) fn trigger_event_parts(&mut self, t: &TriggerCond) -> (String, String) {
        let e = self.trigger_event(t, Det::A);
        (e.subj, e.vp)
    }

    /// `trigger_event_parts` without side effects on target introduction or gaps.
    fn trigger_event_peek(&mut self, t: &TriggerCond) -> (String, String) {
        let saved_i = self.introduced.clone();
        let saved_g = self.gaps.len();
        let r = self.trigger_event_parts(t);
        self.introduced = saved_i;
        self.gaps.truncate(saved_g);
        r
    }

    /// Event with a determiner for its object ("a creature" / "one or more creatures").
    fn trigger_event(&mut self, t: &TriggerCond, det: Det) -> Ev {
        let obj = |r: &mut Self, f: &Filter| -> String { r.noun_det(f, det.clone()) };
        match t {
            // "When ~ enters untapped".
            TriggerCond::EntersBattlefield(Filter::And(v))
                if v.len() == 2
                    && matches!(v[0], Filter::Source)
                    && matches!(v[1], Filter::Untapped | Filter::Tapped) =>
            {
                let how = if matches!(v[1], Filter::Untapped) {
                    "untapped"
                } else {
                    "tapped"
                };
                Ev::new(self.me(), format!("enters {how}"))
            }
            TriggerCond::EntersBattlefield(f) => Ev::new(obj(self, f), "enters"),
            TriggerCond::LeavesBattlefield(f) => Ev::new(obj(self, f), "leaves the battlefield"),
            // "Whenever a creature is put into an opponent's graveyard from the
            // battlefield": a permanent goes to its owner's graveyard (CR 400.3), so it's one
            // that player owns dying.
            TriggerCond::Dies(Filter::And(v))
                if v.iter().any(|x| {
                    matches!(x, Filter::OwnedBy(PlayerRel::Opponent | PlayerRel::You))
                }) =>
            {
                let o = obj(self, &Filter::And(v.clone()));
                let whose = if v
                    .iter()
                    .any(|x| matches!(x, Filter::OwnedBy(PlayerRel::You)))
                {
                    "your"
                } else {
                    "an opponent's"
                };
                let rest: Vec<Filter> = v
                    .iter()
                    .filter(|x| !matches!(x, Filter::OwnedBy(_)))
                    .cloned()
                    .collect();
                let n = obj(self, &Filter::and(rest));
                Ev::new(
                    "",
                    format!(
                        "{{alt:{o} dies|{n} is put into {whose} graveyard from the battlefield}}"
                    ),
                )
            }
            TriggerCond::Dies(f) => Ev::new(obj(self, f), "dies"),
            // "Whenever a spell or ability an opponent controls destroys a land you
            // control" (CR 701.8).
            TriggerCond::DestroyedBy { filter, by } => {
                let s = self.spell_or_ability_of(*by);
                let o = obj(self, filter);
                Ev::new(s, format!("destroys {o}"))
            }
            // "Whenever a spell you control is countered by a spell or ability an opponent
            // controls" (CR 701.6).
            TriggerCond::CounteredBy { filter, by } => {
                let s = self.spell_or_ability_of(*by);
                Ev::new(obj(self, filter), format!("is countered by {s}"))
            }
            // "Whenever you get one or more {E}" (energy counters, CR 107.14), from anyone.
            TriggerCond::CountersPutBy {
                who: PlayerRel::Any,
                on_objects: None,
                on_players: Some(PlayerFilter::You),
                kind: Some(k),
                each: false,
            } if k == "energy" => Ev::new("you", "get one or more {E}"),
            // "Whenever you put one or more +1/+1 counters on a creature you control".
            TriggerCond::CountersPutBy {
                who,
                on_objects,
                on_players,
                kind,
                each,
            } => {
                let w = self.rel_subject(*who);
                let c = match kind {
                    Some(k) => counter_name(k),
                    None => "counter".into(),
                };
                let counters = if *each {
                    with_article(&c)
                } else {
                    format!("one or more {}", plural(&c))
                };
                let on = match (on_objects, on_players) {
                    (Some(f), None) => self.noun_det(f, Det::A),
                    (None, Some(pf)) => {
                        let p = self.player_filter_noun(pf, Num::One);
                        if p.contains("player") || p.contains("opponent") {
                            with_article(&p)
                        } else {
                            p
                        }
                    }
                    (Some(f), Some(pf)) => {
                        let o = self.noun_det(f, Det::A);
                        let p = self.player_filter_noun(pf, Num::One);
                        format!("{o} or {p}")
                    }
                    (None, None) => self.gap("counters put on nothing"),
                };
                let verb = if w == "you" { "put" } else { "puts" };
                Ev::new(w, format!("{verb} {counters} on {on}"))
            }
            TriggerCond::ZoneChange { filter, from, to } => {
                // "one or more cards leave your graveyard": the owner is in the zone.
                let shown = match filter {
                    Filter::And(v)
                        if from.is_some_and(|z| z != ZoneKind::Battlefield)
                            || to.is_some_and(|z| z != ZoneKind::Battlefield) =>
                    {
                        Filter::and(
                            v.iter()
                                .filter(|x| !matches!(x, Filter::OwnedBy(_)))
                                .cloned()
                                .collect(),
                        )
                    }
                    other => other.clone(),
                };
                let o = obj(self, &shown);
                let vp = match (from, to) {
                    (Some(ZoneKind::Battlefield), Some(ZoneKind::Graveyard)) => "dies".to_string(),
                    (_, Some(ZoneKind::Exile)) => {
                        let f = from
                            .map(|z| format!(" from {}", self.zone_from(z, filter)))
                            .unwrap_or_default();
                        format!("is put into exile{f}")
                    }
                    // "enters from your graveyard".
                    (Some(fz), Some(ZoneKind::Battlefield)) => {
                        // "When ~ enters from a graveyard": it comes from its owner's
                        // graveyard (CR 400.3), and whoever controls it as it enters
                        // controls the ability; "from your graveyard" says the card is
                        // yours (`Filter::OwnedBy`).
                        let fz_s = if matches!(filter, Filter::Source)
                            && matches!(
                                fz,
                                ZoneKind::Graveyard | ZoneKind::Hand | ZoneKind::Library
                            ) {
                            with_article(zone_word(*fz))
                        } else {
                            self.zone_from(*fz, filter)
                        };
                        format!("enters from {fz_s}")
                    }
                    (Some(fz), Some(tz)) => {
                        let tz_s = self.zone_into(*tz, filter);
                        let fz_s = self.zone_from(*fz, filter);
                        format!("is put into {tz_s} from {fz_s}")
                    }
                    // "Whenever ~ is put into a graveyard from anywhere": its owner's (CR
                    // 400.3).
                    (None, Some(ZoneKind::Graveyard)) if matches!(filter, Filter::Source) => {
                        let tz_s = self.zone_into(ZoneKind::Graveyard, filter);
                        format!("is put into {{alt:{tz_s}|a graveyard}} from anywhere")
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
                // Once for the whole attack: "attacks one or more of your opponents".
                let d = if self.batch_once && matches!(defender, PlayerRel::Opponent) {
                    "one or more of your opponents".to_string()
                } else {
                    self.rel_object(*defender)
                };
                Ev::new(a, format!("attack {d}"))
            }
            // Only creatures attack and block (CR 506.1): a filter with no type is a
            // creature.
            TriggerCond::BlocksCreature { blocker, attacker } => {
                let saved = self.default_head;
                self.default_head = Some("creature");
                let a = self.noun_det(attacker, Det::A);
                self.default_head = saved;
                Ev::new(obj(self, blocker), format!("blocks {a}"))
            }
            TriggerCond::BlockedByCreature { attacker, blocker } => {
                let saved = self.default_head;
                self.default_head = Some("creature");
                let b = self.noun_det(blocker, Det::A);
                self.default_head = saved;
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
                let saved = self.default_head;
                self.default_head = Some("source");
                let src = obj(self, source);
                self.default_head = saved;
                let to_s = match to {
                    DamageRecipient::Any => String::new(),
                    other => format!(" to {}", self.recipient(other)),
                };
                Ev::new(src, format!("deals {c}{to_s}"))
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
                let w = self.rel_subject(*who);
                if w == "you" {
                    // "Whenever you're dealt damage".
                    return Ev::new("", format!("you're dealt {c}"));
                }
                Ev::new(w, format!("is dealt {c}"))
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
                    (TriggerStep::BeginningOfCombat, PlayerRel::Opponent) => {
                        "the beginning of combat on each opponent's turn".into()
                    }
                    (TriggerStep::BeginningOfCombat, r) => {
                        let p = self.rel_possessive(*r, Num::One);
                        format!("the beginning of combat on {p} turn")
                    }
                    (TriggerStep::EndOfCombat, PlayerRel::Any) => "end of combat".into(),
                    (TriggerStep::Turn, PlayerRel::You) => "the beginning of your turn".into(),
                    (TriggerStep::Turn, PlayerRel::Any) => "the beginning of each turn".into(),
                    (_, PlayerRel::You) => format!("the beginning of your {s}"),
                    // "At the beginning of each player's first main phase".
                    (_, PlayerRel::Any) if s.contains("main phase") => {
                        format!("the beginning of each player's {s}")
                    }
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
                let many = matches!(det, Det::OneOrMore);
                let n = if matches!(filter, Filter::Any | Filter::Card) {
                    if many {
                        "one or more cards".to_string()
                    } else {
                        "a card".to_string()
                    }
                } else {
                    let n = self.noun(
                        &filter.clone().in_zone(ZoneKind::Hand),
                        if many { Num::Many } else { Num::One },
                    );
                    let n = n
                        .trim_end_matches(" in a hand")
                        .trim_end_matches(" in hands");
                    if many {
                        format!("one or more {n}")
                    } else {
                        with_article(n)
                    }
                };
                // "Whenever one or more players discard one or more cards" (once for all).
                let w = self.batch_players(*who);
                Ev::new(w, format!("discard {n}"))
            }
            // "Whenever one or more opponents lose life" (once for all).
            TriggerCond::GainsLife { who } => {
                let w = self.batch_players(*who);
                Ev::new(w, "gain life")
            }
            TriggerCond::LosesLife { who } => {
                let w = self.batch_players(*who);
                Ev::new(w, "lose life")
            }
            TriggerCond::CountersPut { filter, kind, each } => {
                let k = match kind {
                    Some(k) => counter_name(k),
                    None => "counter".into(),
                };
                // "... are put on one or more Humans you control" (once for all).
                let o = if self.batch_once && !matches!(filter, Filter::Source) {
                    self.noun_det(filter, Det::OneOrMore)
                } else {
                    self.noun_det(filter, Det::A)
                };
                // "Whenever a +1/+1 counter is put on ~" triggers for each counter.
                if *each {
                    Ev::new(with_article(&k), format!("is put on {o}"))
                } else {
                    Ev::new(
                        format!("one or more {}", plural(&k)),
                        format!("are put on {o}"),
                    )
                }
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
                // A land is played (CR 305.1): the filter only narrows which land.
                let saved = self.default_head;
                self.default_head = Some("land");
                let n = if matches!(filter, Filter::Any) {
                    "a land".to_string()
                } else {
                    self.noun_det(filter, Det::A)
                };
                self.default_head = saved;
                Ev::new(w, format!("play {n}"))
            }
            TriggerCond::Cycled { who, filter } => {
                let w = self.rel_subject(*who);
                let n = if matches!(filter, Filter::Source) {
                    self.me()
                } else if matches!(filter, Filter::Any) {
                    "a card".into()
                } else if matches!(filter, Filter::Other) {
                    "another card".into()
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
            // "When ~ transforms into [this face]": an ability of a face triggers only
            // when the permanent has that face up after it transforms (it has the ability
            // only then; CR 701.27e), so cards say either.
            TriggerCond::Transforms(Filter::Source) => {
                Ev::new(obj(self, &Filter::Source), "transforms {opt:into ~}")
            }
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
            // "Whenever another creature you control leaves the battlefield without dying":
            // to any zone but a graveyard (CR 700.4).
            TriggerCond::AnyOf(v)
                if v.len() == 4
                    && v.iter().all(|x| {
                        matches!(
                            x,
                            TriggerCond::ZoneChange {
                                from: Some(ZoneKind::Battlefield),
                                to: Some(_),
                                ..
                            }
                        )
                    })
                    && {
                        let mut zones: Vec<String> = v
                            .iter()
                            .filter_map(|x| match x {
                                TriggerCond::ZoneChange { to: Some(z), .. } => {
                                    Some(format!("{z:?}"))
                                }
                                _ => None,
                            })
                            .collect();
                        zones.sort();
                        zones == ["Command", "Exile", "Hand", "Library"]
                    }
                    && v.windows(2).all(|w| match (&w[0], &w[1]) {
                        (
                            TriggerCond::ZoneChange { filter: a, .. },
                            TriggerCond::ZoneChange { filter: b, .. },
                        ) => format!("{a:?}") == format!("{b:?}"),
                        _ => false,
                    }) =>
            {
                let TriggerCond::ZoneChange { filter, .. } = &v[0] else {
                    return Ev::new("", self.gap("leaves without dying"));
                };
                let o = self.noun_det(filter, det.clone());
                Ev::new(o, "leaves the battlefield without dying")
            }
            TriggerCond::AnyOf(v) => {
                let salient = self.self_salient;
                let evs: Vec<Ev> = v
                    .iter()
                    .map(|x| {
                        self.self_salient = salient;
                        self.trigger_event(x, det.clone())
                    })
                    .collect();
                if evs.iter().all(|e| e.subj == evs[0].subj) && !evs[0].subj.is_empty() {
                    let vps: Vec<String> = evs.iter().map(|e| e.vp.clone()).collect();
                    // "cast or copy an instant or sorcery spell": one object for both verbs.
                    let split: Vec<Option<(&str, &str)>> =
                        vps.iter().map(|v| v.split_once(' ')).collect();
                    if split.iter().all(|x| x.is_some()) {
                        let objs: Vec<&str> =
                            split.iter().map(|x| x.unwrap_or(("", "")).1).collect();
                        if objs.iter().all(|o| *o == objs[0]) {
                            let verbs: Vec<String> = split
                                .iter()
                                .map(|x| x.unwrap_or(("", "")).0.to_string())
                                .collect();
                            return Ev::new(
                                evs[0].subj.clone(),
                                format!("{} {}", join_list(&verbs, "or"), objs[0]),
                            );
                        }
                    }
                    // "blocks or becomes blocked by a creature": the object said once.
                    if let [a, b] = vps.as_slice() {
                        if let Some((verb, obj)) = a.split_once(' ') {
                            if !verb.is_empty() && b.ends_with(&format!(" {obj}")) {
                                return Ev::new(
                                    evs[0].subj.clone(),
                                    format!("{verb} {{opt:{obj}}} or {b}"),
                                );
                            }
                        }
                    }
                    // "deals combat damage to a player or battle": the shared words said
                    // once.
                    if let [a, b] = vps.as_slice() {
                        let wa: Vec<&str> = a.split(' ').collect();
                        let wb: Vec<&str> = b.split(' ').collect();
                        let k = wa.iter().zip(&wb).take_while(|(x, y)| x == y).count();
                        if k >= 2 && k < wa.len() && k < wb.len() && !a.contains('{') {
                            let merged = format!(
                                "{} {} or {}",
                                wa[..k].join(" "),
                                wa[k..].join(" "),
                                wb[k..].join(" ")
                            );
                            return Ev::new(
                                evs[0].subj.clone(),
                                format!("{{alt:{} or {b}|{merged}}}", a),
                            );
                        }
                    }
                    Ev::new(evs[0].subj.clone(), join_list(&vps, "or"))
                } else {
                    // "Whenever A and whenever B" (each condition keeps its trigger word):
                    // separate clauses, so ~ named in one is "it" in the next ("When ~
                    // enters and when you sacrifice it").
                    self.self_salient = salient;
                    let evs: Vec<Ev> = v
                        .iter()
                        .map(|x| self.trigger_event(x, det.clone()))
                        .collect();
                    // Or one trigger word for all: "Whenever you cast a blue spell or an
                    // Island you control enters" (CR 603.2: either event triggers it).
                    let n = evs.len();
                    let mut s = String::new();
                    for (i, e) in evs.iter().enumerate() {
                        let t = e.text();
                        let begins = v
                            .get(i)
                            .is_some_and(|x| matches!(x, TriggerCond::BeginningOf { .. }));
                        if i > 0 {
                            let word = if begins { "at" } else { "whenever" };
                            // "At the beginning of your upkeep or whenever you cast a
                            // black spell".
                            let or = if i + 1 == n { "or" } else { "" };
                            s.push_str(&format!(" {{alt:{or}|and {word}|or {word}}} "));
                        }
                        s.push_str(&t);
                    }
                    Ev::new("", s)
                }
            }
            // "Whenever you draw your second card each turn".
            TriggerCond::Where { trigger, cond }
                if matches!(trigger.as_ref(), TriggerCond::Draws { .. })
                    && matches!(
                        cond,
                        Condition::Compare(Value::EventAmount, Cmp::Eq, Value::Const(_))
                    ) =>
            {
                let (TriggerCond::Draws { who }, Condition::Compare(_, _, Value::Const(n))) =
                    (trigger.as_ref(), cond)
                else {
                    return Ev::new("", self.gap("draws-nth"));
                };
                let w = self.rel_subject(*who);
                let p = if w == "you" { "your" } else { "their" };
                Ev::new(
                    w,
                    format!("draw {p} {} card each turn", ordinal_word(*n as u32)),
                )
            }
            // "Whenever ~ becomes crewed", "When ~ becomes plotted": an event about the
            // object itself (`kw/crew.rs`, `kw/plot.rs`).
            TriggerCond::Where {
                trigger,
                cond: Condition::SelMatches(Sel::TriggerObject, Filter::Source),
            } if matches!(trigger.as_ref(), TriggerCond::PlayerAction { name, who: PlayerRel::Any }
                if name == crate::kw::crew::CREWED || name == crate::kw::plot::BECAME_PLOTTED) =>
            {
                let TriggerCond::PlayerAction { name, .. } = trigger.as_ref() else {
                    return Ev::new("", self.gap("an event about the object"));
                };
                let vp = if name == crate::kw::crew::CREWED {
                    "becomes crewed"
                } else {
                    "becomes plotted"
                };
                Ev::new(self.me(), vp)
            }
            // "Whenever you win a coin flip" / "lose a coin flip": the flip's result, 1 for
            // a win (CR 705.2).
            TriggerCond::Where {
                trigger,
                cond: Condition::Compare(Value::EventAmount, Cmp::Eq, Value::Const(k @ (0 | 1))),
            } if matches!(trigger.as_ref(), TriggerCond::FlipCoin(_)) => {
                let TriggerCond::FlipCoin(r) = trigger.as_ref() else {
                    return Ev::new("", self.gap("coin flip"));
                };
                let w = self.rel_subject(*r);
                let vp = if *k == 1 {
                    "win a coin flip"
                } else {
                    "lose a coin flip"
                };
                Ev::new(w, vp)
            }
            // "Whenever ~ is dealt 3 or more damage", "Whenever ~ deals 4 or more damage":
            // the event's amount (CR 120.1).
            TriggerCond::Where {
                trigger,
                cond: Condition::Compare(Value::EventAmount, Cmp::Ge, Value::Const(n)),
            } if {
                let inner = match trigger.as_ref() {
                    TriggerCond::Batched { trigger, .. } => trigger.as_ref(),
                    t => t,
                };
                matches!(
                    inner,
                    TriggerCond::IsDealtDamage { .. } | TriggerCond::DealsDamage { .. }
                )
            } =>
            {
                let e = self.trigger_event(trigger, det);
                let k = number_word(*n);
                let vp = if e.vp.contains("combat damage") {
                    e.vp.replacen("combat damage", &format!("{k} or more combat damage"), 1)
                } else {
                    e.vp.replacen("damage", &format!("{k} or more damage"), 1)
                };
                Ev::new(e.subj, vp)
            }
            // "Whenever an opponent sacrifices a nontoken permanent": the player who
            // sacrificed it is the trigger's player.
            TriggerCond::Where {
                trigger,
                cond: Condition::PlayerMatches(PlayerRef::TriggerPlayer, pf),
            } if matches!(trigger.as_ref(), TriggerCond::Sacrificed(_))
                && matches!(pf, PlayerFilter::Any | PlayerFilter::Opponent) =>
            {
                let TriggerCond::Sacrificed(f) = trigger.as_ref() else {
                    return Ev::new("", self.gap("sacrifices"));
                };
                let w = if matches!(pf, PlayerFilter::Opponent) {
                    "an opponent"
                } else {
                    "a player"
                };
                let o = self.noun_det(&super::effects::strip_controller(f), det);
                Ev::new(w, format!("sacrifice {o}"))
            }
            // "At the beginning of your second main phase".
            TriggerCond::Where { trigger, cond } if matches!(cond, Condition::Custom(n) if n.starts_with("main_phase:")) =>
            {
                let (TriggerCond::BeginningOf { whose, .. }, Condition::Custom(n)) =
                    (trigger.as_ref(), cond)
                else {
                    return Ev::new("", self.gap("main phase trigger"));
                };
                let k: u32 = n["main_phase:".len()..].parse().unwrap_or(1);
                // "At the beginning of each player's first main phase".
                let p = if matches!(whose, PlayerRel::Any) {
                    "each player's".to_string()
                } else {
                    self.rel_possessive(*whose, Num::One)
                };
                Ev::new(
                    "",
                    format!("the beginning of {p} {} main phase", ordinal_word(k)),
                )
            }
            // "Whenever one or more Demons you control attack a player": the attacked
            // player is a player (not a planeswalker or battle, CR 508.1b).
            TriggerCond::Where {
                trigger,
                cond: Condition::And(cs),
            } if matches!(trigger.as_ref(), TriggerCond::Attacks(_))
                && matches!(cs.as_slice(), [
                    Condition::PlayerMatches(PlayerRef::TriggerPlayer, PlayerFilter::Any),
                    Condition::Not(n),
                ] if matches!(n.as_ref(), Condition::SelNonEmpty(Sel::TriggerOtherObject))) =>
            {
                let e = self.trigger_event(trigger, det);
                Ev::new(e.subj, format!("{} a player", e.vp))
            }
            // "Whenever ~ and at least two other creatures attack".
            TriggerCond::Where { trigger, cond }
                if matches!(trigger.as_ref(), TriggerCond::Attacks(_))
                    && matches!(
                        cond,
                        Condition::Compare(Value::EventAmount, Cmp::Ge, Value::Const(_))
                    ) =>
            {
                let (TriggerCond::Attacks(f), Condition::Compare(_, _, Value::Const(n))) =
                    (trigger.as_ref(), cond)
                else {
                    return Ev::new("", self.gap("attacks-with"));
                };
                let o = self.noun_det(f, det.clone());
                let others = n - 1;
                let c = if others == 1 {
                    "at least one other creature".to_string()
                } else {
                    format!("at least {} other creatures", number_word(others))
                };
                Ev::new(format!("{o} and {c}"), "attack")
            }
            // "Whenever ~ becomes the target of a spell an opponent controls".
            TriggerCond::Where {
                trigger,
                cond: Condition::SelMatches(_, Filter::Spell | Filter::SpellOnStack),
            } if matches!(trigger.as_ref(), TriggerCond::BecomesTarget { .. }) => {
                let e = self.trigger_event(trigger, det);
                Ev::new(e.subj, e.vp.replacen("a spell or ability", "a spell", 1))
            }
            // "Whenever ~ attacks while saddled".
            TriggerCond::Where {
                trigger,
                cond: Condition::SelMatches(Sel::This, f),
            } => {
                let e = self.trigger_event(trigger, det);
                let p = self.is_predicate(f, false);
                let p = p.strip_prefix("is ").map(|x| x.to_string()).unwrap_or(p);
                Ev::new(e.subj, format!("{} while {p}", e.vp))
            }
            // "Whenever you cast your first spell during each opponent's turn": the first
            // spell each turn, on an opponent's turn.
            TriggerCond::Where {
                trigger,
                cond: Condition::NotYourTurn,
            } if matches!(trigger.as_ref(), TriggerCond::NthSpellCast { n: 1, .. }) => {
                let e = self.trigger_event(trigger, det);
                let vp = e.vp.replacen(
                    " spell each turn",
                    " spell {alt:each turn during an opponent's turn|during each opponent's turn}",
                    1,
                );
                Ev::new(e.subj, vp)
            }
            // "Whenever you cast a spell during an opponent's turn".
            TriggerCond::Where {
                trigger,
                cond: cond @ (Condition::NotYourTurn | Condition::YourTurn),
            } => {
                let e = self.trigger_event(trigger, det);
                let when = match cond {
                    Condition::NotYourTurn => "during an opponent's turn",
                    _ => "during your turn",
                };
                Ev::new(e.subj, format!("{} {when}", e.vp))
            }
            // "Whenever you roll a 1".
            TriggerCond::Where {
                trigger,
                cond: Condition::Compare(Value::EventAmount, Cmp::Eq, Value::Const(n)),
            } if matches!(trigger.as_ref(), TriggerCond::RollDie(_)) => {
                let e = self.trigger_event(trigger, det);
                Ev::new(e.subj, format!("roll {}", with_article(&n.to_string())))
            }
            // "When ~ becomes monstrous" (CR 701.37b).
            TriggerCond::Where {
                trigger,
                cond: Condition::SelMatches(Sel::TriggerObject, Filter::Source),
            } if matches!(trigger.as_ref(), TriggerCond::PlayerAction { name, .. } if name == "monstrous") => {
                Ev::new(self.me(), "becomes monstrous")
            }
            // "Whenever enchanted player draws a card": a player's event, for one player.
            TriggerCond::Where {
                trigger,
                cond: Condition::PlayerMatches(PlayerRef::TriggerPlayer, PlayerFilter::Ref(p)),
            } if self.trigger_event_peek(trigger).0 == "a player" => {
                let e = self.trigger_event(trigger, det);
                let who = self.player(p, Case::Subj);
                Ev::new(who, e.vp)
            }
            // "Whenever a creature attacks enchanted player": attacking that player (not a
            // planeswalker or battle).
            TriggerCond::Where {
                trigger,
                cond: Condition::And(v),
            } if matches!(trigger.as_ref(), TriggerCond::Attacks(_))
                && matches!(v.as_slice(), [Condition::PlayerMatches(PlayerRef::TriggerPlayer, PlayerFilter::Ref(_)), Condition::Not(n)]
                    if matches!(n.as_ref(), Condition::SelNonEmpty(Sel::TriggerOtherObject))) =>
            {
                let [Condition::PlayerMatches(_, PlayerFilter::Ref(p)), _] = v.as_slice() else {
                    return Ev::new("", self.gap("attacks a player"));
                };
                let e = self.trigger_event(trigger, det);
                let who = self.player(p, Case::Obj);
                Ev::new(e.subj, format!("{} {who}", e.vp))
            }
            // "Whenever you attack with three or more creatures": the event's amount is
            // the number of attacking creatures.
            TriggerCond::Where {
                trigger,
                cond: Condition::Compare(Value::EventAmount, Cmp::Ge, Value::Const(n)),
            } if matches!(trigger.as_ref(), TriggerCond::PlayerAttacks(_)) => {
                let TriggerCond::PlayerAttacks(r) = trigger.as_ref() else {
                    return Ev::new("", self.gap("attack with"));
                };
                let w = self.rel_subject(*r);
                Ev::new(
                    w,
                    format!("attack with {} or more creatures", number_word(*n)),
                )
            }
            // "Whenever ~ attacks while you control two or more artifacts".
            TriggerCond::Where { trigger, cond } => {
                // "Whenever an opponent mills a nonland card", "whenever an opponent draws a
                // card except the first one they draw in each of their draw steps"
                // (`trigger_causes.rs`).
                if let Some((s, vp)) = self.trigger_with_cause(trigger, cond, &det) {
                    return Ev::new(s, vp);
                }
                let e = self.trigger_event(trigger, det);
                let c = self.condition(cond);
                Ev::new(e.subj, format!("{} {{alt:while|if}} {c}", e.vp))
            }
            TriggerCond::FirstTimeEachTurn(inner) => {
                let e = self.trigger_event(inner, det);
                // "Whenever you cast your first instant or sorcery spell each turn".
                let (verb, rest) = match e.vp.split_once(' ') {
                    Some((v @ ("cast" | "casts"), r)) => (v, r),
                    _ => ("", ""),
                };
                let first = rest
                    .strip_prefix("a ")
                    .or_else(|| rest.strip_prefix("an "))
                    .filter(|r| !verb.is_empty() && r.contains("spell") && !r.contains('|'));
                if let Some(r) = first {
                    let (head, tail) = match r.find("spell") {
                        Some(i) => (&r[..i + 5], &r[i + 5..]),
                        None => (r, ""),
                    };
                    let whose = if e.subj == "you" { "your" } else { "their" };
                    return Ev::new(
                        e.subj.clone(),
                        format!(
                            "{verb} {{alt:{rest} for the first time each turn|{whose} first {head}{tail} each turn}}"
                        ),
                    );
                }
                Ev::new(e.subj, format!("{} for the first time each turn", e.vp))
            }
            // "Whenever you roll one or more dice" (once per roll of several dice).
            TriggerCond::Batched {
                trigger,
                per: BatchPer::Batch,
            } if matches!(trigger.as_ref(), TriggerCond::RollDie(_)) => {
                let e = self.trigger_event(trigger, det);
                match e.vp.strip_suffix("roll a die") {
                    Some(head) => Ev::new(e.subj, format!("{head}roll one or more dice")),
                    None => Ev::new(e.subj, e.vp),
                }
            }
            TriggerCond::Batched { trigger, per } => {
                // Once per object (or per source) in a batch is how "whenever a creature is
                // dealt damage" works anyway (CR 603.2c); once per batch or per player is
                // "one or more".
                let d = match (det, per) {
                    (Det::A, BatchPer::Batch | BatchPer::Player) => Det::OneOrMore,
                    (other, _) => other,
                };
                let saved = std::mem::replace(&mut self.batch_once, matches!(per, BatchPer::Batch));
                let e = self.trigger_event(trigger, d);
                self.batch_once = saved;
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
                // "Whenever enchanted player is attacked".
                if a == "a player" && w == " with one or more creatures" {
                    return Ev::new(
                        "",
                        format!("{{alt:a player attacks {d}{w}|{d} is attacked}}"),
                    );
                }
                // Only creatures attack (CR 508.1a): "whenever you attack a player" is
                // with one or more creatures.
                if w == " with one or more creatures" {
                    return Ev::new(a, format!("attack {d} {{opt:with one or more creatures}}"));
                }
                // "Whenever two or more creatures you control attack a player".
                if a == "you" && *min >= 2 && !w.contains('{') {
                    let n = self.noun(with, Num::Many);
                    return Ev::new(
                        "",
                        format!(
                            "{{alt:you attack {d}{w}|{} or more {n} attack {d}}}",
                            number_word(*min as i32)
                        ),
                    );
                }
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
                // "Whenever you tap a land for mana": only its controller can activate a
                // permanent's mana abilities (CR 602.2), so it's a land you control that's
                // tapped for mana.
                let yours = match filter {
                    Filter::And(v)
                        if v.iter()
                            .any(|x| matches!(x, Filter::ControlledBy(PlayerRel::You))) =>
                    {
                        Some(Filter::and(
                            v.iter()
                                .filter(|x| !matches!(x, Filter::ControlledBy(PlayerRel::You)))
                                .cloned()
                                .collect(),
                        ))
                    }
                    _ => None,
                };
                if let (PlayerRel::Any, Some(rest)) = (who, yours) {
                    let a = obj(self, filter);
                    let n = self.noun_det(&rest, Det::A);
                    return Ev::new(
                        "",
                        format!("{{alt:{a} is tapped for mana|you tap {n} for mana}}"),
                    );
                }
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
        // The object itself goes to its owner's zones: "your graveyard".
        if matches!(f, Filter::Source) {
            return format!("your {zw}");
        }
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

/// Whether a rendered trigger condition names an object besides the source itself that a
/// later "it" could refer to ("by a creature", "or another Spirit you control", "to a
/// Spider"). Players ("a player", "an opponent"), counters, and spells or abilities
/// ("becomes the target of a spell") don't count, nor does a "while" condition.
fn names_another_object(trig: &str) -> bool {
    let t = trig.to_lowercase();
    let t = t
        .split(" while ")
        .next()
        .and_then(|t| t.split(" {alt:while|if} ").next())
        .unwrap_or(&t);
    let words: Vec<&str> = t
        .split(|c: char| c.is_whitespace() || c == ',' || c == '.')
        .filter(|w| !w.is_empty())
        .collect();
    const DETERMINERS: &[&str] = &["a", "an", "another", "equipped", "enchanted", "fortified"];
    const NOT_OBJECTS: &[&str] = &[
        "player",
        "players",
        "opponent",
        "opponents",
        "spell",
        "spells",
        "ability",
        "abilities",
        "counter",
        "counters",
        "source",
        "sources",
        "time",
        "turn",
        // Zones aren't objects ("Whenever ~ enters from a graveyard").
        "graveyard",
        "graveyards",
        "library",
        "hand",
    ];
    for (i, w) in words.iter().enumerate() {
        let quantity = (*w == "more" && i >= 2 && words[i - 1] == "or")
            && matches!(words[i - 2], "one" | "two" | "three");
        if !DETERMINERS.contains(w) && !quantity {
            continue;
        }
        // The noun: skip modifiers ("a +1/+1 counter", "a nontoken creature").
        let noun = words[i + 1..]
            .iter()
            .find(|n| !n.starts_with(['+', '-']) && !n.ends_with("/+1"));
        let Some(noun) = noun else {
            continue;
        };
        let next_is_counter = words
            .get(i + 2)
            .is_some_and(|n| matches!(*n, "counter" | "counters"));
        if NOT_OBJECTS.contains(noun) || next_is_counter {
            continue;
        }
        return true;
    }
    false
}

impl Renderer<'_> {
    /// "a spell or ability", "a spell or ability an opponent controls".
    fn spell_or_ability_of(&mut self, by: PlayerRel) -> String {
        match by {
            PlayerRel::Any => "a spell or ability".into(),
            r => {
                let c = self.controls_phrase(r, Num::One);
                format!("a spell or ability {c}")
            }
        }
    }
}
