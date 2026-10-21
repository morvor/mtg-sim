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
    // Choosing to exile five when there aren't five is choosing an impossible option.
    cr!("608.2d", "701.23a");
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

#[test]
fn beseech_the_mirror_casts_the_card_if_bargained_else_puts_it_into_your_hand() {
    cr!("608.2g", "702.166a");
    ruling!(
        "Beseech the Mirror",
        "Bargain represents an optional additional cost. A spell cast with that additional cost paid is \"bargained.\""
    );
    assert_supported("Beseech the Mirror");
    for bargain in [true, false] {
        let mut t = TestGame::new(2);
        t.advance_to(P0, Step::PrecombatMain);
        t.lands(P0, "Swamp", 4);
        let thopter = t.battlefield(P0, "Ornithopter");
        let bears = t.library_top(P0, "Grizzly Bears");
        let spell = t.hand(P0, "Beseech the Mirror");
        t.answer(P0, DecisionKind::OptionalCost, mtg_engine::decision::Answer::Bool(bargain));
        if bargain {
            t.answer_choose(P0, &[Entity::Object(thopter)]);
        }
        t.answer_choose(P0, &[Entity::Object(bears)]);
        t.answer_yes(P0, true);
        t.cast(P0, spell).go();
        t.resolve_all();
        if bargain {
            assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1, "{}", t.dump_log());
        } else {
            assert!(t.in_hand(P0, "Grizzly Bears"), "{}", t.dump_log());
        }
    }
}

#[test]
fn conduit_of_worlds_casts_the_card_only_before_any_spell_and_then_no_more() {
    cr!("608.2g", "601.3");
    ruling!(
        "Conduit of Worlds",
        "If you cast it, you can't cast any other spells this turn, even if another effect would allow you to."
    );
    assert_supported("Conduit of Worlds");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let conduit = t.battlefield(P0, "Conduit of Worlds");
    t.lands(P0, "Forest", 4);
    let bears = t.graveyard(P0, "Grizzly Bears");
    let other = t.hand(P0, "Llanowar Elves");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_yes(P0, true);
    t.activate(P0, conduit, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert!(!can_cast(&mut t, P0, other, &[]));
    // After a spell was cast this turn: not cast.
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let conduit = t.battlefield(P0, "Conduit of Worlds");
    t.lands(P0, "Forest", 4);
    let bears = t.graveyard(P0, "Grizzly Bears");
    let elves = t.hand(P0, "Llanowar Elves");
    assert!(can_cast(&mut t, P0, elves, &[]));
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_yes(P0, true);
    t.activate(P0, conduit, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 0);
}

#[test]
fn mechtitan_core_returns_the_other_exiled_cards_when_the_token_leaves() {
    cr!("607.2a", "603.7");
    assert_supported("Mechtitan Core");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let core = t.battlefield(P0, "Mechtitan Core");
    let thopters: Vec<ObjectId> = (0..4).map(|_| t.battlefield(P0, "Ornithopter")).collect();
    t.lands(P0, "Wastes", 5);
    t.activate(P0, core, 0, &[]).expect("activate Mechtitan Core");
    t.resolve_all();
    let titan = t.named_on_battlefield("Mechtitan");
    assert_eq!(titan.len(), 1, "{}", t.dump_log());
    for o in &thopters {
        assert_eq!(t.zone(t.g.current(*o)), Zone::Exile);
    }
    t.g.destroy(titan[0], None);
    t.settle();
    t.resolve_all();
    for o in &thopters {
        let x = t.g.current(*o);
        assert!(t.on_battlefield(x), "{}", t.dump_log());
        assert!(t.obj_now(x).tapped);
    }
    // Not the Core.
    assert_eq!(t.zone(t.g.current(core)), Zone::Exile);
}

#[test]
fn mirror_of_life_trapping_returns_the_other_exiled_permanent_cards() {
    cr!("607.2a", "603.4");
    assert_supported("Mirror of Life Trapping");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    t.battlefield(P0, "Mirror of Life Trapping");
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Mountain", 4);
    let bears = t.hand(P0, "Grizzly Bears");
    let giant = t.hand(P0, "Hill Giant");
    t.cast(P0, bears).go();
    t.resolve_all();
    let bears_x = t.g.current(bears);
    assert_eq!(t.zone(bears_x), Zone::Exile, "{}", t.dump_log());
    // A creature put onto the battlefield without being cast: no trigger.
    t.battlefield(P1, "Gray Ogre");
    t.settle();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Gray Ogre").len(), 1);
    // The Giant is exiled; the Bears come back.
    t.cast(P0, giant).go();
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(giant)), Zone::Exile);
    assert!(t.on_battlefield(t.g.current(bears)));
}

