//! CR 701.55: face a villainous choice.
//!
//! * "[A player] faces a villainous choice — [option A], or [option B]" means that player
//!   chooses one of the options, then all its actions are performed (CR 701.55a). In the
//!   options, "that player" / "they" is the player facing the choice
//!   (`PlayerRef::Iterated`) and "you" the controller of the effect.
//! * The player may choose an option that's illegal or impossible; as much of it as
//!   possible is done (CR 701.55b).
//! * A replacement effect may have a player face the choice additional times; the whole
//!   process is performed that many times, one at a time (CR 701.55c): the static ability
//!   [`FACE_AGAIN`] ("If an opponent would face a villainous choice, they face that choice
//!   an additional time").
//! * Several players facing a villainous choice each do so in turn, in APNAP order
//!   (CR 701.55d).

use super::*;

/// `StaticEffect::Custom` name: "If an opponent would face a villainous choice, they face
/// that choice an additional time."
pub const FACE_AGAIN: &str = "villainous choice: opponents face it an additional time";
/// `Event::Custom` name reported each time a player faces a villainous choice; the amount
/// is the index of the option they chose.
pub const FACED: &str = "villainous choice";

/// The number of times `p` faces a villainous choice (CR 701.55c).
fn times(g: &Game, p: PlayerId) -> usize {
    1 + g
        .statics
        .customs
        .iter()
        .filter(|(_, ctl, name)| name == FACE_AGAIN && g.are_opponents(*ctl, p))
        .count()
}

/// `p` faces the villainous choice between `options` once (CR 701.55a–b).
pub fn face(g: &mut Game, p: PlayerId, options: &[(String, Effect)], ctx: &mut Ctx) {
    if options.is_empty() {
        return;
    }
    let labels = options.iter().map(|(l, _)| l.clone()).collect();
    let i = g.ask_option(p, ctx.source, "Villainous choice", labels);
    let i = i.min(options.len() - 1);
    g.log(|_| format!("{p} chooses: {}", options[i].0));
    let saved = ctx.iter_player.replace(p);
    g.exec(&options[i].1, ctx);
    ctx.iter_player = saved;
    emit(g, FACED, p, None, i as i32);
}

pub struct VillainousChoice;

impl KeywordActionRules for VillainousChoice {
    fn actions(&self) -> &'static [KeywordAction] {
        &[KeywordAction::VillainousChoice]
    }

    fn perform(&self, g: &mut Game, a: &Args, ctx: &mut Ctx) {
        let Some(spec) = a.spec else {
            return;
        };
        // CR 701.55d: one player at a time, in APNAP order.
        let mut players = g.eval_players(a.who, ctx);
        let order = g.apnap();
        players.sort_by_key(|p| order.iter().position(|q| q == p).unwrap_or(usize::MAX));
        for p in players {
            for _ in 0..times(g, p) {
                if !g.player(p).in_game() {
                    break;
                }
                face(g, p, &spec.options, ctx);
            }
        }
    }
}

inventory::submit! { KeywordActionRegistration(&VillainousChoice) }
