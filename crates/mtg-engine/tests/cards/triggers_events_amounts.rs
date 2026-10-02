//! Trigger events with amounts and counters: "deals N or more damage to ...", "is dealt N
//! or more damage by a single source" (one damage event, CR 120.3), counters put on one or
//! more objects (CR 122.6, once per batch, CR 603.2c), counters removed, the Nth counter
//! (CR 122.7), and a Saga's final chapter ability resolving (CR 714.2c).

use mtg_engine::testing::*;
use mtg_engine::*;

fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.is_fully_supported(),
        "{name}: unsupported {:?}",
        c.unsupported_text()
    );
}

#[test]
fn deals_six_or_more_damage_to_an_opponent() {
    cr!("120.3", "603.2");
    supported("Deus of Calamity");
    ruling!(
        "Deus of Calamity",
        "when Deus of Calamity deals 6 damage to an opponent at one time"
    );
    let mut t = TestGame::new(2);
    let deus = t.battlefield(P0, "Deus of Calamity");
    let land = t.battlefield(P1, "Forest");
    t.answer_targets(P0, &[Entity::Object(land)]);
    t.g.deal_damage(deus, Entity::Player(P1), 5, false);
    t.resolve_all();
    t.g.deal_damage(deus, Entity::Player(P1), 1, false);
    t.resolve_all();
    assert!(t.on_battlefield(land), "5 and then 1 damage: no");
    t.answer_targets(P0, &[Entity::Object(land)]);
    t.g.deal_damage(deus, Entity::Player(P1), 6, false);
    t.resolve_all();
    assert!(!t.on_battlefield(land));
}

#[test]
fn a_source_you_control_deals_five_or_more_damage_to_a_player() {
    cr!("120.3");
    supported("Dragonborn Champion");
    ruling!(
        "Dragonborn Champion",
        "only triggers if 5 or more damage is dealt by the same source at the same time"
    );
    let mut t = TestGame::new(2);
    let champ = t.battlefield(P0, "Dragonborn Champion");
    let bear = t.battlefield(P0, "Grizzly Bears");
    let hand = t.hand_size(P0);
    // Two sources at once, 2 + 5: one draw (only the 5).
    t.g.deal_damage_batch(
        vec![
            (bear, Entity::Player(P1), 2),
            (champ, Entity::Player(P1), 5),
        ],
        true,
    );
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // An opponent's source: no.
    let theirs = t.battlefield(P1, "Craw Wurm");
    t.g.deal_damage(theirs, Entity::Player(P0), 6, false);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn an_opponent_is_dealt_three_or_more_damage_by_a_single_source() {
    cr!("120.3");
    supported("Pain Magnification");
    ruling!(
        "Pain Magnification",
        "this ability will trigger once for each of those opponents"
    );
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Pain Magnification");
    let wurm = t.battlefield(P0, "Craw Wurm");
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.hand(P1, "Island");
    t.hand(P2, "Island");
    t.g.deal_damage_batch(
        vec![(wurm, Entity::Player(P1), 3), (wurm, Entity::Player(P2), 3)],
        false,
    );
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 0);
    assert_eq!(t.hand_size(P2), 0);
    // Two sources adding up to 3: no.
    t.hand(P1, "Island");
    t.g.deal_damage_batch(
        vec![(bear, Entity::Player(P1), 2), (wurm, Entity::Player(P1), 1)],
        false,
    );
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 1);
}

#[test]
fn counters_put_on_one_or_more_humans_you_control() {
    cr!("122.6", "603.2c");
    supported("Cloaked Cadet");
    ruling!(
        "Cloaked Cadet",
        "You draw only one card no matter how many counters are placed"
    );
    let mut t = TestGame::new(2);
    let cadet = t.battlefield(P0, "Cloaked Cadet");
    let human = t.battlefield(P0, "Raging Goblin");
    let hand = t.hand_size(P0);
    // Not a Human: no.
    t.g.add_counters(Entity::Object(human), "+1/+1", 1, None);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    t.g.add_counters(Entity::Object(cadet), "+1/+1", 2, None);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    t.g.add_counters(Entity::Object(cadet), "+1/+1", 1, None);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1, "only once each turn");
}

#[test]
fn you_put_counters_on_one_or_more_other_heroes() {
    cr!("122.6");
    supported("Invisible Woman, Sue Storm");
    let walls = |t: &TestGame| {
        t.g.battlefield
            .iter()
            .filter(|id| t.g.obj(**id).chars.has_subtype("Wall"))
            .count()
    };
    let mut t = TestGame::new(2);
    let sue = t.battlefield(P0, "Invisible Woman, Sue Storm");
    // On Sue herself (not another Hero): no.
    t.g.add_counters(Entity::Object(sue), "+1/+1", 1, None);
    t.resolve_all();
    assert_eq!(walls(&t), 0);
    let hero = t.battlefield(P0, "Invisible Woman, Sue Storm");
    t.answer_yes(P0, true);
    t.g.add_counters(Entity::Object(hero), "+1/+1", 1, Some(sue));
    t.resolve_all();
    assert!(walls(&t) >= 1);
}

#[test]
fn counters_put_on_it_for_the_first_time_each_turn_granted() {
    cr!("603.2", "122.6");
    supported("Danny Pink");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Danny Pink");
    let bear = t.battlefield(P0, "Grizzly Bears");
    let hand = t.hand_size(P0);
    t.g.add_counters(Entity::Object(bear), "+1/+1", 1, None);
    t.resolve_all();
    t.g.add_counters(Entity::Object(bear), "+1/+1", 1, None);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn loyalty_counters_removed_deal_that_much_damage() {
    cr!("306.8", "603.2c");
    supported("Chandra, Fire Artisan");
    ruling!(
        "Chandra, Fire Artisan",
        "her first ability triggers only once"
    );
    let mut t = TestGame::new(2);
    let chandra = t.battlefield(P0, "Chandra, Fire Artisan");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.g.deal_damage_batch(
        vec![
            (a, Entity::Object(chandra), 1),
            (b, Entity::Object(chandra), 1),
        ],
        true,
    );
    t.resolve_all();
    assert_eq!(t.life(P1), 18, "one trigger for both: 2 damage");
}

#[test]
fn the_twelfth_hour_counter() {
    cr!("122.7");
    supported("Midnight Clock");
    let mut t = TestGame::new(2);
    let clock = t.battlefield(P0, "Midnight Clock");
    t.hand(P0, "Island");
    t.g.add_counters(Entity::Object(clock), "hour", 11, None);
    t.resolve_all();
    assert!(t.on_battlefield(clock));
    assert_eq!(t.hand_size(P0), 1);
    t.g.add_counters(Entity::Object(clock), "hour", 1, None);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 7);
    assert!(t.in_exile("Midnight Clock"));
}

#[test]
fn the_final_chapter_ability_of_a_saga_resolves() {
    cr!("714.2c", "608.2p");
    supported("Narci, Fable Singer");
    ruling!(
        "Narci, Fable Singer",
        "the ability with the greatest chapter number among chapter abilities"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Narci, Fable Singer");
    let saga = t.battlefield(P0, "History of Benalia");
    t.g.add_counters(Entity::Object(saga), "lore", 1, None);
    t.resolve_all();
    assert_eq!(t.life(P1), 20, "chapter I isn't the final chapter");
    t.g.add_counters(Entity::Object(saga), "lore", 2, None);
    t.resolve_all();
    // History of Benalia's mana value is 3.
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P0), 23);
}