#[test]
fn thought_prison_exiles_a_chosen_card_from_the_revealed_hand() {
    cr!("607.2a", "701.20a");
    assert_supported("Thought Prison");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.hand(P1, "Forest");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    t.enter(P0, "Thought Prison");
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(bolt)), Zone::Exile, "{}", t.dump_log());
    // A red spell shares a color with the exiled Bolt: 2 damage to its caster.
    t.lands(P1, "Mountain", 1);
    let shock = t.hand(P1, "Shock");
    t.advance_to(P1, Step::PrecombatMain);
    t.cast_with(P1, shock, &[Entity::Player(P0)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 18);
}

#[test]
fn thought_prison_does_nothing_if_you_dont() {
    cr!("608.2c");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_yes(P0, false);
    t.enter(P0, "Thought Prison");
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(bolt)), Zone::Hand(P1));
}

#[test]
fn vault_13_returns_two_exiled_cards_and_puts_the_rest_on_the_bottom() {
    cr!("607.2a", "714.2b");
    assert_supported("Vault 13: Dweller's Journey");
    let mut t = TestGame::new(3);
    t.advance_to(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let ogre = t.battlefield(P1, "Gray Ogre");
    let giant = t.battlefield(P2, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_targets(P0, &[Entity::Object(ogre)]);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    let saga = t.enter(P0, "Vault 13: Dweller's Journey");
    t.resolve_all();
    for o in [bears, ogre, giant] {
        assert_eq!(t.zone(t.g.current(o)), Zone::Exile, "{}", t.dump_log());
    }
    // Chapter III: the Bears and the Ogre return; the Giant goes to the bottom.
    t.g.add_counters(Entity::Object(saga), "lore", 1, None);
    t.resolve_all();
    t.answer_choose(P0, &[Entity::Object(bears), Entity::Object(ogre)]);
    t.g.add_counters(Entity::Object(saga), "lore", 1, None);
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(bears)), "{}", t.dump_log());
    assert!(t.on_battlefield(t.g.current(ogre)));
    assert_eq!(t.g.obj(t.g.current(ogre)).controller, P1);
    let giant_x = t.g.current(giant);
    assert_eq!(t.zone(giant_x), Zone::Library(P2));
    assert_ne!(t.g.library_top(P2), Some(giant_x));
}

#[test]
fn dubious_challenge_opponent_takes_one_exiled_creature_and_you_get_the_rest() {
    cr!("608.2c", "608.2d");
    assert_supported("Dubious Challenge");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    t.lands(P0, "Forest", 4);
    let bears = t.library_top(P0, "Grizzly Bears");
    let giant = t.library_top(P0, "Hill Giant");
    let spell = t.hand(P0, "Dubious Challenge");
    t.answer_choose(P0, &[Entity::Object(bears), Entity::Object(giant)]);
    t.answer_choose(P1, &[Entity::Object(bears)]);
    t.cast(P0, spell).target(P1).go();
    t.resolve_all();
    let bears_x = t.g.current(bears);
    let giant_x = t.g.current(giant);
    assert!(t.on_battlefield(bears_x), "{}", t.dump_log());
    assert_eq!(t.g.obj(bears_x).controller, P1);
    assert!(t.on_battlefield(giant_x));
    assert_eq!(t.g.obj(giant_x).controller, P0);
}

