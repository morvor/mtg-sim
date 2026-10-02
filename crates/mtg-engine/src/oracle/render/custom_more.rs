//! More numbers, conditions, static abilities, and modifications implemented in code
//! (`Value::Custom`, `Condition::Custom`, `StaticEffect::Custom`, `Modification::Custom`,
//! `PlayerModification::Custom`), put into words from what each one checks or does.

use super::*;

/// A rule for players ("you can't get poison counters"): `subj` is "you", "players", ...
pub(crate) fn custom_player_mod_more(name: &str, subj: &str) -> Option<String> {
    let you_like = subj == "you" || subj.ends_with('s');
    Some(match name {
        "can't get poison counters" => format!("{subj} can't get poison counters"),
        "don't lose the game for having 0 or less life" => {
            let dont = if you_like { "don't" } else { "doesn't" };
            format!("{subj} {dont} lose the game for having 0 or less life")
        }
        // Predatory Focus.
        "creatures assign combat damage as though unblocked" => {
            "have creatures you control assign their combat damage as though they weren't blocked"
                .into()
        }
        _ => return None,
    })
}

impl Renderer<'_> {
    pub(crate) fn custom_value_more(&mut self, name: &str) -> Option<String> {
        Some(match name {
            "cards put into enchanted player's graveyard this turn" => {
                "the number of cards put into {alt:their|enchanted player's} graveyard from anywhere this turn".into()
            }
            "creatures_exiled_from_opponents_this_turn" => {
                "the number of creatures that were exiled under your opponents' control this turn".into()
            }
            n if n.starts_with("devour:number devoured of type:") => {
                let t = &n["devour:number devoured of type:".len()..];
                format!("the number of {} ~it devoured", plural(t))
            }
            crate::dice::STORED_SAME => {
                "the greatest number of stored results on ~it of the same value".into()
            }
            "draft:highest number noted for cards with this name" => {
                format!("the highest number you noted for cards named {}", self.me())
            }
            // You can attack only opponents (CR 508.1b): "players you attacked".
            "melee:opponents you attacked with a creature this combat" => {
                "{alt:the number of players you attacked this combat|the number of opponents you attacked with a creature this combat}".into()
            }
            "opponents being attacked" => "the number of opponents you're attacking".into(),
            "opponents you attacked this turn" => {
                "the number of opponents you attacked this turn".into()
            }
            "spells_cast_this_turn" => "the number of spells cast this turn".into(),
            "times_descended" => "the number of times you descended this turn".into(),
            // CR 903.3e: the greatest mana value among them if you have two.
            crate::commander_rules::YOUR_COMMANDER_MANA_VALUE => {
                "the greatest mana value among your commanders".into()
            }
            "number of targets of the triggering spell" => {
                "the number of targets {alt:it has|that spell has}".into()
            }
            n if n.starts_with("spells cast before the trigger spell:") => {
                let f: Filter =
                    serde_json::from_str(&n["spells cast before the trigger spell:".len()..])
                        .ok()?;
                let saved = self.alt_and;
                self.alt_and = true;
                let s = self.noun(&f, Num::Many);
                self.alt_and = saved;
                format!("the number of other {s} spells you've cast before it this turn")
            }
            _ => return None,
        })
    }

    pub(crate) fn custom_condition_more(&mut self, name: &str) -> Option<String> {
        Some(match name {
            "creature_died_under_an_opponents_control_this_turn" => {
                "a creature died under an opponent's control this turn".into()
            }
            "crime:you've committed a crime this turn" => {
                "you've committed a crime this turn".into()
            }
            "embalm:~ was embalmed" => format!("{} was embalmed", self.me()),
            "emerge:you're casting a spell with emerge" => {
                "{opt:you're} casting a spell with emerge".into()
            }
            "end_step:1" => "it's the first end step of the turn".into(),
            "upkeep:1" => "it's the first upkeep step of the turn".into(),
            crate::search_rules::YOU_SEARCHED_THIS_WAY => "you search your library this way".into(),
            "you_were_the_starting_player" => "you were the starting player".into(),
            // CR 701.42a.
            crate::merge::MELD_PAIR_CONDITION => {
                let (partner, _) = self.info.meld.clone()?;
                format!("you both own and control {} and {partner}", self.me())
            }
            n if n.starts_with("you control N creatures that share a creature type:") => {
                let k: i32 = n["you control N creatures that share a creature type:".len()..]
                    .parse()
                    .ok()?;
                let w = number_word(k);
                format!("you control {{alt:at least {w}|{w} or more}} creatures that share a creature type")
            }
            _ => return None,
        })
    }

    pub(crate) fn custom_static_more(&mut self, name: &str) -> Option<String> {
        // "You may cast this card from your graveyard by discarding two cards in addition
        // to paying its other costs." The cost is the one the ability's rules read
        // (`kw/cast_self_from_graveyard.rs`).
        for (prefix, instead) in [
            ("cast from graveyard instead:", true),
            ("cast from graveyard also:", false),
        ] {
            if let Some(text) = name.strip_prefix(prefix) {
                let (cost, loyalty) = crate::oracle::costs::parse_cost(text)?;
                if loyalty {
                    return None;
                }
                let by = self.cost_gerund(&cost);
                let m = self.me();
                let tail = if instead {
                    "rather than paying its mana cost"
                } else {
                    "in addition to paying its other costs"
                };
                return Some(format!(
                    "you may cast {m} from your graveyard by {by} {tail}"
                ));
            }
        }
        let m = self.me();
        Some(match name {
            "tap_total_power:Crew:toughness" => {
                format!("{m} crews Vehicles using its toughness rather than its power")
            }
            "tap_total_power:Saddle:toughness" => {
                format!("{m} saddles Mounts using its toughness rather than its power")
            }
            "tap_total_power:Station:toughness" => {
                format!("{m} stations permanents using its toughness rather than its power")
            }
            // "Enchanted creature can't attack, block, or crew Vehicles."
            "tap_total_power:Crew:attached can't" => {
                let a = self.attached_noun();
                format!("{a} can't crew Vehicles")
            }
            "draft:reveal and note cards drafted this round" => format!(
                "reveal {m} as you draft it and note how many cards you've drafted this draft round, including {m}"
            ),
            _ => return None,
        })
    }

    /// A cost as what's done to pay it: "paying {3}{R} and exiling four other cards from
    /// your graveyard".
    fn cost_gerund(&mut self, c: &Cost) -> String {
        let mut parts = Vec::new();
        if let Some(m) = &c.mana {
            parts.push(format!("paying {m}"));
        }
        for p in &c.parts {
            let s = self.cost_part(p);
            let (verb, rest) = s.split_once(' ').unwrap_or((s.as_str(), ""));
            let v = verb.to_lowercase();
            let ing = match v.as_str() {
                "pay" => "paying".to_string(),
                "tap" => "tapping".to_string(),
                v if v.ends_with('e') => format!("{}ing", &v[..v.len() - 1]),
                v => format!("{v}ing"),
            };
            parts.push(format!("{ing} {rest}").trim().to_string());
        }
        join_list(&parts, "and")
    }

    pub(crate) fn custom_modification_more(&mut self, name: &str) -> Option<String> {
        // "has all activated abilities of all creature cards in all graveyards".
        let rest =
            name.strip_prefix(crate::kw::hand_graveyard_actions::ACTIVATED_ABILITIES_OF_GRAVEYARD)?;
        let (whose, kind) = rest.split_once(':')?;
        let where_ = match whose {
            "all" => "all graveyards",
            "your" => "your graveyard",
            _ => return None,
        };
        Some(format!(
            "has all activated abilities of all {kind} cards in {where_}"
        ))
    }
}
