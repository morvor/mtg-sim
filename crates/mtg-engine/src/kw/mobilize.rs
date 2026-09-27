//! CR 702.181 Mobilize: "Mobilize N" means "Whenever this creature attacks, create N 1/1
//! red Warrior creature tokens. Those tokens enter tapped and attacking. Sacrifice them at
//! the beginning of the next end step." (CR 702.181a).
//!
//! * The tokens' controller chooses what each of them is attacking as it enters: they
//!   don't have to attack what the creature with mobilize is attacking (CR 508.4).
//! * "Mobilize X, where X is [value]" keeps the value in [`Keyword::x`]; X is determined
//!   as the ability resolves. A granted "mobilize X" (e.g. "has mobilize X, where X is its
//!   power") is determined the same way (see [`KeywordRules::x_determined_on_resolution`]).
//! * The delayed triggered ability sacrifices only the tokens its controller still
//!   controls (CR 701.21a).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::replacement::TokenCreate;
use crate::types::*;
use smol_str::SmolStr;

/// `Effect::Custom`: creates the tokens (the number is in [`COUNT`]).
const MOBILIZE: &str = "mobilize:create attacking warriors";
/// The number variable holding how many tokens to create.
const COUNT: Var = vars::USER + 181;

/// The token mobilize creates: a 1/1 red Warrior creature token.
pub fn warrior() -> TokenSpec {
    TokenSpec {
        name: SmolStr::default(),
        colors: ColorSet::single(Color::Red),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: vec![Subtype::new("Warrior")],
        power: Some(1),
        toughness: Some(1),
        abilities: vec![],
        scryfall_name: None,
    }
}

/// How many tokens a mobilize keyword instance creates: N, or X as its value defines it.
fn count(kw: &Keyword) -> Value {
    match (&kw.x, kw.n) {
        (Some(x), _) => x.clone(),
        (None, Some(n)) => Value::c(n.max(0)),
        (None, None) => Value::c(1),
    }
}

pub struct Mobilize;

impl KeywordRules for Mobilize {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Mobilize]
    }

    fn x_determined_on_resolution(&self) -> bool {
        true
    }

    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        Some(vec![AbilityDef::new(
            AbilityKind::Triggered(TriggeredAbility::new(
                TriggerCond::Attacks(Filter::Source),
                Body::effect(Effect::Seq(vec![
                    Effect::StoreValue {
                        var: COUNT,
                        value: count(kw),
                    },
                    Effect::Custom(SmolStr::new(MOBILIZE)),
                    // "Sacrifice them at the beginning of the next end step."
                    Effect::AtNext {
                        step: TriggerStep::End,
                        effect: Box::new(Effect::SacrificeObjects {
                            what: Sel::All(Filter::And(vec![
                                Filter::In(Box::new(Sel::Var(vars::CREATED))),
                                Filter::ControlledBy(PlayerRel::You),
                            ])),
                        }),
                    },
                ])),
            )),
            KeywordKind::Mobilize.name(),
        )])
    }

    /// Creates the tokens tapped and attacking, each attacking what its controller chooses
    /// (CR 508.4).
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != MOBILIZE {
            return false;
        }
        let n = ctx.nums.get(&COUNT).copied().unwrap_or(0).max(0) as u32;
        let p = ctx.controller;
        // What each token will attack, chosen as they enter; tokens attacking the same
        // thing are created together.
        let mut groups: Vec<(Option<Entity>, u32)> = Vec::new();
        for _ in 0..n {
            let target = if g.combat.is_some() {
                crate::combat::choose_attack_target_for_new_attacker(g, p)
            } else {
                None
            };
            match groups.iter_mut().find(|(t, _)| *t == target) {
                Some(gr) => gr.1 += 1,
                None => groups.push((target, 1)),
            }
        }
        let mut created: Vec<ObjectId> = Vec::new();
        for (target, k) in groups {
            let spec = warrior();
            let tc = TokenCreate {
                chars: crate::tokens::token_characteristics(&spec),
                card: crate::tokens::predefined_card(&spec),
                tapped: true,
                attacking: target,
                copy_of: None,
                copy_exceptions: vec![],
            };
            created.extend(g.create_tokens(p, tc, k, ctx.source));
        }
        ctx.prev_value = created.len() as i64;
        ctx.set_var(
            vars::CREATED,
            created.into_iter().map(Entity::Object).collect(),
        );
        true
    }
}

inventory::submit! { KeywordRegistration(&Mobilize) }
