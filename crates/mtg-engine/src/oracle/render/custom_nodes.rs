//! Named behaviors implemented in code (custom filters, conditions, values, effects and
//! static abilities) that earlier renderer modules left unworded, each put into words from
//! what its implementation does (see the constant each name is defined by).

use super::custom_filters::CustomQuality;
use super::players::Case;
use super::*;

/// The quality of a custom filter after (or before) a noun.
pub(crate) fn custom_rel_nodes(name: &str) -> Option<CustomQuality> {
    use CustomQuality::*;
    let rel = |s: &str| Some(Rel(s.to_string()));
    match name {
        // A card whose card types number two or more (CR 300.2, 205.2a).
        crate::oracle::patterns::filter_qualities::TWO_CARD_TYPES => {
            rel("with two or more card types")
        }
        // CR 601.2: cast this turn (a permanent that was a spell cast this turn).
        crate::kw::cast_history::CAST_THIS_TURN => rel("{alt:that was|that were} cast this turn"),
        // The player who put it into exile is the evaluating player
        // (`kw/exile_provenance.rs`).
        crate::kw::exile_provenance::EXILED_BY_YOU => rel("you exiled"),
        crate::kw::exile_provenance::EXILED_BY_THEM => rel("they exiled"),
        // "If an effect causes you to discard a card": not discarded as a cost
        // (`discard_rules.rs`).
        crate::discard_rules::DISCARDED_BY_EFFECT => rel("an effect causes you to discard"),
        // "with an activated ability" (CR 602.1).
        crate::kw::craft::HAS_ACTIVATED_ABILITY => rel("with activated abilities"),
        // An Aura card whose last object on the battlefield was attached to the triggering
        // object (`kw/attached_before.rs`).
        crate::kw::attached_before::WAS_ATTACHED_TO_TRIGGER_OBJECT => {
            rel("that were attached to it")
        }
        // Of the cards the latest instruction acted on, one with the same mana value as
        // another of them (`kw/referents_5.rs`).
        crate::kw::referents_5::SAME_MANA_VALUE_AS_ANOTHER_OF_IT => {
            rel("that has the same mana value as another card revealed this way")
        }
        // The power's quality (odd or even) the linked choice chose (`kw/choices_4.rs`).
        crate::kw::choices_4::POWER_OF_CHOSEN_QUALITY => rel("with power of the chosen quality"),
        _ => None,
    }
}

/// A clause saying the subject has a custom quality, for a condition ("if that creature
/// shares a color with the mana that land produced").
pub(crate) fn custom_clause_nodes(name: &str, subj: &str, negated: bool) -> Option<String> {
    let (yes, no) = match name {
        crate::kw::damage_shares_a_color::SHARES_COLOR_WITH_PRODUCED_MANA => (
            format!("{subj} shares a color with the mana that land produced"),
            format!("{subj} doesn't share a color with the mana that land produced"),
        ),
        _ => return None,
    };
    Some(if negated { no } else { yes })
}