#[test]
fn mavinda_costs_eight_more_unless_the_spell_targets_your_creature() {
    cr!("601.2f", "601.2c", "614.1a");
    assert_supported("Mavinda, Students' Advocate");
    // Targeting a creature you control: its normal cost.
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let mavinda = t.battlefield(P0, "Mavinda, Students' Advocate");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    let growth = t.graveyard(P0, "Giant Growth");
    t.activate(P0, mavinda, 0, &[Entity::Object(growth)]).unwrap();
    t.resolve_all();
    assert!(can_cast(&mut t, P0, growth, &[Entity::Object(bears)]));
    assert_eq!(t.pt(bears), (5, 5));
    assert_eq!(t.zone(t.g.current(growth)), Zone::Exile);
    // Targeting a player: {8} more.
    for lands in [8, 9] {
        let mut t = TestGame::new(2);
        t.advance_to(P0, Step::PrecombatMain);
        let mavinda = t.battlefield(P0, "Mavinda, Students' Advocate");
        t.lands(P0, "Mountain", lands);
        let bolt = t.graveyard(P0, "Lightning Bolt");
        t.activate(P0, mavinda, 0, &[Entity::Object(bolt)]).unwrap();
        t.resolve_all();
        assert_eq!(can_cast(&mut t, P0, bolt, &[Entity::Player(P1)]), lands == 9);
    }
}

#[test]
fn amped_raptor_casts_the_exiled_card_for_energy_equal_to_its_mana_value() {
    cr!("118.9", "608.2g", "107.14");
    ruling!(
        "Amped Raptor",
        "If you choose not to cast the exiled nonland card (either because you don't have enough {E} or you just don't want to), that card will remain in exile as well."
    );
    assert_supported("Amped Raptor");
    for (card, castable) in [("Grizzly Bears", true), ("Hill Giant", false)] {
        let mut t = TestGame::new(2);
        t.advance_to(P0, Step::PrecombatMain);
        t.lands(P0, "Mountain", 2);
        let found = t.library_top(P0, card);
        t.library_top(P0, "Forest");
        let raptor = t.hand(P0, "Amped Raptor");
        t.answer_yes(P0, true);
        t.cast(P0, raptor).go();
        t.resolve_all();
        let on = t.named_on_battlefield(card).len();
        if castable {
            // Two energy for the Bears' mana value of 2.
            assert_eq!(on, 1, "{}", t.dump_log());
            assert_eq!(t.g.player(P0).counter("energy"), 0);
        } else {
            // Mana value 3: not enough energy; it stays exiled.
            assert_eq!(on, 0);
            assert_eq!(t.zone(t.g.current(found)), Zone::Exile);
            assert_eq!(t.g.player(P0).counter("energy"), 2);
        }
    }
}

#[test]
fn hama_casts_the_exiled_card_during_your_turn_by_waterbending() {
    cr!("118.9", "701.67a", "611.2a");
    ruling!(
        "Hama, the Bloodbender",
        "You may only cast a spell this way by paying the waterbend cost."
    );
    assert_supported("Hama, the Bloodbender");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    t.library_top(P1, "Forest");
    let bears = t.library_top(P1, "Grizzly Bears");
    let bolt = t.library_top(P1, "Lightning Bolt");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    let hama = t.enter(P0, "Hama, the Bloodbender");
    t.resolve_all();
    let bolt_x = t.g.current(bolt);
    assert_eq!(t.zone(bolt_x), Zone::Exile, "{}", t.dump_log());
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    let _ = bears;
    // Not during an opponent's turn.
    t.lands(P0, "Island", 1);
    t.advance_to(P1, Step::Upkeep);
    let pay = CastMethod::Alternative(mtg_engine::casting::PERMISSION_COST);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.g.turn.priority = Some(P0);
    assert!(t.g.cast_spell(P0, bolt_x, pay.clone()).is_err());
    t.clear_answers();
    // During P0's turn, waterbending {1} (its mana value): an Island pays for it.
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.g.turn.priority = Some(P0);
    t.g.cast_spell(P0, bolt_x, pay).expect("cast by waterbending");
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    let _ = hama;
}

