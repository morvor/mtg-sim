//! Casting as though a spell had flash, or for free, depending on which spell it is and how
//! it's proposed (`oracle/patterns/cast_timing_options.rs`; CR 601.3b, 601.3c, 601.2e,
//! 118.9).

use mtg_engine::object::CastMethod;
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
fn silver_scrutiny_has_flash_only_with_x_three_or_less() {
    cr!("601.3b", "601.2e");
    assert_supported(&["Silver Scrutiny"]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 6);
    for _ in 0..6 {
        t.library_top(P0, "Island");
    }
    let s = t.hand(P0, "Silver Scrutiny");
    t.advance_to(P0, Step::Upkeep);
    // X = 4 during the upkeep: illegal, and the card stays in hand.
    assert!(t.cast(P0, s).x(4).try_go().is_err());
    assert!(t.in_hand(P0, "Silver Scrutiny"));
    let before = t.hand_size(P0);
    t.cast(P0, s).x(2).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), before - 1 + 2);
}

#[test]
fn flash_photography_has_flash_only_targeting_your_permanent() {
    cr!("601.3b", "601.2e");
    assert_supported(&["Flash Photography"]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 4);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    let fp = t.hand(P0, "Flash Photography");
    t.advance_to(P0, Step::Upkeep);
    assert!(t.cast(P0, fp).target(theirs).try_go().is_err());
    t.cast(P0, fp).target(mine).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 2);
}

#[test]
fn the_blue_spirit_first_creature_spell_each_turn_has_flash() {
    cr!("601.3b");
    assert_supported(&["The Blue Spirit"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Blue Spirit");
    t.lands(P0, "Forest", 4);
    let a = t.hand(P0, "Grizzly Bears");
    let b = t.hand(P0, "Grizzly Bears");
    t.advance_to(P0, Step::Upkeep);
    t.cast(P0, a).go();
    t.resolve_all();
    // The second creature spell this turn: no flash.
    assert!(t.cast(P0, b).try_go().is_err());
}

#[test]
fn radagast_first_creature_spell_costs_less_and_has_flash() {
    cr!("601.3b", "601.2f");
    assert_supported(&["Radagast of Rhosgobel"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Radagast of Rhosgobel");
    t.lands(P0, "Forest", 4);
    let wurm = t.hand(P0, "Craw Wurm");
    t.advance_to(P0, Step::Upkeep);
    // {4}{G}{G} costs {2}{G}{G}, during the upkeep.
    t.cast(P0, wurm).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Craw Wurm").len(), 1);
}

#[test]
fn weftwalking_first_spell_each_player_casts_on_their_turn_is_free() {
    cr!("118.9");
    assert_supported(&["Weftwalking"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Weftwalking");
    let wurm = t.hand(P0, "Craw Wurm");
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, wurm).method(CastMethod::Free).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Craw Wurm").len(), 1);
    // The second spell: not free.
    assert!(t.cast(P0, bears).method(CastMethod::Free).try_go().is_err());
    // The opponent's first spell on their turn.
    t.advance_to(P1, Step::PrecombatMain);
    let giant = t.hand(P1, "Hill Giant");
    t.cast(P1, giant).method(CastMethod::Free).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    // Not during another player's turn: an instant on P0's turn.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Weftwalking");
    let bolt = t.hand(P1, "Lightning Bolt");
    assert!(t
        .cast(P1, bolt)
        .method(CastMethod::Free)
        .target(Entity::Player(P0))
        .try_go()
        .is_err());
}

#[test]
fn qasali_ambusher_free_with_flash_while_attacked() {
    cr!("601.3c", "118.9");
    assert_supported(&["Qasali Ambusher"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Forest");
    t.battlefield(P0, "Plains");
    let q = t.hand(P0, "Qasali Ambusher");
    // Not attacked: no.
    assert!(t.cast(P0, q).method(CastMethod::Free).try_go().is_err());
    let attacker = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P1, Step::BeginningOfCombat);
    t.answer(
        P1,
        DecisionKind::Attackers,
        mtg_engine::decision::Answer::Attackers(vec![(attacker, Entity::Player(P0))]),
    );
    t.advance_to(P1, Step::DeclareAttackers);
    t.cast(P0, q).method(CastMethod::Free).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Qasali Ambusher").len(), 1);
}
