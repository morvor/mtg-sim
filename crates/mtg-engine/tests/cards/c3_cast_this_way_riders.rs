//! Riders on spells cast "this way" (cards3-cast-zones): "If you cast it this way and it
//! would be put into your graveyard, exile it instead." (Eelectrocute), "If you do, ~
//! enters with a +1/+1 counter on it." (Undead Sprinter), the spell a delayed triggered
//! ability casts exiled instead (Power Pack), "If you cast a spell this way, put a +1/+1
//! counter on ~." (Helmut Zemo, Mastermind), an additional cost and "When you cast that
//! spell" (Ogre Battlecaster), "If you don't, you may cast the exiled nonland card until
//! end of turn" (Black Widow, Super Spy), and the other exiled cards put on the bottom
//! after a permission in a delayed triggered ability (Codie, Vociferous Codex).

use mtg_engine::dice::DiceIgnore;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

fn can_cast(t: &mut TestGame, p: PlayerId, card: ObjectId, targets: &[Entity]) -> bool {
    let c = t.g.current(card);
    for e in targets {
        t.answer_targets(p, &[*e]);
    }
    t.g.turn.priority = Some(p);
    let ok = t.g.cast_spell(p, c, CastMethod::Normal).is_ok();
    t.g.flush_events();
    if ok {
        t.resolve_all();
    } else {
        t.clear_answers();
    }
    ok
}

fn roll(t: &mut TestGame, p: PlayerId, natural: u32) {
    t.g.dice.loaded.push_back(natural);
    mtg_engine::dice::roll_dice(&mut t.g, p, 1, 6, DiceIgnore::None, 0, None);
    t.g.flush_events();
    t.settle();
    t.resolve_all();
}

#[test]
fn eelectrocute_cast_from_your_graveyard_is_exiled_after() {
    cr!("601.3", "614.1a");
    ruling!("Eelectrocute", "If you cast Eelectrocute from your graveyard, you still pay its costs.");
    assert_supported("Eelectrocute");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 2);
    let eel = t.graveyard(P0, "Eelectrocute");
    // Not before a 6 was rolled.
    roll(&mut t, P0, 3);
    assert!(!can_cast(&mut t, P0, eel, &[Entity::Player(P1)]));
    roll(&mut t, P0, 6);
    assert!(can_cast(&mut t, P0, eel, &[Entity::Player(P1)]));
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.zone(t.g.current(eel)), Zone::Exile);
}

#[test]
fn eelectrocute_cast_from_hand_goes_to_the_graveyard() {
    cr!("614.1a");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 2);
    let eel = t.hand(P0, "Eelectrocute");
    roll(&mut t, P0, 6);
    assert!(can_cast(&mut t, P0, eel, &[Entity::Player(P1)]));
    assert_eq!(t.zone(t.g.current(eel)), Zone::Graveyard(P0));
}

#[test]
fn undead_sprinter_enters_with_a_counter_when_cast_from_the_graveyard() {
    cr!("601.3", "614.1c");
    ruling!(
        "Undead Sprinter",
        "You still pay all costs and obey timing restrictions when casting Undead Sprinter from your graveyard"
    );
    assert_supported("Undead Sprinter");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Mountain", 2);
    let sprinter = t.graveyard(P0, "Undead Sprinter");
    // Not until a non-Zombie creature died this turn.
    assert!(!can_cast(&mut t, P0, sprinter, &[]));
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.destroy(bears, None);
    t.settle();
    assert!(can_cast(&mut t, P0, sprinter, &[]));
    let s = t.g.current(sprinter);
    assert!(t.on_battlefield(s));
    assert_eq!(t.counters(s, counters::PLUS1), 1);
    // Cast from the hand: no counter.
    let second = t.hand(P0, "Undead Sprinter");
    assert!(can_cast(&mut t, P0, second, &[]));
    assert_eq!(t.counters(t.g.current(second), counters::PLUS1), 0);
}

#[test]
fn power_pack_casts_the_card_next_upkeep_and_exiles_the_spell() {
    cr!("603.7", "614.1a", "118.9");
    assert_supported("Power Pack");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let pack = t.battlefield(P0, "Power Pack");
    let bolt = t.graveyard(P0, "Lightning Bolt");
    t.attack(&[(pack, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.zone(t.g.current(bolt)), Zone::Exile);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P1), 13, "{}", t.dump_log());
    // Exiled instead of put into the graveyard.
    assert_eq!(t.zone(t.g.current(bolt)), Zone::Exile);
    assert!(!t.in_graveyard(P0, "Lightning Bolt"));
}

