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
        let p = self.player(who, Case::Poss);
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
