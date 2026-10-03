//! Oracle patterns for controlling another player (CR 723):
//!
//! * "You control target player during that player's next turn." (Mindslaver, Worst
//!   Fears, Sorin Markov) and "you gain control of target opponent during that player's
//!   next turn. After that turn, that player takes an extra turn." (Emrakul, the Promised
//!   End);
//! * "You control target opponent during their next combat phase. If ~'s additional cost
//!   was paid, you control that player during their next turn instead." (Secret of
//!   Bloodbending);
//! * "You control your opponents while they're searching their libraries." (Opposition
//!   Agent);
//! * Word of Command's instructions (CR 723.7).

use super::{EffectPattern, FollowupPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;
use crate::player_control::{
    control_effect, parse_control_effect, word_of_command, ControlSpan, ADDITIONAL_COST_PAID,
    CONTROL_WHILE_SEARCHING, WORD_OF_COMMAND,
};
use smol_str::SmolStr;

fn control_target_player(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l)
        .strip_prefix("you control ")
        .or_else(|| end(l).strip_prefix("you gain control of "))?;
    let (filter, text, r) = if let Some(r) = r.strip_prefix("target player ") {
        (PlayerFilter::Any, "target player", r)
    } else if let Some(r) = r.strip_prefix("target opponent ") {
        (PlayerFilter::Opponent, "target opponent", r)
    } else {
        return None;
    };
    let span = match r {
        "during that player's next turn" | "during their next turn" => ControlSpan::NextTurn,
        "during their next combat phase" | "during that player's next combat phase" => {
            ControlSpan::NextCombatPhase
        }
        _ => return None,
    };
    let slot = b.add_target(TargetSpec::player(filter, text), text);
    b.it_player = PlayerRef::Target(slot);
    Some(control_effect(span, slot, false))
}

inventory::submit! { EffectPattern { name: "r723 control target player during their next turn", priority: 60, parse: control_target_player } }

/// "After that turn, that player takes an extra turn." / "If ~'s additional cost was
/// paid, you control that player during their next turn instead."
fn control_followup(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let Effect::Custom(name) = prev else {
        return false;
    };
    let Some((span, slot, extra)) = parse_control_effect(name) else {
        return false;
    };
    match end(l) {
        "after that turn, that player takes an extra turn" if span == ControlSpan::NextTurn => {
            *prev = control_effect(span, slot, true);
            true
        }
        "if ~'s additional cost was paid, you control that player during their next turn instead"
        | "if this spell's additional cost was paid, you control that player during their next turn instead"
            if span == ControlSpan::NextCombatPhase =>
        {
            *prev = Effect::If {
                cond: Condition::Custom(SmolStr::new(ADDITIONAL_COST_PAID)),
                then: Box::new(control_effect(ControlSpan::NextTurn, slot, extra)),
                otherwise: Box::new(prev.clone()),
            };
            true
        }
        _ => false,
    }
}

inventory::submit! { FollowupPattern { name: "r723 player control followups", priority: 60, apply: control_followup } }

/// "You control your opponents while they're searching their libraries."
fn control_while_searching(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    if end(l) != "you control your opponents while they're searching their libraries" {
        return None;
    }
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Custom(SmolStr::new(
            CONTROL_WHILE_SEARCHING,
        )))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "r723 control opponents while searching", priority: 60, parse: control_while_searching } }

/// The target slot "that player" refers to.
fn that_player_slot(b: &Builder) -> Option<u8> {
    match b.it_player {
        PlayerRef::Target(slot) => Some(slot),
        _ => None,
    }
}

/// Word of Command's instructions: "Look at target opponent's hand and choose a card from
/// it.", "You control that player until ~ finishes resolving.", "The player plays that
/// card if able.", "If the chosen card is cast as a spell, you control the player while
/// that spell is resolving."
fn word_of_command_steps(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if l == "look at target opponent's hand and choose a card from it" {
        let slot = b.add_target(
            TargetSpec::player(PlayerFilter::Opponent, "target opponent"),
            "target opponent",
        );
        b.it_player = PlayerRef::Target(slot);
        return Some(word_of_command("choose", slot));
    }
    let step = match l {
        "you control that player until ~ finishes resolving" => "control",
        "the player plays that card if able" => "play",
        "if the chosen card is cast as a spell, you control the player while that spell is resolving" => {
            "spell"
        }
        _ => return None,
    };
    Some(word_of_command(step, that_player_slot(b)?))
}

inventory::submit! { EffectPattern { name: "r723 word of command", priority: 60, parse: word_of_command_steps } }

/// "While doing so, the player can activate mana abilities only if they're from lands that
/// player controls and only if mana they produce is spent to activate other mana abilities
/// of lands the player controls and/or to play that card."
fn lands_only_while_playing(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if end(l)
        != "while doing so, the player can activate mana abilities only if they're from lands that player controls and only if mana they produce is spent to activate other mana abilities of lands the player controls and/or to play that card"
    {
        return false;
    }
    let target = match prev {
        Effect::Seq(v) => v.last_mut(),
        e => Some(e),
    };
    let Some(Effect::Custom(name)) = target else {
        return false;
    };
    let Some(slot) = name.strip_prefix(&format!("{WORD_OF_COMMAND}play:")) else {
        return false;
    };
    let Ok(slot) = slot.parse::<u8>() else {
        return false;
    };
    *name = SmolStr::new(format!("{WORD_OF_COMMAND}play-lands:{slot}"));
    true
}

inventory::submit! { FollowupPattern { name: "r723 word of command mana restriction", priority: 60, apply: lands_only_while_playing } }