#[test]
fn helmut_zemo_gets_a_counter_if_a_spell_was_cast() {
    cr!("608.2g", "614.1a");
    assert_supported("Helmut Zemo, Mastermind");
    for cast in [true, false] {
        let mut t = TestGame::new(2);
        t.advance_to(P0, Step::PrecombatMain);
        let zemo = t.battlefield(P0, "Helmut Zemo, Mastermind");
        t.lands(P0, "Mountain", 1);
        let bolt = t.graveyard(P0, "Lightning Bolt");
        t.answer_targets(P0, &[Entity::Object(bolt)]);
        t.answer_yes(P0, cast);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.attack(&[(zemo, Entity::Player(P1))], &[]);
        if cast {
            // 3 from the Bolt, 3 from Zemo with the counter.
            assert_eq!(t.life(P1), 14, "{}", t.dump_log());
            assert_eq!(t.counters(zemo, counters::PLUS1), 1);
            assert_eq!(t.zone(t.g.current(bolt)), Zone::Exile);
        } else {
            assert_eq!(t.life(P1), 18);
            assert_eq!(t.counters(zemo, counters::PLUS1), 0);
            assert_eq!(t.zone(t.g.current(bolt)), Zone::Graveyard(P0));
        }
    }
}

#[test]
fn ogre_battlecaster_casts_the_card_for_rr_more_and_gets_its_mana_value() {
    cr!("601.2b", "601.2f", "603.12", "614.1a");
    ruling!(
        "Ogre Battlecaster",
        "You must cast the spell as Ogre Battlecaster's triggered ability resolves."
    );
    assert_supported("Ogre Battlecaster");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let ogre = t.battlefield(P0, "Ogre Battlecaster");
    let lands = t.lands(P0, "Mountain", 4);
    let divination = t.graveyard(P0, "Lava Spike");
    t.answer_targets(P0, &[Entity::Object(divination)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.attack(&[(ogre, Entity::Player(P1))], &[]);
    // {R} for Lava Spike and {R}{R} more: three Mountains tapped.
    let tapped = lands.iter().filter(|l| t.obj_now(**l).tapped).count();
    assert_eq!(tapped, 3, "{}", t.dump_log());
    // 3 from Lava Spike, 3 + 1 from the Ogre (Lava Spike's mana value).
    assert_eq!(t.life(P1), 13);
    assert_eq!(t.zone(t.g.current(divination)), Zone::Exile);
}

#[test]
fn ogre_battlecaster_cant_cast_it_without_rr_more() {
    cr!("601.2f");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let ogre = t.battlefield(P0, "Ogre Battlecaster");
    t.lands(P0, "Mountain", 2);
    let spike = t.graveyard(P0, "Lava Spike");
    t.answer_targets(P0, &[Entity::Object(spike)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.attack(&[(ogre, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.zone(t.g.current(spike)), Zone::Graveyard(P0));
}

#[test]
fn black_widow_casts_the_exiled_card_if_no_counter_was_put() {
    cr!("611.2a", "118.14");
    assert_supported("Black Widow, Super Spy");
    for counter in [true, false] {
        let mut t = TestGame::new(2);
        t.advance_to(P0, Step::PrecombatMain);
        let widow = t.battlefield(P0, "Black Widow, Super Spy");
        t.lands(P0, "Swamp", 1);
        let bolt = t.library_top(P1, "Lightning Bolt");
        t.library_top(P1, "Forest");
        t.answer_yes(P0, counter);
        t.attack(&[(widow, Entity::Player(P1))], &[]);
        let bolt_x = t.g.current(bolt);
        assert_eq!(t.zone(bolt_x), Zone::Exile);
        t.advance_to(P0, Step::PostcombatMain);
        // A Swamp's mana pays for the Bolt (mana of any type).
        let cast = can_cast(&mut t, P0, bolt_x, &[Entity::Player(P1)]);
        assert_eq!(cast, !counter, "{}", t.dump_log());
        assert_eq!(t.counters(widow, counters::PLUS1), counter as u32);
    }
}

#[test]
fn codie_casts_the_found_card_free_and_puts_the_others_on_the_bottom() {
    cr!("603.7", "118.9");
    ruling!(
        "Codie, Vociferous Codex",
        "The last card exiled is not cast immediately. You may cast it until the end of the current turn."
    );
    assert_supported("Codie, Vociferous Codex");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let codie = t.battlefield(P0, "Codie, Vociferous Codex");
    t.lands(P0, "Island", 4);
    let bolt = t.library_top(P0, "Lightning Bolt");
    let divination = t.library_top(P0, "Divination");
    let forest = t.library_top(P0, "Forest");
    let spell = t.hand(P0, "Divination");
    t.activate(P0, codie, 0, &[]).unwrap();
    // Divination (mana value 3): exile until an instant or sorcery with lesser mana value:
    // the Forest and Divination are passed, the Bolt is found.
    t.cast(P0, spell).go();
    t.resolve_all();
    let bolt_x = t.g.current(bolt);
    assert_eq!(t.zone(bolt_x), Zone::Exile, "{}", t.dump_log());
    assert_eq!(t.zone(t.g.current(forest)), Zone::Library(P0));
    assert_eq!(t.zone(t.g.current(divination)), Zone::Library(P0));
    t.g.turn.priority = Some(P0);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, bolt_x).method(CastMethod::Free).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn dragons_approach_exiles_itself_and_four_more_to_find_a_dragon() {
    cr!("608.2c", "701.23a");
    ruling!(
        "Dragon's Approach",
        "In most cases, you'll need to exile five cards named Dragon's Approach"
    );
    assert_supported("Dragon's Approach");
    assert_supported("Sphinx's Approach");
    for in_graveyard in [3, 4] {
        let mut t = TestGame::new(2);
        t.advance_to(P0, Step::PrecombatMain);
        t.lands(P0, "Mountain", 3);
        for _ in 0..in_graveyard {
            t.graveyard(P0, "Dragon's Approach");
        }
        t.library_top(P0, "Shivan Dragon");
        let spell = t.hand(P0, "Dragon's Approach");
        t.answer_yes(P0, true);
        t.cast(P0, spell).go();
        t.resolve_all();
        assert_eq!(t.life(P1), 17);
        let found = t.named_on_battlefield("Shivan Dragon").len();
        let left = t
            .g
            .player(P0)
            .graveyard
            .iter()
            .filter(|c| t.g.obj(**c).chars.name == "Dragon's Approach")
            .count();
        if in_graveyard == 4 {
            assert_eq!(found, 1, "{}", t.dump_log());
            assert_eq!(left, 0);
            assert_eq!(t.zone(t.g.current(spell)), Zone::Exile);
        } else {
            assert_eq!(found, 0);
            assert_eq!(left, 4);
        }
    }
}

#[test]
fn rashmi_and_ragavan_cast_it_later_this_turn_if_not_cast_this_way() {
    cr!("608.2g", "611.2a");
    ruling!(
        "Rashmi and Ragavan",
        "If you don’t cast it without paying its mana cost but rather cast it later in the turn, you must pay its costs"
    );
    assert_supported("Rashmi and Ragavan");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    t.battlefield(P0, "Rashmi and Ragavan");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Forest", 2);
    // Mana value 2, not less than the one artifact (the Treasure): not cast for free.
    let bears = t.library_top(P1, "Grizzly Bears");
    let bolt = t.hand(P0, "Lightning Bolt");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, bolt).go();
    t.resolve_all();
    let bears_x = t.g.current(bears);
    assert_eq!(t.zone(bears_x), Zone::Exile, "{}", t.dump_log());
    assert!(can_cast(&mut t, P0, bears_x, &[]));
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn rashmi_and_ragavan_permission_only_if_not_cast_and_only_this_turn() {
    cr!("611.2a");
    ruling!("Rashmi and Ragavan", "If you don’t cast the card at all, it remains exiled.");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    t.battlefield(P0, "Rashmi and Ragavan");
    t.lands(P0, "Mountain", 1);
    let bears = t.library_top(P1, "Grizzly Bears");
    let bolt = t.hand(P0, "Lightning Bolt");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, bolt).go();
    t.resolve_all();
    let bears_x = t.g.current(bears);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    t.lands(P0, "Forest", 2);
    assert!(!can_cast(&mut t, P0, bears_x, &[]));
    assert_eq!(t.zone(bears_x), Zone::Exile);
}

#[test]
fn taster_of_wares_casts_an_exiled_instant_while_you_control_it() {
    cr!("611.2a", "118.14", "601.3e");
    ruling!(
        "Taster of Wares",
        "You pay all costs and follow all timing rules for cards cast this way."
    );
    assert_supported("Taster of Wares");
    for (card, castable) in [("Lightning Bolt", true), ("Grizzly Bears", false)] {
        let mut t = TestGame::new(2);
        t.advance_to(P0, Step::PrecombatMain);
        t.lands(P0, "Swamp", 2);
        let c = t.hand(P1, card);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.answer_choose(P1, &[Entity::Object(c)]);
        t.answer_choose(P0, &[Entity::Object(c)]);
        let taster = t.enter(P0, "Taster of Wares");
        t.resolve_all();
        let x = t.g.current(c);
        assert_eq!(t.zone(x), Zone::Exile, "{}", t.dump_log());
        // Swamps pay for the Bolt (mana of any type).
        let targets: &[Entity] = if castable { &[Entity::Player(P1)] } else { &[] };
        assert_eq!(can_cast(&mut t, P0, x, targets), castable);
        if castable {
            assert_eq!(t.life(P1), 17);
        }
        let _ = taster;
    }
}

#[test]
fn taster_of_wares_permission_ends_when_you_lose_control() {
    cr!("611.2a");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose(P1, &[Entity::Object(bolt)]);
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    let taster = t.enter(P0, "Taster of Wares");
    t.resolve_all();
    let x = t.g.current(bolt);
    assert_eq!(t.zone(x), Zone::Exile);
    t.g.destroy(t.g.current(taster), None);
    t.settle();
    assert!(!can_cast(&mut t, P0, x, &[Entity::Player(P1)]));
}
