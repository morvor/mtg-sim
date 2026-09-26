//! CR 701.63: endure.
//!
//! * To endure N, the permanent's controller creates an N/N white Spirit creature token
//!   unless they put N +1/+1 counters on that permanent (CR 701.63a). They choose as the
//!   instruction is performed; if counters can't be put on it (it left the battlefield),
//!   they create the token. A permanent that left uses its last known information for who
//!   controlled it.
//! * Enduring 0 does nothing: no counters, no token (CR 701.63b).

use super::*;
use crate::types::counters;

/// `Event::Custom` name reported when a permanent endures.
pub const ENDURED: &str = "endure";

fn spirit(n: u32) -> TokenSpec {
    TokenSpec {
        name: SmolStr::default(),
        colors: ColorSet::single(Color::White),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: vec![SmolStr::new("Spirit")],
        power: Some(n as i32),
        toughness: Some(n as i32),
        abilities: vec![],
        scryfall_name: None,
    }
}

/// `obj` endures N (CR 701.63a–b).
pub fn endure(g: &mut Game, obj: ObjectId, n: u32, ctx: &mut Ctx) {
    if n == 0 {
        return;
    }
    let p = controller_or_last(g, obj);
    let counters_ok = on_battlefield(g, obj);
    let use_counters = counters_ok
        && g.ask_option(
            p,
            ctx.source,
            "Endure",
            vec![
                format!("Put {n} +1/+1 counters on it"),
                format!("Create a {n}/{n} white Spirit creature token"),
            ],
        ) == 0;
    if use_counters {
        g.add_counters(Entity::Object(obj), counters::PLUS1, n, ctx.source);
    } else {
        let mut c = ctx.clone();
        c.controller = p;
        g.exec(
            &Effect::CreateToken {
                spec: spirit(n),
                count: Value::c(1),
                controller: PlayerRef::You,
                tapped: false,
                attacking: false,
            },
            &mut c,
        );
    }
    emit(g, ENDURED, p, Some(obj), n as i32);
}

pub struct Endure;

impl KeywordActionRules for Endure {
    fn actions(&self) -> &'static [KeywordAction] {
        &[KeywordAction::Endure]
    }

    fn perform(&self, g: &mut Game, a: &Args, ctx: &mut Ctx) {
        let n = number(g, a.n, ctx);
        for obj in g.resolve_objects(a.what, ctx) {
            endure(g, obj, n, ctx);
        }
    }
}

inventory::submit! { KeywordActionRegistration(&Endure) }
