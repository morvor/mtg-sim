//! CR 107.3f: X in a triggered ability's text that the ability doesn't define is chosen by
//! its controller "at the appropriate time". When the ability's targets depend on X
//! ("you may pay {X}. If you do, return another target Zombie creature card with mana
//! value X", Relentless Dead), that time is as it's put on the stack, before its targets
//! are chosen (CR 603.3d, 601.2b–c); the {X} paid as it resolves uses that value.

use crate::ability::{Body, Effect, TargetKind};
use crate::eval::Ctx;
use crate::game::Game;

fn pays_x_on_resolution(e: &Effect) -> bool {
    match e {
        Effect::PayOptional { cost, .. } => cost.mana.as_ref().is_some_and(|m| m.has_x()),
        Effect::Seq(v) => v.iter().any(pays_x_on_resolution),
        _ => false,
    }
}

fn targets_mention_x(body: &Body) -> bool {
    body.targets.iter().any(|t| match &t.what {
        TargetKind::Object(f) | TargetKind::ObjectOrPlayer(f, _) | TargetKind::Spell(f) => {
            crate::casting::filter_mentions_x(f)
        }
        _ => false,
    })
}

/// Chooses X for a triggered ability being put on the stack whose targets depend on an X
/// it pays as it resolves. Returns the value chosen, or `None` if the ability isn't such.
pub fn choose_before_targets(g: &mut Game, body: &Body, ctx: &mut Ctx) -> Option<i32> {
    if !targets_mention_x(body) || !pays_x_on_resolution(&body.effect) {
        return None;
    }
    let p = ctx.controller;
    let max = g.max_mana_available(p) as i64;
    let src = ctx.source.unwrap_or(crate::types::ObjectId(0));
    let x = match g.ask(
        p,
        crate::decision::Decision::ChooseX {
            source: src,
            min: 0,
            max,
        },
    ) {
        crate::decision::Answer::Number(n) if n >= 0 => n.min(max) as i32,
        _ => 0,
    };
    ctx.x = x;
    ctx.x_defined = true;
    Some(x)
}
