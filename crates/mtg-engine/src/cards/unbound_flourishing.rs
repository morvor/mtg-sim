//! Unbound Flourishing: "Whenever you cast a permanent spell with a mana cost that
//! contains {X}, double the value of X." The trigger is compiled; the effect doubles the
//! X of that spell while it's still on the stack (its mana value there, and the X its
//! permanent enters with, CR 107.3m), without any more mana being paid (rulings). If the
//! spell has left the stack, nothing happens. Several instances multiply (×4, ×8, ...).

use super::{map_effect, parse, ManualAbility};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::Zone;

const DOUBLE_X: &str = "card:Unbound Flourishing:double the value of X of that spell";
const TEXT: &str = "Whenever you cast a permanent spell with a mana cost that contains {X}, double the value of X.";

inventory::submit! { ManualAbility {
    card: "Unbound Flourishing",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "Whenever you cast a permanent spell with a mana cost that contains {X}, you gain 1 life.")
            .iter()
            .map(|a| map_effect(a, TEXT, |_| Effect::Custom(DOUBLE_X.into())))
            .collect()
    },
    reason: "doubles the value of X of a spell on the stack: the only legal card that changes a spell's X",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != DOUBLE_X {
            return false;
        }
        for s in g.resolve_objects(&Sel::TriggerObject, ctx) {
            if !g.is_live(s) || g.obj(s).zone != Zone::Stack {
                continue;
            }
            let o = &mut g.objects[s.0 as usize];
            if let Some(si) = o.stack.as_mut() {
                if let Some(x) = si.x.as_mut() {
                    *x = x.saturating_mul(2);
                }
                if let Some(x) = si.cast.x.as_mut() {
                    *x = x.saturating_mul(2);
                }
            }
            let x = g.obj(s).stack.as_ref().and_then(|si| si.x);
            g.log(|g| format!("{}: X is now {:?}", g.describe(s), x));
            g.dirty = true;
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
