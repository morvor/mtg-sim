//! CR 702.116 Myriad: "Whenever this creature attacks, for each opponent other than
//! defending player, you may create a token that's a copy of this creature that's tapped
//! and attacking that player or a planeswalker they control. If one or more tokens are
//! created this way, exile the tokens at end of combat." (CR 702.116a). Each instance
//! triggers separately (CR 702.116b).
//!
//! * The defending player is the player the creature was attacking (or the controller of
//!   the planeswalker or protector of the battle it was attacking) as it became an
//!   attacking creature (CR 508.5).
//! * For each such opponent, its controller chooses whether to create a token and, if so,
//!   whether it's attacking that player or a planeswalker they control. The token copies
//!   the creature's copiable values (as it last existed, if it has left the battlefield,
//!   CR 707.2); it was never declared as an attacker, so its own abilities that trigger
//!   on attacking don't trigger (CR 508.4).
//! * The tokens created are exiled by a delayed triggered ability at end of combat
//!   (CR 603.7).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::replacement::TokenCreate;
use crate::types::*;

/// `Effect::Custom`: the myriad trigger's effect.
const MYRIAD: &str = "myriad";

/// The variable holding the tokens a myriad ability created.
const TOKENS: Var = vars::USER + 116;

/// The defending player for the attacking creature `id` (CR 508.5), including if it was
/// removed from combat.
fn defending_player(g: &Game, id: ObjectId) -> Option<PlayerId> {
    let c = g.combat.as_ref()?;
    c.attacker(id)
        .and_then(|a| a.defending_player)
        .or_else(|| {
            c.removed_attackers
                .iter()
                .find(|(x, _)| *x == id)
                .map(|(_, p)| *p)
        })
}

fn myriad(g: &mut Game, ctx: &mut Ctx) {
    let Some(src) = ctx.source else {
        return;
    };
    let you = ctx.controller;
    let defending = defending_player(g, src);
    let others: Vec<PlayerId> = g
        .apnap()
        .into_iter()
        .filter(|q| g.are_opponents(you, *q) && Some(*q) != defending)
        .collect();
    let mut created: Vec<ObjectId> = Vec::new();
    for q in others {
        let name = g.obj(src).chars.name.clone();
        if !g.ask_yes_no(
            you,
            Some(src),
            &format!("Create a token copy of {name} attacking {q}?"),
            true,
        ) {
            continue;
        }
        // That player or a planeswalker they control.
        let mut choices = vec![Entity::Player(q)];
        choices.extend(
            g.permanents()
                .filter(|o| o.controller == q && o.is(CardType::Planeswalker))
                .map(|o| Entity::Object(o.id)),
        );
        let target = if choices.len() == 1 {
            choices[0]
        } else {
            g.ask_entities(
                you,
                Some(src),
                "Choose what the token is attacking",
                choices.clone(),
                1,
                1,
            )
            .first()
            .copied()
            .filter(|e| choices.contains(e))
            .unwrap_or(Entity::Player(q))
        };
        let o = g.obj(src);
        let spec = TokenCreate {
            chars: o.copiable.clone(),
            card: o.card.clone(),
            tapped: true,
            attacking: Some(target),
            copy_of: Some(src),
            copy_exceptions: vec![],
        };
        created.extend(g.create_tokens(you, spec, 1, Some(src)));
    }
    ctx.prev_happened = !created.is_empty();
    if created.is_empty() {
        return;
    }
    ctx.set_var(TOKENS, created.into_iter().map(Entity::Object).collect());
    // "Exile the tokens at end of combat."
    g.exec(
        &Effect::AtNext {
            step: TriggerStep::EndOfCombat,
            effect: Box::new(Effect::Exile {
                what: Sel::Var(TOKENS),
                face_down: false,
                link: false,
            }),
        },
        ctx,
    );
}

pub struct Myriad;

impl KeywordRules for Myriad {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Myriad]
    }

    fn derived(&self, _kw: &Keyword) -> Option<Vec<Ability>> {
        Some(vec![AbilityDef::new(
            AbilityKind::Triggered(TriggeredAbility::new(
                TriggerCond::Attacks(Filter::Source),
                Body::effect(Effect::Custom(MYRIAD.into())),
            )),
            KeywordKind::Myriad.name(),
        )])
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != MYRIAD {
            return false;
        }
        myriad(g, ctx);
        true
    }
}

inventory::submit! { KeywordRegistration(&Myriad) }