#[test]
fn tibalts_trickery_the_controller_casts_the_found_card_and_bottoms_the_rest() {
    cr!("608.2g", "701.17a", "201.2");
    assert_supported("Tibalt's Trickery");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 1);
    t.lands(P1, "Mountain", 2);
    // P0's library: three Forests (one to three are milled), a Lightning Bolt (the same
    // name as the countered spell: passed over), then a Shock (found).
    let shock = t.library_top(P0, "Shock");
    let bolt2 = t.library_top(P0, "Lightning Bolt");
    for _ in 0..3 {
        t.library_top(P0, "Forest");
    }
    let bolt = t.hand(P0, "Lightning Bolt");
    let trickery = t.hand(P1, "Tibalt's Trickery");
    t.cast(P0, bolt).target(P1).go();
    // The countered spell's controller (P0) casts the Shock for free, at P1.
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.g.turn.priority = Some(P1);
    let target = t.g.current(bolt);
    t.cast(P1, trickery).target(target).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 18, "{}", t.dump_log());
    assert!(t.in_graveyard(P0, "Shock"));
    let _ = shock;
    // The other exiled cards went to the bottom of P0's library.
    assert_eq!(t.zone(t.g.current(bolt2)), Zone::Library(P0));
    assert!(t.g.exile.iter().all(|c| t.g.obj(*c).owner != P0), "{}", t.dump_log());
}

#[test]
fn doomsday_excruciator_leaves_each_player_six_cards() {
    cr!("603.4", "406.3");
    assert_supported("Doomsday Excruciator");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    t.lands(P0, "Swamp", 6);
    let bottom = t.library_top(P1, "Grizzly Bears");
    mtg_engine::library::put_on_bottom(&mut t.g, P1, &[bottom]);
    let spell = t.hand(P0, "Doomsday Excruciator");
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(t.library_size(P0), 6, "{}", t.dump_log());
    assert_eq!(t.library_size(P1), 6);
    assert_eq!(t.zone(t.g.current(bottom)), Zone::Library(P1));
    assert!(t.g.exile.iter().all(|c| t.g.obj(*c).face_down));
    // Not cast: nothing.
    let mut t = TestGame::new(2);
    t.enter(P0, "Doomsday Excruciator");
    t.resolve_all();
    assert_eq!(t.library_size(P0), 30);
}

#[test]
fn light_paws_finds_a_cheaper_aura_with_a_new_name_and_attaches_it() {
    cr!("303.4f", "201.2", "701.23a");
    assert_supported("Light-Paws, Emperor's Voice");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let paws = t.battlefield(P0, "Light-Paws, Emperor's Voice");
    t.lands(P0, "Forest", 1);
    let rancor2 = t.library_top(P0, "Rancor");
    let pacifism = t.library_top(P0, "Pacifism");
    let armor = t.library_top(P0, "Ethereal Armor");
    let rancor = t.hand(P0, "Rancor");
    t.answer_yes(P0, true);
    // Only Ethereal Armor qualifies: Pacifism costs more, the other Rancor has the name of
    // an Aura you control.
    t.answer_choose(P0, &[Entity::Object(armor)]);
    t.cast(P0, rancor).target(paws).go();
    t.resolve_all();
    let armor_x = t.g.current(armor);
    assert!(t.on_battlefield(armor_x), "{}", t.dump_log());
    assert_eq!(t.obj_now(armor_x).attached_to, Some(Entity::Object(paws)));
    assert_eq!(t.zone(t.g.current(pacifism)), Zone::Library(P0));
    assert_eq!(t.zone(t.g.current(rancor2)), Zone::Library(P0));
    let offered: Vec<Entity> = t
        .asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseEntities { candidates, .. } => Some(candidates),
            _ => None,
        })
        .last()
        .unwrap_or_default();
    assert_eq!(offered.len(), 1, "{offered:?}");
}

