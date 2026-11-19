//! Who orders one player's objects when an instruction is performed for each of several
//! objects one at a time (CR 608.2f; register `per-object-actions-in-given-order`,
//! decided by the user, Nov 19).
//!
//! The players' groups go in APNAP order (608.2f's primary order; `simultaneous.rs`). Within
//! one player's group, before each object is considered, a player chooses which of the
//! group's remaining objects comes next, knowing what the earlier ones did (608.2f's
//! Soulfire Eruption example: "they first choose which target they are considering, then
//! ..."):
//!
//! * the player the instruction has perform it for that object ("its controller reveals
//!   cards ...", Chaotic Transformation and its ruling: "that player first chooses which
//!   target they're considering"), ordering their own repetitions of it;
//! * otherwise the player performing the instruction: the controller of the resolving
//!   spell or ability (608.2f; Kaboom!, whose caster chooses the next target as it
//!   resolves), or, inside "each player ...", that player.
//!
//! Nobody is asked when the order can't matter: when the instruction is the same for every
//! object (it doesn't refer to the object beyond naming who performs it, as Divergent
//! Transformations' "its controller reveals cards ... until they reveal a creature card"),
//! or when it's performed for all of them at the same time. The default answer keeps the
//! order given (register `default-order-as-listed`).

use crate::ability::{Effect, PlayerRef, Var};
use crate::eval::Ctx;
use crate::game::Game;
use crate::types::{Entity, ObjectId, PlayerId};

/// The groups of a "for each object" instruction's objects, and who orders each group.
#[derive(Clone, Debug)]
pub(crate) struct ObjectOrder {
    /// The variable each object is bound to.
    pub var: Var,
    /// Each object, its group (the APNAP position of the player it belongs to) and the
    /// player who chooses the next object of that group.
    groups: Vec<(Entity, usize, PlayerId)>,
}

impl ObjectOrder {
    /// The groups of `objects` (each bound to `var`), as `simultaneous::for_each_object`
    /// sorts them: by the player `deciding` names for the object if the body names one
    /// ("its controller ..."), otherwise by the object's controller (a player: itself).
    pub fn new(
        g: &Game,
        objects: &[Entity],
        var: Var,
        deciding: Option<&PlayerRef>,
        ctx: &Ctx,
    ) -> ObjectOrder {
        let order = g.pregame_order_or_apnap();
        let pos = |p: PlayerId| order.iter().position(|x| *x == p).unwrap_or(usize::MAX);
        let groups = objects
            .iter()
            .map(|&e| {
                let (key, chooser) = match deciding {
                    Some(who) => {
                        let mut c = ctx.clone();
                        c.vars.insert(var, vec![e]);
                        match g.eval_player(who, &c) {
                            Some(p) => (pos(p), p),
                            None => (usize::MAX, ctx.controller),
                        }
                    }
                    None => {
                        let key = match e {
                            Entity::Player(p) => pos(p),
                            Entity::Object(o) => g.try_obj(o).map_or(0, |x| pos(x.controller)),
                        };
                        (key, ctx.controller)
                    }
                };
                (e, key, chooser)
            })
            .collect();
        ObjectOrder { var, groups }
    }

    /// The group of `e` and the player who orders it.
    pub fn group_of(&self, e: Entity) -> Option<(usize, PlayerId)> {
        self.groups
            .iter()
            .find(|(x, ..)| *x == e)
            .map(|&(_, k, p)| (k, p))
    }
}

/// Whether performing `e` for an object depends on which object it is (so the order of
/// one player's objects can matter): `e` refers to the object's variable other than in
/// naming who performs it or in only recording it ("flip a coin for each creature. Destroy
/// each creature whose coin comes up tails", Rakdos, the Showstopper: the flips don't
/// depend on the creature, which is only noted for the later destruction).
pub(crate) fn depends_on(e: &Effect, var: Var) -> bool {
    fn mentions(j: &serde_json::Value, var: Var) -> bool {
        match j {
            serde_json::Value::Object(m) => m.iter().any(|(k, x)| {
                (k == "Var" && x.as_u64().is_some_and(|n| n == u64::from(var)))
                    || (!matches!(k.as_str(), "Store" | "StoreValue" | "Note") && mentions(x, var))
            }),
            serde_json::Value::Array(a) => a.iter().any(|x| mentions(x, var)),
            _ => false,
        }
    }
    match e {
        Effect::AsPlayer { effect, .. } => depends_on(effect, var),
        e => serde_json::to_value(e).is_ok_and(|j| mentions(&j, var)),
    }
}

/// `chooser` chooses which of `options` (one player's remaining objects) is considered
/// next; returns its index. The first by default (`default-order-as-listed`).
pub(crate) fn choose_next(
    g: &mut Game,
    chooser: PlayerId,
    source: Option<ObjectId>,
    options: &[Entity],
) -> usize {
    let pick = g.ask_entities(
        chooser,
        source,
        "Choose the one to consider next",
        options.to_vec(),
        1,
        1,
    );
    pick.first()
        .and_then(|e| options.iter().position(|x| x == e))
        .unwrap_or(0)
}
