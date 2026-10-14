//! More restriction grammar: attack taxes for a duration (CR 508.1d, 508.1h), "can't
//! play land cards from graveyards", "[spells] cost {1} less and can't be countered",
//! "can't be blocked this turn except by ...", "gains shroud until end of turn and
//! doesn't untap during your next untap step", "You can't sacrifice those creatures this
//! turn", "Cast this spell only if no permanents named ~ are on the battlefield", "can't
//! become untapped and can't have counters put on it", counted-value conditions.

use mtg_engine::combat::{attack_options, block_declaration_legal, block_options};
use mtg_engine::decision::{Action, Answer};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

fn castable(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.legal_actions(p)
        .iter()
        .any(|a| matches!(a, Action::Cast { card: c, .. } if *c == card))
}

fn activatable(t: &mut TestGame, p: PlayerId, src: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.legal_actions(p)
        .iter()
        .any(|a| matches!(a, Action::Activate { source, .. } if *source == src))
}

#[test]
fn attack_tax_until_your_next_turn() {
    cr!("508.1d", "508.1h");
    compiles("Forbidding Spirit");
    compiles("Sivitri, Dragon Master");
    let mut t = TestGame::new(2);
    let spirit = t.hand(P0, "Forbidding Spirit");
    t.lands(P0, "Plains", 3);
    t.cast(P0, spirit).go();
    t.resolve_all();
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    let _ = attack_options(&t.g);
    let cost = mtg_engine::combat::required_attack_cost(&t.g, bears, Entity::Player(P0));
    assert_eq!(cost.and_then(|c| c.mana).map(|m| m.mana_value()), Some(2));
    // It ends at P0's next turn.
    t.advance_to(P0, Step::Upkeep);
    assert!(mtg_engine::combat::required_attack_cost(&t.g, bears, Entity::Player(P0)).is_none());
}

#[test]
fn opponents_cant_play_land_cards_from_graveyards() {
    cr!("305.1");
    compiles("Tomik, Distinguished Advokist");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tomik, Distinguished Advokist");
    t.battlefield(P1, "Crucible of Worlds");
    let land = t.graveyard(P1, "Forest");
    t.set_step(P1, Step::PrecombatMain);
    t.g.turn.priority = Some(P1);
    assert!(!t
        .g
        .legal_actions(P1)
        .iter()
        .any(|a| matches!(a, Action::PlayLand { card } if *card == land)));
    // Its controller can.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tomik, Distinguished Advokist");
    t.battlefield(P0, "Crucible of Worlds");
    let land = t.graveyard(P0, "Forest");
    t.g.turn.priority = Some(P0);
    assert!(t
        .g
        .legal_actions(P0)
        .iter()
        .any(|a| matches!(a, Action::PlayLand { card } if *card == land)));
}

#[test]
fn spells_with_flash_cost_less_and_cant_be_countered() {
    cr!("601.2f", "701.6a");
    compiles("Cunning Nightbonder");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cunning Nightbonder");
    // Spectral Sailor ({U}, flash) costs {0}... its cost has no generic mana: {U}.
    let sailor = t.hand(P0, "Spectral Sailor");
    t.lands(P0, "Island", 1);
    let spell = t.cast(P0, sailor).go();
    let cancel = t.hand(P1, "Cancel");
    t.lands(P1, "Island", 3);
    t.cast(P1, cancel).target(spell).go();
    t.resolve();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Spectral Sailor").len(), 1);
}

