//! Bludgeon Brawl: "Each noncreature, non-Equipment artifact is an Equipment with equip
//! {X} and "Equipped creature gets +X/+0", where X is that artifact's mana value." Each
//! affected artifact keeps its other types and abilities (ruling); X is its own mana value.

use super::{parse, ManualAbility};
use crate::ability::*;

const TEXT: &str = "Each noncreature, non-Equipment artifact is an Equipment with equip {X} and \"Equipped creature gets +X/+0\", where X is that artifact's mana value.";

inventory::submit! { ManualAbility {
    card: "Bludgeon Brawl",
    face: 0,
    text: TEXT,
    build: |ctx| {
        let equip = parse(ctx, "Equip {X}").iter().find_map(|a| match &a.kind {
            AbilityKind::Keyword(k) => Some(k.clone()),
            _ => None,
        });
        let Some(equip) = equip else {
            return parse(ctx, "Equip {X}");
        };
        parse(ctx, "Each noncreature, non-Equipment artifact has \"Equipped creature gets +1/+0.\"")
            .iter()
            .map(|a| {
                let mut kind = a.kind.clone();
                if let AbilityKind::Static(s) = &mut kind {
                    if let StaticEffect::Continuous { mods, .. } = &mut s.effect {
                        // "Equipped creature gets +X/+0": X is the mana value of the
                        // artifact that has the ability.
                        for m in mods.iter_mut() {
                            if let Modification::AddAbility(granted) = m {
                                let mut g = (**granted).clone();
                                if let AbilityKind::Static(gs) = &mut g.kind {
                                    if let StaticEffect::Continuous { mods: gm, .. } = &mut gs.effect {
                                        *gm = vec![Modification::ModifyPT(
                                            Value::ManaValueOf(Box::new(Sel::This)),
                                            Value::Const(0),
                                        )];
                                    }
                                }
                                g.text = "Equipped creature gets +X/+0, where X is this artifact's mana value.".into();
                                *granted = std::sync::Arc::new(g);
                            }
                        }
                        mods.insert(0, Modification::AddSubtypes(vec!["Equipment".into()]));
                        mods.insert(
                            1,
                            Modification::AddKeywordX(
                                equip.clone(),
                                Value::ManaValueOf(Box::new(Sel::Var(vars::AFFECTED))),
                            ),
                        );
                    }
                }
                AbilityDef::new(kind, TEXT)
            })
            .collect()
    },
    reason: "each noncreature artifact becomes an Equipment with equip and a bonus set by its own mana value: only this card",
} }
