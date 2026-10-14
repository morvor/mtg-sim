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

#[test]
fn lord_of_the_nazgul_wraiths_have_protection_from_ring_bearers() {
    cr!("702.16b", "701.54e");
    assert_supported(&["Lord of the Nazgûl"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lord of the Nazgûl");
    let wraith = t.battlefield(P0, "Nazgûl");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.players[1].ring_bearer = Some(bears);
    t.g.recompute();
    let ctx = mtg_engine::eval::Ctx::new(None, P0);
    let prot = t
        .obj_now(wraith)
        .chars
        .keywords()
        .filter(|k| k.kind == mtg_engine::keywords::KeywordKind::Protection)
        .filter_map(|k| k.filter.clone())
        .collect::<Vec<_>>();
    assert_eq!(prot.len(), 1);
    assert!(t.g.matches(bears, &prot[0], &ctx));
    assert!(!t.g.matches(giant, &prot[0], &ctx));
}

#[test]
fn medomai_cant_attack_during_extra_turns() {
    cr!("500.7", "508.1c");
    assert_supported(&["Medomai the Ageless"]);
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Medomai the Ageless");
    t.set_step(P0, Step::BeginningOfCombat);
    t.g.turn.extra = true;
    t.attack(&[(m, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 20);
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Medomai the Ageless");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(m, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 16);
}

#[test]
fn vraska_gives_poison_counters_up_to_nine() {
    cr!("122.1f");
    assert_supported(&["Vraska, Betrayal's Sting"]);
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Vraska, Betrayal's Sting");
    t.g.add_counters(Entity::Object(v), "loyalty", 10, None);
    t.g.add_counters(Entity::Player(P1), "poison", 3, None);
    t.activate(P0, v, 2, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.g.player(P1).counter("poison"), 9);
    assert!(!t.has_lost(P1));
}

#[test]
fn radiant_destiny_gives_vigilance_with_the_citys_blessing() {
    cr!("702.131c");
    assert_supported(&["Radiant Destiny"]);
    let mut t = TestGame::new(2);
    let i = mtg_engine::types::subtype_lists()
        .creature
        .iter()
        .position(|s| s == "Bear")
        .expect("creature type");
    t.answer(P0, DecisionKind::Option, Answer::Index(i));
    t.enter(P0, "Radiant Destiny");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.recompute();
    assert_eq!(t.pt(bears), (3, 3));
    assert!(!t.obj_now(bears).chars.has_keyword(keywords::KeywordKind::Vigilance));
    t.g.players[0].has_citys_blessing = true;
    t.g.recompute();
    assert!(t.obj_now(bears).chars.has_keyword(keywords::KeywordKind::Vigilance));
}
