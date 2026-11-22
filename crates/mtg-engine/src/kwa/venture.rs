//! CR 701.49: venture into the dungeon (the process itself is in `dungeons.rs`).
//!
//! * Without a dungeon in the command zone, the player chooses a dungeon card they own
//!   from outside the game, puts it into the command zone and their venture marker on its
//!   topmost room (CR 701.49a).
//! * Otherwise they move the marker along an arrow of their choice (CR 701.49b), or, from
//!   the bottommost room, complete the dungeon and start another (CR 701.49c).
//! * "Venture into [quality]" (CR 701.49d) — "venture into Undercity" — starts only a
//!   dungeon with that quality, and otherwise follows the normal procedure. A dungeon that
//!   says "You can't enter this dungeon unless you 'venture into [this dungeon]'"
//!   ([`ONLY_BY_VENTURING_INTO_IT`]) is never started by venturing into the dungeon.
//!
//! The quality is given as the name in `Sel::All(Filter::Named(name))`.

use super::*;

/// `StaticEffect::Custom` name: "You can't enter this dungeon unless you 'venture into
/// [this dungeon].'"
pub const ONLY_BY_VENTURING_INTO_IT: &str = "dungeon: enter only by venturing into it";

/// Whether the dungeon card `o` can be entered only by venturing into it by name.
pub fn only_by_venturing_into_it(o: &crate::object::GameObject) -> bool {
    let chars = o.card.as_ref().map(|c| &c.front().chars).unwrap_or(&o.chars);
    chars.abilities.iter().any(|a| {
        matches!(&a.kind, AbilityKind::Static(s)
            if matches!(&s.effect, StaticEffect::Custom(n) if n.as_str() == ONLY_BY_VENTURING_INTO_IT))
    })
}

/// The instruction "venture into [name]".
pub fn venture_into(name: &str) -> Effect {
    Effect::KeywordAction {
        action: KeywordAction::Venture,
        who: PlayerRef::You,
        what: Sel::All(Filter::Named(SmolStr::new(name))),
        n: Value::c(1),
    }
}

pub struct Venture;

impl KeywordActionRules for Venture {
    fn actions(&self) -> &'static [KeywordAction] {
        &[KeywordAction::Venture]
    }

    fn perform(&self, g: &mut Game, a: &Args, ctx: &mut Ctx) {
        let name = match a.what {
            Sel::All(Filter::Named(n)) => Some(n.to_string()),
            _ => None,
        };
        for p in g.eval_players(a.who, ctx) {
            crate::dungeons::venture_into(g, p, ctx.source, name.as_deref());
        }
    }
}

inventory::submit! { KeywordActionRegistration(&Venture) }
