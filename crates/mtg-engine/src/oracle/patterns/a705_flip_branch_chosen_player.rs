//! "Flip a coin. If you lose the flip, choose one of your opponents. That player gains
//! control of ~." (Goblin Festival): the sentence about the player chosen in the "if you
//! win/lose the flip" part continues that part (CR 705.2), so it happens only if the
//! player was chosen.

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_sentence, Builder};

/// The coin flip the effect ends with.
fn last_flip(e: &mut Effect) -> Option<&mut crate::dice::CoinFlip> {
    match e {
        Effect::Seq(v) => v.last_mut().and_then(last_flip),
        Effect::FlipCoins(spec) => Some(spec),
        _ => None,
    }
}

fn ends_choosing_player(e: &Effect) -> bool {
    match e {
        Effect::Seq(v) => v.last().is_some_and(ends_choosing_player),
        Effect::Choose {
            kind: ChoiceKind::Opponent | ChoiceKind::Player,
            ..
        } => true,
        _ => false,
    }
}

fn flip_branch_chosen_player(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if !l.starts_with("that player ") {
        return false;
    }
    let Some(spec) = last_flip(prev) else {
        return false;
    };
    let branch = if ends_choosing_player(&spec.on_lose) {
        &mut spec.on_lose
    } else if ends_choosing_player(&spec.on_win) {
        &mut spec.on_win
    } else {
        return false;
    };
    let old_player = b.it_player.clone();
    b.it_player = PlayerRef::ChosenOpponent;
    let Some(e) = parse_sentence(l, b) else {
        b.it_player = old_player;
        return false;
    };
    let old = std::mem::replace(branch, Effect::Noop);
    *branch = Effect::seq(vec![old, e]);
    true
}

inventory::submit! { FollowupPattern { name: "a705 flip: that player (chosen in a win/lose part)", priority: 30, apply: flip_branch_chosen_player } }
