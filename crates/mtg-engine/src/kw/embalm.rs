//! CR 702.128 Embalm and CR 702.129 Eternalize: activated abilities that function while
//! the card is in a graveyard.
//!
//! * "Embalm [cost]" means "[Cost], Exile this card from your graveyard: Create a token
//!   that's a copy of this card, except it's white, it has no mana cost, and it's a Zombie
//!   in addition to its other types. Activate only as a sorcery." (CR 702.128a)
//! * "Eternalize [cost]" means "[Cost], Exile this card from your graveyard: Create a
//!   token that's a copy of this card, except it's black, it's 4/4, it has no mana cost,
//!   and it's a Zombie in addition to its other types. Activate only as a sorcery."
//!   (CR 702.129a)
//!
//! The token copies the card as it last existed in the graveyard; the exceptions become
//! part of its copiable values (CR 707.9b). A token is "embalmed" if it's created by a
//! resolving embalm ability (CR 702.128b): the created tokens are marked, and while the
//! ability resolves, a token entering the battlefield is being created by it (for "if ~
//! was embalmed" in an ability that applies as the token enters, [`EMBALMED`]).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::{ObjKind, StackKind};
use crate::types::*;
use smol_str::SmolStr;

/// `Effect::Custom`: marks the tokens the resolving embalm ability created as embalmed.
const MARK_EMBALMED: &str = "embalm:mark embalmed";
/// `Condition::Custom`: "if ~ was embalmed" — the source is an embalmed token.
pub const EMBALMED: &str = "embalm:~ was embalmed";
/// The key (in `GameObject::linked_choices`) marking an embalmed token.
const EMBALMED_LINK: u16 = 0x7ffb;

pub struct Embalm;

fn zombie() -> Modification {
    Modification::AddSubtypes(vec![Subtype::new("Zombie")])
}

/// The copy exceptions of the token an embalm or eternalize ability creates.
pub fn exceptions(kind: KeywordKind) -> Vec<Modification> {
    match kind {
        KeywordKind::Eternalize => vec![
            Modification::SetColors(ColorSet::single(Color::Black)),
            Modification::SetPT(Some(Value::c(4)), Some(Value::c(4))),
            Modification::NoManaCost,
            zombie(),
        ],
        _ => vec![
            Modification::SetColors(ColorSet::single(Color::White)),
            Modification::NoManaCost,
            zombie(),
        ],
    }
}

/// Whether `ability` is an embalm ability (one this implementation derives).
fn is_embalm_ability(ability: &Ability) -> bool {
    match &ability.kind {
        AbilityKind::Activated(a) => match &a.body.effect {
            Effect::Seq(v) => v
                .iter()
                .any(|e| matches!(e, Effect::Custom(n) if n == MARK_EMBALMED)),
            _ => false,
        },
        _ => false,
    }
}

/// Whether `obj` is an embalmed token: one marked as created by an embalm ability, or a
/// token being created by the embalm ability that's resolving now.
pub fn is_embalmed(g: &Game, obj: ObjectId) -> bool {
    let o = g.obj(obj);
    if o.kind != ObjKind::Token {
        return false;
    }
    if o.linked_choices.contains_key(&EMBALMED_LINK) {
        return true;
    }
    let resolving_embalm = g.stack.last().is_some_and(|s| {
        matches!(
            g.obj(*s).stack.as_deref().map(|si| &si.kind),
            Some(StackKind::Activated { ability, .. }) if is_embalm_ability(ability)
        )
    });
    resolving_embalm && o.zone != crate::object::Zone::Battlefield
}

impl KeywordRules for Embalm {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Embalm, KeywordKind::Eternalize]
    }

    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        // A granted ability "whose cost is equal to its mana cost" has no cost of its own.
        let mut cost = kw.cost.clone().unwrap_or_else(|| Cost {
            mana: None,
            parts: vec![CostPart::PayManaCostOf(Box::new(Sel::This))],
        });
        cost.parts.push(CostPart::ExileSelf);
        let mut effects = vec![Effect::CreateTokenCopy {
            of: Sel::This,
            count: Value::c(1),
            controller: PlayerRef::You,
            tapped: false,
            attacking: false,
            mods: exceptions(kw.kind),
        }];
        if kw.kind == KeywordKind::Embalm {
            effects.push(Effect::Custom(SmolStr::new(MARK_EMBALMED)));
        }
        let mut act = ActivatedAbility::new(cost, Body::effect(Effect::Seq(effects)));
        act.timing = ActivationTiming::Sorcery;
        act.zone = FunctionZone::Graveyard;
        Some(vec![AbilityDef::new(
            AbilityKind::Activated(act),
            kw.kind.name(),
        )])
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != MARK_EMBALMED {
            return false;
        }
        for t in ctx.var_objects(vars::CREATED) {
            g.objects[t.0 as usize]
                .linked_choices
                .entry(EMBALMED_LINK)
                .or_default()
                .text = Some(SmolStr::new("embalmed"));
        }
        true
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        (name == EMBALMED).then(|| ctx.source.is_some_and(|s| is_embalmed(g, s)))
    }
}

inventory::submit! { KeywordRegistration(&Embalm) }
