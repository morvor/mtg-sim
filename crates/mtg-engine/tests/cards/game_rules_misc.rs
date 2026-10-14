//! Mana equal to your devotion to a chosen color (CR 700.5), player counters after "you
//! may" (CR 122.1), several extra turns (CR 500.7), the player to your right (CR 101.4),
//! and phasing: "can't phase out", "this creature and that creature phase out"
//! (CR 702.26).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let c = card(n);
        assert!(
            c.unsupported_text().is_empty(),
            "{n} has unsupported text: {:?}",
            c.unsupported_text()
        );
    }
}

#[test]
fn nykthos_adds_mana_equal_to_devotion_to_the_chosen_color() {
    cr!("700.5", "605.1a");
    assert_supported(&["Nykthos, Shrine to Nyx", "Nyx Lotus"]);
    let mut t = TestGame::new(2);
    let n = t.battlefield(P0, "Nykthos, Shrine to Nyx");
    // Devotion to green: Llanowar Elves {G} + Leatherback Baloth {G}{G}{G} = 4.
    t.battlefield(P0, "Llanowar Elves");
    t.battlefield(P0, "Leatherback Baloth");
    t.lands(P0, "Wastes", 2);
    t.answer(P0, DecisionKind::Option, Answer::Index(4));
    t.activate(P0, n, 1, &[]).unwrap();
    assert_eq!(t.g.player(P0).mana_pool.count(mana::ManaType::G), 4);
}

#[test]
fn tato_farmer_may_get_rad_counters() {
    cr!("122.1");
    assert_supported(&["Tato Farmer"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tato Farmer");
    t.answer_yes(P0, true);
    t.enter(P0, "Forest");
    t.resolve_all();
    assert_eq!(t.g.player(P0).counter("rad"), 2);
}

#[test]
fn teferi_master_of_time_takes_two_extra_turns() {
    cr!("500.7");
    let mut t = TestGame::new(2);
    let teferi = t.battlefield(P0, "Teferi, Master of Time");
    t.g.add_counters(Entity::Object(teferi), "loyalty", 7, None);
    t.activate(P0, teferi, 2, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.g.extra_turns, vec![P0, P0]);
}

#[test]
fn hithlain_rope_goes_to_the_player_on_your_right() {
    cr!("101.4");
    assert_supported(&["Hithlain Rope", "Bucknard's Everfull Purse"]);
    let mut t = TestGame::new(3);
    let rope = t.battlefield(P0, "Hithlain Rope");
    t.lands(P0, "Wastes", 2);
    t.activate(P0, rope, 1, &[]).unwrap();
    t.resolve_all();
    // Turns pass to the left (P0, P1, P2): P0's right is P2.
    assert_eq!(t.obj_now(rope).controller, P2);
}

#[test]
fn spatial_binding_keeps_a_phasing_permanent_in_until_your_next_upkeep() {
    cr!("702.26a", "500.4");
    assert_supported(&["Spatial Binding", "Ertai's Familiar"]);
    let mut t = TestGame::new(2);
    let f = t.battlefield(P1, "Ertai's Familiar");
    let sb = t.battlefield(P0, "Spatial Binding");
    t.activate(P0, sb, 0, &[Entity::Object(f)]).unwrap();
    t.resolve_all();
    // P1's untap step: it doesn't phase out.
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj_now(f).phased_out);
}

#[test]
fn dream_fighter_and_its_blocker_phase_out() {
    cr!("702.26b");
    assert_supported(&["Dream Fighter"]);
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Dream Fighter");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(d, Entity::Player(P1))], &[(bears, d)]);
    assert!(t.obj_now(d).phased_out);
    assert!(t.obj_now(bears).phased_out);
}

#[test]
fn brothers_yamazaki_two_copies_are_exempt_from_the_legend_rule() {
    cr!("704.5j");
    assert_supported(&["Brothers Yamazaki", "Hall of Echoes"]);
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Brothers Yamazaki");
    let b = t.battlefield(P0, "Brothers Yamazaki");
    t.settle();
    assert!(t.on_battlefield(a) && t.on_battlefield(b));
    // A third: there aren't exactly two any more, and the legend rule applies.
    t.battlefield(P0, "Brothers Yamazaki");
    t.settle();
    assert_eq!(t.named_on_battlefield("Brothers Yamazaki").len(), 1);
}

#[test]
fn hall_of_echoes_suspends_the_legend_rule_this_turn() {
    cr!("704.5j", "707.2");
    let mut t = TestGame::new(2);
    let hall = t.battlefield(P0, "Hall of Echoes");
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    t.lands(P0, "Wastes", 5);
    t.activate(P0, hall, 1, &[Entity::Object(isamaru)]).unwrap();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Isamaru, Hound of Konda").len(), 2);
}
