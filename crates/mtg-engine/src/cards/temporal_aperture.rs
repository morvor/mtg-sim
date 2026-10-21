//! Temporal Aperture: "{5}, {T}: Shuffle your library, then reveal the top card. Until end
//! of turn, for as long as that card remains on top of your library, play with the top
//! card of your library revealed and you may play that card without paying its mana cost."
//! The permission ends if the card leaves the top of your library (ruling).

use super::{map_effect, parse, ManualAbility};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::Zone;
use crate::types::ObjectId;

const REVEAL_AND_PERMIT: &str = "card:Temporal Aperture:reveal the top card and allow playing it";
const ON_TOP: &str = "card:Temporal Aperture:still on top this turn:";
const TEXT: &str = "{5}, {T}: Shuffle your library, then reveal the top card. Until end of turn, for as long as that card remains on top of your library, play with the top card of your library revealed and you may play that card without paying its mana cost.";

inventory::submit! { ManualAbility {
    card: "Temporal Aperture",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(ctx, "{5}, {T}: Shuffle your library.")
            .iter()
            .map(|a| map_effect(a, TEXT, |e| Effect::Seq(vec![e, Effect::Custom(REVEAL_AND_PERMIT.into())])))
            .collect()
    },
    reason: "a free-play permission for the revealed card for as long as it stays on top this turn: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != REVEAL_AND_PERMIT {
            return false;
        }
        let you = ctx.controller;
        let Some(top) = g.player(you).library.last().copied() else {
            return true;
        };
        crate::reveal::reveal(g, you, &[top], ctx.source);
        let cond = Condition::Custom(format!("{ON_TOP}{}:{}", top.0, g.turn.number).into());
        crate::casting::grant_play_permission(
            g,
            you,
            vec![top],
            Duration::WhileCondition(cond),
            true,
            ctx.source,
        );
        true
    }
    fn custom_condition(&self, g: &Game, name: &str, _ctx: &Ctx) -> Option<bool> {
        let rest = name.strip_prefix(ON_TOP)?;
        let (id, turn) = rest.split_once(':')?;
        let (id, turn): (u32, u32) = (id.parse().ok()?, turn.parse().ok()?);
        let c = ObjectId(id);
        Some(
            g.turn.number == turn
                && g.is_live(c)
                && matches!(g.obj(c).zone, Zone::Library(p) if g.player(p).library.last() == Some(&c)),
        )
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