impl Renderer<'_> {
    /// "the top two cards of your graveyard", "the top three black cards of your
    /// graveyard" (`kw/graveyard_top_n.rs`, CR 404.2).
    pub(crate) fn top_cards_of_graveyard(&mut self, f: &Filter) -> Option<String> {
        use crate::kw::graveyard_top_n as g;
        let atoms: Vec<&Filter> = match f {
            Filter::And(v) => v.iter().collect(),
            other => vec![other],
        };
        let (n, word) = atoms.iter().find_map(|a| match a {
            Filter::Custom(n) => n.strip_prefix(g::AMONG_TOP_OF_GRAVEYARD)?.split_once(':'),
            _ => None,
        })?;
        let n: i32 = n.parse().ok()?;
        let q = g::quality(word)?;
        let mut owner = None;
        for a in &atoms {
            match a {
                Filter::OwnedBy(r) => owner = Some(*r),
                Filter::Custom(_) | Filter::InZone(ZoneKind::Graveyard) | Filter::Card => {}
                x if format!("{x:?}") == format!("{q:?}") => {}
                _ => return None,
            }
        }
        let whose = match owner {
            Some(r) => self.rel_possessive(r, Num::One),
            None => "a".into(),
        };
        let quality = if word.is_empty() {
            String::new()
        } else {
            format!("{} ", word.to_lowercase())
        };
        Some(format!(
            "the top {} {quality}cards of {whose} graveyard",
            number_word(n)
        ))
    }

    /// Conditions implemented in code that this module words.
    pub(crate) fn custom_condition_nodes(&mut self, name: &str) -> Option<String> {
        Some(match name {
            // CR 503.2: any time after the first upkeep step of the turn has ended.
            crate::turn_structure::AFTER_UPKEEP => "after their upkeep step".into(),
            crate::kw::trigger_events::CAUSED_BY_YOURS => {
                "a spell or ability you control caused it".into()
            }
            n if n.starts_with(crate::dice::ANY_RESULT_AT_LEAST) => {
                let k: i32 = n[crate::dice::ANY_RESULT_AT_LEAST.len()..]
                    .trim_start_matches(':')
                    .parse()
                    .ok()?;
                format!("any of those results was {k} or higher")
            }
            _ => return None,
        })
    }

    /// Numbers implemented in code that this module words.
    pub(crate) fn custom_value_nodes(&mut self, name: &str) -> Option<String> {
        Some(match name {
            // CR 709.5: each unlocked door's name, counted once.
            crate::kw::room_door_names::DIFFERENT_DOOR_NAMES => {
                "the number of different names among unlocked doors of Rooms you control".into()
            }
            // CR 309.7.
            crate::kw::game_rules_4::DIFFERENT_DUNGEONS_COMPLETED => {
                "the number of differently named dungeons you've completed".into()
            }
            _ => return None,
        })
    }

    /// Effects implemented in code that this module words.
    pub(crate) fn custom_effect_nodes(&mut self, name: &str) -> Option<String> {
        // "you may activate ~'s loyalty abilities two more times this turn".
        if let Some(n) = crate::kw::game_rules_4::more_loyalty_count(name) {
            let m = self.me();
            return Some(format!(
                "you may activate {m}'s loyalty ability {} more times this turn",
                number_word(n as i32)
            ));
        }
        Some(match name {
            // CR 104.4a.
            crate::game_end::DRAW_EFFECT => "the game is a draw".into(),
            // CR 706.3c.
            crate::dice::ROLL_AGAIN => "roll again".into(),
            _ => return None,
        })
    }

    /// Static abilities implemented in code that this module words.
    pub(crate) fn custom_static_nodes(&mut self, name: &str) -> Option<String> {
        Some(match name {
            // CR 406.3: each player may look at the face-down cards they exiled with it.
            crate::kw::exile_provenance::EACH_MAY_LOOK_AT_OWN_EXILED => {
                format!("each player may look at cards they exiled with {}", self.me())
            }
            _ => return None,
        })
    }

    /// "the number of instant and sorcery spells cast before that spell this turn": the
    /// spells cast this turn no later than that spell, but for that spell
    /// (`kw/cast_history.rs`, [`crate::kw::cast_history::CAST_BY_THEN`]).
    pub(crate) fn cast_before_count(&mut self, v: &Value) -> Option<String> {
        // "you get X {E}, where X is that excess damage": the excess damage dealt to the
        // triggering creature this turn (CR 120.10), after "if that creature was dealt
        // excess damage this turn".
        if let Value::EventsThisTurn(t, Tally::Amount) = v {
            if let TriggerCond::DealtExcessDamage {
                filter: Filter::In(s),
                noncombat_only: false,
            } = t.as_ref()
            {
                if matches!(s.as_ref(), Sel::TriggerObject) {
                    return Some("that excess damage".into());
                }
            }
        }
        let Value::Sum(parts) = v else {
            return None;
        };
        let [Value::EventsThisTurn(t, Tally::Events), Value::Const(-1)] = parts.as_slice() else {
            return None;
        };
        let TriggerCond::CastSpell {
            who: PlayerRel::Any,
            filter: Filter::And(atoms),
        } = t.as_ref()
        else {
            return None;
        };
        let by_then = format!("{}that spell", crate::kw::cast_history::CAST_BY_THEN);
        if !atoms
            .iter()
            .any(|a| matches!(a, Filter::Custom(n) if *n == by_then))
        {
            return None;
        }
        let rest: Vec<Filter> = atoms
            .iter()
            .filter(|a| !matches!(a, Filter::Custom(n) if *n == by_then))
            .cloned()
            .collect();
        let saved = (self.alt_and, self.plural_alts);
        (self.alt_and, self.plural_alts) = (true, true);
        let n = self.noun(&Filter::and(rest), Num::Many);
        (self.alt_and, self.plural_alts) = saved;
        Some(format!("the number of {n} cast before that spell this turn"))
    }

    /// The player-subject form of a custom effect, for an effect another player performs:
    /// "If that opponent's speed is greater than each other player's speed, reduce that
    /// opponent's speed by 1. This effect can't reduce their speed below 1."
    pub(crate) fn custom_effect_as_player(&mut self, who: &PlayerRef, name: &str) -> Option<String> {
        let (n, floor) = crate::kw::game_rules_4::reduce_highest_speed_args(name)?;
        // The controller of a target "an opponent controls" is "that opponent".
        let opponents_target = match who {
            PlayerRef::ControllerOf(s) => match s.as_ref() {
                Sel::Target(i) => matches!(
                    self.targets.get(*i as usize).map(|t| &t.what),
                    Some(TargetKind::Object(Filter::And(v)))
                        if v.iter().any(|x| matches!(x, Filter::ControlledBy(PlayerRel::Opponent)))
                ),
                _ => false,
            },
            _ => false,
        };
        let p = if opponents_target {
            "that opponent's".to_string()
        } else {
            self.player(who, Case::Poss)
        };
        let mut s = format!(
            "if {p} speed is greater than each other player's speed, reduce {p} speed by {}",
            number_word(n as i32)
        );
        if floor > 0 {
            s.push_str(&format!(
                ". This effect can't reduce {{alt:their|{p}}} speed below {}",
                number_word(floor as i32)
            ));
        }
        Some(s)
    }
}

