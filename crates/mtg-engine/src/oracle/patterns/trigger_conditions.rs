//! Intervening "if" conditions of triggered abilities (CR 603.4) that look at the game or
//! at what happened this turn or last turn: "if you attacked this turn", "if a creature
//! died this turn", "if no spells were cast last turn" (werewolves), "if ~ is tapped",
//! "if there are four or more card types among cards in your graveyard".

use crate::ability::*;
use crate::oracle::patterns::ConditionPattern;
use crate::oracle::phrases::*;

inventory::submit! {
    ConditionPattern { name: "turn history and state conditions", priority: 100, parse: parse_condition }
}

fn custom(name: &str) -> Condition {
    Condition::Custom(name.into())
}

fn parse_condition(c: &str) -> Option<Condition> {
    let c = end(c);
    Some(match c {
        "~ is tapped" => Condition::SelMatches(Sel::This, Filter::Tapped),
        "~ is untapped" => Condition::SelMatches(Sel::This, Filter::Untapped),
        "a creature died this turn" => {
            Condition::Compare(Value::CreaturesDiedThisTurn, Cmp::Gt, Value::c(0))
        }
        "you gained life this turn" => Condition::Compare(
            Value::LifeGainedThisTurn(PlayerRef::You),
            Cmp::Gt,
            Value::c(0),
        ),
        "you lost life this turn" => Condition::Compare(
            Value::LifeLostThisTurn(PlayerRef::You),
            Cmp::Gt,
            Value::c(0),
        ),
        "you attacked this turn" | "you attacked with a creature this turn" => {
            custom("you_attacked_this_turn")
        }
        "a permanent left the battlefield under your control this turn"
        | "a permanent you controlled left the battlefield this turn" => {
            custom("permanent_you_controlled_left_this_turn")
        }
        "a creature died under your control this turn" => {
            custom("creature_died_under_your_control_this_turn")
        }
        "you descended this turn" => custom("you_descended_this_turn"),
        "a card left your graveyard this turn" => custom("card_left_your_graveyard_this_turn"),
        "you've cast a noncreature spell this turn" => {
            custom("you_cast_noncreature_spell_this_turn")
        }
        "no spells were cast last turn" => custom("no_spells_cast_last_turn"),
        "a player cast two or more spells last turn" => {
            custom("a_player_cast_two_spells_last_turn")
        }
        "you lost life last turn" => custom("you_lost_life_last_turn"),
        // CR 400.7d: the spell that became this permanent.
        "you cast it from your hand" | "you cast ~ from your hand" => {
            Condition::CastFrom(ZoneKind::Hand)
        }
        "you cast it from exile" | "you cast ~ from exile" => Condition::CastFrom(ZoneKind::Exile),
        "you cast it from your graveyard" | "you cast ~ from your graveyard" => {
            Condition::CastFrom(ZoneKind::Graveyard)
        }
        "no creatures are on the battlefield" => {
            Condition::Not(Box::new(Condition::Exists(Filter::creature())))
        }
        "you control your commander" => Condition::Exists(Filter::and(vec![
            Filter::Commander,
            Filter::OwnedBy(PlayerRel::You),
            Filter::ControlledBy(PlayerRel::You),
        ])),
        _ => return parse_counted(c),
    })
}

/// "you gained N or more life this turn", "there are N or more card types among cards in
/// your graveyard", "there are N or more creature cards in your graveyard", "you control
/// no untapped lands".
fn parse_counted(c: &str) -> Option<Condition> {
    if let Some(r) = c.strip_prefix("you gained ") {
        let (n, r) = parse_number(r)?;
        if end(r) != "or more life this turn" {
            return None;
        }
        return Some(Condition::Compare(
            Value::LifeGainedThisTurn(PlayerRef::You),
            Cmp::Ge,
            n,
        ));
    }
    if let Some(r) = c.strip_prefix("you control no ") {
        let (f, _, tail) = parse_object_phrase(r)?;
        if !end(tail).is_empty() {
            return None;
        }
        return Some(Condition::Not(Box::new(Condition::Exists(f.you_control()))));
    }
    let r = c.strip_prefix("there are ")?;
    let (n, r) = parse_number(r)?;
    let r = r.trim_start().strip_prefix("or more ")?;
    let in_gy = || {
        Filter::and(vec![
            Filter::InZone(ZoneKind::Graveyard),
            Filter::OwnedBy(PlayerRel::You),
        ])
    };
    if r == "card types among cards in your graveyard" {
        // CR 205.2a card types; delirium-style counts.
        return Some(Condition::Compare(
            Value::CardTypesAmong(in_gy()),
            Cmp::Ge,
            n,
        ));
    }
    let phrase = r.strip_suffix(" in your graveyard")?;
    let filter = if phrase == "cards" {
        Filter::Any
    } else {
        let (f, plural, tail) = parse_object_phrase(phrase)?;
        if !plural || !end(tail).is_empty() {
            return None;
        }
        f
    };
    Some(Condition::Compare(
        Value::CardsInGraveyard(PlayerRef::You, filter),
        Cmp::Ge,
        n,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conditions_parse() {
        for c in [
            "you attacked this turn",
            "a creature died this turn",
            "no spells were cast last turn",
            "a player cast two or more spells last turn",
            "~ is tapped",
            "you gained 3 or more life this turn",
            "there are four or more card types among cards in your graveyard",
            "there are three or more creature cards in your graveyard",
            "you control no untapped lands",
        ] {
            assert!(parse_condition(c).is_some(), "{c}");
        }
    }
}
