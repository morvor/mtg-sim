//! "Create a 1/1 green Squirrel creature token. Put a +1/+1 counter on that token for each
//! ..." (Animal Friend's granted ability): after an instruction that created tokens, "that
//! token" in the next sentence is the token created (CR 111.1), even inside a triggered
//! ability, whose trigger object "that token" would otherwise name.

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_effect_text, Builder};
use crate::oracle::phrases::end;

fn last_instruction(e: &Effect) -> &Effect {
    match e {
        Effect::Seq(v) => v.last().map_or(e, last_instruction),
        _ => e,
    }
}

fn that_token_after_create(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l).trim();
    if !matches!(
        last_instruction(prev),
        Effect::CreateToken { .. } | Effect::CreateTokenWithPT { .. }
    ) || !l.contains(" that token")
        || l.starts_with("when ")
        || l.starts_with("if ")
    {
        return false;
    }
    let text = l.replace(" that token's ", " its ").replace(" that token", " it");
    let mut sub = Builder::new(b.ctx);
    sub.in_trigger = b.in_trigger;
    sub.it = Sel::Var(vars::CREATED);
    sub.its_is_it = true;
    let Some(e) = parse_effect_text(&text, &mut sub) else {
        return false;
    };
    // Targets of its own would belong to the ability's targets.
    if !sub.targets.is_empty() {
        return false;
    }
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, e]);
    true
}

inventory::submit! { FollowupPattern { name: "\"that token\" after creating a token", priority: 10, apply: that_token_after_create } }
