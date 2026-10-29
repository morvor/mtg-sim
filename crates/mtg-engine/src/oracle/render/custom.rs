//! Named custom behaviors (`Custom(name)` nodes): behaviors implemented in code, named by
//! what they do. Each name the renderer can put into words is listed here; any other
//! name is a gap, reported as a mismatch.

use super::custom_more::custom_player_mod_more;
use super::players::Case;
use super::*;

/// A color word for a mana letter ("U" → "blue").
fn color_of_letter(l: &str) -> Option<&'static str> {
    Some(match l {
        "W" => "white",
        "U" => "blue",
        "B" => "black",
        "R" => "red",
        "G" => "green",
        _ => return None,
    })
}

/// "a", "two or more" for a count threshold.
fn at_least(n: i32, noun: &str) -> String {
    if n <= 1 {
        with_article(noun)
    } else {
        format!("{} or more {}", number_word(n), plural(noun))
    }
}

impl Renderer<'_> {
    pub(crate) fn custom_effect(&mut self, name: &str) -> String {
        let me = |r: &mut Self| r.me();
        let s: String = match name {
            // Bookkeeping: the number of cards a card action affected, per player
            // (`kw/hand_graveyard_actions.rs`).
            crate::kw::hand_graveyard_actions::RECORD_THAT_MANY => String::new(),
            "case: becomes solved" => format!("{} becomes solved", me(self)),
            "flip this permanent" => format!("flip {}", me(self)),
            "end the turn" => "end the turn".into(),
            "parley:each player reveals the top card of their library" => {
                "each player reveals the top card of their library".into()
            }
            "reveal top card if it shares a creature type with ~" => format!(
                "if it shares a creature type with {}, you may reveal it",
                me(self)
            ),
            "forecast:reveal this card from your hand" => {
                format!("reveal {} from your hand", me(self))
            }
            "foretell:it becomes foretold" => "it becomes foretold".into(),
            "chaos ensues (effect)" => "chaos ensues".into(),
            "ascend:spell" => "ascend".into(),
            // A saddled permanent stays saddled until end of turn (CR 702.171b).
            "saddle:becomes saddled" => {
                let w = match (self.each_target, &self.for_each_subject) {
                    (Some(i), _) => self.target_mention(i, Case::Subj),
                    (None, Some(s)) => s.clone(),
                    (None, None) => me(self),
                };
                format!("{w} becomes saddled {{opt:until end of turn}}")
            }
            "plot:becomes plotted" => "it becomes plotted".into(),
            "friend or foe:choose" => "for each player, choose friend or foe".into(),
            n if n.starts_with("cast from hand free with mana value at most:") => {
                let v = &n["cast from hand free with mana value at most:".len()..];
                format!(
                    "you may cast a spell with mana value {v} or less from your hand without paying its mana cost"
                )
            }
            n if n.starts_with("named-token:") => {
                let mut it = n.split(':').skip(1);
                let count: i32 = it.next().and_then(|x| x.parse().ok()).unwrap_or(1);
                let mut tname = it.next().unwrap_or("");
                let tapped = tname == "tapped";
                if tapped {
                    tname = it.next().unwrap_or("");
                }
                if count == 1 && tapped {
                    format!("create a tapped {tname} token")
                } else if tapped {
                    format!("create {} tapped {tname} tokens", number_word(count))
                } else if count == 1 {
                    format!("create {} token", with_article(tname))
                } else {
                    format!("create {} {tname} tokens", number_word(count))
                }
            }
            n if n.starts_with("divided-counters:") => {
                let mut it = n.split(':').skip(1);
                let slot: u8 = it.next().and_then(|x| x.parse().ok()).unwrap_or(0);
                let kind = it.next().unwrap_or("+1/+1");
                let amount = self
                    .targets
                    .get(slot as usize)
                    .and_then(|t| t.divide.clone());
                let (c, w) = match amount {
                    Some(v) => self.counted(&v, &counter_name(kind)),
                    None => (self.gap("divided counters without amount"), None),
                };
                let t = self.target_mention(slot, players::Case::Obj);
                let c = c
                    .trim_start_matches("a ")
                    .trim_start_matches("an ")
                    .to_string();
                format!("distribute {c} among {t}{}", w.unwrap_or_default())
            }
            "end the combat phase" => "end the combat phase".into(),
            "redistribute life totals" => "redistribute any number of players' life totals".into(),
            "roll the planar die (effect)" => "roll the planar die".into(),
            "play a subgame" => {
                "players play a Magic subgame, using their libraries as their decks".into()
            }
            "subgame: non-winners lose half their life" => {
                "each player who doesn't win the subgame loses half their life, rounded up".into()
            }
            "party:choose a party, sacrifice the rest" => {
                "each player chooses a party from among creatures they control, then sacrifices the rest".into()
            }
            n if n.starts_with("each player keeps creatures with total power at most:") => {
                let k = &n["each player keeps creatures with total power at most:".len()..];
                format!("each player chooses any number of creatures they control with total power {k} or less, then sacrifices all other creatures they control")
            }
            "zones:each other player copies that spell with new targets" => {
                "each other player copies that spell. Each of those players may choose new targets for their copy".into()
            }
            n if n.starts_with("conjure:") => {
                // "conjure:[count]:[zone]:each:[name]".
                let parts: Vec<&str> = n["conjure:".len()..].splitn(4, ':').collect();
                match parts.as_slice() {
                    [count, zone, _, name] => {
                        let k: i32 = count.parse().unwrap_or(1);
                        let cards = if k == 1 {
                            "a card".to_string()
                        } else {
                            format!("{} cards", number_word(k))
                        };
                        let to = match *zone {
                            "battlefield" => "onto the battlefield",
                            "library" => "into your library",
                            "hand" => "into your hand",
                            "graveyard" => "into your graveyard",
                            _ => "",
                        };
                        format!("conjure {cards} named {name} {to}")
                    }
                    _ => return self.gap(format!("Effect::Custom({n})")),
                }
            }
            // "Target opponent reveals a card at random from their hand."
            n if n.starts_with("reveal a card at random from hand:target ") => {
                match n["reveal a card at random from hand:target ".len()..].parse::<u8>() {
                    Ok(i) => {
                        let t = self.target_mention(i, Case::Subj);
                        format!("{t} reveals a card at random from their hand")
                    }
                    Err(_) => return self.gap(format!("Effect::Custom({n})")),
                }
            }
            // "You control target opponent during their next turn" (CR 723).
            n if n.starts_with("player control:") => {
                let parts: Vec<&str> = n["player control:".len()..].split(':').collect();
                // "The first player controls the second player during the second
                // player's next turn" (Cruel Entertainment): two target players, called
                // by the order they were chosen in.
                if let ["turn", slot, by] = parts.as_slice() {
                    if let (Ok(i), Some(Ok(j))) = (
                        slot.parse::<u8>(),
                        by.strip_prefix("by").map(|b| b.parse::<u8>()),
                    ) {
                        let a = format!("the {} player", ordinal_word(u32::from(i) + 1));
                        let b = format!("the {} player", ordinal_word(u32::from(j) + 1));
                        return format!("{b} controls {a} during {a}'s next turn");
                    }
                }
                let (span, slot, extra) = match parts.as_slice() {
                    [span, slot] => (*span, *slot, false),
                    [span, slot, "extra"] => (*span, *slot, true),
                    _ => return self.gap(format!("Effect::Custom({n})")),
                };
                let Ok(i) = slot.parse::<u8>() else {
                    return self.gap(format!("Effect::Custom({n})"));
                };
                let t = self.target_mention(i, Case::Obj);
                let when = match span {
                    "turn" => "next turn",
                    "combat" => "next combat phase",
                    _ => return self.gap(format!("Effect::Custom({n})")),
                };
                let mut s =
                    format!("{{alt:you control|you gain control of}} {t} during {{alt:their|that player's}} {when}");
                if extra {
                    s.push_str(". After that turn, that player takes an extra turn");
                }
                s
            }
            n if n.starts_with("play permission terms:") => match self.permission_terms(n) {
                Some(s) => s,
                None => return self.gap(format!("Effect::Custom({n})")),
            },
            other => match self.custom_effect_more(other) {
                Some(s) => s,
                None => match self
                    .custom_effect_extra(other)
                    .or_else(|| self.custom_effect_tail(other))
                    .or_else(|| self.custom_effect_nodes(other))
                {
                    Some(s) => s,
                    None => return self.gap(format!("Effect::Custom({other})")),
                },
            },
        };
        s
    }

    /// A custom filter that is a whole noun phrase ("the creature it haunts").
    pub(crate) fn custom_noun(&mut self, name: &str) -> Option<String> {
        Some(match name {
            // CR 702.55c.
            crate::kw::haunt::HAUNTED => "the creature ~it haunts".into(),
            // CR 702.95: "both creatures have flying" / "each of those creatures has ...".
            crate::kw::soulbond::THE_PAIR => "{alt:each of those creatures|both creatures}".into(),
            // In a granted ability: the object granting it, by its name.
            crate::granted_by::GRANTER => "~".into(),
            _ => return None,
        })
    }

    /// A custom object quality: (adjective?, text). Adjectives go before the noun
    /// ("saddled Mount"); other qualities after it ("creature that saddled it this turn").
    pub(crate) fn custom_filter_quality(&mut self, name: &str) -> (bool, String) {
        let adj = |s: &str| (true, s.to_string());
        let rel = |s: &str| (false, s.to_string());
        match name {
            "saddled" | "suspected" | "monstrous" | "renowned" | "transformed" | "damaged" => {
                adj(name)
            }
            "saddle:saddled it this turn" => rel("that saddled it this turn"),
            "crew:crewed it this turn" => rel("that crewed it this turn"),
            "convoke:convoked it" => rel("that convoked it"),
            "has an adventure" => rel("that has an Adventure"),
            "attached_to_host" => rel("attached to it"),
            "attacking the event's player" => rel("attacking that player"),
            "toughness_gt_power" => rel("with toughness greater than its power"),
            "activated_this_turn" => rel("that was activated this turn"),
            crate::kw::spell_not_cast::NOT_CAST => rel("that wasn't cast"),
            n if n.starts_with("base:p=") => (false, format!("with base power {}", &n[7..])),
            n if n.starts_with("base:t=") => (false, format!("with base toughness {}", &n[7..])),
            "attached to you" => rel("attached to you"),
            // "with base power 0" (CR 208.4b).
            n if n.starts_with(crate::kw::offspring::BASE_POWER) => (
                false,
                format!(
                    "with base power {}",
                    &n[crate::kw::offspring::BASE_POWER.len()..]
                ),
            ),
            "has_nonmana_activated_ability" => {
                rel("with an activated ability that isn't a mana ability")
            }
            "umbra armor:attached to a permanent you control" => {
                rel("attached to a permanent you control")
            }
            "attached to a creature you control" => rel("attached to a creature you control"),
            crate::kw::grant_filters::DEALT_DAMAGE_THIS_TURN => rel("that dealt damage this turn"),
            crate::kw::grant_filters::ATTACKING_OPPONENT_OR_THEIR_PLANESWALKER => {
                rel("attacking your opponents and/or planeswalkers they control")
            }
            crate::kw::grant_filters::ATTACKING_ENCHANTED_PLAYER => {
                rel("attacking enchanted player")
            }
            n if n.starts_with("has landwalk:") => {
                let k = &n["has landwalk:".len()..];
                (false, format!("with {k}"))
            }
            // "if you paid life this way" (see `kw/offered_costs.rs`).
            n if n.starts_with(crate::kw::offered_costs::PAID_OFFERED_COST) => {
                let cost = &n[crate::kw::offered_costs::PAID_OFFERED_COST.len()..];
                (false, format!("for which you chose to {cost}"))
            }
            other => match self.custom_filter_extra(other) {
                Some(q) => q,
                None => {
                    let g = self.gap(format!("Filter::Custom({other})"));
                    (false, g)
                }
            },
        }
    }

    pub(crate) fn custom_value(&mut self, name: &str) -> String {
        // "the amount of {B} spent on X" (see `payment_rules`).
        if let Some(t) = name.strip_prefix(crate::payment_rules::MANA_SPENT_ON_X) {
            return format!("the amount of {{{t}}} spent on X");
        }
        match name {
            "party_size" => "the number of creatures in your party".into(),
            crate::choices::CHOSEN_OBJECT_POWER => self.beheld_power(),
            crate::kw::hand_graveyard_actions::THAT_MANY => "that many".into(),
            "spell_targets_beyond_first" => "the number of targets beyond the first".into(),
            "opus:mana spent to cast that spell" => {
                "the amount of mana spent to cast that spell".into()
            }
            "parley:nonland cards revealed this way" => {
                "the number of nonland cards revealed this way".into()
            }
            "dungeons completed" => "the number of dungeons you've completed".into(),
            "spells_you_cast_this_turn" => "the number of spells you've cast this turn".into(),
            "mutate:times this has mutated" => {
                format!("the number of times {} has mutated", self.me())
            }
            "devour:number devoured" => "the number of creatures it devoured".into(),
            "die:the result" => "the result".into(),
            "colors_of:host" | "colors_of:affected" => "the number of its colors".into(),
            "creatures_you_controlled_died_this_turn" => {
                "the number of creatures that died under your control this turn".into()
            }
            "turn:creatures attacked" => "the number of creatures that attacked this turn".into(),
            "snow mana spent to cast it" => {
                format!("the amount of {{S}} spent to cast {}", self.me())
            }
            "web-slinging:mana value of the returned creature" => {
                "the mana value of the returned creature".into()
            }
            "sticker unique vowels" => "the number of unique vowels on that sticker".into(),
            "commander:times cast from the command zone" => {
                "the number of times you've cast your commander from the command zone this game"
                    .into()
            }
            n if n.starts_with("mana_spent_of:") => {
                let l = &n["mana_spent_of:".len()..];
                format!("the amount of {{{l}}} spent to cast {}", self.me())
            }
            // "that many": the amount paid for "you may pay any amount of mana" (see
            // `kw/offered_costs.rs`).
            n if n.starts_with(crate::kw::offered_costs::PAID_OFFERED_AMOUNT) => {
                "the amount of mana you paid this way".into()
            }
            "opponents_counters:poison" => {
                "the number of poison counters your opponents have".into()
            }
            "players being attacked" => "the number of players being attacked".into(),
            crate::kw::mana_grammar::UNLOCKED_DOORS => {
                "the number of unlocked doors among Rooms you control".into()
            }
            n if n.starts_with(crate::kw::mana_grammar::CHROMA_GRAVEYARD) => {
                let c = n[crate::kw::mana_grammar::CHROMA_GRAVEYARD.len()..]
                    .chars()
                    .next()
                    .and_then(crate::types::Color::from_letter)
                    .map_or("", |c| c.word());
                format!(
                    "the number of {c} mana symbols in the mana costs of cards in your graveyard"
                )
            }
            "cards_in_all_hands" => "the total number of cards in all players' hands".into(),
            "life_lost_by_opponents_this_turn" => {
                "the total life lost by your opponents this turn".into()
            }
            "crew:creatures that crewed it this turn" => {
                "the number of creatures that crewed it this turn".into()
            }
            "convoke:number of creatures that convoked it" => {
                "the number of creatures that convoked it".into()
            }
            "bushido:points of bushido it has" => "the number of points of bushido it has".into(),
            other => match self.custom_value_more(other) {
                Some(s) => s,
                None => match self
                    .custom_value_extra(other)
                    .or_else(|| self.cast_history_value(other))
                    .or_else(|| self.chosen_number_value(other))
                    .or_else(|| self.draft_value(other))
                    .or_else(|| self.custom_value_nodes(other))
                {
                    Some(s) => s,
                    None => self.gap(format!("Value::Custom({other})")),
                },
            },
        }
    }

    /// A comparison of a custom value with a number, as cards word it ("{U} was spent to
    /// cast ~", "you've cast two or more spells this turn").
    pub(crate) fn custom_compare(&mut self, name: &str, cmp: Cmp, n: i32) -> Option<String> {
        if let Some(s) = self.custom_compare_named(name, cmp, n) {
            return Some(s);
        }
        if let Some(s) = self.custom_compare_extra(name, cmp, n) {
            return Some(s);
        }
        // Thresholds: "N or more".
        let min = match (cmp, n) {
            (Cmp::Ge, n) => n,
            (Cmp::Gt, n) => n + 1,
            _ => {
                return match (name, cmp, n) {
                    ("turn:creatures attacked", Cmp::Eq, 0) => {
                        Some("no creatures attacked this turn".into())
                    }
                    ("hideaway:cards in the smallest library", Cmp::Le, n) => Some(format!(
                        "a library has {} or fewer cards in it",
                        number_word(n)
                    )),
                    _ => None,
                }
            }
        };
        Some(match name {
            n2 if n2.starts_with("mana_spent_of:") && min >= 1 => {
                let l = &n2["mana_spent_of:".len()..];
                if min == 1 {
                    format!("{{{l}}} was spent to cast {}", self.me())
                } else {
                    let c = color_of_letter(l).unwrap_or(l);
                    format!(
                        "at least {} {c} mana was spent to cast {}",
                        number_word(min),
                        self.me()
                    )
                }
            }
            "opus:mana spent to cast that spell" => format!(
                "{} or more mana was spent to cast that spell",
                number_word(min)
            ),
            "spells_you_cast_this_turn" => {
                format!("you've cast {} this turn", at_least(min, "spell"))
            }
            "dungeons completed" if min == 1 => "you've completed a dungeon".into(),
            "devour:number devoured" if min == 1 => "it devoured a creature".into(),
            "max_opponent_counters:poison" => format!(
                "an opponent has {} or more poison counters",
                number_word(min)
            ),
            "coven:creatures you control with different powers" => format!(
                "you control {} or more creatures with different powers",
                number_word(min)
            ),
            "hideaway:creatures you attacked with this turn" => format!(
                "you attacked with {} or more creatures this turn",
                number_word(min)
            ),
            "party_size" if min == 4 => "you have a full party".to_string(),
            "turn:sources you controlled dealt damage" => format!(
                "{} or more sources you controlled dealt damage this turn",
                number_word(min)
            ),
            "turn:creature cards put into graveyards" => format!(
                "{} or more creature cards were put into graveyards from anywhere this turn",
                number_word(min)
            ),
            // "if an opponent was dealt 7 or more damage this turn".
            "hideaway:most damage dealt to an opponent this turn" => format!(
                "an opponent was dealt {} or more damage this turn",
                number_word(min)
            ),
            "max_mana_spent_of_one_color" => format!(
                "at least {} mana of the same color was spent to cast {}",
                number_word(min),
                self.me()
            ),
            _ => return None,
        })
    }

    pub(crate) fn custom_condition(&mut self, name: &str) -> String {
        let me = |r: &mut Self| r.me();
        match name {
            "you_attacked_this_turn" => "you attacked this turn".into(),
            "eminence:in the command zone or on the battlefield" => {
                format!("{} is in the command zone or on the battlefield", me(self))
            }
            "an opponent this turn:gained life" => "an opponent gained life this turn".into(),
            "untap:control_and_source_remains_tapped" => {
                let m = me(self);
                format!("you control {m} and {m} remains tapped")
            }
            "soulbond:this is paired with a creature with soulbond" => {
                format!("{} is paired with a creature with soulbond", me(self))
            }
            "combat:this attacked or blocked this combat" => {
                format!("{} attacked or blocked this combat", me(self))
            }
            "exploit:it exploited that creature" => "it exploited that creature".into(),
            n if n.starts_with("you_cast_another_spell_this_turn:") => {
                let c = &n["you_cast_another_spell_this_turn:".len()..];
                match color_of_letter(c) {
                    Some(w) => format!("you've cast another {w} spell this turn"),
                    None => self.gap(format!("Condition::Custom({n})")),
                }
            }
            crate::kw::storied::HAS_ENDURING_STORY => "you have an enduring story".into(),
            // CR 505.1a: a turn's combat phases are numbered in order.
            crate::kw::activated_ability_kind::FIRST_COMBAT_PHASE => {
                "it's the first combat phase of the turn".into()
            }
            "foretell:this spell was foretold" => "this spell was foretold".into(),
            "a_player_cast_two_spells_last_turn" => {
                "a player cast two or more spells last turn".into()
            }
            "no_spells_cast_last_turn" => "no spells were cast last turn".into(),
            "case: is solved" => format!("{} is solved", me(self)),
            "untap:source_remains_tapped" => format!("{} remains tapped", me(self)),
            "soulbond:this is paired with another creature" => {
                format!("{} is paired with another creature", me(self))
            }
            "permanent_left_under_your_control_this_turn" => {
                "a permanent left the battlefield under your control this turn".into()
            }
            "permanent_you_controlled_left_this_turn" => {
                "a permanent you controlled left the battlefield this turn".into()
            }
            "opponent_lost_life_this_turn" => "an opponent lost life this turn".into(),
            "cast during your main phase" => {
                format!("you cast {} during your main phase", me(self))
            }
            "warp:a spell was warped this turn" => "a spell was warped this turn".into(),
            "warp:a nonland permanent left the battlefield this turn" => {
                "a nonland permanent left the battlefield this turn".into()
            }
            "tribute:tribute wasn't paid" => "tribute wasn't paid".into(),
            "restrictions:you_were_attacked_this_step" => "you've been attacked this step".into(),
            "you_descended_this_turn" => "you descended this turn".into(),
            "you_cast_noncreature_spell_this_turn" => {
                "you've cast a noncreature spell this turn".into()
            }
            "this spell's additional cost was paid" => {
                "this spell's additional cost was paid".into()
            }
            "creature_died_under_your_control_this_turn" => {
                "a creature died under your control this turn".into()
            }
            "card_left_your_graveyard_this_turn" => "a card left your graveyard this turn".into(),
            "you_lost_life_last_turn" => "you lost life last turn".into(),
            crate::oracle::patterns::r603_entered_last_turn::CREATURE_ENTERED_LAST_TURN => {
                "you had another creature enter the battlefield under your control last turn".into()
            }
            "turn:this dealt damage to an opponent" => {
                format!("{} dealt damage to an opponent this turn", me(self))
            }
            // A spell's own cost: the spell isn't cast until its costs are paid (CR
            // 601.2i), so "a spell" cast this turn is another one.
            "spell_costs:cast_another_spell_this_turn" => {
                "you've cast {alt:another|a} spell this turn".into()
            }
            "spell_costs:cast_another_instant_or_sorcery_this_turn" => {
                "you've cast {alt:another|an} instant or sorcery spell this turn".into()
            }
            "dethrone:it's attacking the player with the most life" => {
                "it's attacking the player with the most life or tied for most life".into()
            }
            "enlist:it enlisted a creature this combat" => {
                "it enlisted a creature this combat".into()
            }
            "facedown:revealed is a creature card" => "it's a creature card".into(),
            "mill:two milled cards share all their card types" => {
                "two cards that share all their card types were milled this way".into()
            }
            n if most_common_chosen(n).is_some() => most_common_chosen(n).unwrap_or_default(),
            n if n.starts_with("most_common_color:") => {
                let c = color_of_letter(&n["most_common_color:".len()..]).unwrap_or("?");
                format!(
                    "{c} is the most common color among all permanents or is tied for most common"
                )
            }
            n if n.starts_with("an opponent this turn:entered:") => {
                let mut it = n["an opponent this turn:entered:".len()..].split(':');
                let k: i32 = it.next().and_then(|x| x.parse().ok()).unwrap_or(1);
                let t = it.next().unwrap_or("permanent").to_lowercase();
                format!(
                    "an opponent had {} enter the battlefield under their control this turn",
                    at_least(k, &t)
                )
            }
            n if n.starts_with("an opponent this turn:drew:") => {
                let k: i32 = n["an opponent this turn:drew:".len()..]
                    .parse()
                    .unwrap_or(1);
                format!("an opponent drew {} this turn", at_least(k, "card"))
            }
            crate::oracle::patterns::grant_conditions::COMMITTED_CRIME_THIS_TURN => {
                "you've committed a crime this turn".into()
            }
            n if n.starts_with(crate::oracle::patterns::grant_conditions::ACTION_THIS_TURN) => {
                match &n[crate::oracle::patterns::grant_conditions::ACTION_THIS_TURN.len()..] {
                    "surveil" => "you've surveilled this turn".into(),
                    "scry" => "you've scried this turn".into(),
                    other => self.gap(format!("Condition::Custom(did {other})")),
                }
            }
            n if n.starts_with(crate::oracle::patterns::grant_conditions::SACRIFICED_THIS_TURN) => {
                let t = &n[crate::oracle::patterns::grant_conditions::SACRIFICED_THIS_TURN.len()..];
                let t = t
                    .strip_prefix(crate::oracle::patterns::grant_conditions::SACRIFICED_SUBTYPE)
                    .unwrap_or(t);
                format!("you've sacrificed {} this turn", with_article(t))
            }
            other => match self
                .custom_condition_more(other)
                .or_else(|| self.custom_condition_terms(other, false))
                .or_else(|| self.custom_condition_tail(other))
                .or_else(|| self.chosen_number_condition(other))
                .or_else(|| self.draft_condition(other))
                .or_else(|| self.custom_condition_nodes(other))
            {
                Some(s) => s,
                None => self.gap(format!("Condition::Custom({other})")),
            },
        }
    }

    /// Whether a custom trigger renders as a complete trigger phrase (with its own
    /// trigger word).
    pub(crate) fn custom_trigger_is_complete(&self, name: &str) -> bool {
        name == "visit"
            || name.starts_with("door unlocked:")
            || name == crate::kw::exert::EXERTED
            || name == crate::kw::damage_prevented_trigger::DAMAGE_TO_YOU_PREVENTED
    }

    /// A custom trigger event ("~ mutates", "you unlock this door").
    pub(crate) fn custom_trigger(&mut self, name: &str) -> String {
        match name {
            crate::kw::trigger_events::FINAL_CHAPTER_TRIGGERS => {
                "the final chapter ability of a Saga you control triggers".into()
            }
            crate::kw::teamwork::SELF_TAPPED_FOR_TEAMWORK => {
                format!("{} becomes tapped to pay a teamwork cost", self.me())
            }
            crate::kw::trigger_events::ENTERS_ATTACHED_TO_CREATURE => {
                format!("{} enters attached to a creature", self.me())
            }
            "visit" => "visit —".into(),
            crate::kw::damage_prevented_trigger::DAMAGE_TO_YOU_PREVENTED => {
                "whenever damage that would be dealt to you is prevented".into()
            }
            // "You may exert ~ as it attacks. When you do, ..." (CR 701.43).
            crate::kw::exert::EXERTED => "when you do".into(),
            n if n.starts_with("door unlocked:") => "when you unlock this door".into(),
            "mutates:self" => format!("{} mutates", self.me()),
            "die:highest natural result:you" => "you roll a die's highest natural result".into(),
            // "When you roll a natural 20", "whenever you roll your third die each turn"
            // (CR 706.2).
            n if n.starts_with(crate::dice::NATURAL_N) || n.starts_with(crate::dice::NTH_DIE) => {
                let natural = n.starts_with(crate::dice::NATURAL_N);
                let rest = n
                    .strip_prefix(crate::dice::NATURAL_N)
                    .or_else(|| n.strip_prefix(crate::dice::NTH_DIE))
                    .unwrap_or_default();
                let (k, who) = match rest.trim_start_matches(':').split_once(':') {
                    Some((k, w)) => (k.parse::<u32>().ok(), w),
                    None => (None, ""),
                };
                let (subj, poss) = match who {
                    "you" => ("you roll", "your"),
                    "opponent" => ("an opponent rolls", "their"),
                    "any" => ("a player rolls", "their"),
                    _ => return self.gap(format!("TriggerCond::Custom({n})")),
                };
                match (k, natural) {
                    (Some(k), true) => format!("{subj} a natural {k}"),
                    (Some(k), false) => format!("{subj} {poss} {} die each turn", ordinal_word(k)),
                    _ => self.gap(format!("TriggerCond::Custom({n})")),
                }
            }
            n if n.starts_with("class becomes level:") => {
                let l = &n["class becomes level:".len()..];
                format!("{} becomes level {l}", self.me())
            }
            "suspend:a time counter is removed from this card" => format!(
                "a time counter is removed from {} while it's exiled",
                self.me()
            ),
            "crew:~ crews a vehicle" => format!("{} crews a Vehicle", self.me()),
            "saddle:~ saddles a mount" => format!("{} saddles a Mount", self.me()),
            "a spell or ability an opponent controls causes you to discard this" => format!(
                "a spell or ability an opponent controls causes you to discard {}",
                self.me()
            ),
            n => {
                // "[keyword]:[event]" names describe the event: "exploit:this exploits a
                // creature", "training:this creature trains".
                let known = [
                    "foretell:",
                    "mutate:",
                    "mentor:",
                    "visit:",
                    "exploit:",
                    "training:",
                    "enlist:",
                    "evolve:",
                    "exhaust:",
                    "boast:",
                ];
                if let Some(p) = known.iter().find(|p| n.starts_with(*p)) {
                    let ev = &n[p.len()..];
                    let me = self.me();
                    return ev
                        .replace("this creature", &me)
                        .replace("this ", &format!("{me} "))
                        .replace('~', &me);
                }
                self.gap(format!("TriggerCond::Custom({n})"))
            }
        }
    }

    pub(crate) fn custom_static(&mut self, name: &str) -> String {
        let me = |r: &mut Self| r.me();
        match name {
            "exert as it attacks" => format!("you may exert {} as it attacks", me(self)),
            "commander:can be your commander" => format!("{} can be your commander", me(self)),
            "deck:any number" => format!("a deck can have any number of cards named {}", me(self)),
            "assigns combat damage as though unblocked" => format!(
                "{} assigns its combat damage as though it weren't blocked",
                me(self)
            ),
            "may assign combat damage as though unblocked" => format!(
                "you may have {} assign its combat damage as though it weren't blocked",
                me(self)
            ),
            "may look at cards exiled with this" => {
                format!("you may look at cards exiled with {}", me(self))
            }
            crate::kw::pay_life_exile_instead::PAY_LIFE_EXILE_INSTEAD => "if you would pay life while your library has at least that many cards in it, exile that many cards from the top of your library instead".into(),
            crate::game_terms::DEVOTION_INCREASED => "your devotion to each color and each combination of colors is increased by one".into(),
            "hands revealed:each" => "players play with their hands revealed".into(),
            "hands revealed:opponents" => "your opponents play with their hands revealed".into(),
            "opponents' creatures targetable as though no hexproof" => "creatures your opponents control with hexproof can be the targets of spells and abilities you control as though they didn't have hexproof".into(),
            "opponents and their permanents targetable as though no hexproof" => "your opponents and permanents your opponents control with hexproof can be the targets of spells and abilities you control as though they didn't have hexproof".into(),
            "warp:cast from graveyard using warp" => {
                format!("you may cast {} from your graveyard using its warp ability", me(self))
            }
            "you may cast this card from your graveyard using its blitz ability" => {
                format!("you may cast {} from your graveyard using its blitz ability", me(self))
            }
            "vote: you may vote an additional time" => {
                "while voting, you may vote an additional time".into()
            }
            n if n.starts_with("partner:") => {
                let what = &n["partner:".len()..];
                match what {
                    "choose a background" => "choose a Background".into(),
                    "doctor's companion" => "doctor's companion".into(),
                    other => format!("partner—{other}"),
                }
            }
            n if n.starts_with("tap_total_power:") && !n.ends_with(":toughness") => {
                let mut it = n["tap_total_power:".len()..].split(':');
                let kw = it.next().unwrap_or("");
                let delta = it.next().unwrap_or("");
                let (verb, obj) = match kw {
                    "Crew" => ("crews", "Vehicles"),
                    "Saddle" => ("saddles", "Mounts"),
                    _ => ("crews", "Vehicles"),
                };
                match delta.strip_prefix('+') {
                    Some(d) => format!(
                        "{} {verb} {obj} as though its power were {} greater",
                        me(self),
                        number_word(d.parse().unwrap_or(0))
                    ),
                    None => match self.custom_static_more(n) {
                        Some(s) => s,
                        None => self.gap(format!("StaticEffect::Custom({n})")),
                    },
                }
            }
            n if n.starts_with(crate::kw::torpor::DOESNT_TRIGGER) => {
                let mut it = n[crate::kw::torpor::DOESNT_TRIGGER.len()..].split(';');
                let events = it.next().unwrap_or("");
                let types: Vec<String> = it
                    .next()
                    .unwrap_or("")
                    .split(',')
                    .filter(|t| !t.is_empty())
                    .map(|t| plural(t))
                    .collect();
                let subject = if types.is_empty() {
                    "permanents".to_string()
                } else {
                    join_list(&types, "and")
                };
                let verb = if events == "enter,die" {
                    "entering or dying"
                } else {
                    "entering"
                };
                let whose = if it.next() == Some("opponents") {
                    "abilities of permanents your opponents control"
                } else {
                    "abilities"
                };
                format!("{subject} {verb} don't cause {whose} to trigger")
            }
            n if n.starts_with("blockable as though no landwalk:") => {
                let k = &n["blockable as though no landwalk:".len()..];
                format!("creatures with {k} can be blocked as though they didn't have {k}")
            }
            n if n.starts_with("mill multiplier:2:opponents") => {
                "if an opponent would mill one or more cards, they mill twice that many cards instead".into()
            }
            n if n.starts_with("power-up:other creatures' cost less:") => {
                let k = &n["power-up:other creatures' cost less:".len()..];
                format!("power-up abilities of other creatures you control cost {{{k}}} less to activate")
            }
            "cascade:as you cascade, put a land card from among the exiled cards onto the battlefield tapped" => {
                "as you cascade, you may put a land card from among the exiled cards onto the battlefield tapped".into()
            }
            n if n.starts_with("deck:up to ") => {
                let k: i32 = n["deck:up to ".len()..].parse().unwrap_or(0);
                format!("a deck can have up to {} cards named {}", number_word(k), me(self))
            }
            "deck:circle two colors" => "as you create your deck, circle two of the colors below".into(),
            "exhaust:during your turn, you may activate exhaust abilities as though they haven't been activated" => {
                "during your turn, as long as you haven't activated an exhaust ability this turn, you may activate exhaust abilities as though they haven't been activated".into()
            }
            "teamwork:flash if cast using teamwork" => format!(
                "you may cast {} as though it had flash if it's cast using teamwork",
                me(self)
            ),
            "bestow:you may cast this card from your graveyard using its bestow ability" => {
                format!("you may cast {} from your graveyard using its bestow ability", me(self))
            }
            "you may cast this card from your graveyard using its mutate ability" => {
                format!("you may cast {} from your graveyard using its mutate ability", me(self))
            }
            "foretell:costs {1} less and can be done on any player's turn" => {
                "foretelling cards from your hand costs {1} less and can be done on any player's turn".into()
            }
            "can block creatures with shadow as though they didn't have shadow" => format!(
                "{} can block creatures with shadow as though they didn't have shadow",
                me(self)
            ),
            "plot:the top card of your library has plot equal to its mana cost" => {
                "the top card of your library has plot. The plot cost is equal to its mana cost".into()
            }
            "plot:you may plot nonland cards from the top of your library" => {
                "you may plot nonland cards from the top of your library".into()
            }
            "opponents skip extra turns" => {
                "if an opponent would begin an extra turn, that player skips that turn instead".into()
            }
            n if n.starts_with("surveil extra:") => {
                let k: i32 = n["surveil extra:".len()..].parse().unwrap_or(0);
                format!(
                    "you may look at an additional {} cards each time you surveil",
                    number_word(k)
                )
            }
            "wither:all damage as though its source had wither" => {
                "all damage is dealt as though its source had wither".into()
            }
            "power-up:can be activated an additional time" => {
                "each power-up ability of permanents you control can be activated an additional time".into()
            }
            n if n.starts_with("collect evidence instead of mana cost:") => {
                let k = &n["collect evidence instead of mana cost:".len()..];
                format!("you may collect evidence {k} rather than pay the mana cost for spells you cast")
            }
            "villainous choice: opponents face it an additional time" => {
                "if an opponent would face a villainous choice, they face that choice an additional time".into()
            }
            "radiation: gain life rather than lose life" => {
                "you gain life rather than lose life from radiation".into()
            }
            "boast:creatures you control can boast twice during each of your turns" => {
                "creatures you control can boast twice during each of your turns rather than once".into()
            }
            "vote: you get an additional vote" => "while voting, you get an additional vote".into(),
            n if n.starts_with("search portion:") && n.ends_with(":opponents") => {
                let k: i32 = n["search portion:".len()..n.len() - ":opponents".len()]
                    .parse()
                    .unwrap_or(0);
                format!(
                    "if an opponent would search a library, that player searches the top {} cards of that library instead",
                    number_word(k)
                )
            }
            "tap_total_power:Crew:toughness" => {
                format!("{} crews Vehicles using its toughness rather than its power", me(self))
            }
            "tap_total_power:Station:toughness" => format!(
                "{} stations permanents using its toughness rather than its power",
                me(self)
            ),
            other => match self.custom_static_more(other) {
                Some(s) => s,
                None => match self
                    .custom_static_extra(other)
                    .or_else(|| self.named_static(other))
                    .or_else(|| self.draft_static(other))
                    .or_else(|| self.custom_static_nodes(other))
                {
                    Some(s) => s,
                    None => self.gap(format!("StaticEffect::Custom({other})")),
                },
            },
        }
    }

    pub(crate) fn custom_restriction(&mut self, name: &str) -> String {
        match name {
            "players can't cycle cards" => "players can't cycle cards".into(),
            "combat: controller chooses how creatures block" => {
                "you choose which creatures block and how those creatures block".into()
            }
            "combat: controller chooses which creatures attack" => {
                "you choose which creatures attack".into()
            }
            "combat: controller chooses how opponents' creatures block" => {
                "you choose how those creatures block".into()
            }
            "untap:may_choose_not_to_untap" => format!(
                "you may choose not to untap {} during your untap step",
                self.me()
            ),
            crate::kw::game_rules::NO_LOYALTY_EACH => {
                "players can't activate planeswalkers' loyalty abilities".into()
            }
            crate::kw::game_rules::NO_LOYALTY_OPPONENTS => {
                "your opponents can't activate planeswalkers' loyalty abilities".into()
            }
            crate::kw::game_rules::ENCHANTED_ONLY_MANA_OR_LOYALTY => {
                "enchanted player can't activate abilities that aren't mana abilities or loyalty abilities".into()
            }
            crate::kw::durations_4::POWER_UP_CANT_BE_ACTIVATED => {
                "power-up abilities can't be activated".into()
            }
            crate::block_choice::ATTACKING_PLAYER_CHOOSES_BLOCKS => {
                "the attacking player chooses how each creature blocks each combat".into()
            }
            other => self.gap(format!("Restriction::Custom({other})")),
        }
    }

    /// A modification implemented in code (`Modification::Custom`), as a verb phrase.
    pub(crate) fn custom_modification(&mut self, name: &str) -> String {
        // Layer 6 (CR 613.1f).
        if name == crate::kw::mana_grammar::EXCEPT_MANA_ABILITIES {
            return "lose all abilities except mana abilities".into();
        }
        match self.custom_modification_more(name) {
            Some(s) => s,
            None => match self
                .custom_modification_extra(name)
                .or_else(|| self.named_modification(name))
            {
                Some(s) => s,
                None => self.gap(format!("Modification::Custom({name})")),
            },
        }
    }

    /// A custom rule for players: `subj` is "you", "players", ...; `poss` its possessive.
    pub(crate) fn custom_player_mod(&mut self, name: &str, subj: &str, poss: &str) -> String {
        let you = subj == "you";
        match name {
            "keep unspent mana" => {
                let s = if subj == "each player" {
                    "players"
                } else {
                    subj
                };
                let dont = if you || s == "players" || s.ends_with('s') {
                    "don't"
                } else {
                    "doesn't"
                };
                format!("{s} {dont} lose unspent mana as steps and phases end")
            }
            // "You don't lose unspent red mana as steps and phases end" (CR 500.5).
            n if n
                .strip_prefix(crate::mana_abilities::KEEP_UNSPENT_MANA)
                .is_some_and(|t| t.len() == 2 && t.starts_with(' ')) =>
            {
                let color = match &n[n.len() - 1..] {
                    "W" => "white",
                    "U" => "blue",
                    "B" => "black",
                    "R" => "red",
                    "G" => "green",
                    _ => "colorless",
                };
                let dont = if you || subj.ends_with('s') {
                    "don't"
                } else {
                    "doesn't"
                };
                format!("{subj} {dont} lose unspent {color} mana as steps and phases end")
            }
            // "If you would lose unspent mana, that mana becomes red instead." (CR 614.1a).
            n if you
                && n.strip_prefix(crate::mana_abilities::UNSPENT_MANA_BECOMES)
                    .is_some_and(|t| t.len() == 2 && t.starts_with(' ')) =>
            {
                let t = crate::mana::ManaType::from_letter_str(&n[n.len() - 1..]);
                let what = match t.and_then(|t| t.color()) {
                    Some(c) => c.word(),
                    None => "colorless",
                };
                format!("if you would lose unspent mana, that mana becomes {what} instead")
            }
            // Mana burn (CR 500.5 as it once was): each player affected loses life.
            crate::kw::mana_grammar::MANA_BURN if matches!(subj, "players" | "each player") => {
                "a player losing unspent mana causes that player to lose that much life".into()
            }
            crate::kw::mana_grammar::MANA_BURN => {
                format!("{subj} losing unspent mana causes {subj} to lose that much life")
            }
            // CR 704.5j: the legend rule, for the permanents this player controls.
            crate::legend_rule::NOT_FOR_YOUR_PERMANENTS if you => {
                "the \"legend rule\" doesn't apply to permanents you control".into()
            }
            "spend mana as though any color" => {
                format!("{subj} may spend mana as though it were mana of any color")
            }
            "unspent mana becomes C" => {
                format!("if {subj} would lose unspent mana, that mana becomes colorless instead")
            }
            crate::reveal::PLAYS_WITH_HAND_REVEALED => {
                let verb = if you || subj.ends_with('s') {
                    "play"
                } else {
                    "plays"
                };
                format!("{subj} {verb} with {poss} hand revealed")
            }
            "may look at the top card of their library any time" => {
                format!("{subj} may look at the top card of {poss} library any time")
            }
            other => match custom_player_mod_more(other, subj)
                .or_else(|| self.named_player_mod(other, subj))
            {
                Some(s) => s,
                None => self.gap(format!("PlayerModification::Custom({other})")),
            },
        }
    }
}

/// "the chosen color is the most common color among nontoken permanents the chosen player
/// controls but isn't tied for most common" (`kw::most_common_chosen_color`).
fn most_common_chosen(n: &str) -> Option<String> {
    let (c, _, strict) = crate::kw::most_common_chosen_color::parse_name(n)?;
    let c = match c {
        Some(c) => color_of_letter(&c.letter().to_string())?.to_string(),
        None => "the chosen color".to_string(),
    };
    let tie = if strict {
        "but isn't tied for most common"
    } else {
        "or is tied for most common"
    };
    Some(format!(
        "{c} is the most common color among nontoken permanents the chosen player controls {tie}"
    ))
}
