//! Oubliette: "When ~ enters, target creature phases out until ~ leaves the battlefield.
//! Tap that creature as it phases in this way." The phasing out until it leaves compiles;
//! the creature phases in tapped without becoming tapped (no "becomes tapped" triggers,
//! ruling).

use super::{map_effect, parse, ManualAbility};
use crate::ability::{Effect, Sel};
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::Zone;

const TAP_AS_IT_PHASES_IN: &str = "card:Oubliette:tap that creature as it phases in this way";
const TEXT: &str = "When ~ enters, target creature phases out until ~ leaves the battlefield. Tap that creature as it phases in this way.";

inventory::submit! { ManualAbility {
    card: "Oubliette",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "When ~ enters, target creature phases out until ~ leaves the battlefield.")
            .iter()
            .map(|a| {
                map_effect(a, TEXT, |e| Effect::Seq(vec![e, Effect::Custom(TAP_AS_IT_PHASES_IN.into())]))
            })
            .collect()
    },
    reason: "the creature it phased out phases in tapped when it leaves: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    /// Records (source, creature) for each creature it phased out.
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != TAP_AS_IT_PHASES_IN {
            return false;
        }
        let src = ctx.source.map_or(-1, |s| i64::from(s.0));
        for o in g.resolve_objects(&Sel::Target(0), ctx) {
            if g.obj(o).phased_out {
                g.cards.push(TAP_AS_IT_PHASES_IN, vec![src, i64::from(o.0)]);
            }
        }
        true
    }
    /// It phases in "this way" once the Oubliette has left the battlefield.
    fn on_event(&self, g: &mut Game, ev: &Event) {
        let Event::PhasedIn { obj } = ev else {
            return;
        };
        let id = i64::from(obj.0);
        let rows: Vec<Vec<i64>> = g.cards.get(TAP_AS_IT_PHASES_IN).to_vec();
        let mut tap = false;
        for r in rows.iter().filter(|r| r[1] == id) {
            let src = crate::types::ObjectId(r[0] as u32);
            let gone = r[0] < 0 || !g.is_live(src) || g.obj(src).zone != Zone::Battlefield;
            tap |= gone;
        }
        g.cards.retain(TAP_AS_IT_PHASES_IN, |r| r[1] != id);
        if tap && g.is_live(*obj) && g.obj(*obj).zone == Zone::Battlefield {
            g.objects[obj.0 as usize].tapped = true;
            g.dirty = true;
        }
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
