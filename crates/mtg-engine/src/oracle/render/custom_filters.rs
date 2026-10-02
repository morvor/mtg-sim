//! Object qualities implemented in code (`Filter::Custom`), put into words from what each
//! one checks (see the constant each name is defined by).

use super::players::Case;
use super::*;
use crate::kw::hand_graveyard_actions as hga;

/// How a custom quality is worded.
pub(crate) enum CustomQuality {
    /// After the noun: "creature that dealt damage to you this turn".
    Rel(String),
    /// The kind of object, as the head noun: "triggered ability".
    Kind(&'static str),
    /// After the noun, and says where it is: "card exiled with it".
    RelInExile(String),
    /// Says nothing the noun doesn't: a rule of the instruction it's in ("attach it to a
    /// creature": only one it can legally be attached to, CR 701.3b).
    Implied,
}

/// The quality of a custom filter after a noun, if this module knows it.
pub(crate) fn custom_rel(name: &str) -> Option<CustomQuality> {
    use CustomQuality::*;
    let rel = |s: &str| Some(Rel(s.to_string()));
    match name {
        crate::kw::attached_to_creature::ATTACHED_TO_A_CREATURE => {
            rel("{alt:attached to a creature|that's attached to a creature|that are attached to a creature}")
        }
        // "each Equipment attached to it", "it" being each object the effect is about.
        "attached_to_affected" => rel("attached to it"),
        crate::battle::ATTACKED_A_BATTLE_THIS_TURN => rel("that attacked a battle this turn"),
        crate::kw::dealt_damage_to_you::DEALT_DAMAGE_TO_YOU_THIS_TURN => {
            rel("that dealt damage to you this turn")
        }
        crate::kw::noncombat_damage::DEALT_NONCOMBAT_DAMAGE_THIS_TURN => {
            rel("that was dealt noncombat damage this turn")
        }
        crate::game_terms::ACTIVATED_ABILITY => Some(Kind("activated ability")),
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
        crate::stack_ability_filters::X_IN_ACTIVATION_COST => {
            rel("with {X} in its activation cost")
        }
        crate::kwa::vote::GOT_VOTES => rel("with one or more votes"),
        crate::battle::PROTECTED_BY_OPPONENT => rel("an opponent protects"),
        // An object the source can legally be attached to (CR 301.5c, 303.4k, 701.3b).
        crate::attach::SOURCE_CAN_ATTACH | crate::kw::attach_each::CAN_ATTACH_IT => {
            Some(Implied)
        }
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
        n if n.starts_with("came from:") => match &n["came from:".len()..] {
            "Exile" => rel("from exile"),
            "Graveyard" => rel("from your graveyard"),
            "Library" => rel("from the top of your library"),
            _ => None,
        },
        _ => None,
    }
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
        _ => return None,
    };
    Some(if negated { no } else { yes })
}

impl Renderer<'_> {
    /// A custom quality whose name carries data (a JSON filter or player).
    pub(crate) fn custom_rel_with_data(&mut self, name: &str) -> Option<String> {
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
        let atoms: Vec<&Filter> = match f {
            Filter::And(v) => v.iter().collect(),
            other => vec![other],
        };
        let name = atoms.iter().find_map(|a| match a {
            Filter::Custom(n) => Some(n.as_str()),
            _ => None,
        })?;
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
}
