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

/// Placeholder subtypes of a [`TokenSpec`]: the token's color, or a creature type, is the
/// one chosen for the source of the effect creating it ("a 1/1 creature token of the
/// chosen color and type").
pub const CHOSEN_COLOR: &str = "\u{1}chosen color";
pub const CHOSEN_TYPE: &str = "\u{1}chosen type";

/// Characteristics of a token the resolving effect `ctx` creates from `spec`: a P/T given
/// by values ("an X/X ... token, where X is ...") is determined now, once, and is part of
/// the token's characteristics as created — its copiable values (CR 111.3, 107.3, 608.2h).
pub fn token_characteristics_in(
    g: &Game,
    spec: &TokenSpec,
    ctx: &crate::eval::Ctx,
) -> Characteristics {
    // "a 1/1 creature token of the chosen color and type": the choices made for the
    // source (CR 607.2d); an undefined choice gives none (CR 607.5a).
    let resolved;
    let spec = if spec
        .subtypes
        .iter()
        .any(|t| t.as_str() == CHOSEN_COLOR || t.as_str() == CHOSEN_TYPE)
    {
        let choices = g.source_choices(ctx);
        let own = ctx.source.map(|s| &g.obj(s).choices);
        let color = choices
            .and_then(|ch| ch.color)
            .or_else(|| own.and_then(|ch| ch.color));
        let creature_type = choices
            .and_then(|ch| ch.creature_type.clone())
            .or_else(|| own.and_then(|ch| ch.creature_type.clone()));
        let mut s = spec.clone();
        let mut subtypes = Vec::new();
        for t in &spec.subtypes {
            match t.as_str() {
                CHOSEN_COLOR => {
                    if let Some(c) = color {
                        s.colors = ColorSet::from_iter([c]);
                    }
                }
                CHOSEN_TYPE => {
                    if let Some(ct) = creature_type.clone() {
                        subtypes.push(ct);
                    }
                }
                _ => subtypes.push(t.clone()),
            }
        }
        s.subtypes = subtypes;
        resolved = s;
        &resolved
    } else {
        spec
    };
    let mut chars = token_characteristics(spec);
    // "This token's power and toughness are each equal to the number of slime counters on
    // Gutter Grime": that specific object, the one creating the token; as part of the
    // token's copiable values, a copy of the token refers to it too (CR 201.5a, 707.2).
    if let Some(src) = ctx.source {
        for a in chars.abilities.iter_mut() {
            if crate::granted_by::refers_to_granter(a) {
                *a = crate::granted_by::bind_granter(a, src);
            }
        }
    }
    if let Some(pt) = &spec.pt_values {
        chars.power = Some(g.eval_value(&pt.0, ctx) as i32);
        chars.toughness = Some(g.eval_value(&pt.1, ctx) as i32);
    }
    chars
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

/// The player tokens created attacking must attack ("a 1/1 ... token that's tapped and
/// attacking that player", `Effect::TokensJoinCombat`; empty: no such player).
pub const TOKEN_DEFENDER: Var = vars::USER + 7356;

/// The player tokens created attacking now must attack, if an effect says which (`None`
/// inside: it names no player, so they can't attack).
pub fn forced_defender(ctx: &crate::eval::Ctx) -> Option<Option<Entity>> {
    ctx.vars.get(&TOKEN_DEFENDER).map(|v| v.first().copied())
}

/// Creates tokens by name ("create a Tarmogoyf token", CR 111.11): the characteristics
/// come from the card with that name in the Oracle card reference. `spec` is "N:Name".
pub fn create_named_tokens(g: &mut Game, spec: &str, ctx: &mut crate::eval::Ctx) {
    let Some((n, name)) = spec.split_once(':') else {
        return;
    };
    let n: u32 = n.parse().unwrap_or(1);
    // "N:tapped:Name": "create a tapped Mutavault token".
    let (name, tapped) = match name.strip_prefix("tapped:") {
        Some(x) => (x, true),
        None => (name, false),
    };
    let Some(card) = crate::card::CardDb::global().get(name) else {
        return;
    };
    let tc = crate::replacement::TokenCreate {
        chars: card.characteristics(FaceState::Front),
        card: None,
        tapped,
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

/// Performs `effect`, whose tokens enter with `counters` (CR 122.6; see
/// [`Effect::TokensEnterWithCounters`]). The numbers are determined first, while the
/// tokens don't exist yet: "the tokens just created" are none.
pub fn enter_with_counters(
    g: &mut Game,
    counters: &[(CounterKind, Value)],
    effect: &Effect,
    ctx: &mut crate::eval::Ctx,
) {
    let mut before = ctx.clone();
    before.vars.remove(&vars::CREATED);
    let with: Vec<(CounterKind, u32)> = counters
        .iter()
        .map(|(k, v)| (k.clone(), g.eval_value(v, &before).max(0) as u32))
        .filter(|(_, n)| *n > 0)
        .collect();
    let prev = std::mem::replace(&mut g.token_counters, with);
    g.exec(effect, ctx);
    g.token_counters = prev;
}
