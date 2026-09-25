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
    }
}

/// Creates an emblem in the command zone (CR 114.1).
pub fn create_emblem(
    g: &mut Game,
    owner: PlayerId,
    abilities: Vec<Ability>,
    source: Option<ObjectId>,
) -> ObjectId {
    let name = source
        .map(|s| format!("Emblem ({})", g.obj(s).chars.name))
        .unwrap_or_else(|| "Emblem".into());
    let chars = Characteristics {
        name: SmolStr::new(name),
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
