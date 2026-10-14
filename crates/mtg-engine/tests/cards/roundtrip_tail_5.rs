//! roundtrip-tail-5 (docs/agents/items/roundtrip-tail-5.md): the second half of the 3-card
//! wording clusters, and the behavior doubts the roundtrip-clusters-4 review left open.

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn vesuva_enters_untapped_when_it_copies_nothing() {
    cr!("614.1c", "707.9");
    ruling!(
        "Vesuva",
        "If you don't choose a land on the battlefield, Vesuva enters the battlefield untapped as itself"
    );
    // "You may have this land enter tapped as a copy of any land on the battlefield."
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Forest");
    t.answer_choose(P0, &[]);
    let v = t.enter(P0, "Vesuva");
    t.settle();
    assert_eq!(t.obj_now(v).chars.name, "Vesuva");
    assert!(!t.obj_now(v).tapped, "{}", t.dump_log());
}

#[test]
fn vesuva_enters_tapped_as_a_copy() {
    cr!("614.1c", "707.9");
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P1, "Forest");
    t.answer_choose(P0, &[Entity::Object(forest)]);
    let v = t.enter(P0, "Vesuva");
    t.settle();
    assert_eq!(t.obj_now(v).chars.name, "Forest");
    assert!(t.obj_now(v).tapped, "{}", t.dump_log());
}

#[test]
fn echoing_deeps_enters_untapped_when_it_copies_nothing() {
    cr!("614.1c", "707.9");
    ruling!(
        "Echoing Deeps",
        "If you choose to have Echoing Deeps not copy anything, it enters the battlefield untapped."
    );
    let mut t = TestGame::new(2);
    t.graveyard(P1, "Forest");
    t.answer_choose(P0, &[]);
    let d = t.enter(P0, "Echoing Deeps");
    t.settle();
    let o = t.obj_now(d);
    assert_eq!(o.chars.name, "Echoing Deeps");
    assert!(!o.tapped, "{}", t.dump_log());
}

#[test]
fn callidus_assassin_enters_untapped_when_it_copies_nothing() {
    cr!("614.1c", "707.9");
    ruling!(
        "Callidus Assassin",
        "You can choose not to copy anything. In that case, Callidus Assassin enters the battlefield as a 3/3 creature."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Hill Giant");
    t.answer_choose(P0, &[]);
    let c = t.enter(P0, "Callidus Assassin");
    t.settle();
    assert_eq!(t.obj_now(c).chars.name, "Callidus Assassin");
    assert!(!t.obj_now(c).tapped, "{}", t.dump_log());
    // No "destroy up to one other target creature with the same name" trigger.
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn clackbridge_troll_sacrifices_the_chosen_creatures_at_the_same_time() {
    cr!("101.4", "603.10a", "701.21a");
    ruling!(
        "Clackbridge Troll",
        "Then all the chosen creatures are sacrificed at the same time."
    );
    // P1 sacrifices Blood Artist and P2 a Grizzly Bears: they die together, so Blood
    // Artist sees the Bears die too (CR 603.10a) and triggers twice.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Clackbridge Troll");
    let artist = t.battlefield(P1, "Blood Artist");
    let bears = t.battlefield(P2, "Grizzly Bears");
    t.answer_choose(P1, &[Entity::Object(artist)]);
    t.answer_choose(P2, &[Entity::Object(bears)]);
    t.set_step(P0, Step::PrecombatMain);
    let life = t.life(P1);
    t.advance_to_step(Step::BeginningOfCombat);
    t.resolve_all();
    assert!(!t.on_battlefield(artist) && !t.on_battlefield(bears));
    assert_eq!(t.life(P1), life + 2, "{}", t.dump_log());
}

#[test]
fn underworld_fires_exiles_only_permanents_it_actually_damaged() {
    cr!("614.1a", "702.16e");
    // "If a permanent dealt damage this way would die this turn, exile it instead": a
    // creature whose damage was prevented wasn't dealt damage this way.
    let mut t = TestGame::new(2);
    // Protection from red prevents the damage to Kor Firewalker (CR 702.16e).
    let a = t.battlefield(P1, "Kor Firewalker");
    let b = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 2);
    let fires = t.hand(P0, "Underworld Fires");
    if let Err(e) = t.cast_with(P0, fires, &[]) {
        panic!("cast: {e:?}\n{}", t.dump_log());
    }
    t.resolve_all();
    t.g.destroy(a, None);
    t.g.destroy(b, None);
    t.settle();
    assert!(t.in_graveyard(P1, "Kor Firewalker"), "{}", t.dump_log());
    assert!(t.in_exile("Hill Giant"), "{}", t.dump_log());
}
