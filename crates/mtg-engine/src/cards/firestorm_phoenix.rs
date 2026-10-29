//! Firestorm Phoenix: "If ~ would die, return it to its owner's hand instead. Until that
//! player's next turn, that player plays with that card revealed in their hand and can't
//! play it." The replacement is the compiled one (CR 614.1a); what's hand-written is the
//! card it returned being tracked: until its owner's next turn begins, that card (only
//! that one; another Firestorm Phoenix can be played) is revealed and can't be played
//! (rulings). It can still be put onto the battlefield by other effects.

use super::ManualAbility;
use crate::ability::{
    AbilityDef, AbilityKind, Effect, ReplacementAction, StaticAbility, StaticEffect,
};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::{Characteristics, Zone};
use crate::types::{ObjectId, PlayerId};

const TEXT: &str = "If ~ would die, return it to its owner's hand instead. Until that player's next turn, that player plays with that card revealed in their hand and can't play it.";
const TRACK: &str = "card:Firestorm Phoenix:the returned card is revealed and can't be played";
/// Rows `[card, owner, turns the owner had taken]`.
const ROWS: &str = "card:Firestorm Phoenix:returned cards";

inventory::submit! { ManualAbility {
    card: "Firestorm Phoenix",
    face: 0,
    text: TEXT,
    build: |ctx| {
        super::parse(ctx, "If ~ would die, return it to its owner's hand instead.")
            .iter()
            .map(|a| {
                let mut kind = a.kind.clone();
                if let AbilityKind::Static(StaticAbility {
                    effect: StaticEffect::Replacement(def),
                    ..
                }) = &mut kind
                {
                    let action = def.action.clone();
                    def.action = ReplacementAction::Then(
                        Box::new(action),
                        Box::new(Effect::Custom(TRACK.into())),
                    );
                }
                AbilityDef::new(kind, TEXT)
            })
            .collect()
    },
    reason: "tracks the one card it returned to hand: revealed and unplayable until its owner's next turn",
} }

/// Whether `card` is (or, as last known, was) in a hand as a card this ability returned,
/// still before its owner's next turn.
fn row_for(g: &Game, card: ObjectId) -> bool {
    let Some(Zone::Hand(p)) = g.try_obj(card).map(|o| o.zone) else {
        return false;
    };
    g.cards.get(ROWS).iter().any(|r| {
        r[0] == card.0 as i64 && r[1] == p.0 as i64 && r[2] == g.player(p).turns_taken as i64
    })
}

/// Whether `card`, in a hand, is a card this ability returned (until its owner's next
/// turn).
fn tracked(g: &Game, card: ObjectId) -> bool {
    g.is_live(card) && row_for(g, card)
}

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != TRACK {
            return false;
        }
        // "It": the card the event moved, where it went.
        let moved: Vec<ObjectId> = ctx
            .vars
            .get(&crate::ability::vars::IT)
            .into_iter()
            .flatten()
            .filter_map(|e| e.object())
            .collect();
        for now in moved {
            if let Some(Zone::Hand(p)) = g.try_obj(now).map(|o| o.zone) {
                let turns = g.player(p).turns_taken as i64;
                g.cards.push(ROWS, vec![now.0 as i64, p.0 as i64, turns]);
            }
        }
        true
    }

    fn card_revealed(&self, g: &Game, card: ObjectId) -> bool {
        tracked(g, card)
    }

    fn cast_prohibited(
        &self,
        g: &Game,
        _p: PlayerId,
        card: ObjectId,
        _chars: &Characteristics,
    ) -> bool {
        tracked(g, card) || g.try_obj(card).and_then(|o| o.prev).is_some_and(|c| row_for(g, c))
    }

    fn land_play_prohibited(&self, g: &Game, _p: PlayerId, card: ObjectId) -> bool {
        tracked(g, card)
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
