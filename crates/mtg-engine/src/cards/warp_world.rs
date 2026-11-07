//! Warp World: "Each player shuffles all permanents they own into their library, then
//! reveals that many cards from the top of their library. Each player puts all artifact,
//! creature, and land cards revealed this way onto the battlefield, then does the same for
//! enchantment cards, then puts all cards revealed this way that weren't put onto the
//! battlefield on the bottom of their library."
//!
//! Following the rulings' steps: tokens count but aren't revealed; the artifacts,
//! creatures and lands enter at the same time, then the enchantments (Auras can enchant
//! what entered first); abilities wait until the spell has finished resolving.

use super::{spell, ManualAbility};
use crate::ability::*;
use crate::eval::Ctx;
use crate::events::MoveCause;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::Zone;
use crate::replacement::{EtbInfo, MoveEv};
use crate::types::{CardType, ObjectId, PlayerId};

const WARP: &str = "card:Warp World:shuffle permanents away and put new ones onto the battlefield";
const TEXT: &str = "Each player shuffles all permanents they own into their library, then reveals that many cards from the top of their library. Each player puts all artifact, creature, and land cards revealed this way onto the battlefield, then does the same for enchantment cards, then puts all cards revealed this way that weren't put onto the battlefield on the bottom of their library.";

inventory::submit! { ManualAbility {
    card: "Warp World",
    face: 0,
    text: TEXT,
    build: |_ctx| vec![spell(vec![], Effect::Custom(WARP.into()), TEXT)],
    reason: "re-dealing every player's permanents from their libraries: only this card",
} }

fn onto_battlefield(g: &mut Game, cards: &[(PlayerId, ObjectId)], ctx: &Ctx) {
    let moves: Vec<MoveEv> = cards
        .iter()
        .filter(|(p, c)| g.player(*p).library.contains(c))
        .map(|(p, c)| MoveEv {
            obj: *c,
            to: Zone::Battlefield,
            pos: LibraryPosition::Top,
            cause: MoveCause::Effect,
            by: Some(*p),
            etb: EtbInfo {
                controller: Some(*p),
                ..Default::default()
            },
            source: ctx.source,
        })
        .collect();
    g.move_objects(moves);
}

fn warp(g: &mut Game, ctx: &Ctx) {
    let players: Vec<PlayerId> = g.apnap();
    // 1) Each player counts the permanents they own.
    let counts: Vec<(PlayerId, u32)> = players
        .iter()
        .map(|p| (*p, g.permanents().filter(|o| o.owner == *p).count() as u32))
        .collect();
    // 2) Each player shuffles them into their library.
    let perms: Vec<ObjectId> = g.permanents().map(|o| o.id).collect();
    crate::shuffle_rules::shuffle_into(g, &perms, Vec::new(), ctx);
    // 3) Each player reveals that many cards.
    let mut revealed: Vec<(PlayerId, Vec<ObjectId>)> = Vec::new();
    for (p, n) in counts {
        let cards = crate::library::top_cards(g, p, n);
        crate::reveal::reveal(g, p, &cards, ctx.source);
        revealed.push((p, cards));
    }
    let of_types = |g: &Game, types: &[CardType]| -> Vec<(PlayerId, ObjectId)> {
        revealed
            .iter()
            .flat_map(|(p, cs)| cs.iter().map(move |c| (*p, *c)))
            .filter(|(_, c)| types.iter().any(|t| g.obj(*c).chars.is(*t)))
            .collect()
    };
    // 4) Artifacts, creatures and lands, at the same time.
    let first = of_types(g, &[CardType::Artifact, CardType::Creature, CardType::Land]);
    onto_battlefield(g, &first, ctx);
    // 5) Then the enchantments.
    let second = of_types(g, &[CardType::Enchantment]);
    onto_battlefield(g, &second, ctx);
    // 6) The rest go to the bottom.
    for (p, cs) in &revealed {
        crate::library::put_on_bottom(g, *p, cs);
    }
}

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != WARP {
            return false;
        }
        let c = ctx.clone();
        g.atomically(|g| warp(g, &c));
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
