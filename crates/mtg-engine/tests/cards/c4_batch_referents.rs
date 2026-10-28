//! cards4-referents-statics, family `batch-referents` (and Twilight Diviner's group
//! condition): the objects of a "one or more" trigger (CR 603.2c) as a group a choice is
//! made from on resolution.

use mtg_engine::testing::*;
use mtg_engine::events::MoveCause;
use mtg_engine::object::Zone;
use mtg_engine::*;

fn supported(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

#[test]
fn rinoa_returns_one_of_the_attackers_that_died_tapped_with_a_flying_counter() {
    cr!("603.2c", "603.10a", "400.7", "122.1b");
    supported("Rinoa, Angel Wing");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rinoa, Angel Wing");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_yes(P0, true);
    t.attack(&[(bears, Entity::Player(P1))], &[(giant, bears)]);
    t.resolve_all();
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1, "{}", t.dump_log());
    let b = back[0];
    assert!(t.g.obj(b).tapped);
    assert_eq!(t.counters(b, "flying"), 1);
    assert!(t.g.obj(b).has_keyword(mtg_engine::keywords::KeywordKind::Flying));
}

#[test]
fn rinoa_returns_a_creature_only_once_each_turn() {
    cr!("603.2h");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rinoa, Angel Wing");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_yes(P0, true);
    t.attack(&[(bears, Entity::Player(P1))], &[(giant, bears)]);
    t.resolve_all();
    let b = t.named_on_battlefield("Grizzly Bears")[0];
    // It dies again this turn (not attacking: doesn't trigger anyway); a second attacker
    // dying this turn isn't returned.
    t.g.destroy(b, None);
    t.settle();
    t.resolve_all();
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
}

#[test]
fn rinoa_doesnt_return_creatures_that_werent_attacking() {
    cr!("603.2c");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rinoa, Angel Wing");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.g.destroy(bears, None);
    t.settle();
    t.resolve_all();
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
}

fn diviner_sees(from_graveyard: bool) -> usize {
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Twilight Diviner");
    let bears = if from_graveyard {
        t.graveyard(P0, "Grizzly Bears")
    } else {
        t.hand(P0, "Grizzly Bears")
    };
    // Put onto the battlefield by an effect.
    t.g.move_object(bears, Zone::Battlefield, MoveCause::Effect, Some(P0));
    t.settle();
    t.resolve_all();
    t.named_on_battlefield("Grizzly Bears").len()
}

#[test]
fn twilight_diviner_copies_a_creature_that_entered_from_a_graveyard() {
    cr!("603.2c", "603.4", "707.2");
    interp!("batch-they-condition-all");
    supported("Twilight Diviner");
    assert_eq!(diviner_sees(true), 2);
    // From the hand: no copy.
    assert_eq!(diviner_sees(false), 1);
}

#[test]
fn hordewing_skaab_draws_and_discards_for_each_opponent_dealt_damage() {
    cr!("603.2c", "510.2");
    ruling!(
        "Hordewing Skaab",
        "The number of cards you discard is equal to the number of opponents that were dealt combat damage by your Zombies"
    );
    supported("Hordewing Skaab");
    let mut t = TestGame::new(3);
    let skaab = t.battlefield(P0, "Hordewing Skaab");
    let corpse = t.battlefield(P0, "Walking Corpse");
    let corpse2 = t.battlefield(P0, "Walking Corpse");
    for _ in 0..5 {
        t.library_top(P0, "Island");
    }
    let hand = t.hand_size(P0);
    let library = t.library_size(P0);
    t.answer_yes(P0, true);
    // Three Zombies, two opponents dealt damage: draw two, discard two.
    t.attack(
        &[
            (skaab, Entity::Player(P1)),
            (corpse, Entity::Player(P2)),
            (corpse2, Entity::Player(P2)),
        ],
        &[],
    );
    t.resolve_all();
    assert_eq!(t.library_size(P0), library - 2, "{}", t.dump_log());
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.graveyard_size(P0), 2);
}

#[test]
fn breeches_exiles_the_top_card_of_each_opponent_dealt_damage_only() {
    cr!("603.2c");
    supported("Breeches, Brazen Plunderer");
    let mut t = TestGame::new(3);
    let breeches = t.battlefield(P0, "Breeches, Brazen Plunderer");
    let top1 = t.library_top(P1, "Forest");
    let top2 = t.library_top(P2, "Forest");
    t.attack(&[(breeches, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.zone(top1), Zone::Exile, "{}", t.dump_log());
    assert_ne!(t.zone(top2), Zone::Exile);
}

#[test]
fn soundwave_exiles_a_card_with_mana_value_equal_to_the_damage_from_that_players_graveyard() {
    cr!("603.2c", "707.12");
    supported("Soundwave, Sonic Spy // Soundwave, Superior Captain");
    let mut t = TestGame::new(2);
    let sw = t.battlefield(P0, "Soundwave, Sonic Spy // Soundwave, Superior Captain");
    let giant = t.battlefield(P0, "Hill Giant");
    t.g.objects[giant.0 as usize].kind = mtg_engine::object::ObjKind::Token;
    let div = t.graveyard(P1, "Divination");
    let shock = t.graveyard(P1, "Shock");
    for _ in 0..3 {
        t.library_top(P0, "Island");
    }
    let hand = t.hand_size(P0);
    t.answer_targets(P0, &[Entity::Object(div)]);
    t.answer_yes(P0, true);
    t.attack(&[(giant, Entity::Player(P1))], &[]);
    t.resolve_all();
    // 3 combat damage: Divination (mana value 3) is exiled and its copy cast.
    assert_eq!(t.zone(div), Zone::Exile, "{}", t.dump_log());
    assert_ne!(t.zone(shock), Zone::Exile);
    assert_eq!(t.hand_size(P0), hand + 2);
    assert_eq!(t.obj_now(sw).face, mtg_engine::object::FaceState::Back);
}

#[test]
fn turf_war_the_attackers_controller_gains_one_of_the_contested_lands() {
    cr!("603.4", "608.2c");
    supported("Turf War");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Turf War");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let contested = t.battlefield(P1, "Forest");
    let plain = t.battlefield(P1, "Forest");
    t.g.add_counters(Entity::Object(contested), "contested", 1, None);
    t.g.objects[contested.0 as usize].tapped = true;
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.obj_now(contested).controller, P0, "{}", t.dump_log());
    assert!(!t.obj_now(contested).tapped);
    assert_eq!(t.obj_now(plain).controller, P1);
}