impl Renderer<'_> {
    /// The (subject, verb phrase) of a trigger event worded here.
    pub(crate) fn trigger_event_nodes(
        &mut self,
        t: &TriggerCond,
        det: &super::nouns::Det,
    ) -> Option<(String, String)> {
        let TriggerCond::Where { trigger, cond } = t else {
            return None;
        };
        match (trigger.as_ref(), cond) {
            // "When the chosen player loses the game" (even once they've left it).
            (TriggerCond::PlayerLoses, Condition::Custom(n))
                if n == crate::kw::trigger_events::EVENT_PLAYER_IS_CHOSEN =>
            {
                Some(("the chosen player".into(), "loses the game".into()))
            }
            // "Whenever an instant or sorcery spell is cast during your turn": by any
            // player.
            (
                TriggerCond::CastSpell {
                    who: PlayerRel::Any,
                    filter,
                },
                Condition::YourTurn,
            ) if matches!(det, super::nouns::Det::A) => {
                let s = self.spell_noun(filter, det.clone());
                Some((
                    String::new(),
                    format!("{{alt:a player casts {s}|{s} is cast}} during your turn"),
                ))
            }
            // "When you remove the last intervention counter from ~": you removed one and
            // none are left.
            (
                TriggerCond::CountersRemoved {
                    filter: Filter::Source,
                    kind: Some(k),
                    each: false,
                },
                Condition::And(cs),
            ) if matches!(cs.as_slice(), [
                    Condition::Compare(Value::CountersOn(s, Some(k2)), Cmp::Eq, Value::Const(0)),
                    Condition::PlayerMatches(PlayerRef::TriggerPlayer, PlayerFilter::You),
                ] if matches!(s.as_ref(), Sel::This) && format!("{k2:?}") == format!("{k:?}")) =>
            {
                let m = self.me();
                Some((
                    "you".into(),
                    format!("remove the last {} from {m}", counter_name(k)),
                ))
            }
            // "a spell or ability you control exiles one or more permanents from the
            // battlefield": the event's cause (`kw/trigger_events.rs`).
            (
                TriggerCond::ZoneChange {
                    filter,
                    from: Some(ZoneKind::Battlefield),
                    to: Some(ZoneKind::Exile),
                },
                Condition::Custom(n),
            ) if n == crate::kw::trigger_events::CAUSED_BY_YOURS => {
                let o = self.noun_det(filter, det.clone());
                Some((
                    "a spell or ability you control".into(),
                    format!("exiles {o} from the battlefield"),
                ))
            }
            _ => None,
        }
    }
}

