//! Shrouded Lore: "Target opponent chooses a card in your graveyard. You may pay {B}. If you
//! do, repeat this process except that opponent can't choose a card already chosen for ~.
//! Then put the last chosen card into your hand." When no other card is left to choose, the
//! process stops (ruling).

use super::{map_effect, parse, ManualAbility};
use crate::ability::*;
use crate::eval::Ctx;
use crate::events::MoveCause;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::mana::ManaCost;
use crate::object::Zone;
use crate::replacement::{EtbInfo, MoveEv};
use crate::types::{Entity, ObjectId};

const LORE: &str = "card:Shrouded Lore:the opponent chooses until you stop paying";
const TEXT: &str = "Target opponent chooses a card in your graveyard. You may pay {B}. If you do, repeat this process except that opponent can't choose a card already chosen for ~. Then put the last chosen card into your hand.";

inventory::submit! { ManualAbility {
    card: "Shrouded Lore",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "Target opponent loses 1 life.")
            .iter()
            .map(|a| map_effect(a, TEXT, |_| Effect::Custom(LORE.into())))
            .collect()
    },
    reason: "an opponent chooses graveyard cards for as long as you pay {B}; you get the last one: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != LORE {
            return false;
        }
        let Some(Entity::Player(opp)) = ctx.targets.first().and_then(|v| v.first()).copied()
        else {
            return true;
        };
        let you = ctx.controller;
        let src = ctx.source;
        let b = Cost::mana(ManaCost::parse("{B}").unwrap_or_default());
        let mut chosen: Vec<ObjectId> = Vec::new();
        let mut last: Option<ObjectId> = None;
        loop {
            let cands: Vec<ObjectId> = g
                .player(you)
                .graveyard
                .iter()
                .copied()
                .filter(|c| !chosen.contains(c))
                .collect();
            let Some(c) = g
                .ask_objects(opp, src, "Choose a card in their graveyard", cands, 1, 1)
                .first()
                .copied()
            else {
                break;
            };
            chosen.push(c);
            last = Some(c);
            let more = g.player(you).graveyard.iter().any(|x| !chosen.contains(x));
            if !more
                || !g.can_pay_cost(you, &b, src, ctx)
                || !g.ask_yes_no(you, src, "Pay {B} to have them choose again?", false)
                || !g.pay_cost(you, &b, src, ctx)
            {
                break;
            }
        }
        if let Some(c) = last.filter(|c| g.obj(*c).zone == Zone::Graveyard(you)) {
            g.move_object_ev(MoveEv {
                obj: c,
                to: Zone::Hand(you),
                pos: LibraryPosition::Top,
                cause: MoveCause::Effect,
                by: Some(you),
                etb: EtbInfo::default(),
                source: src,
            });
        }
        true
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
