//! Magar of the Magic Strings: "{1}{B}{R}: Note the name of target instant or sorcery
//! card in your graveyard and put it onto the battlefield face down. It's a 3/3 creature
//! with "Whenever ~ deals combat damage to a player, you may create a copy of the card
//! with the noted name. You may cast the copy without paying its mana cost" and "If ~
//! would leave the battlefield, exile it instead of putting it anywhere else.""
//!
//! The face-down permanent remembers the card it was in the graveyard; its granted
//! trigger copies that card's last known information (CR 707.14).

use super::{map_effect, parse, ManualAbility};
use crate::ability::*;
use crate::copy_rules::NamedCopy;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::types::Entity;

const NOTE: &str = "card:Magar of the Magic Strings:note the name";
const RECALL: &str = "card:Magar of the Magic Strings:the card with the noted name";
const NOTED: Var = vars::USER + 31000;
const TEXT: &str = "{1}{B}{R}: Note the name of target instant or sorcery card in your graveyard and put it onto the battlefield face down. It's a 3/3 creature with \"Whenever ~ deals combat damage to a player, you may create a copy of the card with the noted name. You may cast the copy without paying its mana cost\" and \"If ~ would leave the battlefield, exile it instead of putting it anywhere else.\"";

inventory::submit! { ManualAbility {
    card: "Magar of the Magic Strings",
    face: 0,
    text: TEXT,
    build: |ctx| {
        let mut granted: Vec<Ability> = parse(
            ctx,
            "Whenever ~ deals combat damage to a player, draw a card.",
        )
        .iter()
        .map(|a| {
            map_effect(a, "Whenever ~ deals combat damage to a player, you may create a copy of the card with the noted name. You may cast the copy without paying its mana cost.", |_| {
                Effect::May {
                    who: PlayerRef::You,
                    effect: Box::new(Effect::Seq(vec![
                        Effect::Custom(RECALL.into()),
                        Effect::CopyCard {
                            what: Sel::Var(NOTED),
                            named: Some(NamedCopy::LastKnown),
                        },
                        Effect::CastCard {
                            who: PlayerRef::You,
                            what: Sel::Var(vars::CREATED),
                            free: true,
                            optional: true,
                        },
                    ])),
                }
            })
        })
        .collect();
        granted.extend(parse(
            ctx,
            "If ~ would leave the battlefield, exile it instead of putting it anywhere else.",
        ));
        parse(ctx, "{1}{B}{R}: Put target instant or sorcery card in your graveyard onto the battlefield face down. It's a 3/3 creature.")
            .iter()
            .map(|a| {
                map_effect(a, TEXT, |e| match e {
                    Effect::Move { what, mut to } => {
                        to.with_mods
                            .extend(granted.iter().cloned().map(Modification::AddAbility));
                        Effect::Seq(vec![Effect::Move { what, to }, Effect::Custom(NOTE.into())])
                    }
                    other => other,
                })
            })
            .collect()
    },
    reason: "a face-down permanent that remembers the graveyard card it was and copies it: only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        match name {
            // Records (face-down permanent, the card as it was in the graveyard).
            NOTE => {
                for t in ctx.targets.iter().flatten() {
                    if let Entity::Object(o) = t {
                        let now = g.current(*o);
                        if now != *o && g.is_live(now) {
                            g.cards.push(NOTE, vec![i64::from(now.0), i64::from(o.0)]);
                        }
                    }
                }
                true
            }
            RECALL => {
                let src = ctx.source.map_or(-1, |s| i64::from(s.0));
                let noted: Vec<Entity> = g
                    .cards
                    .get(NOTE)
                    .iter()
                    .filter(|r| r[0] == src)
                    .map(|r| Entity::Object(crate::types::ObjectId(r[1] as u32)))
                    .collect();
                ctx.set_var(NOTED, noted);
                true
            }
            _ => false,
        }
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
