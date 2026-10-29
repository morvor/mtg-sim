//! In-game behaviors of the draft abilities of `oracle/patterns/draft_4.rs` (CR 905.2b):
//! the information noted during the draft and the cards removed from it, as the game
//! refers to them (see `crate::draft`).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::{AbilityDef, AbilityKind};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::{same_instance, KeywordKind};
use crate::object::Characteristics;
use crate::types::*;

/// `Modification::Custom` prefix: the keyword abilities of kind `K` of the creature cards
/// the controller removed from the draft with cards with the source's name
/// (`draft:removed keywords:Flying`; Animus of Predation: "If you removed a creature card
/// with flying from the draft with cards named ~, ~ has flying."). Variants are kept
/// ("hexproof from white", its rulings).
pub const REMOVED_KEYWORDS: &str = "draft:removed keywords";

/// `Condition::Custom` name: the iterated player is a player the controller noted for
/// cards with the source's name (Cogwork Tracker: "a player you noted for cards named ~").
pub const NOTED_PLAYER: &str = "draft:a player noted for cards with this name";

struct Draft4;

impl KeywordRules for Draft4 {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        crate::draft::custom_filter(g, name, id, ctx)
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        if name == NOTED_PLAYER {
            let p = ctx.iter_player?;
            let src = ctx
                .source
                .and_then(|s| g.try_obj(g.ability_source_of(s)))
                .map(|o| o.chars.name.clone())?;
            return Some(g.start.draft.notes.iter().any(|n| {
                n.player == ctx.controller && n.card == src && n.noted_player == Some(p)
            }));
        }
        crate::draft::custom_condition(g, name, ctx)
    }

    fn custom_modification(
        &self,
        g: &Game,
        name: &str,
        chars: &mut Characteristics,
        ctx: &Ctx,
        _target: ObjectId,
    ) -> bool {
        let Some(kind) = name
            .strip_prefix(REMOVED_KEYWORDS)
            .and_then(|r| r.strip_prefix(':'))
            .and_then(KeywordKind::from_name)
        else {
            return false;
        };
        let Some(src) = ctx
            .source
            .and_then(|s| g.try_obj(s))
            .map(|o| o.chars.name.clone())
        else {
            return true;
        };
        for r in &g.start.draft.removed {
            if r.player != ctx.controller || r.with != src {
                continue;
            }
            for f in &r.card.faces {
                if !f.chars.card_types.contains(CardType::Creature) {
                    continue;
                }
                let new: Vec<_> = f
                    .chars
                    .keywords_of(kind)
                    .filter(|k| !chars.keywords_of(kind).any(|x| same_instance(x, k)))
                    .cloned()
                    .collect();
                for k in new {
                    let name = k.kind.name();
                    chars
                        .abilities
                        .push(AbilityDef::new(AbilityKind::Keyword(k), name));
                }
            }
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&Draft4) }