#[test]
fn the_great_aurora_each_player_draws_as_many_as_they_shuffled_in() {
    cr!("608.2e", "701.24");
    ruling!(
        "The Great Aurora",
        "The number of cards you draw is equal to the number of permanents on the battlefield that you shuffle into your library plus the number of cards from your hand that you shuffle into your library."
    );
    assert_supported("The Great Aurora");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    t.lands(P0, "Forest", 9);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Ornithopter");
    t.hand(P0, "Lightning Bolt");
    t.hand(P0, "Shock");
    t.battlefield(P1, "Gray Ogre");
    t.hand(P1, "Shock");
    let spell = t.hand(P0, "The Great Aurora");
    t.answer_choose(P0, &[]);
    t.answer_choose(P1, &[]);
    t.cast(P0, spell).go();
    t.resolve_all();
    // P0: two cards in hand and eleven permanents; P1: one and one.
    assert_eq!(t.hand_size(P0), 13, "{}", t.dump_log());
    assert_eq!(t.hand_size(P1), 2);
    assert!(t.named_on_battlefield("Gray Ogre").is_empty());
    assert!(t.in_exile("The Great Aurora"));
}

#[test]
fn glimpse_of_tomorrow_reveals_as_many_as_shuffled_in_and_puts_auras_last() {
    cr!("701.20a", "303.4f");
    assert_supported("Glimpse of Tomorrow");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    // Only these cards will be in the library.
    t.g.player_mut(P0).library.clear();
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Ornithopter");
    let rancor = t.hand(P0, "Rancor");
    t.lands(P0, "Forest", 1);
    t.cast(P0, rancor).target(bears).go();
    t.resolve_all();
    // (Cast for free with Omniscience, also one of P0's permanents.)
    t.battlefield(P0, "Omniscience");
    let spell = t.hand(P0, "Glimpse of Tomorrow");
    t.g.turn.priority = Some(P0);
    t.cast(P0, spell).method(CastMethod::Free).go();
    t.resolve_all();
    // Five permanents (the Forest and Omniscience too) were shuffled in, five cards
    // revealed: all of them come back, the Aura after the others (attached to one of
    // them).
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1, "{}", t.dump_log());
    assert_eq!(t.named_on_battlefield("Ornithopter").len(), 1);
    assert_eq!(t.named_on_battlefield("Forest").len(), 1);
    assert_eq!(t.named_on_battlefield("Omniscience").len(), 1);
    let r = t.named_on_battlefield("Rancor");
    assert_eq!(r.len(), 1);
    assert!(t.obj_now(r[0]).attached_to.is_some());
    assert_eq!(t.library_size(P0), 0);
}

#[test]
fn hew_the_entwood_puts_chosen_nonlands_then_lands_tapped_onto_the_battlefield() {
    cr!("701.20a", "701.21a");
    assert_supported("Hew the Entwood");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 5);
    let lands = t.lands(P0, "Forest", 3);
    let bears = t.library_top(P0, "Grizzly Bears");
    let forest = t.library_top(P0, "Forest");
    let thopter = t.library_top(P0, "Ornithopter");
    let spell = t.hand(P0, "Hew the Entwood");
    // Sacrifice three lands: reveal three cards; choose the Ornithopter and the Forest.
    let sac: Vec<Entity> = lands.iter().map(|l| Entity::Object(*l)).collect();
    t.answer_choose(P0, &sac);
    t.answer_choose(P0, &[Entity::Object(thopter), Entity::Object(forest)]);
    t.cast(P0, spell).go();
    t.resolve_all();
    let thopter_x = t.g.current(thopter);
    let forest_x = t.g.current(forest);
    assert!(t.on_battlefield(thopter_x), "{}", t.dump_log());
    assert!(!t.obj_now(thopter_x).tapped);
    assert!(t.on_battlefield(forest_x));
    assert!(t.obj_now(forest_x).tapped);
    let bears_x = t.g.current(bears);
    assert_eq!(t.zone(bears_x), Zone::Library(P0));
    assert_ne!(t.g.library_top(P0), Some(bears_x));
}
