//! The Book of Vile Darkness: "{T}, Exile ~ and artifacts you control named Eye of Vecna
//! and Hand of Vecna: Create Vecna, a legendary 8/8 black Zombie God creature token with
//! indestructible and all triggered abilities of the exiled cards."
//!
//! The triggered abilities come from the exiled cards as they are in exile (rulings) and
//! are part of the token's copiable values, so its enters triggers trigger.

use super::{map_effect, parse, ManualAbility};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};

const GRANT: &str = "card:The Book of Vile Darkness:all triggered abilities of the exiled cards";
static VECNA: std::sync::OnceLock<Effect> = std::sync::OnceLock::new();
const TEXT: &str = "{T}, Exile ~ and artifacts you control named Eye of Vecna and Hand of Vecna: Create Vecna, a legendary 8/8 black Zombie God creature token with indestructible and all triggered abilities of the exiled cards.";

fn artifact_named(name: &str) -> CostPart {
    CostPart::Exile {
        filter: Filter::And(vec![
            Filter::Type(crate::types::CardType::Artifact),
            Filter::ControlledBy(PlayerRel::You),
            Filter::Named(name.into()),
        ]),
        zone: ZoneKind::Battlefield,
        count: Value::Const(1),
    }
}

inventory::submit! { ManualAbility {
    card: "The Book of Vile Darkness",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "{T}: Create Vecna, a legendary 8/8 black Zombie God creature token with indestructible.")
            .iter()
            .map(|a| {
                let mut d: AbilityDef = (*map_effect(a, TEXT, |e| {
                    // The token as compiled; the triggered abilities are added to it as
                    // it's created.
                    if let Effect::CreateToken { .. } = &e {
                        let _ = VECNA.set(e);
                    }
                    Effect::Custom(GRANT.into())
                }))
                .clone();
                if let AbilityKind::Activated(act) = &mut d.kind {
                    act.cost.parts.push(CostPart::ExileSelf);
                    act.cost.parts.push(artifact_named("Eye of Vecna"));
                    act.cost.parts.push(artifact_named("Hand of Vecna"));
                }
                std::sync::Arc::new(d)
            })
            .collect()
    },
    reason: "a token with the triggered abilities of the cards its cost exiled: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != GRANT {
            return false;
        }
        let exiled: Vec<_> = ctx
            .vars
            .get(&crate::zones::COST_MOVED)
            .map(|es| es.iter().filter_map(|e| e.object()).collect())
            .unwrap_or_default();
        let granted: Vec<Ability> = exiled
            .iter()
            .flat_map(|o| g.obj(*o).chars.abilities.clone())
            .filter(|a| matches!(a.kind, AbilityKind::Triggered(_)))
            .collect();
        if let Some(Effect::CreateToken {
            spec,
            count,
            controller,
            tapped,
            attacking,
        }) = VECNA.get()
        {
            let mut spec = spec.clone();
            spec.abilities.extend(granted);
            g.exec(
                &Effect::CreateToken {
                    spec,
                    count: count.clone(),
                    controller: controller.clone(),
                    tapped: *tapped,
                    attacking: *attacking,
                },
                ctx,
            );
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
