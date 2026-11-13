//! CR 608.2h / 113.7a audit: last known information. An effect that needs information
//! from an object that is no longer in the public zone it was expected to be in — or that
//! the effect itself moved to a hidden zone — uses the object as it last existed there:
//! its controller and its power or toughness with every effect that applied to it then,
//! including an anthem that left at the same time. An ability's source that left and came
//! back is a new object; the ability still uses the source as it last existed.

use mtg_engine::card::card;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.is_fully_supported(),
        "{name}: unsupported {:?}",
        c.unsupported_text()
    );
}

fn pt(t: &mut TestGame, id: ObjectId) -> (i32, i32) {
    t.g.recompute();
    t.pt(id)
}

/// `p` casts `name` (an instant or sorcery) from hand with `lands` of mana, targeting
/// `target`, and it resolves.
fn cast_on(t: &mut TestGame, p: PlayerId, name: &str, land: &str, n: usize, target: Entity) {
    supported(name);
    t.lands(p, land, n);
    let c = t.hand(p, name);
    t.cast(p, c).target(target).go();
    t.resolve();
}

/// P0 gains control of P1's Hill Giant with Act of Treason and makes it 6/6 with Giant
/// Growth. Returns the Giant.
fn stolen_6_6_giant(t: &mut TestGame) -> ObjectId {
    let giant = t.battlefield(P1, "Hill Giant");
    cast_on(t, P0, "Act of Treason", "Mountain", 3, Entity::Object(giant));
    cast_on(t, P0, "Giant Growth", "Forest", 1, Entity::Object(giant));
    assert_eq!(t.obj_now(giant).controller, P0);
    assert_eq!(pt(t, giant), (6, 6));
    giant
}

#[test]
fn swords_to_plowshares_gives_the_last_controller_life_equal_to_the_last_known_power() {
    cr!("608.2h");
    ruling!(
        "Swords to Plowshares",
        "Use the power of the creature from when it was last on the battlefield to determine how much life is gained."
    );
    let mut t = TestGame::new(2);
    let giant = stolen_6_6_giant(&mut t);
    // "Exile target creature. Its controller gains life equal to its power." Its controller
    // and its power are as it last existed on the battlefield: P0, 6 (the card in exile is
    // P1's 3/3 Hill Giant).
    cast_on(&mut t, P1, "Swords to Plowshares", "Plains", 1, Entity::Object(giant));
    assert!(t.in_exile("Hill Giant"));
    assert_eq!(t.life(P0), 26);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn condemn_uses_the_last_known_toughness_and_controller_of_what_it_put_into_a_library() {
    cr!("608.2h");
    supported("Condemn");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    // Act of Treason gives it haste: P0 attacks P1 with it, then makes it 6/6.
    cast_on(&mut t, P0, "Act of Treason", "Mountain", 3, Entity::Object(giant));
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(giant, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.settle();
    assert!(t.g.is_attacking(giant));
    cast_on(&mut t, P0, "Giant Growth", "Forest", 1, Entity::Object(giant));
    // "Put target attacking creature on the bottom of its owner's library. Its controller
    // gains life equal to its toughness." The effect moved it to a hidden zone: its last
    // known information is used, so P0 (its controller) gains 6.
    cast_on(&mut t, P1, "Condemn", "Plains", 1, Entity::Object(giant));
    let bottom = t.g.players[P1.idx()].library[0];
    assert_eq!(t.g.obj(bottom).chars.name.as_str(), "Hill Giant");
    assert_eq!(t.life(P0), 26);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn a_dies_trigger_uses_its_power_with_an_anthem_destroyed_at_the_same_time() {
    cr!("608.2h", "113.7a", "603.10a");
    ruling!(
        "Dreadhorde Butcher",
        "Dreadhorde Butcher’s last known existence on the battlefield is checked to determine its power as its last ability resolves."
    );
    supported("Dreadhorde Butcher");
    supported("Akroma's Vengeance");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Glorious Anthem");
    let butcher = t.battlefield(P0, "Dreadhorde Butcher");
    assert_eq!(pt(&mut t, butcher), (2, 2));
    // Akroma's Vengeance destroys the Anthem and the Butcher at the same time. "When this
    // creature dies, it deals damage equal to its power to any target": immediately
    // before, the Anthem still gave it +1/+1, so it deals 2.
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.lands(P0, "Plains", 6);
    let v = t.hand(P0, "Akroma's Vengeance");
    t.cast(P0, v).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Glorious Anthem"));
    assert!(t.in_graveyard(P0, "Dreadhorde Butcher"));
    assert_eq!(t.life(P1), 18);
}

#[test]
fn an_ability_of_a_source_that_left_and_returned_uses_the_source_as_it_last_existed() {
    cr!("113.7a", "400.7", "608.2h");
    supported("Spikeshot Elder");
    supported("Cloudshift");
    let mut t = TestGame::new(2);
    let elder = t.battlefield(P0, "Spikeshot Elder");
    t.lands(P0, "Mountain", 3);
    // "{1}{R}{R}: This creature deals damage equal to its power to any target."
    t.activate(P0, elder, 0, &[Entity::Player(P1)])
        .expect("ability");
    // In response it becomes 4/4, then is exiled and returned (a new 1/1 object).
    cast_on(&mut t, P0, "Giant Growth", "Forest", 1, Entity::Object(elder));
    cast_on(&mut t, P0, "Cloudshift", "Plains", 1, Entity::Object(elder));
    let new_elder = t.g.current(elder);
    assert_ne!(new_elder, elder);
    assert!(t.on_battlefield(new_elder));
    assert_eq!(pt(&mut t, new_elder), (1, 1));
    // The ability's source is the Elder as it last existed on the battlefield: 4 damage.
    t.resolve();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.life(P1), 16);
}

#[test]
fn an_upkeep_trigger_sees_its_source_enchanted_as_it_last_existed() {
    cr!("608.2h", "113.7a", "603.4");
    ruling!(
        "Dreampod Druid",
        "Notably, if Dreampod Druid leaves the battlefield while its ability is on the stack, that ability will use Dreampod Druid’s last known information to determine whether or not it was enchanted."
    );
    supported("Dreampod Druid");
    let mut t = TestGame::new(2);
    let druid = t.battlefield(P0, "Dreampod Druid");
    let aura = t.battlefield(P0, "Holy Strength");
    t.g.attach(aura, Entity::Object(druid));
    // "At the beginning of each upkeep, if this creature is enchanted, create a 1/1 green
    // Saproling creature token." It's destroyed (and its Aura goes to the graveyard) with
    // the trigger on the stack: as it last existed it was enchanted.
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.g.destroy(druid, None);
    t.settle();
    assert!(t.in_graveyard(P0, "Holy Strength"));
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Saproling Token").len(), 1);
    // The Aura is destroyed first, then the Druid: as it last existed it wasn't enchanted,
    // so the condition is false as the ability resolves.
    let mut t = TestGame::new(2);
    let druid = t.battlefield(P0, "Dreampod Druid");
    let aura = t.battlefield(P0, "Holy Strength");
    t.g.attach(aura, Entity::Object(druid));
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.g.destroy(aura, None);
    t.g.destroy(druid, None);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Saproling Token").len(), 0);
}
