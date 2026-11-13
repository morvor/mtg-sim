//! CR 701.53: incubate.
//!
//! * To incubate N, create an Incubator token that enters the battlefield with N +1/+1
//!   counters on it (CR 701.53a): the counters are placed as it enters (CR 122.6).
//! * An Incubator token is a double-faced token: its front face is a colorless Incubator
//!   artifact with "{2}: Transform this token."; its back face is a 0/0 colorless
//!   Phyrexian artifact creature named Phyrexian Token (CR 701.53b, 111.10i; see
//!   `tokens_predefined::incubator_card`).

use super::*;
use crate::types::counters;

/// `p` incubates N. Returns the Incubator tokens created.
pub fn incubate(g: &mut Game, p: PlayerId, n: u32, ctx: &mut Ctx) -> Vec<ObjectId> {
    let Some(spec) = crate::tokens::predefined("Incubator") else {
        return vec![];
    };
    let with = if n > 0 {
        vec![(SmolStr::new(counters::PLUS1), n)]
    } else {
        vec![]
    };
    let prev = std::mem::replace(&mut g.token_counters, with);
    let mut c = ctx.clone();
    c.controller = p;
    g.exec(
        &Effect::CreateToken {
            spec,
            count: Value::c(1),
            controller: PlayerRef::You,
            tapped: false,
            attacking: false,
        },
        &mut c,
    );
    g.token_counters = prev;
    c.var_objects(vars::CREATED)
}

pub struct Incubate;

impl KeywordActionRules for Incubate {
    fn actions(&self) -> &'static [KeywordAction] {
        &[KeywordAction::Incubate]
    }

    fn perform(&self, g: &mut Game, a: &Args, ctx: &mut Ctx) {
        let n = number(g, a.n, ctx);
        let mut created = Vec::new();
        for p in g.eval_players(a.who, ctx) {
            created.extend(incubate(g, p, n, ctx));
        }
        ctx.set_var(
            vars::CREATED,
            created.into_iter().map(Entity::Object).collect(),
        );
    }
}

inventory::submit! { KeywordActionRegistration(&Incubate) }
