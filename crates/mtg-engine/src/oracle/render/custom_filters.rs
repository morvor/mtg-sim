//! Object qualities implemented in code (`Filter::Custom`), put into words from what each
//! one checks (see the constant each name is defined by).

use super::players::Case;
use super::*;
use crate::kw::hand_graveyard_actions as hga;

/// How a custom quality is worded.
pub(crate) enum CustomQuality {
    /// After the noun: "creature that dealt damage to you this turn".
    Rel(String),
    /// Before the noun: "goaded creatures".
    Adj(&'static str),
    /// The kind of object, as the head noun: "triggered ability".
    Kind(&'static str),
    /// After the noun, and says where it is: "card exiled with it".
    RelInExile(String),
    /// Says nothing the noun doesn't: a rule of the instruction it's in ("attach it to a
    /// creature": only one it can legally be attached to, CR 701.3b).
    Implied,
    /// An adjective that says where the object is, and a quality after the noun: "exiled
    /// card used to craft it".
    AdjRelInExile(&'static str, String),
}

/// The quality of a custom filter after a noun, if this module knows it.
pub(crate) fn custom_rel(name: &str) -> Option<CustomQuality> {
    use CustomQuality::*;
    let rel = |s: &str| Some(Rel(s.to_string()));
    match name {
        crate::kw::paper_history::DAMAGED_BY_SOURCE => rel("~it has dealt damage to this game"),
        crate::kw::paper_history::NOT_SACRIFICED => rel("that wasn't sacrificed"),
        crate::kw::paper_history::NOT_FIRST_LAND_PLAYED => {
            rel("that wasn't the first land you played this turn")
        }
        crate::kw::attached_to_creature::ATTACHED_TO_A_CREATURE => {
            rel("{alt:attached to a creature|that's attached to a creature|that are attached to a creature}")
        }
        // "other creatures that spell could target": a legal target for the triggering
        // spell's one target (CR 115.4, 707.10c).
        crate::copy_rules::TRIGGER_SPELL_COULD_TARGET => rel("that spell could target"),
        // "each Equipment attached to it", "it" being each object the effect is about.
        "attached_to_affected" => rel("attached to it"),
        // Noted during the draft (CR 905.2b; `draft.rs`).
        crate::draft::NOTED_NAME => rel("with a name you noted for cards named ~"),
        crate::draft::NOTED_TYPE => rel("of a type you noted for cards named ~"),
        crate::draft::CHOSEN_COLOR => {
            rel("that's one or more of the colors chosen as you drafted cards named ~")
        }
        crate::kw::game_rules_4::COLOR_OUTSIDE_COMMANDER_IDENTITY => {
            rel("that's a color not in your commander's color identity")
        }
        crate::kw::cast_history::USING_SOURCE_MANA => rel("using mana produced by ~"),
        crate::kw::damage_shares_a_color::SHARES_COLOR_WITH_PRODUCED_MANA => {
            rel("that shares a color with the mana that land produced")
        }
        crate::kw::your_commander_terms::SHARES_TYPE_WITH_YOUR_COMMANDER => {
            rel("that shares a creature type with your commander")
        }
        crate::battle::ATTACKED_A_BATTLE_THIS_TURN => rel("that attacked a battle this turn"),
        crate::kw::blocked_this_turn::BLOCKED_BY_REFERENT => {
            rel("{alt:that were blocked by {alt:it|that creature} this turn|that was blocked by {alt:it|that creature} this turn}")
        }
        crate::kw::blocked_this_turn::BLOCKED_BY_TARGET_THIS_COMBAT => {
            rel("that were blocked by that creature this combat")
        }
        crate::kw::blocked_this_turn::BLOCKED_OR_BLOCKED_BY_REFERENT => {
            rel("that blocked or were blocked by {alt:it|that creature} this turn")
        }
        crate::kw::dealt_damage_to_you::DEALT_DAMAGE_TO_YOU_THIS_TURN => {
            rel("that dealt damage to you this turn")
        }
        crate::kw::noncombat_damage::DEALT_NONCOMBAT_DAMAGE_THIS_TURN => {
            rel("that was dealt noncombat damage this turn")
        }
        crate::game_terms::ACTIVATED_ABILITY => Some(Kind("activated ability")),
        crate::oracle::patterns::zone_move_grammar::EVEN_MANA_VALUE | "restrictions:even_mana_value" => {
            rel("with {alt:an even mana value|even mana values}")
        }
        crate::oracle::patterns::zone_move_grammar::ODD_MANA_VALUE | "restrictions:odd_mana_value" => {
            rel("with {alt:an odd mana value|odd mana values}")
        }
        // Cycling a card discards it (CR 702.29a).
        crate::oracle::patterns::zone_move_grammar::DISCARDED_BY_YOU_THIS_TURN => {
            rel("{alt:that you discarded this turn|that you cycled or discarded this turn}")
        }
        crate::oracle::patterns::zone_move_grammar::MILLED_THIS_TURN => rel("milled this turn"),
        crate::kw::graveyard_history::PUT_THERE_FROM_LIBRARY => rel(
            "{alt:that were put there from your library this turn|that were put there from libraries this turn}",
        ),
        crate::kw::graveyard_history::PUT_THERE_NOT_FROM_BATTLEFIELD => {
            rel("that were put there from anywhere other than the battlefield this turn")
        }
        crate::kw::graveyard_history::SURVEILLED_THIS_TURN => rel("you've surveilled this turn"),
        crate::kw::graveyard_history::PUT_THERE_THIS_TURN => rel("this turn"),
        crate::oracle::patterns::zone_move_grammar::DEALT_DAMAGE_THIS_TURN => {
            rel("that dealt damage this turn")
        }
        "restrictions:goaded" => Some(Adj("goaded")),
        crate::game_terms::TRIGGERED_ABILITY => Some(Kind("triggered ability")),
        crate::kw::basic_effects::SECOND_SPELL_CAST_THIS_TURN => {
            rel("{alt:that's the second spell cast this turn|that is the second spell cast this turn}")
        }
        crate::kw::basic_effects::DEALT_DAMAGE_THIS_TURN => rel("that dealt damage this turn"),
        crate::kw::basic_effects::BLOCKED_THIS_TURN => rel("that blocked this turn"),
        crate::kw::basic_effects::BLOCKED_BY_SOURCE_THIS_TURN => rel("~it blocked this turn"),
        // CR 608.2b: the source as it last existed ("target creature ~ is blocking").
        crate::kw::basic_effects::BLOCKED_BY_SOURCE_LKI => rel("~ is blocking"),
        crate::kw::delve::EXILED_WITH_IT => Some(RelInExile("exiled with ~it".into())),
        crate::attach::ENCHANTED_BY_YOUR_AURA => {
            rel("{alt:that are enchanted by Auras you control|that's enchanted by an Aura you control|enchanted by an Aura you control}")
        }
        crate::search_rules::HAS_MANA_ABILITY => rel("with a mana ability"),
        crate::search_rules::ENCHANT_CREATURE => rel("with enchant creature"),
        crate::kw::dig_filters::DOCTORS_COMPANION => rel("with doctor's companion"),
        crate::stack_ability_filters::X_IN_ACTIVATION_COST => {
            rel("with {X} in its activation cost")
        }
        crate::kwa::vote::GOT_VOTES => rel("with one or more votes"),
        crate::battle::PROTECTED_BY_OPPONENT => rel("an opponent protects"),
        // An object the source can legally be attached to (CR 301.5c, 303.4k, 701.3b).
        // "a creature you control it could be attached to": said or not.
        crate::attach::SOURCE_CAN_ATTACH => rel("{opt:it could be attached to}"),
        crate::kw::attach_each::CAN_ATTACH_IT => Some(Implied),
        n if n.starts_with(hga::PUT_THERE_THIS_TURN) => {
            match &n[hga::PUT_THERE_THIS_TURN.len()..] {
                "" => rel("{alt:that was put there this turn|that were put there this turn|that was put there from anywhere this turn|that were put there from anywhere this turn}"),
                "battlefield" => rel("{alt:that was put there from the battlefield this turn|that were put there from the battlefield this turn}"),
                "library" => rel("{alt:that was put there from your library this turn|that were put there from your library this turn}"),
                _ => None,
            }
        }
        // "with base power 1 or less".
        n if n.starts_with("base:p<=") || n.starts_with("base:t<=") => {
            let what = if n.starts_with("base:p") {
                "power"
            } else {
                "toughness"
            };
            let k: i32 = n[8..].parse().ok()?;
            Some(Rel(format!("with base {what} {} or less", number_word(k))))
        }
        // "you may attach an Equipment you control to it": only one that can be
        // attached to it (CR 701.3b).
        crate::kw::attach_choice::CAN_ATTACH_TO_RECIPIENT => Some(Implied),
        crate::kw::attach_choice::ANOTHER_AURA_ATTACHED => rel("with another Aura attached to it"),
        // CR 702.167a: "the exiled cards used to craft it".
        crate::kw::craft::USED_TO_CRAFT => Some(AdjRelInExile("exiled", "used to craft ~it".into())),
        // CR 702.143c.
        crate::kw::foretell::FORETOLD_CARD => Some(Adj("foretold")),
        crate::kwa::harness::HARNESSED => Some(Adj("harnessed")),
        // CR 712.1.
        crate::kw::dig_filters::DOUBLE_FACED => Some(Adj("double-faced")),
        crate::kw::choice_grammar::DEALT_COMBAT_DAMAGE_TO_YOU => {
            rel("that dealt combat damage to you this turn")
        }
        crate::kw::grant_filters::ATTACKING_SAME_AS_SOURCE => {
            rel("attacking the same player or planeswalker")
        }
        crate::kw::permanent_cast_this_turn::YOU_CAST_THIS_TURN => rel("you cast this turn"),
        crate::kw::dealt_damage_to_you::CONTROLLER_DEALT_COMBAT_DAMAGE_BY_SOURCE => {
            rel("whose controller was dealt combat damage by ~ this turn")
        }
        crate::kw::dealt_damage_to_source::DEALT_DAMAGE_TO_SOURCE => {
            rel("that dealt damage to ~it this turn")
        }
        crate::kw::activated_ability_kind::TURNED_FACE_UP_THIS_TURN => {
            rel("that was turned face up this turn")
        }
        crate::kw::activated_ability_kind::ATTACKING_YOU_OR_YOUR_PLANESWALKER => {
            rel("{alt:attacking you and/or planeswalkers you control|that's attacking you or a planeswalker you control}")
        }
        crate::oracle::patterns::noun_phrase_predicates::ATTACKING_YOUR_PLANESWALKER => {
            rel("that's attacking a planeswalker you control")
        }
        n if n.starts_with(crate::kw::spell_cost_grammar::MANA_COST_IS) => Some(Rel(format!(
            "with mana cost {}",
            &n[crate::kw::spell_cost_grammar::MANA_COST_IS.len()..]
        ))),
        crate::kw::crew::CREWED_BY_IT_THIS_TURN => rel("crewed by ~ this turn"),
        // CR 509.1: the creatures the source was declared blocking this combat.
        crate::kw::combat_history_4::SOURCE_BLOCKED_THIS_COMBAT => rel("~it blocked this combat"),
        crate::kw::combat_history_4::NOT_DECLARED => {
            rel("not declared as an attacking or blocking creature this combat")
        }
        // "for each creature that player controls": the player each creature is
        // attacking.
        crate::kw::combat_history_4::CONTROLLED_BY_ITS_DEFENDER => rel("that player controls"),
        // The cards still the same objects in exile (CR 400.7).
        crate::oracle::patterns::referents_values_4::STILL_EXILED_CARD => {
            rel("{alt:that are still exiled|that is still exiled}")
        }
        n if n.starts_with("came from:") => match &n["came from:".len()..] {
            "Exile" => rel("from exile"),
            "Graveyard" => rel("from your graveyard"),
            "Library" => rel("from the top of your library"),
            _ => None,
        },
        n => super::tail_filters::custom_rel_more(n),
    }
}

/// What an object with a custom quality is, after "is": "your Ring-bearer" (CR
/// 701.54a), "a double-faced card" (CR 712.1), "harnessed" (CR 701.64b).
pub(crate) fn custom_predicate(name: &str) -> Option<&'static str> {
    Some(match name {
        crate::kwa::ring::RING_BEARER => "your Ring-bearer",
        crate::kw::dig_filters::DOUBLE_FACED => "a double-faced card",
        crate::kwa::harness::HARNESSED => "harnessed",
        _ => return None,
    })
}

/// A clause saying the subject has a custom quality, for a condition: "you cast it from
/// your hand", "an opponent protects it", "it attacked a battle this turn".
fn custom_clause(name: &str, subj: &str, negated: bool) -> Option<String> {
    let (yes, no) = match name {
        crate::custom::PERMANENT_CAST_FROM_HAND => (
            format!("you cast {subj} from your hand"),
            format!("you didn't cast {subj} from your hand"),
        ),
        crate::custom::PERMANENT_WAS_CAST => (
            format!("you cast {subj}"),
            format!("you didn't cast {subj}"),
        ),
        crate::battle::PROTECTED_BY_OPPONENT => (
            format!("an opponent protects {subj}"),
            format!("no opponent protects {subj}"),
        ),
        crate::battle::ATTACKED_A_BATTLE_THIS_TURN => (
            format!("{subj} attacked a battle this turn"),
            format!("{subj} didn't attack a battle this turn"),
        ),
        crate::kw::noncombat_damage::DEALT_NONCOMBAT_DAMAGE_THIS_TURN => (
            format!("{subj} was dealt noncombat damage this turn"),
            format!("{subj} wasn't dealt noncombat damage this turn"),
        ),
        crate::kw::attached_to_creature::ATTACHED_TO_A_CREATURE => (
            format!("{subj} is attached to a creature"),
            format!("{subj} isn't attached to a creature"),
        ),
        crate::kw::dealt_damage_to_you::DEALT_DAMAGE_TO_YOU_THIS_TURN => (
            format!("{subj} dealt damage to you this turn"),
            format!("{subj} didn't deal damage to you this turn"),
        ),
        _ => return super::named_qualities::custom_clause_named(name, subj, negated),
    };
    Some(if negated { no } else { yes })
}

impl Renderer<'_> {
    /// A custom quality whose name carries data (a JSON filter or player).
    pub(crate) fn custom_rel_with_data(&mut self, name: &str) -> Option<String> {
        // "that shares a creature type with each creature tapped this way".
        if let Some(j) = name.strip_prefix(crate::kw::referents_values_4::SHARES_TYPE_WITH_EACH) {
            let sel: Sel = serde_json::from_str(j).ok()?;
            let s = self.sel(&sel, Case::Obj);
            let s = if s.starts_with("each ") || s.starts_with("all ") {
                s
            } else {
                format!("each of {s}")
            };
            return Some(format!("that shares a creature type with {s}"));
        }
        if let Some(j) = name.strip_prefix("basic_effects:blocked or was blocked by:") {
            let f: Filter = serde_json::from_str(j).ok()?;
            let n = match f {
                Filter::Any => "a creature".to_string(),
                f => self.noun_det(&f, super::nouns::Det::A),
            };
            return Some(format!("that blocked or was blocked by {n} this turn"));
        }
        if let Some(j) = name.strip_prefix("basic_effects:battle protected by:") {
            let p: PlayerRef = serde_json::from_str(j).ok()?;
            return Some(match p {
                PlayerRef::EachOpponent => "{alt:they protect|your opponents protect}".into(),
                PlayerRef::You => "you protect".into(),
                other => {
                    let w = self.player(&other, Case::Subj);
                    format!("{w} protects")
                }
            });
        }
        None
    }

    /// A condition that an object has a custom quality ("if you cast it from your hand"):
    /// the filter is the quality, maybe with the kind of object it implies (a permanent
    /// or a card).
    pub(crate) fn custom_condition_clause(
        &mut self,
        s: &Sel,
        f: &Filter,
        negated: bool,
    ) -> Option<String> {
        if let Some(c) = self.attempt(|r| r.cast_history_clause(s, f, negated)) {
            return Some(c);
        }
        let atoms: Vec<&Filter> = match f {
            Filter::And(v) => v.iter().collect(),
            other => vec![other],
        };
        let mut name = None;
        for a in &atoms {
            match a {
                Filter::Custom(n) if name.is_none() => name = Some(n.as_str()),
                Filter::Permanent | Filter::Card | Filter::Any => {}
                _ => return None,
            }
        }
        let name = name?;
        // Check the name before rendering the subject (which may introduce a target).
        custom_clause(name, "it", negated)?;
        let subj = self.sel(s, Case::Obj);
        custom_clause(name, &subj, negated)
    }

    /// A noun phrase a custom filter makes with the qualities around it: "the top card of
    /// target player's graveyard", "the top creature card of defending player's
    /// graveyard", "~" (the source, or the object it became).
    pub(crate) fn custom_whole_noun(&mut self, f: &Filter) -> Option<String> {
        if let Some(s) = self.top_cards_of_graveyard(f) {
            return Some(s);
        }
        // The object noted for the linked abilities, as it last existed (CR 607.2c,
        // 607.2d).
        if let Some(n) = &self.info.noted {
            let noted = |a: &Filter| matches!(a, Filter::Custom(x) if x == crate::kw::trigger_events::NOTED_AS_IT_LAST_EXISTED);
            let atoms: Vec<&Filter> = match f {
                Filter::And(v) => v.iter().collect(),
                other => vec![other],
            };
            if atoms.iter().any(|a| noted(a))
                && atoms
                    .iter()
                    .all(|a| noted(a) || matches!(a, Filter::Type(_) | Filter::Permanent))
            {
                return Some(n.clone());
            }
        }
        let atoms: Vec<&Filter> = match f {
            Filter::And(v) => v.iter().collect(),
            other => vec![other],
        };
        let name = atoms.iter().find_map(|a| match a {
            Filter::Custom(n) => Some(n.as_str()),
            _ => None,
        })?;
        // "Equipped creature has '{T}, Sacrifice Blazing Torch: ...'": the object granting
        // the ability, named by its name in the quoted ability.
        if name == crate::granted_by::GRANTER && atoms.len() == 1 {
            return Some("~".into());
        }
        if name == hga::SOURCE_OR_NEXT {
            let m = self.me();
            if atoms
                .iter()
                .any(|a| matches!(a, Filter::InZone(ZoneKind::Graveyard)))
            {
                return Some(format!("{m} {{opt:from your graveyard}}"));
            }
            return Some(m);
        }
        if name == crate::kwa::ring::RING_BEARER {
            return Some("your Ring-bearer".into());
        }
        // "If it's paired with a creature, that creature also gets ...".
        if name == crate::kw::soulbond::PAIRED_WITH_TARGET && atoms.len() == 1 {
            return Some("that creature {opt:also}".into());
        }
        // "a spell you've cast": a spell's controller is the player who cast it (CR
        // 601.2a), unless it wasn't cast.
        if name == crate::custom::WAS_CAST {
            let who = atoms.iter().find_map(|a| match a {
                Filter::ControlledBy(r) => Some(*r),
                _ => None,
            })?;
            let rest: Vec<Filter> = atoms
                .iter()
                .filter(|a| {
                    !matches!(a, Filter::ControlledBy(_))
                        && !matches!(a, Filter::Custom(n) if n == crate::custom::WAS_CAST)
                })
                .map(|a| (*a).clone())
                .collect();
            let n = self.noun_det(&Filter::and(rest), super::nouns::Det::A);
            let w = self.rel_subject(who);
            let have = if w == "you" { "you've" } else { "has" };
            let have = if w == "you" {
                have.to_string()
            } else {
                format!("{w} {have}")
            };
            return Some(format!("{n} {have} cast"));
        }
        if name == "soulbond:the creature this is paired with" {
            return Some("a creature ~it is paired with".into());
        }
        let which = if name == hga::TOP_OF_GRAVEYARD {
            ("top", None)
        } else if name == hga::BOTTOM_OF_GRAVEYARD {
            ("bottom", None)
        } else if let Some(t) = name.strip_prefix(crate::kw::graveyard_order::TOPMOST_IN_GRAVEYARD)
        {
            ("top", Some(t.to_lowercase()))
        } else {
            return None;
        };
        let mut owner = None;
        for a in &atoms {
            match a {
                Filter::OwnedBy(r) => owner = Some(*r),
                Filter::Custom(_) | Filter::InZone(ZoneKind::Graveyard) | Filter::Card => {}
                Filter::Type(t) if which.1.as_deref() == Some(&t.word().to_lowercase()) => {}
                _ => return None,
            }
        }
        let whose = match owner {
            Some(r) => self.rel_possessive(r, Num::One),
            None => "a".into(),
        };
        let kind = match &which.1 {
            Some(t) => format!("{t} card"),
            None => "card".into(),
        };
        Some(format!("the {} {kind} of {whose} graveyard", which.0))
    }

    /// "the total power of the exiled cards used to craft it", "the mana value of the
    /// exiled card used to craft it" (CR 702.167a).
    pub(crate) fn craft_value(&mut self, v: &Value) -> Option<String> {
        let crafted = |s: &Sel| {
            matches!(s, Sel::All(Filter::And(p))
                if p.iter().any(|x| matches!(x, Filter::Custom(n) if n == crate::kw::craft::USED_TO_CRAFT)))
        };
        match v {
            Value::Aggregate(AggOp::Sum, Stat::Power, s) if crafted(s) => {
                Some("the total power of the exiled cards used to craft ~it".into())
            }
            Value::ManaValueOf(s) if crafted(s) => {
                Some("the mana value of the exiled card used to craft ~it".into())
            }
            _ => None,
        }
    }
}
