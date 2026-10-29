//! Static abilities and player rules implemented in code (`StaticEffect::Custom`,
//! `PlayerModification::Custom`), and the spell markers effects give (where a spell goes
//! as it resolves), put into words from what each one does.

use super::*;

/// "white" for "W".
fn color_word(l: char) -> Option<&'static str> {
    Some(match l {
        'W' => "white",
        'U' => "blue",
        'B' => "black",
        'R' => "red",
        'G' => "green",
        _ => return None,
    })
}

/// A spell marker (`kw/spell_destination_markers.rs`) given by `mods`, if that's all they
/// give.
fn marker(mods: &[Modification]) -> Option<&str> {
    let [Modification::AddAbility(a)] = mods else {
        return None;
    };
    let AbilityKind::Static(s) = &a.kind else {
        return None;
    };
    match &s.effect {
        StaticEffect::Custom(n) => Some(n.as_str()),
        _ => None,
    }
}

impl Renderer<'_> {
    /// The words of a custom static ability this module knows.
    pub(crate) fn named_static(&mut self, name: &str) -> Option<String> {
        use crate::rule_statics::{caused_actions as ca, opponents_steps as os};
        // CR 609.4b: "You may spend white mana as though it were red mana", "... as though
        // it were mana of any color".
        if let Some(r) = name.strip_prefix("spend rules:as though:") {
            let (from, to) = r.split_once(':')?;
            let from = color_word(from.chars().next()?)?;
            let to = if to.len() == 5 {
                "mana of any color".to_string()
            } else {
                let words: Option<Vec<String>> = to
                    .chars()
                    .map(|c| color_word(c).map(String::from))
                    .collect();
                format!("{} mana", join_list(&words?, "or"))
            };
            return Some(format!("you may spend {from} mana as though it were {to}"));
        }
        // "You may spend blue, black, red, or green mana only as though it were colorless
        // mana"; with every color but one, "mana other than white mana" (printed "other
        // mana" after the ability naming that color, `joined_lines.rs` checks).
        if let Some(r) = name.strip_prefix("spend rules:only as though colorless:") {
            let listed: Vec<char> = r.chars().collect();
            let missing: Vec<char> = "WUBRG".chars().filter(|c| !listed.contains(c)).collect();
            let what = match missing.as_slice() {
                [m] if listed.len() == 4 => format!("mana other than {} mana", color_word(*m)?),
                _ => {
                    let words: Option<Vec<String>> = listed
                        .iter()
                        .map(|c| color_word(*c).map(String::from))
                        .collect();
                    format!("{} mana", join_list(&words?, "or"))
                }
            };
            return Some(format!(
                "you may spend {what} only as though it were colorless mana"
            ));
        }
        // "If ~ is in a graveyard, effects from spells named Flame Burst count it as a card
        // named Flame Burst."
        if let Some(n) = name.strip_prefix(crate::counts_as_named::PREFIX) {
            let m = self.me();
            return Some(format!(
                "if {m} is in a graveyard, effects from spells named {n} count it as a card named {n}"
            ));
        }
        // "You can't choose an untapped creature as ~'s target as you cast it."
        if let Some(j) =
            name.strip_prefix(crate::oracle::patterns::targets_grammar_2::CAST_TARGET_FORBIDDEN)
        {
            let f: Filter = serde_json::from_str(j).ok()?;
            let n = self.noun_det(&f, super::nouns::Det::A);
            let m = self.me();
            return Some(format!(
                "you can't choose {n} as {m}'s target as you cast it"
            ));
        }
        // "Activated abilities of creatures with flying with {T} in their costs can't be
        // activated" (CR 602.5).
        if let Some(j) = name.strip_prefix(crate::kw::tap_abilities_prohibited::PREFIX) {
            let f: Filter = serde_json::from_str(j).ok()?;
            let n = self.noun(&f, Num::Many);
            return Some(format!(
                "activated abilities of {n} with {{T}} in their costs can't be activated"
            ));
        }
        if let Some(who) = name.strip_prefix(crate::reveal::HANDS_REVEALED) {
            return Some(match who {
                "you" => "play with your hand revealed".into(),
                "opponents" => "your opponents play with their hands revealed".into(),
                _ => "players play with their hands revealed".into(),
            });
        }
        Some(match name {
            ca::CANT_CAUSE_CONTROLLER_TO_SEARCH => {
                "spells and abilities your opponents control can't cause their controller to search their library".into()
            }
            ca::CANT_CAUSE_YOU_TO_DISCARD => {
                "spells and abilities your opponents control can't cause you to discard cards"
                    .into()
            }
            os::UNTAP_DURING_OPPONENTS_UNTAP_STEPS => {
                "untap all permanents you control during each opponent's untap step".into()
            }
            os::DRAW_DURING_OPPONENTS_DRAW_STEPS => {
                "you draw a card during each opponent's draw step".into()
            }
            crate::kw::look_face_down::LOOK_AT_FACE_DOWN_CREATURES => {
                "you may look at face-down creatures you don't control any time".into()
            }
            crate::kw::hexproof::OPPONENT_CREATURES_TARGETABLE_BY_ANY => {
                "creatures your opponents control can be the targets of spells and abilities as though they didn't have hexproof. Ward abilities of those creatures don't trigger".into()
            }
            crate::kw::echo::ECHO_FOR_ZERO => {
                "you may pay {0} rather than pay the echo cost for permanents you control".into()
            }
            crate::choices::AS_ATTACHED_CHOOSE_COLOR => {
                let m = self.me();
                format!("as {m} becomes attached to a creature, choose a color")
            }
            _ => return None,
        })
    }

    /// The words of a custom modification this module knows, as a verb phrase.
    pub(crate) fn named_modification(&mut self, name: &str) -> Option<String> {
        Some(match name {
            // Layer 4: the second land type chosen replaces their land types (CR 305.7).
            crate::choices::SECOND_CHOSEN_TYPE => "{alt:is|becomes} the second chosen type".into(),
            // Layer 6.
            crate::choices::CHOSEN_ABILITIES => "has each of the chosen abilities".into(),
            _ => return None,
        })
    }

    /// The words of a custom player rule this module knows (`subj`: "you", ...).
    pub(crate) fn named_player_mod(&mut self, name: &str, subj: &str) -> Option<String> {
        if subj != "you" {
            return None;
        }
        Some(match name {
            crate::as_though::OTHER_GRAVEYARDS_AS_YOURS => {
                "you may play lands and cast spells from other players' graveyards as though those cards were in your graveyard".into()
            }
            crate::kw::look_face_down::LOOK_AT_FACE_DOWN_CREATURES => {
                "you may look at face-down creatures you don't control any time".into()
            }
            crate::kw::hexproof::OPPONENTS_AND_CREATURES_AS_THOUGH_NO_HEXPROOF => {
                "your opponents and creatures your opponents control with hexproof can be the targets of spells and abilities you control as though they didn't have hexproof".into()
            }
            _ => return None,
        })
    }

    /// "Exile that card instead of putting it into your graveyard as it resolves. If you
    /// do, return it to your hand at the beginning of the next end step." (a marker given
    /// to the triggering spell, `kw/spell_destination_markers.rs`).
    pub(crate) fn spell_marker_effect(
        &mut self,
        what: &Sel,
        mods: &[Modification],
    ) -> Option<String> {
        use crate::kw::spell_destination_markers as m;
        if !matches!(what, Sel::TriggerSpell | Sel::TriggerObject) {
            return None;
        }
        Some(match marker(mods)? {
            m::EXILE_THEN_HAND => "exile that card instead of putting it into your graveyard as it resolves. If you do, return it to your hand at the beginning of the next end step".into(),
            m::TO_HAND => "put that card into your hand instead of into your graveyard as it resolves".into(),
            _ => return None,
        })
    }

    /// "The next time you cast an instant or sorcery spell from your hand this turn, put
    /// that card into your hand instead of into your graveyard as it resolves."
    pub(crate) fn next_spell_marker(
        &mut self,
        filter: &Filter,
        mods: &[Modification],
        expires: &Duration,
    ) -> Option<String> {
        let name = marker(mods)?;
        if name != crate::kw::spell_destination_markers::TO_HAND
            || !matches!(expires, Duration::EndOfTurn | Duration::ThisTurn)
        {
            return None;
        }
        let Filter::And(parts) = filter else {
            return None;
        };
        let from_hand = parts
            .iter()
            .any(|f| matches!(f, Filter::CastFrom(ZoneKind::Hand)));
        let rest: Vec<Filter> = parts
            .iter()
            .filter(|f| !matches!(f, Filter::CastFrom(_)))
            .cloned()
            .collect();
        let n = self.noun_det(&Filter::and(rest), super::nouns::Det::A);
        let n = if n.contains("spell") {
            n
        } else {
            format!("{n} spell")
        };
        let from = if from_hand { " from your hand" } else { "" };
        Some(format!(
            "the next time you cast {n}{from} this turn, put that card into your hand instead of into your graveyard as it resolves"
        ))
    }
}
