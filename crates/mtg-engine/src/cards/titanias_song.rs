//! Titania's Song: "Each noncreature artifact loses all abilities and becomes an artifact
//! creature with power and toughness each equal to its mana value. If ~ leaves the
//! battlefield, this effect continues until end of turn." The static ability is the
//! compiled March of the Machines wording with the loss of abilities added to the same
//! effect (one ability: its affected set is fixed as it begins to apply in layer 4,
//! CR 613.6). Hand-written: when Titania's Song leaves the battlefield, the effect goes on
//! until end of turn for the artifacts it was affecting then (a continuous effect whose
//! objects are fixed as it begins, CR 611.2c), each with its P/T equal to its mana value
//! as it ends (CR 608.2h).

use super::{has_marker, marker, parse, ManualAbility};
use crate::ability::*;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::Zone;

const TEXT: &str = "Each noncreature artifact loses all abilities and becomes an artifact creature with power and toughness each equal to its mana value. If ~ leaves the battlefield, this effect continues until end of turn.";
const LINGERS: &str = "card:Titania's Song:the effect continues until end of turn after it leaves";

inventory::submit! { ManualAbility {
    card: "Titania's Song",
    face: 0,
    text: TEXT,
    build: |ctx| {
        let mut out: Vec<Ability> = parse(
            ctx,
            "Each noncreature artifact is an artifact creature with power and toughness each equal to its mana value.",
        )
        .iter()
        .map(|a| {
            let mut kind = a.kind.clone();
            if let AbilityKind::Static(StaticAbility {
                effect: StaticEffect::Continuous { mods, .. },
                ..
            }) = &mut kind
            {
                mods.push(Modification::RemoveAllAbilities);
            }
            AbilityDef::new(kind, TEXT)
        })
        .collect();
        out.push(marker(LINGERS, TEXT));
        out
    },
    reason: "a static ability's effect that outlives its source until end of turn: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn on_event(&self, g: &mut Game, ev: &Event) {
        let Event::ZoneChange {
            old,
            from: Zone::Battlefield,
            ..
        } = ev
        else {
            return;
        };
        if !has_marker(g, *old, LINGERS) {
            return;
        }
        // The effect of its static ability, as it last existed (CR 603.10).
        let effect = g.obj(*old).chars.abilities.iter().find_map(|a| match &a.kind {
            AbilityKind::Static(StaticAbility {
                effect: StaticEffect::Continuous { affected, mods },
                ..
            }) if mods.iter().any(|m| matches!(m, Modification::RemoveAllAbilities)) => {
                Some((affected.clone(), mods.clone()))
            }
            _ => None,
        });
        let Some((affected, mods)) = effect else {
            return;
        };
        // Without the static ability, the objects it affected are those it describes.
        g.recompute();
        let controller = g.obj(*old).controller;
        let mut ctx = crate::eval::Ctx::new(Some(*old), controller);
        g.exec(
            &Effect::Modify {
                what: Sel::All(affected),
                mods,
                duration: Duration::EndOfTurn,
            },
            &mut ctx,
        );
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
