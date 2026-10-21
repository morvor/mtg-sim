//! Soul Sculptor: "{1}{W}, {T}: Target creature becomes an enchantment and loses all
//! abilities until a player casts a creature spell." It's only an enchantment with no
//! abilities until any player casts a creature spell (rulings): the effect ends as one is
//! cast.

use super::{map_effect, parse, ManualAbility};
use crate::ability::*;
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};

const UNTIL_CREATURE_SPELL: &str = "card:Soul Sculptor:until a player casts a creature spell";
const TEXT: &str = "{1}{W}, {T}: Target creature becomes an enchantment and loses all abilities until a player casts a creature spell.";

fn until_creature_spell() -> Duration {
    Duration::WhileCondition(Condition::Custom(UNTIL_CREATURE_SPELL.into()))
}

inventory::submit! { ManualAbility {
    card: "Soul Sculptor",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "{1}{W}, {T}: Target creature becomes an enchantment and loses all abilities until end of turn.")
            .iter()
            .map(|a| {
                map_effect(a, TEXT, |e| match e {
                    Effect::Modify { what, mods, .. } => Effect::Modify {
                        what,
                        mods,
                        duration: until_creature_spell(),
                    },
                    e => e,
                })
            })
            .collect()
    },
    reason: "an effect that lasts until a player casts a creature spell: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    /// A creature spell was cast: the effects end.
    fn on_event(&self, g: &mut Game, ev: &Event) {
        let Event::SpellCast { spell, .. } = ev else {
            return;
        };
        if !g.obj(*spell).is_creature() {
            return;
        }
        let ends = |e: &crate::game::ContinuousEffect| {
            matches!(&e.duration, Duration::WhileCondition(Condition::Custom(n)) if n == UNTIL_CREATURE_SPELL)
        };
        if g.effects.iter().any(ends) {
            g.effects.retain(|e| !ends(e));
            g.dirty = true;
        }
    }
    /// Until then, it applies.
    fn custom_condition(&self, _g: &Game, name: &str, _ctx: &Ctx) -> Option<bool> {
        (name == UNTIL_CREATURE_SPELL).then_some(true)
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