impl Renderer<'_> {
    /// Instructions of a sequence put into words together here, and how many.
    pub(crate) fn seq_nodes(&mut self, v: &[Effect]) -> Option<(String, usize)> {
        self.paid_up_to_times(v)
            .or_else(|| self.whenever_drawn_this_way(v))
    }

    /// "Draw a card, then each other player may draw a card. Whenever a card is drawn
    /// this way, ...": the cards drawn this turn counted before and after the draws, and
    /// a reflexive trigger repeated once for each card drawn in between (CR 603.12).
    fn whenever_drawn_this_way(&mut self, v: &[Effect]) -> Option<(String, usize)> {
        let [Effect::StoreValue { var, value }, rest @ ..] = v else {
            return None;
        };
        let drawn = |x: &Value| {
            matches!(x, Value::OverPlayers(AggOp::Sum, PlayerFilter::Any, c)
                if matches!(c.as_ref(), Value::CardsDrawnThisTurn(PlayerRef::Iterated)))
        };
        if !drawn(value) {
            return None;
        }
        let j = rest.iter().position(|e| {
            matches!(e, Effect::Repeat { times: Value::Diff(a, b), effect }
                if drawn(a)
                    && matches!(b.as_ref(), Value::Var(x) if x == var)
                    && matches!(effect.as_ref(), Effect::Reflexive { .. }))
        })?;
        if j == 0 {
            return None;
        }
        let Effect::Repeat { effect, .. } = &rest[j] else {
            return None;
        };
        let Effect::Reflexive { body } = effect.as_ref() else {
            return None;
        };
        let draws = self.seq(&rest[..j]);
        let b = self.in_event_scope(|r| r.body(body));
        Some((
            format!(
                "{}. Whenever a card is drawn this way, {}",
                draws.trim_end_matches('.'),
                effects::lower_first(&b)
            ),
            j + 2,
        ))
    }

    /// "You may pay {G} up to three times. When you pay this cost one or more times,
    /// ...": the payment offered until it's declined or made N times, the payments counted
    /// as X, then a reflexive trigger if it was paid at least once (CR 603.12).
    fn paid_up_to_times(&mut self, v: &[Effect]) -> Option<(String, usize)> {
        let [Effect::StoreValue {
            var: paid,
            value: Value::Const(0),
        }, Effect::StoreValue {
            var: declined,
            value: Value::Const(0),
        }, Effect::Repeat {
            times: Value::Const(n),
            effect,
        }, Effect::SetX {
            value: Value::Var(x),
        }, Effect::If {
            cond: Condition::Compare(Value::X, Cmp::Ge, Value::Const(1)),
            then,
            otherwise,
        }, ..] = v
        else {
            return None;
        };
        if x != paid || !matches!(otherwise.as_ref(), Effect::Noop) {
            return None;
        }
        let Effect::If {
            cond: Condition::Compare(Value::Var(d), Cmp::Eq, Value::Const(0)),
            then: pay,
            otherwise: no,
        } = effect.as_ref()
        else {
            return None;
        };
        if d != declined || !matches!(no.as_ref(), Effect::Noop) {
            return None;
        }
        let Effect::PayOptional {
            who: PlayerRef::You,
            cost,
            then: counted,
            otherwise: stop,
        } = pay.as_ref()
        else {
            return None;
        };
        let counts = matches!(counted.as_ref(), Effect::StoreValue { var, value: Value::Sum(p) }
            if var == paid && matches!(p.as_slice(), [Value::Var(a), Value::Const(1)] if a == paid));
        let stops = matches!(stop.as_ref(), Effect::StoreValue { var, value: Value::Const(1) } if var == declined);
        if !counts || !stops {
            return None;
        }
        let Effect::Reflexive { body } = then.as_ref() else {
            return None;
        };
        let c = self.cost(cost);
        let saved = std::mem::replace(&mut self.extra.x_paid, true);
        let b = self.in_event_scope(|r| r.body(body));
        self.extra.x_paid = saved;
        Some((
            format!(
                "you may pay {c} up to {} times. {{alt:When you pay this cost one or more times|When you do}}, {}",
                number_word(*n as i32),
                effects::lower_first(&b)
            ),
            5,
        ))
    }
}

impl Renderer<'_> {
    /// "if you've done all four this turn": each of the player actions the trigger event
    /// is one of happened this turn.
    pub(crate) fn all_actions_done(&mut self, c: &Condition) -> Option<String> {
        if self.trigger_actions.len() < 2 {
            return None;
        }
        fn flat<'a>(c: &'a Condition, out: &mut Vec<&'a Condition>) {
            match c {
                Condition::And(v) => v.iter().for_each(|x| flat(x, out)),
                other => out.push(other),
            }
        }
        let mut parts = Vec::new();
        flat(c, &mut parts);
        let names: Vec<&smol_str::SmolStr> = parts
            .iter()
            .map(|p| match p {
                Condition::Compare(Value::EventsThisTurn(t, Tally::Events), Cmp::Ge, Value::Const(1)) => {
                    match t.as_ref() {
                        TriggerCond::PlayerAction {
                            name,
                            who: PlayerRel::You,
                        } => Some(name),
                        _ => None,
                    }
                }
                _ => None,
            })
            .collect::<Option<Vec<_>>>()?;
        let same = names.len() == self.trigger_actions.len()
            && names.iter().all(|n| self.trigger_actions.contains(n));
        same.then(|| format!("you've done all {} this turn", number_word(names.len() as i32)))
    }
}
