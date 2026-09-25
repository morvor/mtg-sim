//! Battles (CR 310): protectors, defense, and the Siege's intrinsic ability.

use crate::ability::*;
use crate::casting::CastOption;
use crate::eval::Ctx;
use crate::game::Game;
use crate::object::{CastMethod, FaceState, Zone};
use crate::types::*;

/// `Effect::Custom` name of a Siege's intrinsic ability: "exile it, then you may cast it
/// transformed without paying its mana cost" (CR 310.12b).
pub const SIEGE_DEFEATED: &str = "siege defeated";

/// The protector of a battle (stored as the battle's chosen player).
pub fn protector(g: &Game, battle: ObjectId) -> Option<PlayerId> {
    g.obj(battle).choices.player
}

/// The players who can be a battle's protector, determined by its battle type (CR 310.9a):
/// a Siege's controller's opponents (CR 310.12a); with no battle type, only its
/// controller. (A subtype the rules give no protector rule to doesn't restrict the
/// choice.) Only players in the game can be chosen.
pub fn eligible_protectors(g: &Game, battle: ObjectId) -> Vec<PlayerId> {
    let o = g.obj(battle);
    let controller = o.controller;
    if o.chars.has_subtype("Siege") {
        g.opponents(controller)
    } else if o.chars.subtypes.is_empty() {
        if g.player(controller).in_game() {
            vec![controller]
        } else {
            vec![]
        }
    } else {
        g.players_in_game()
    }
}

/// Whether `p` can be the battle's protector (CR 310.9a, 310.12a).
fn can_protect(g: &Game, battle: ObjectId, p: PlayerId) -> bool {
    g.player(p).in_game() && eligible_protectors(g, battle).contains(&p)
}

/// The battle's controller chooses its protector among the eligible players. Returns false
/// if no player can be chosen.
fn choose_protector(g: &mut Game, battle: ObjectId) -> bool {
    let cands = eligible_protectors(g, battle);
    if cands.is_empty() {
        return false;
    }
    let controller = g.obj(battle).controller;
    let ents: Vec<Entity> = cands.iter().map(|p| Entity::Player(*p)).collect();
    let chosen = g.ask_entities(
        controller,
        Some(battle),
        "Choose the battle's protector",
        ents,
        1,
        1,
    );
    let p = chosen
        .first()
        .and_then(|e| e.player())
        .filter(|p| cands.contains(p))
        .unwrap_or(cands[0]);
    // CR 310.9f: a battle has only one protector at a time.
    g.objects[battle.0 as usize].choices.player = Some(p);
    g.dirty = true;
    true
}

/// CR 310.9a: as a battle enters the battlefield, its controller chooses a player to be its
/// protector. (With no player to choose, the state-based action of CR 310.11 puts it into
/// its owner's graveyard.)
pub fn choose_protector_as_it_enters(g: &mut Game, id: ObjectId) {
    if g.obj(id).is(CardType::Battle) && g.obj(id).zone == Zone::Battlefield {
        choose_protector(g, id);
    }
}

/// Whether any attacking creature is attacking the battle.
fn is_being_attacked(g: &Game, battle: ObjectId) -> bool {
    g.combat.as_ref().is_some_and(|c| {
        c.attackers
            .iter()
            .any(|a| a.target == Some(Entity::Object(battle)))
    })
}

/// SBAs 704.5x and 704.5y (CR 310.11): a battle with no protector in the game (and not
/// being attacked), or whose protector can't be its protector, gets a new protector chosen
/// by its controller; if no player can be chosen, it's put into its owner's graveyard.
/// Returns true if any action was performed.
pub fn protector_sba(g: &mut Game, to_graveyard: &mut Vec<ObjectId>) -> bool {
    let mut did = false;
    let battles: Vec<ObjectId> = g
        .permanents()
        .filter(|o| o.is(CardType::Battle))
        .map(|o| o.id)
        .collect();
    for b in battles {
        let needs = match protector(g, b) {
            // 704.5x: no player in the game designated as its protector.
            None => !is_being_attacked(g, b),
            Some(p) if !g.player(p).in_game() => !is_being_attacked(g, b),
            // 704.5y: a protector who can't be its protector.
            Some(p) => !can_protect(g, b, p),
        };
        if !needs {
            continue;
        }
        if choose_protector(g, b) {
            did = true;
        } else {
            to_graveyard.push(b);
        }
    }
    did
}

/// Abilities a permanent has because of its battle type: a Siege's "When the last defense
/// counter is removed from this permanent, exile it, then you may cast it transformed
/// without paying its mana cost" (CR 310.12b).
pub fn intrinsic_abilities(chars: &crate::object::Characteristics) -> Vec<Ability> {
    if !chars.is(CardType::Battle) || !chars.has_subtype("Siege") {
        return vec![];
    }
    vec![siege_ability().clone()]
}

/// Siege abilities (CR 310.12b) that have triggered and are on the stack.
pub fn defeat_triggers_on_stack(g: &Game) -> Vec<ObjectId> {
    let uid = siege_ability().uid;
    g.stack
        .iter()
        .copied()
        .filter(|s| {
            matches!(
                g.obj(*s).stack.as_deref().map(|x| &x.kind),
                Some(crate::object::StackKind::Triggered { ability, .. }) if ability.uid == uid
            )
        })
        .collect()
}

/// The Siege's intrinsic ability: one definition (one uid) shared by every Siege.
fn siege_ability() -> &'static Ability {
    use std::sync::OnceLock;
    static SIEGE: OnceLock<Ability> = OnceLock::new();
    SIEGE.get_or_init(|| {
        let defense = Value::CountersOn(Box::new(Sel::This), Some(counters::DEFENSE.into()));
        let t = TriggeredAbility::new(
            TriggerCond::Where {
                trigger: Box::new(TriggerCond::CountersRemoved {
                    filter: Filter::Source,
                    kind: Some(counters::DEFENSE.into()),
                }),
                cond: Condition::Compare(defense, Cmp::Eq, Value::c(0)),
            },
            Body::effect(Effect::Custom(SIEGE_DEFEATED.into())),
        );
        AbilityDef::new(
            AbilityKind::Triggered(t),
            "When the last defense counter is removed from this permanent, exile it, then you may cast it transformed without paying its mana cost.",
        )
    })
}

/// Custom effects of battles. Returns true if `name` was one.
pub fn custom_effect(g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
    if name != SIEGE_DEFEATED {
        return false;
    }
    let Some(src) = ctx.source else {
        return true;
    };
    if !g.is_live(src) || g.obj(src).zone != Zone::Battlefield {
        return true;
    }
    let p = ctx.controller;
    let Some(card) = g.exile_object(src, Some(src)) else {
        return true;
    };
    g.recompute();
    // "Cast it transformed": only a double-faced card can be (CR 712.8c, 712.11a).
    let dfc = g
        .obj(card)
        .card
        .as_ref()
        .is_some_and(|d| d.layout.is_double_faced() && d.faces.len() > 1);
    if !dfc || g.obj(card).zone != Zone::Exile {
        return true;
    }
    if !g.ask_yes_no(
        p,
        Some(card),
        "Cast it transformed without paying its mana cost?",
        true,
    ) {
        return true;
    }
    let mut opt = CastOption::normal(FaceState::Back);
    opt.any_time = true;
    opt.method = CastMethod::Free;
    opt.alt_cost = Some(Cost::free());
    let _ = g.cast_with_option(p, card, opt);
    true
}
