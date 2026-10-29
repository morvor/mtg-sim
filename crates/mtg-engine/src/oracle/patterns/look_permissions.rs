//! Permissions to look at hidden cards (CR 406.3, 708.5):
//!
//! * "You may look at cards exiled with ~." (Bane Alley Broker): the face-down cards linked
//!   to the source (CR 607.2a, `kw/look_at_exiled_with.rs`);
//! * "You may look at face-down creatures you don't control any time." (Keeper of the
//!   Lens), with "the top card of your library and" (Lens of Clarity), or "Until end of
//!   turn, ..." (Lumbering Laundry, an effect on the player): `kw/look_face_down.rs`.

use super::{EffectPattern, StaticPattern};
use crate::ability::*;
use crate::kw::look_face_down::LOOK_AT_FACE_DOWN_CREATURES;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

fn custom_static(name: &str, text: &str) -> Ability {
    AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Custom(name.into()))),
        text,
    )
}

fn s_look_permissions(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    match end(l.trim()) {
        "you may look at cards exiled with ~" | "you may look at the cards exiled with ~" => {
            Some(vec![custom_static(
                crate::kw::look_at_exiled_with::MAY_LOOK_AT_LINKED,
                text,
            )])
        }
        "you may look at face-down creatures you don't control any time" => {
            Some(vec![custom_static(LOOK_AT_FACE_DOWN_CREATURES, text)])
        }
        "you may look at the top card of your library and at face-down creatures you don't control any time" =>
        {
            let mut out = crate::oracle::statics::parse_static(
                "You may look at the top card of your library any time.",
                ctx,
            )?;
            out.push(custom_static(LOOK_AT_FACE_DOWN_CREATURES, text));
            Some(
                out.into_iter()
                    .map(|a| AbilityDef::new(a.kind.clone(), text))
                    .collect(),
            )
        }
        _ => None,
    }
}

inventory::submit! { StaticPattern { name: "look permissions: cards exiled with ~, face-down creatures you don't control", priority: 100, parse: s_look_permissions } }

/// "Until end of turn, you may look at face-down creatures you don't control any time."
fn p_look_for_a_while(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l.trim()).strip_prefix("until end of turn, ")?;
    if r != "you may look at face-down creatures you don't control any time" {
        return None;
    }
    Some(Effect::AddPlayerEffect {
        who: PlayerRef::You,
        effect: PlayerModification::Custom(LOOK_AT_FACE_DOWN_CREATURES.into()),
        duration: Duration::EndOfTurn,
    })
}

inventory::submit! { EffectPattern { name: "look permissions: until end of turn, look at face-down creatures", priority: 100, parse: p_look_for_a_while } }
