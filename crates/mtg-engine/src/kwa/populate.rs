//! CR 701.36: populate.
//!
//! * To populate, choose a creature token you control and create a token that's a copy of
//!   it (CR 701.36a); the copy is created the way any token copy is (CR 707, 111.10).
//! * If you control no creature tokens, no token is created (CR 701.36b).

use super::*;

/// The creature token chosen to populate (for the copy instruction).
const CHOSEN: Var = vars::USER + 1036;

/// `p` populates. Returns the tokens created.
pub fn populate(g: &mut Game, p: PlayerId, ctx: &mut Ctx) -> Vec<ObjectId> {
    if g.dirty {
        g.recompute();
    }
    let tokens: Vec<ObjectId> = g
        .permanents()
        .filter(|o| o.controller == p && o.is_token() && o.is_creature())
        .map(|o| o.id)
        .collect();
    let Some(first) = tokens.first().copied() else {
        return vec![];
    };
    let pick = g
        .ask_objects(
            p,
            ctx.source,
            "Populate: choose a creature token",
            tokens.clone(),
            1,
            1,
        )
        .first()
        .copied()
        .unwrap_or(first);
    let mut c = ctx.clone();
    c.controller = p;
    c.set_var(CHOSEN, vec![Entity::Object(pick)]);
    g.exec(
        &Effect::CreateTokenCopy {
            of: Sel::Var(CHOSEN),
            count: Value::c(1),
            controller: PlayerRef::You,
            tapped: false,
            attacking: false,
            mods: vec![],
        },
        &mut c,
    );
    c.var_objects(vars::CREATED)
}

pub struct Populate;

impl KeywordActionRules for Populate {
    fn actions(&self) -> &'static [KeywordAction] {
        &[KeywordAction::Populate]
    }

    fn perform(&self, g: &mut Game, a: &Args, ctx: &mut Ctx) {
        let times = number(g, a.n, ctx);
        let mut created = Vec::new();
        for p in g.eval_players(a.who, ctx) {
            for _ in 0..times {
                created.extend(populate(g, p, ctx));
            }
        }
        ctx.prev_happened = !created.is_empty();
        ctx.set_var(
            vars::CREATED,
            created.into_iter().map(Entity::Object).collect(),
        );
    }
}

inventory::submit! { KeywordActionRegistration(&Populate) }
