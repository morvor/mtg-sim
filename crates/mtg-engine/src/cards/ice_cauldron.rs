//! Ice Cauldron: "{T}, Remove a charge counter from ~: Add ~'s last noted type and amount
//! of mana. Spend this mana only to cast the last card exiled with ~." The mana and its
//! note are the compiled noted-mana grammar (CR 607.2e, `noted_mana.rs`, shared with
//! Jeweled Amulet); the spending restriction is hand-written: the mana can pay only for
//! the card the Cauldron exiled most recently (its first ability exiles the card and lets
//! its controller cast it for as long as it remains exiled), cast from exile as that card
//! (rulings: "If multiple cards are exiled by the Cauldron, the mana can only be used for
//! the card that was exiled by the Cauldron when the most recent charge counter was put
//! there"; it can pay additional costs of that spell too).

use super::{map_effect, parse, ManualAbility};
use crate::ability::{Effect, Filter};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::mana::{ManaRestriction, SpendFilter};
use crate::types::ObjectId;

const TEXT: &str = "{T}, Remove a charge counter from ~: Add ~'s last noted type and amount of mana. Spend this mana only to cast the last card exiled with ~.";
const LAST_EXILED: &str = "card:Ice Cauldron:the last card exiled with the mana's source";

inventory::submit! { ManualAbility {
    card: "Ice Cauldron",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "{T}, Remove a charge counter from ~: Add ~'s last noted type and amount of mana.")
            .iter()
            .map(|a| {
                map_effect(a, TEXT, |e| match e {
                    Effect::AddMana { who, mana, .. } => Effect::AddMana {
                        who,
                        mana,
                        restriction: Some(ManaRestriction::CastSpell(SpendFilter::new(
                            Filter::Custom(LAST_EXILED.into()),
                        ))),
                    },
                    e => e,
                })
            })
            .collect()
    },
    reason: "mana spendable only on the last card its source exiled: no other card restricts mana to one exiled card",
} }

/// Rows `[source, card]`: the cards a source with this ability let its controller cast
/// from exile, in the order its abilities exiled them.
const EXILED: &str = "card:Ice Cauldron:cards exiled with the source";

/// Whether `o` has this ability (its mana spendable only on the last card it exiled).
fn has_ability(g: &Game, o: ObjectId) -> bool {
    g.try_obj(o).is_some_and(|o| {
        o.chars.abilities.iter().any(|a| match &a.kind {
            crate::ability::AbilityKind::Activated(x) => matches!(
                &x.body.effect,
                Effect::AddMana {
                    restriction: Some(ManaRestriction::CastSpell(f)),
                    ..
                } if matches!(&*f.0, Filter::Custom(n) if n == LAST_EXILED)
            ),
            _ => false,
        })
    })
}

/// The last card `source` exiled.
fn last_exiled(g: &Game, source: ObjectId) -> Option<ObjectId> {
    g.cards
        .get(EXILED)
        .iter()
        .rev()
        .find(|r| r[0] == source.0 as i64)
        .map(|r| ObjectId(r[1] as u32))
}

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    /// Records the cards a source with this ability exiles with a permission to cast them
    /// (its first ability), as the permission is given.
    fn on_event(&self, g: &mut Game, _ev: &crate::events::Event) {
        if g.play_grants.is_empty() {
            return;
        }
        let new: Vec<(ObjectId, ObjectId)> = g
            .play_grants
            .iter()
            .filter_map(|p| p.source.map(|s| (s, p.object)))
            .filter(|(s, o)| {
                !g.cards
                    .get(EXILED)
                    .iter()
                    .any(|r| r[0] == s.0 as i64 && r[1] == o.0 as i64)
            })
            .collect();
        for (s, o) in new {
            if has_ability(g, s) {
                g.cards.push(EXILED, vec![s.0 as i64, o.0 as i64]);
            }
        }
    }

    /// The spell being paid for (`id`) was cast from exile as the last card the mana's
    /// source (`ctx.source`) exiled.
    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        if name != LAST_EXILED {
            return None;
        }
        let Some(card) = ctx.source.and_then(|s| last_exiled(g, s)) else {
            return Some(false);
        };
        Some(g.try_obj(id).is_some_and(|o| o.prev == Some(card)))
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
