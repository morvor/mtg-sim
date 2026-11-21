//! Lodestone Bauble: "{1}, {T}, Sacrifice this artifact: Put up to four target basic land
//! cards from a player's graveyard on top of their library in any order. That player
//! draws a card at the beginning of the next turn's upkeep."
//!
//! The player is chosen, not targeted, as the targets are chosen (CR 601.2c via 602.2b,
//! 115.10), even when no targets are chosen (ruling: "If you choose zero targets, you
//! still choose a player. That player will draw a card at the beginning of the next
//! turn's upkeep."); the targets must be cards in that player's graveyard, and "That
//! player" is that player. The rest is compiled: the ability is the compiled form of the
//! text with "target player's graveyard", with its player target replaced by the player
//! chosen with the targets (the `choose_with_targets` hook, keyed to the marker
//! [`CHOOSE`] in the ability's effect), kept on the ability on the stack as the chosen
//! player (`stack::CHOOSER_KEY`, `PlayerRel::Chosen`; interpretation
//! `player-chosen-before-targets`).

use super::{parse, ManualAbility};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::{CardType, Entity, ObjectId, PlayerId, Supertype};

/// `Effect::Custom` marking the ability whose player is chosen with its targets (it
/// does nothing as the ability resolves).
pub const CHOOSE: &str = "card:Lodestone Bauble:a player chosen with the targets";

const TEXT: &str = "{1}, {T}, Sacrifice ~: Put up to four target basic land cards from a player's graveyard on top of their library in any order. That player draws a card at the beginning of the next turn's upkeep.";

/// The compiled form with a targeted player.
const COMPILED: &str = "{1}, {T}, Sacrifice ~: Put up to four target basic land cards from target player's graveyard on top of their library in any order. That player draws a card at the beginning of the next turn's upkeep.";

/// The compiled ability with its player target (slot 0) replaced by the chosen player.
fn build(ctx: &crate::oracle::CompileContext) -> Vec<Ability> {
    let unsupported = || vec![AbilityDef::new(AbilityKind::Unsupported(TEXT.into()), TEXT)];
    let abilities = parse(ctx, COMPILED);
    let [a] = abilities.as_slice() else {
        return unsupported();
    };
    let mut kind = a.kind.clone();
    let AbilityKind::Activated(act) = &mut kind else {
        return unsupported();
    };
    let body = &mut act.body;
    // [target player, up to four target basic land cards owned by target 0]
    let ok_targets = matches!(
        body.targets.as_slice(),
        [TargetSpec { what: TargetKind::Player(_), .. }, TargetSpec { what: TargetKind::Object(_), .. }]
    );
    if !ok_targets {
        return unsupported();
    }
    let mut cards = body.targets.remove(1);
    let TargetKind::Object(Filter::And(parts)) = &mut cards.what else {
        return unsupported();
    };
    let mut owner = false;
    for f in parts.iter_mut() {
        if matches!(f, Filter::OwnedBy(PlayerRel::Target(0))) {
            *f = Filter::OwnedBy(PlayerRel::Chosen);
            owner = true;
        }
    }
    if !owner {
        return unsupported();
    }
    body.targets = vec![cards];
    // [move target 1 on top, as target player 0: the delayed draw]
    let Effect::Seq(steps) = std::mem::take(&mut body.effect) else {
        return unsupported();
    };
    let [Effect::Move { what: Sel::Target(1), to }, Effect::AsPlayer { who: PlayerRef::Target(0), effect }] =
        steps.as_slice()
    else {
        return unsupported();
    };
    body.effect = Effect::Seq(vec![
        Effect::Custom(CHOOSE.into()),
        Effect::Move {
            what: Sel::Target(0),
            to: to.clone(),
        },
        Effect::AsPlayer {
            who: PlayerRef::ChosenOpponent,
            effect: effect.clone(),
        },
    ]);
    vec![AbilityDef::new(kind, TEXT)]
}

inventory::submit! { ManualAbility {
    card: "Lodestone Bauble",
    face: 0,
    text: TEXT,
    build,
    reason: "a player chosen (not targeted) with the targets, even with no targets, whose graveyard they come from: the only such card",
} }

fn marked(e: &Effect) -> bool {
    match e {
        Effect::Custom(n) => n == CHOOSE,
        Effect::Seq(v) => v.iter().any(marked),
        _ => false,
    }
}

/// Basic land cards in `p`'s graveyard.
fn basic_lands(g: &Game, p: PlayerId) -> usize {
    g.player(p)
        .graveyard
        .iter()
        .filter(|c| {
            let o = g.obj(**c);
            o.is(CardType::Land) && o.chars.supertypes.contains(Supertype::Basic)
        })
        .count()
}

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn choose_with_targets(
        &self,
        g: &mut Game,
        stack_obj: ObjectId,
        body: &Body,
        ctx: &mut Ctx,
    ) -> bool {
        if !marked(&body.effect) {
            return true;
        }
        // Any player (CR 801.5a: within the controller's range of influence); by default
        // the one with the most basic land cards in their graveyard.
        let mut players: Vec<PlayerId> = g
            .players_in_game()
            .into_iter()
            .filter(|p| crate::multiplayer::range::player_in_range(g, ctx.controller, *p))
            .collect();
        players.sort_by_key(|p| std::cmp::Reverse(basic_lands(g, *p)));
        let cands = players.into_iter().map(Entity::Player).collect();
        let pick = g.ask_entities(
            ctx.controller,
            Some(stack_obj),
            "Choose a player (the targets come from their graveyard)",
            cands,
            1,
            1,
        );
        let Some(Entity::Player(p)) = pick.first().copied() else {
            return false;
        };
        ctx.chosen_player = Some(p);
        if let Some(si) = g.objects[stack_obj.0 as usize].stack.as_mut() {
            si.chosen_values
                .insert(crate::stack::CHOOSER_KEY, i64::from(p.0));
        }
        g.log(|_| format!("{p} is chosen with the targets"));
        true
    }
    fn custom_effect(&self, _g: &mut Game, name: &str, _ctx: &mut Ctx) -> bool {
        // The marker does nothing as the ability resolves.
        name == CHOOSE
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
