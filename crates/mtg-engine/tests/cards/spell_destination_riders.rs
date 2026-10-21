//! Where a spell goes as it resolves, changed for that spell only (CR 608.2n, 614.1a,
//! 611.2f): Soulfire Grand Master, Feather, the Redeemed, Forger's Foundry; and Jade Orb
//! of Dragonkind's counter and hexproof for the spell its mana was spent on (CR 400.7a).

use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

#[test]
fn soulfire_grand_master_returns_the_next_instant_from_hand_to_hand() {
    cr!("611.2f", "608.2n", "614.1a");
    assert_supported(&["Soulfire Grand Master"]);
    let mut t = TestGame::new(2);
    let sgm = t.battlefield(P0, "Soulfire Grand Master");
    t.lands(P0, "Mountain", 6);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.activate(P0, sgm, 0, &[]).unwrap();
    t.resolve_all();
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(t.in_hand(P0, "Lightning Bolt"));
    // Only the next one.
    let bolt = t.g.player(P0).hand[0];
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
}

#[test]
fn soulfire_grand_master_not_for_a_spell_cast_from_the_graveyard() {
    cr!("611.2f");
    let mut t = TestGame::new(2);
    let sgm = t.battlefield(P0, "Soulfire Grand Master");
    t.lands(P0, "Mountain", 6);
    t.lands(P0, "Island", 1);
    t.activate(P0, sgm, 0, &[]).unwrap();
    t.resolve_all();
    // Flashback (from the graveyard): exiled as usual.
    let think = t.graveyard(P0, "Think Twice");
    t.cast(P0, think)
        .method(mtg_engine::object::CastMethod::Keyword(KeywordKind::Flashback))
        .go();
    t.resolve_all();
    assert!(t.in_exile("Think Twice"));
    // That spell didn't use it up: the next instant cast from the hand returns to it.
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(t.in_hand(P0, "Lightning Bolt"));
}

#[test]
fn feather_exiles_the_spell_and_returns_it_at_the_end_step() {
    cr!("608.2n", "603.7a");
    assert_supported(&["Feather, the Redeemed"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Feather, the Redeemed");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    let growth = t.hand(P0, "Giant Growth");
    t.cast(P0, growth).target(bears).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5));
    assert!(t.in_exile("Giant Growth"));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.in_hand(P0, "Giant Growth"));
    // Targeting an opponent's creature: no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Feather, the Redeemed");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    let growth = t.hand(P0, "Giant Growth");
    t.cast(P0, growth).target(theirs).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Giant Growth"));
}

#[test]
fn forgers_foundry_exiles_the_spell_its_mana_paid_for() {
    cr!("106.6", "608.2n", "607.2a");
    assert_supported(&["Forger's Foundry"]);
    let mut t = TestGame::new(2);
    let ff = t.battlefield(P0, "Forger's Foundry");
    t.activate(P0, ff, 0, &[]).unwrap();
    let opt = t.hand(P0, "Opt");
    t.library_top(P0, "Island");
    t.library_top(P0, "Island");
    t.answer_yes(P0, true);
    t.cast(P0, opt).go();
    t.resolve_all();
    assert!(t.in_exile("Opt"));
    // Its second ability casts it from exile.
    t.g.objects[ff.0 as usize].tapped = false;
    t.lands(P0, "Island", 5);
    t.library_top(P0, "Island");
    let exiled = t.g.current(opt);
    t.answer_choose(P0, &[Entity::Object(exiled)]);
    t.answer_yes(P0, true);
    t.activate(P0, ff, 1, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Opt"));
}

#[test]
fn jade_orb_dragon_enters_with_a_counter_and_hexproof() {
    cr!("106.6", "614.1c", "400.7a");
    assert_supported(&["Jade Orb of Dragonkind"]);
    let mut t = TestGame::new(2);
    let orb = t.battlefield(P0, "Jade Orb of Dragonkind");
    t.lands(P0, "Mountain", 5);
    t.activate(P0, orb, 0, &[]).unwrap();
    let dragon = t.hand(P0, "Shivan Dragon");
    t.cast(P0, dragon).go();
    t.resolve_all();
    let d = t.named_on_battlefield("Shivan Dragon")[0];
    assert_eq!(t.counters(d, "+1/+1"), 1);
    assert!(t.obj_now(d).has_keyword(KeywordKind::Hexproof));
    let _ = Zone::Exile;
}
