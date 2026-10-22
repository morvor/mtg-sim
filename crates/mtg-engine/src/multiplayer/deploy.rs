//! The deploy creatures option (CR 804): each creature has the ability "{T}: Target
//! teammate gains control of this creature. Activate only as a sorcery." (CR 804.2).
//! Always used in the Emperor variant (CR 804.1, 809.3b).

use crate::ability::*;
use crate::game::Game;
use crate::object::Zone;
use crate::types::*;

/// The rules text of the ability the deploy creatures option grants (CR 804.2).
pub const DEPLOY_TEXT: &str =
    "{T}: Target teammate gains control of this creature. Activate only as a sorcery.";

/// The ability each creature has with the deploy creatures option (CR 804.2).
pub fn deploy_ability() -> Ability {
    use std::sync::OnceLock;
    static ABILITY: OnceLock<Ability> = OnceLock::new();
    ABILITY
        .get_or_init(|| {
            // "Target teammate": a player who is neither you nor your opponent.
            let teammate = PlayerFilter::And(vec![
                PlayerFilter::NotYou,
                PlayerFilter::Not(Box::new(PlayerFilter::Opponent)),
            ]);
            let mut a = ActivatedAbility::new(
                Cost::tap(),
                Body::simple(
                    vec![TargetSpec::player(teammate, "target teammate")],
                    Effect::GainControl {
                        what: Sel::This,
                        who: PlayerRef::Target(0),
                        duration: Duration::Permanent,
                    },
                ),
            );
            a.timing = ActivationTiming::Sorcery;
            AbilityDef::new(AbilityKind::Activated(a), DEPLOY_TEXT)
        })
        .clone()
}

/// Whether the game uses the deploy creatures option (CR 804.1).
pub fn option_used(g: &Game) -> bool {
    g.config.deploy_creatures
}

/// Gives each creature on the battlefield the deploy ability (CR 804.2). Called after the
/// type-changing layer, so it's creatures as their types are then; like other abilities
/// the rules give an object, effects that remove abilities can remove it.
pub fn grant(g: &mut Game, live: &[ObjectId]) {
    if !option_used(g) {
        return;
    }
    let ability = deploy_ability();
    for id in live {
        let o = &mut g.objects[id.0 as usize];
        if o.zone == Zone::Battlefield && o.chars.is(CardType::Creature) {
            o.chars.abilities.push(ability.clone());
        }
    }
}
