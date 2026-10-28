//! Structure-coverage tests for ability structures that only the random card pools of the
//! property and API tests happened to exercise: counting the paper cards with an online-only
//! oracle printing as paper (item `paper-scope-1`) changed those pools, and these structures
//! lost their only exercise. Each test checks a real card's ability does what its text says.

use super::structure_seed_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn pool(t: &TestGame, p: PlayerId) -> usize {
    t.g.players[p.idx()].mana_pool.mana.len()
}

#[test]
fn prismite_and_llanowar_envoy_filter_mana_into_any_color() {
    cr!("605.1a", "605.3a", "106.1b");
    // "{2}: Add one mana of any color." / "{1}{G}: Add one mana of any color."
    for (name, land) in [("Prismite", "Island"), ("Llanowar Envoy", "Forest")] {
        supported(name);
        let mut t = TestGame::new(2);
        let src = t.battlefield(P0, name);
        t.lands(P0, land, 2);
        let r = t.activate(P0, src, 0, &[]).unwrap();
        assert!(r.is_none(), "{name}: a mana ability doesn't use the stack");
        assert_eq!(t.stack_len(), 0);
        assert_eq!(pool(&t, P0), 1, "{name}: two mana in, one out");
    }
}

#[test]
fn broken_fall_returns_itself_to_regenerate_a_creature() {
    cr!("701.19a", "602.2");
    // "Return this enchantment to its owner's hand: Regenerate target creature."
    supported("Broken Fall");
    let mut t = TestGame::new(2);
    let fall = t.battlefield(P0, "Broken Fall");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, fall, 0, &[Entity::Object(bears)]).unwrap();
    assert!(t.in_hand(P0, "Broken Fall"), "the cost returns it");
    t.resolve();
    // The shield replaces the next destruction: the bears stay, tapped.
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(bears).go();
    t.resolve();
    assert!(t.on_battlefield(bears));
    assert!(t.obj_now(bears).tapped);
}

#[test]
fn hoodwink_bounces_a_land() {
    cr!("608.2b");
    // "Return target artifact, enchantment, or land to its owner's hand."
    supported("Hoodwink");
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P1, "Forest");
    t.lands(P0, "Island", 2);
    let h = t.hand(P0, "Hoodwink");
    t.cast(P0, h).target(forest).go();
    t.resolve();
    assert!(t.in_hand(P1, "Forest"));
}

#[test]
fn consign_to_dream_puts_a_green_permanent_on_top_of_its_library() {
    cr!("608.2c");
    // "Return target permanent to its owner's hand. If that permanent is red or green, put
    // it on top of its owner's library instead."
    supported("Consign to Dream");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 3);
    let c = t.hand(P0, "Consign to Dream");
    t.cast(P0, c).target(bears).go();
    t.resolve();
    let top = *t.g.players[P1.idx()].library.last().unwrap();
    assert_eq!(&*t.obj_now(top).chars.name, "Grizzly Bears");
    assert!(!t.in_hand(P1, "Grizzly Bears"));
}

#[test]
fn fated_intervention_scries_on_its_controllers_turn() {
    cr!("701.22a", "111.1");
    // "Create two 3/3 green Centaur enchantment creature tokens. If it's your turn, scry 2."
    supported("Fated Intervention");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 5);
    let f = t.hand(P0, "Fated Intervention");
    t.cast(P0, f).go();
    t.resolve();
    assert_eq!(count_subtype(&t, P0, "Centaur"), 2);
    assert_eq!(scries(&t), vec![2]);
}

#[test]
fn sams_curiosity_gains_life_and_makes_a_lander() {
    cr!("119.3", "111.10");
    // "You gain 2 life. Create a Lander token."
    supported("Sami's Curiosity");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    let s = t.hand(P0, "Sami's Curiosity");
    t.cast(P0, s).go();
    t.resolve();
    assert_eq!(t.life(P0), 22);
    assert_eq!(count_subtype(&t, P0, "Lander"), 1);
}

#[test]
fn hellraiser_goblin_gives_your_creatures_haste() {
    cr!("611.3a", "613.1f");
    // "Creatures you control have haste and attack each combat if able."
    supported("Hellraiser Goblin");
    let mut t = TestGame::new(2);
    let mine = t.battlefield_sick(P0, "Grizzly Bears");
    let theirs = t.battlefield_sick(P1, "Grizzly Bears");
    t.battlefield(P0, "Hellraiser Goblin");
    t.g.recompute();
    assert!(t.obj_now(mine).has_keyword(KeywordKind::Haste));
    assert!(!t.obj_now(theirs).has_keyword(KeywordKind::Haste));
    assert!(t.g.can_attack(mine));
}

#[test]
fn luxknight_breacher_enters_with_a_counter_per_other_creature_and_artifact() {
    cr!("614.1c", "122.6");
    // "This creature enters with a +1/+1 counter on it for each other creature and/or
    // artifact you control."
    supported("Luxknight Breacher");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Ornithopter");
    t.battlefield(P0, "Sol Ring");
    t.battlefield(P1, "Grizzly Bears");
    let lux = t.enter(P0, "Luxknight Breacher");
    assert_eq!(t.counters(lux, "+1/+1"), 3);
}

#[test]
fn goblin_spymaster_gives_each_opponent_a_goblin_at_their_end_step() {
    cr!("603.2", "513.1");
    // "At the beginning of each opponent's end step, that player creates a 1/1 red Goblin
    // creature token with "Creatures you control attack each combat if able.""
    supported("Goblin Spymaster");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Goblin Spymaster");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(count_subtype(&t, P1, "Goblin"), 0, "not on its controller's turn");
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert_eq!(count_subtype(&t, P1, "Goblin"), 1);
    assert_eq!(count_subtype(&t, P0, "Goblin"), 1, "only the Spymaster");
}

#[test]
fn matt_murdock_may_pay_one_for_a_counter_at_the_beginning_of_combat() {
    cr!("603.12", "603.2");
    // "At the beginning of combat on your turn, you may pay {1}. When you do, put a +1/+1
    // counter on target creature you control."
    supported("Matt Murdock, Justice Seeker");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Matt Murdock, Justice Seeker");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 1);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 1);
}

#[test]
fn shrine_of_piercing_vision_looks_at_one_card_per_charge_counter() {
    cr!("701.21a", "122.1");
    // "{T}, Sacrifice this artifact: Look at the top X cards of your library, where X is the
    // number of charge counters on this artifact. Put one of those cards into your hand and
    // the rest on the bottom of your library in any order."
    supported("Shrine of Piercing Vision");
    let mut t = TestGame::new(2);
    let shrine = t.battlefield(P0, "Shrine of Piercing Vision");
    t.g.add_counters(Entity::Object(shrine), "charge", 3, None);
    let hand = t.hand_size(P0);
    let library = t.library_size(P0);
    t.activate(P0, shrine, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Shrine of Piercing Vision"));
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.library_size(P0), library - 1);
}

#[test]
fn pardic_dragon_lets_a_casting_opponent_add_a_time_counter() {
    cr!("702.62b", "603.4");
    // "Whenever an opponent casts a spell, if this card is suspended, that player may put a
    // time counter on it."
    supported("Pardic Dragon");
    let mut t = TestGame::new(2);
    let dragon = t.exile(P0, "Pardic Dragon");
    t.g.add_counters(Entity::Object(dragon), "time", 2, None);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.answer_yes(P1, true);
    t.cast(P1, bolt).target(P0).go();
    t.resolve_all();
    assert_eq!(t.zone(dragon), Zone::Exile);
    assert_eq!(t.counters(dragon, "time"), 3);
}
