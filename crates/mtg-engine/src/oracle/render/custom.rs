//! Named custom behaviors (`Custom(name)` nodes): behaviors implemented in code, named by
//! what they do. Each name the renderer can put into words is listed here; any other
//! name is a gap, reported as a mismatch.

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
            "saddle:becomes saddled" => format!("{} becomes saddled", me(self)),
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
                let tname = it.next().unwrap_or("");
                if count == 1 {
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
            other => return self.gap(format!("Effect::Custom({other})")),
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
            _ => return None,
        })
    }

    /// A custom object quality: (adjective?, text). Adjectives go before the noun
    /// ("saddled Mount"); other qualities after it ("creature that saddled it this turn").
    pub(crate) fn custom_filter_quality(&mut self, name: &str) -> (bool, String) {
        let adj = |s: &str| (true, s.to_string());
        let rel = |s: &str| (false, s.to_string());
        match name {
            "saddled" | "suspected" | "monstrous" | "renowned" | "transformed" => adj(name),
            "saddle:saddled it this turn" => rel("that saddled it this turn"),
            "crew:crewed it this turn" => rel("that crewed it this turn"),
            "convoke:convoked it" => rel("that convoked it"),
            "has an adventure" => rel("that has an Adventure"),
            "attached_to_host" => rel("attached to it"),
            "attacking the event's player" => rel("attacking that player"),
            "toughness_gt_power" => rel("with toughness greater than its power"),
            n if n.starts_with("has landwalk:") => {
                let k = &n["has landwalk:".len()..];
                (false, format!("with {k}"))
            }
            other => {
                let g = self.gap(format!("Filter::Custom({other})"));
                (false, g)
            }
        }
    }

    pub(crate) fn custom_value(&mut self, name: &str) -> String {
        match name {
            "party_size" => "the number of creatures in your party".into(),
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
            other => self.gap(format!("Value::Custom({other})")),
        }
    }

    /// A comparison of a custom value with a number, as cards word it ("{U} was spent to
    /// cast ~", "you've cast two or more spells this turn").
    pub(crate) fn custom_compare(&mut self, name: &str, cmp: Cmp, n: i32) -> Option<String> {
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
            n2 if n2.starts_with("mana_spent_of:") => {
                let l = &n2["mana_spent_of:".len()..];
                if min <= 1 {
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
            "dungeons completed" if min <= 1 => "you've completed a dungeon".into(),
            "devour:number devoured" if min <= 1 => "it devoured a creature".into(),
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
            "party_size" if min >= 4 => "you have a full party".to_string(),
            _ => return None,
        })
    }

    pub(crate) fn custom_condition(&mut self, name: &str) -> String {
        let me = |r: &mut Self| r.me();
        match name {
            "you_attacked_this_turn" => "you attacked this turn".into(),
            crate::kw::storied::HAS_ENDURING_STORY => "you have an enduring story".into(),
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
            "turn:this dealt damage to an opponent" => {
                format!("{} dealt damage to an opponent this turn", me(self))
            }
            "spell_costs:cast_another_spell_this_turn" => {
                "you've cast another spell this turn".into()
            }
            "spell_costs:cast_another_instant_or_sorcery_this_turn" => {
                "you've cast another instant or sorcery spell this turn".into()
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
            other => self.gap(format!("Condition::Custom({other})")),
        }
    }

    /// Whether a custom trigger renders as a complete trigger phrase (with its own
    /// trigger word).
    pub(crate) fn custom_trigger_is_complete(&self, name: &str) -> bool {
        name == "visit" || name.starts_with("door unlocked:") || name == crate::kw::exert::EXERTED
    }

    /// A custom trigger event ("~ mutates", "you unlock this door").
    pub(crate) fn custom_trigger(&mut self, name: &str) -> String {
        match name {
            "visit" => "visit —".into(),
            // "You may exert ~ as it attacks. When you do, ..." (CR 701.43).
            crate::kw::exert::EXERTED => "when you do".into(),
            n if n.starts_with("door unlocked:") => "when you unlock this door".into(),
            "mutates:self" => format!("{} mutates", self.me()),
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
            "may assign combat damage as though unblocked" => format!(
                "you may have {} assign its combat damage as though it weren't blocked",
                me(self)
            ),
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
            n if n.starts_with("tap_total_power:") => {
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
                    None => self.gap(format!("StaticEffect::Custom({n})")),
                }
            }
            n if n.starts_with("entering doesn't cause abilities to trigger:") => {
                let types: Vec<String> = n["entering doesn't cause abilities to trigger:".len()..]
                    .split(',')
                    .map(|t| plural(t))
                    .collect();
                format!(
                    "{} entering don't cause abilities to trigger",
                    join_list(&types, "and")
                )
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
            other => self.gap(format!("StaticEffect::Custom({other})")),
        }
    }

    pub(crate) fn custom_restriction(&mut self, name: &str) -> String {
        match name {
            "untap:may_choose_not_to_untap" => format!(
                "you may choose not to untap {} during your untap step",
                self.me()
            ),
            other => self.gap(format!("Restriction::Custom({other})")),
        }
    }

    pub(crate) fn custom_player_mod(&mut self, name: &str) -> String {
        self.gap(format!("PlayerModification::Custom({name})"))
    }
}
