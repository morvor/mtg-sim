//! Tokens (CR 111) and emblems (CR 114).

use crate::ability::*;
use crate::game::Game;
use crate::object::*;
use crate::types::*;
use smol_str::SmolStr;

/// Characteristics of a token from its spec (CR 111.4: name defaults to its subtypes plus "Token").
pub fn token_characteristics(spec: &TokenSpec) -> Characteristics {
    let mut name = spec.name.clone();
    if name.is_empty() {
        name = SmolStr::new(
            format!(
                "{} Token",
                spec.subtypes
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(" ")
            )
            .trim(),
        );
    }
    Characteristics {
        name,
        mana_cost: None,
        color_indicator: None,
        colors: spec.colors,
        supertypes: spec
            .supertypes
            .iter()
            .copied()
            .fold(SupertypeSet::NONE, |mut s, t| {
                s.insert(t);
                s
            }),
        card_types: spec.card_types.iter().copied().collect(),
        subtypes: spec.subtypes.iter().cloned().collect(),
        abilities: spec.abilities.clone(),
        power: spec.power,
        toughness: spec.toughness,
        loyalty: None,
        defense: None,
        hand_modifier: None,
        life_modifier: None,
        rules_text: std::sync::Arc::from(""),
        all_creature_names: false,
        interchangeable_names: Default::default(),
        all_creature_types: false,
        printed: None,
    }
}

/// Creates an emblem in the command zone, owned and controlled by `owner` (CR 114.1,
/// 114.2). It has no characteristics other than its abilities: no name, types, mana cost
/// or color (CR 114.3).
pub fn create_emblem(
    g: &mut Game,
    owner: PlayerId,
    abilities: Vec<Ability>,
    source: Option<ObjectId>,
) -> ObjectId {
    if let Some(s) = source {
        g.log(|g| format!("{} gets an emblem from {}", owner, g.obj(s).chars.name));
    }
    let chars = Characteristics {
        name: SmolStr::default(),
        abilities,
        rules_text: std::sync::Arc::from(""),
        ..Default::default()
    };
    let mut obj = GameObject::new(ObjectId(0), ObjKind::Emblem, owner, Zone::Command, chars);
    obj.controller = owner;
    let id = ObjectId(g.objects.len() as u32);
    obj.id = id;
    obj.timestamp = g.new_timestamp();
    g.objects.push(obj);
    g.command.push(id);
    g.dirty = true;
    id
}

/// Common predefined tokens (CR 111.10).
pub fn predefined(name: &str) -> Option<TokenSpec> {
    crate::tokens_predefined::predefined(name)
}

/// Creates tokens by name ("create a Tarmogoyf token", CR 111.11): the characteristics
/// come from the card with that name in the Oracle card reference. `spec` is "N:Name".
pub fn create_named_tokens(g: &mut Game, spec: &str, ctx: &mut crate::eval::Ctx) {
    let Some((n, name)) = spec.split_once(':') else {
        return;
    };
    let n: u32 = n.parse().unwrap_or(1);
    let Some(card) = crate::card::CardDb::global().get(name) else {
        return;
    };
    let tc = crate::replacement::TokenCreate {
        chars: card.characteristics(FaceState::Front),
        card: None,
        tapped: false,
        attacking: None,
        copy_of: None,
        copy_exceptions: vec![],
    };
    let created = g.create_tokens(ctx.controller, tc, n, ctx.source);
    ctx.set_var(
        vars::CREATED,
        created.into_iter().map(Entity::Object).collect(),
    );
}

/// The card definition behind a token created from `spec`, for predefined double-faced
/// tokens (an Incubator token, CR 111.10i) that need both faces to transform.
pub fn predefined_card(spec: &TokenSpec) -> Option<std::sync::Arc<crate::card::CardDef>> {
    let incubator = spec.name.is_empty()
        && spec.power.is_none()
        && spec.subtypes.len() == 1
        && spec.subtypes[0].as_str() == "Incubator";
    incubator.then(crate::tokens_predefined::incubator_card)
}