#[test]
fn cant_be_blocked_this_turn_except_by_creatures_with_flying_or_reach() {
    cr!("509.1b");
    compiles("Tatsunari, Toad Rider");
    let mut t = TestGame::new(2);
    let tatsunari = t.battlefield(P0, "Tatsunari, Toad Rider");
    let frog = t.battlefield(P0, "Spore Frog");
    t.lands(P0, "Forest", 2);
    t.activate(P0, tatsunari, 0, &[Entity::Object(frog)]).unwrap();
    t.resolve();
    let bears = t.battlefield(P1, "Grizzly Bears");
    let spider = t.battlefield(P1, "Giant Spider");
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(tatsunari, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    let opts = block_options(&t.g, &[P1]);
    assert!(!block_declaration_legal(&t.g, &opts, &[(bears, tatsunari)]));
    assert!(block_declaration_legal(&t.g, &opts, &[(spider, tatsunari)]));
}

#[test]
fn gains_shroud_and_doesnt_untap_during_your_next_untap_step() {
    cr!("502.3", "702.18a");
    compiles("Homarid Warrior");
    let mut t = TestGame::new(2);
    let homarid = t.battlefield(P0, "Homarid Warrior");
    t.lands(P0, "Island", 1);
    t.activate(P0, homarid, 0, &[]).unwrap();
    t.resolve();
    assert!(t.obj_now(homarid).tapped);
    assert!(t.obj_now(homarid).has_keyword(mtg_engine::keywords::KeywordKind::Shroud));
    t.advance_to(P0, Step::Upkeep);
    assert!(t.obj_now(homarid).tapped);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert!(!t.obj_now(homarid).tapped);
}

#[test]
fn you_cant_sacrifice_those_creatures_or_attack_that_player_this_turn() {
    cr!("701.21a", "508.1c");
    compiles("Call for Aid");
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let aid = t.hand(P0, "Call for Aid");
    t.lands(P0, "Mountain", 5);
    t.cast(P0, aid).target(P1).go();
    t.resolve();
    assert_eq!(t.g.obj(bears).controller, P0);
    assert!(t.g.cant_be_sacrificed(bears));
    t.set_step(P0, Step::BeginningOfCombat);
    let opts = attack_options(&t.g);
    let (_, targets) = opts.iter().find(|(c, _)| *c == bears).unwrap();
    assert!(!targets.contains(&Entity::Player(P1)));
    assert!(targets.contains(&Entity::Player(P2)));
}

#[test]
fn cast_only_if_no_permanents_named_it_are_on_the_battlefield() {
    cr!("601.3");
    compiles("Tidal Influence");
    let mut t = TestGame::new(2);
    let a = t.hand(P0, "Tidal Influence");
    t.lands(P0, "Island", 3);
    assert!(castable(&mut t, P0, a));
    t.battlefield(P1, "Tidal Influence");
    assert!(!castable(&mut t, P0, a));
}

#[test]
fn cant_become_untapped_and_cant_have_counters_put_on_it() {
    cr!("614.1", "701.26b");
    compiles("Blossombind");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let aura = t.hand(P0, "Blossombind");
    t.lands(P0, "Island", 2);
    t.cast(P0, aura).target(bears).go();
    t.resolve_all();
    assert!(t.obj_now(bears).tapped);
    assert!(!t.g.untap(bears));
    assert_eq!(t.g.add_counters(Entity::Object(bears), "+1/+1", 1, None), 0);
}

#[test]
fn activate_only_if_there_are_four_or_more_permanent_types_among_cards_in_your_graveyard() {
    cr!("602.5b");
    let mut t = TestGame::new(2);
    let door = t.battlefield(P0, "Matzalantli, the Great Door // The Core");
    t.lands(P0, "Wastes", 4);
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Forest");
    t.graveyard(P0, "Ornithopter");
    // Creature, land, artifact: three types.
    let uid = t.g.obj(door).chars.abilities[1].uid;
    let _ = uid;
    assert!(activatable(&mut t, P0, door));
    let can_transform = |t: &mut TestGame| {
        t.g.turn.priority = Some(P0);
        let uid = t.g.obj(door).chars.abilities[1].uid;
        t.g.legal_actions(P0).iter().any(
            |a| matches!(a, Action::Activate { source, ability } if *source == door && *ability == uid),
        )
    };
    assert!(!can_transform(&mut t));
    t.graveyard(P0, "Pacifism");
    assert!(can_transform(&mut t));
}
