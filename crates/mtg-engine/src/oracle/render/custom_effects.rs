//! More instructions implemented in code (`Effect::Custom`), put into words from what each
//! one does (see the constant or keyword-rules file each name belongs to).

use super::players::Case;
use super::*;

/// "an artifact spell", "instant and/or sorcery spells": the kinds of spells a card type
/// list (`Instant,Sorcery`) names.
fn spell_kinds(types: &str, many: bool) -> String {
    let words: Vec<String> = types
        .split(',')
        .filter(|t| !t.is_empty())
        .map(|t| t.to_lowercase())
        .collect();
    match (words.as_slice(), many) {
        ([], false) => "a spell".into(),
        ([], true) => "spells".into(),
        (w, false) => with_article(&format!("{} spell", join_list(w, "or"))),
        (w, true) => format!("{} spells", join_list(w, "and/or")),
    }
}

/// "your hand or graveyard", "your graveyard and/or hand".
fn zones_phrase(zones: &str, conj: &str) -> Option<String> {
    let z: Vec<&str> = zones.split(',').filter(|z| !z.is_empty()).collect();
    Some(match z.as_slice() {
        [one] => format!("your {one}"),
        [a, b] => format!("your {a} {conj} {b}"),
        _ => return None,
    })
}

impl Renderer<'_> {
    /// The target an instruction is about: the target of a "for each" over a single
    /// target, or the first one.
    fn each_target_mention(&mut self) -> String {
        let i = self.each_target.unwrap_or(0);
        if (i as usize) < self.targets.len() {
            self.target_mention(i, Case::Obj)
        } else {
            "it".into()
        }
    }

    pub(crate) fn custom_effect_more(&mut self, name: &str) -> Option<String> {
        // "Pay {E}{E}." as an instruction (see `kw/game_rules.rs`).
        if let Some(n) = crate::kw::game_rules::pay_energy_amount(name) {
            return Some(format!("pay {}", "{E}".repeat(n.max(0) as usize)));
        }
        // "After the second main phase this turn, there's an additional combat phase
        // followed by an additional main phase" (CR 500.8).
        if let Some((main, nth, parts)) = crate::kw::added_combat_phases::parse_after(name) {
            let phase = if main { "main" } else { "combat" };
            let nth = match nth {
                1 => "first",
                2 => "second",
                3 => "third",
                _ => return None,
            };
            let added = match parts.as_slice() {
                [TurnPart::CombatPhase] => "an additional combat phase",
                [TurnPart::CombatPhase, TurnPart::MainPhase] => {
                    "an additional combat phase followed by an additional main phase"
                }
                _ => return None,
            };
            return Some(format!(
                "after the {nth} {phase} phase this turn, there's {added}"
            ));
        }
        // "Those creatures are now attacking that player" (CR 508.7).
        if let Some(r) = name.strip_prefix(crate::kw::combat_history_4::NOW_ATTACKING) {
            let (p, c) = r.split_once(':')?;
            let p: u8 = p.parse().ok()?;
            let c: u8 = c.parse().ok()?;
            let creatures = self.target_mention(c, Case::Subj);
            let player = self.target_mention(p, Case::Obj);
            return Some(format!("{creatures} are now attacking {player}"));
        }
        // "you may reselect which player ~ is attacking" (CR 508.7).
        if name == crate::kw::reselect_attack::RESELECT_SELF_PLAYER {
            let m = self.me();
            return Some(format!("you may reselect which player {m} is attacking"));
        }
        // "If ~ and a creature named Phyrexian Dragon Engine are attacking, and you both
        // own and control them, exile them, then meld them into ~ Lost to Phyrexia. It
        // enters tapped and attacking." (CR 701.42a).
        if let Some(rest) = name.strip_prefix(crate::merge::MELD_ATTACKING_PAIR) {
            let (partner, result) = self.info.meld.clone()?;
            let m = self.me();
            let mut s = format!(
                "if {m} and {partner} are attacking, and you both own and control them, exile them, then meld them into {result}"
            );
            if rest == crate::merge::MELD_ENTERS_ATTACKING {
                s.push_str(". It enters tapped and attacking");
            } else if !rest.is_empty() {
                return None;
            }
            return Some(s);
        }
        Some(match name {
            crate::kwa::vote::CHOOSES_VOTES => "you choose how each player votes this turn".into(),
            // Secret choices (`choices.rs`).
            crate::choices::REVEAL_CHOICES => {
                "{alt:those choices|the choices|those numbers|the numbers} are revealed".into()
            }
            crate::choices::REVEAL_CHOSEN_PLAYER => "reveal the player you chose".into(),
            crate::choices::REVEAL_CHOSEN_TYPE => "reveal the creature type you chose".into(),
            // Reinterpret.
            // "equal or lesser" than the mana value of the spell's one target.
            n if n.starts_with("cast from hand free with mana value at most that of target:") => {
                let slot: usize = n["cast from hand free with mana value at most that of target:".len()..]
                    .parse()
                    .ok()?;
                if slot != 0 || self.targets.len() != 1 {
                    return None;
                }
                "cast a spell with equal or lesser mana value from your hand without paying its mana cost".into()
            }
            // Powerbalance.
            "cast it without paying its mana cost if its spell has the triggering spell's mana value" => {
                "you may cast {alt:it|that card} without paying its mana cost if the two spells have the same mana value".into()
            }
            // Kiora, Sovereign of the Deep (X is the triggering spell's mana value).
            "cast one of the cards looked at without paying its mana cost if its spell has lesser mana value than the triggering spell" => {
                "cast a spell with mana value less than {alt:X|that spell's mana value} from among them without paying its mana cost".into()
            }
            "cast that card paying life equal to its mana value" => {
                "cast {alt:it|that card} by paying life equal to the spell's mana value rather than paying its mana cost".into()
            }
            n if n.starts_with("cast paying life equal to its mana value:") => {
                let rest = &n["cast paying life equal to its mana value:".len()..];
                let (zones, types) = rest.split_once(':')?;
                let z = zones_phrase(zones, "or")?;
                format!(
                    "cast {} from {z} by paying life equal to its mana value rather than paying its mana cost",
                    spell_kinds(types, false)
                )
            }
            // "You may cast up to two instant and/or sorcery spells with total mana value 6
            // or less from your graveyard and/or hand without paying their mana costs."
            n if n.starts_with("cast spells with total mana value at most:") => {
                let parts: Vec<&str> = n["cast spells with total mana value at most:".len()..]
                    .split(':')
                    .collect();
                let [count, max, zones, types] = parts.as_slice() else {
                    return None;
                };
                let count = match count.parse::<u32>().ok()? {
                    u32::MAX => "any number of".to_string(),
                    k => format!("up to {}", number_word(k as i32)),
                };
                let max = if *max == "x" { "X".to_string() } else { max.to_string() };
                let spells = spell_kinds(types, true);
                let from = if *zones == "linked" {
                    "from among cards exiled with ~".to_string()
                } else {
                    let a = zones_phrase(zones, "and/or")?;
                    let b = zones_phrase(&zones.split(',').rev().collect::<Vec<_>>().join(","), "and/or")?;
                    format!("from {{alt:{a}|{b}}}")
                };
                let total = format!("with total mana value {max} or less");
                if from.contains('{') {
                    format!("cast {count} {spells} {total} {from} without paying their mana costs")
                } else {
                    format!(
                        "cast {count} {spells} {{alt:{total} {from}|{from} {total}}} without paying their mana costs"
                    )
                }
            }
            crate::dice::REROLL_STORED => {
                format!("reroll any number of {} stored results", super::nouns::possessive(&self.me()))
            }
            // Rod of Absorption.
            "exile the triggering spell as it resolves, exiled with the source" => {
                "exile it instead of putting it into a graveyard as it resolves".into()
            }
            "facedown:reveal it" => {
                let t = self.each_target_mention();
                format!("reveal {t}")
            }
            "foretell:it becomes foretold with its mana cost reduced by 2" => {
                "it becomes foretold. Its foretell cost is its mana cost reduced by {2}".into()
            }
            // Bookkeeping: the order of objects an instruction is done for ("for each card
            // exiled this way, copy it"), chosen by the controller (CR 101.4).
            "order the affected objects" => String::new(),
            "reselect attack of each attacking creature" => {
                "for each attacking creature, you may reselect which player or permanent that creature is attacking".into()
            }
            n if n.starts_with("reselect attack of target:") => {
                let i: u8 = n["reselect attack of target:".len()..].parse().ok()?;
                let t = self.target_mention(i, Case::Subj);
                format!("reselect which player or permanent {t} is attacking")
            }
            "reveal a card at random from hand:you" => {
                "reveal a card at random from your hand".into()
            }
            "room: lock or unlock a door" => {
                let t = self.each_target_mention();
                format!("lock or unlock a door of {t}")
            }
            "room: unlock a locked door" => {
                let t = self.each_target_mention();
                format!("unlock a locked door of {t}")
            }
            "target becomes blocked" => {
                let t = self.each_target_mention();
                format!("{t} becomes blocked")
            }
            "visit:claim the prize" => "claim the prize".into(),
            crate::kw::basic_effects::CLASH_WITH_DEFENDING_PLAYER => {
                "clash with defending player".into()
            }
            // "Look at the top five cards of target opponent's library, then put them back
            // in any order", "... then exile one of them".
            n if n.starts_with("basic_effects:look at top:") => {
                let l: crate::kw::basic_effects::LookAtTop =
                    serde_json::from_str(&n["basic_effects:look at top:".len()..]).ok()?;
                let lib = self.player(&l.library, Case::Poss);
                let looker = self.player(&l.looker, Case::Subj);
                let cards = match &l.n {
                    Value::Const(1) => format!("the top card of {lib} library"),
                    Value::Const(k) => format!("the top {} cards of {lib} library", number_word(*k)),
                    other => {
                        let v = self.value(other);
                        format!("the top {v} cards of {lib} library")
                    }
                };
                let mut s = format!("look at {cards}");
                if l.exile > 0 {
                    s.push_str(&format!(", then exile {} of them", number_word(l.exile as i32)));
                }
                if l.reorder {
                    let pron = if l.exile > 0 { "the rest" } else { "them" };
                    s.push_str(&format!(", then put {pron} back in any order"));
                }
                if looker == "you" {
                    s
                } else {
                    format!("{looker} {}", super::effects::third_person(&s).replace(", then put ", ", then puts ").replace(", then exile ", ", then exiles "))
                }
            }
            // Scour: "Search its controller's graveyard, hand, and library for all cards
            // with the same name as that enchantment and exile them."
            n if n.starts_with("search graveyard hand library same name:") => {
                let slot: u8 = n["search graveyard hand library same name:".len()..]
                    .split(':')
                    .next()?
                    .parse()
                    .ok()?;
                let t = self.target_mention(slot, Case::Obj);
                format!("search its controller's graveyard, hand, and library for all cards with the same name as {t}")
            }
            // "exile them, then meld them into [result]" (CR 701.42a).
            n if n.starts_with(crate::merge::MELD_EFFECT) => {
                let named = &n[crate::merge::MELD_EFFECT.len()..];
                let result = if named.is_empty() {
                    self.info.meld.clone()?.1
                } else {
                    named.to_string()
                };
                format!("exile them, then meld them into {result}")
            }
            _ => return None,
        })
    }
}
