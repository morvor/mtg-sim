//! Rulings batch S31 — lands that check what's on the battlefield as they enter (CR 614.1d,
//! 614.12): lands entering at the same time aren't counted, a Snarl can reveal a land
//! entering from the hand along with it, "put onto the battlefield tapped" wins, opponents
//! are counted as they are now, a land that enters tapped doesn't get its "enters
//! untapped" trigger by untapping later, and Karoo's return is a triggered ability.

use crate::r_s01_common::supported;
use crate::r_s14_common::triggers_from;
use crate::r_s20_common::tap_for_mana;
use crate::r_s25_common::lands_for_cost;
use crate::r_s31_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn a_legendary_creature_entering_with_minas_tirith_doesnt_let_it_enter_untapped() {
    cr!("614.12", "614.1d", "603.6a");
    ruling!(
        "Minas Tirith",
        "The legendary creature must already be on the battlefield as the land enters the battlefield. If it enters the battlefield at the same time, the land will enter tapped."
    );
    supported("Minas Tirith");
    supported("Genesis Wave");
    // "Minas Tirith enters tapped unless you control a legendary creature."
    let mut t = TestGame::new(2);
    let cards = wave(&mut t, &["Minas Tirith", "Isamaru, Hound of Konda"]);
    assert!(t.on_battlefield(cards[1]));
    assert!(entered_tapped(&t, cards[0]));
    // With the legendary creature already on the battlefield, it enters untapped.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Isamaru, Hound of Konda");
    let tirith = t.enter(P0, "Minas Tirith");
    assert!(!entered_tapped(&t, tirith));
}

#[test]
fn a_check_land_doesnt_see_lands_entering_at_the_same_time() {
    cr!("614.12", "614.1d");
    ruling!(
        "Dragonskull Summit",
        "As this is entering, it checks for lands that are already on the battlefield. It won't see lands that are entering at the same time (due to Warp World, for example)."
    );
    supported("Dragonskull Summit");
    // "This land enters tapped unless you control a Swamp or a Mountain."
    let mut t = TestGame::new(2);
    let cards = wave(&mut t, &["Dragonskull Summit", "Swamp"]);
    assert!(t.on_battlefield(cards[1]));
    assert!(entered_tapped(&t, cards[0]));
    // Now that the Swamp is on the battlefield, another one enters untapped.
    let summit = t.enter(P0, "Dragonskull Summit");
    assert!(!entered_tapped(&t, summit));
}

#[test]
fn mystic_sanctuary_doesnt_see_islands_entering_at_the_same_time() {
    cr!("614.12", "614.1d", "603.6a");
    ruling!(
        "Mystic Sanctuary",
        "As these lands are entering the battlefield, they check for lands that are already on the battlefield. They won't see lands that are entering the battlefield at the same time (due to Scapeshift, for example)."
    );
    supported("Mystic Sanctuary");
    // "This land enters tapped unless you control three or more other Islands. When this
    // land enters untapped, you may put target instant or sorcery card from your
    // graveyard on top of your library."
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    t.graveyard(P0, "Lightning Bolt");
    let cards = wave(&mut t, &["Mystic Sanctuary", "Island"]);
    assert!(t.on_battlefield(cards[1]));
    assert!(entered_tapped(&t, cards[0]));
    // It didn't enter untapped, so its last ability didn't trigger.
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    // With the three Islands already there, another one enters untapped.
    let sanctuary = t.enter(P0, "Mystic Sanctuary");
    assert!(!entered_tapped(&t, sanctuary));
}

#[test]
fn a_slow_land_doesnt_count_other_lands_entering_at_the_same_time() {
    cr!("614.12", "614.1d");
    ruling!(
        "Rockfall Vale",
        "If this land enters the battlefield at the same time as any number of other lands, those other lands are not counted when determining if this land enters the battlefield tapped or untapped."
    );
    supported("Rockfall Vale");
    // "This land enters tapped unless you control two or more other lands."
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let cards = wave(&mut t, &["Rockfall Vale", "Forest", "Forest"]);
    assert!(t.on_battlefield(cards[1]) && t.on_battlefield(cards[2]));
    assert!(entered_tapped(&t, cards[0]));
    let vale = t.enter(P0, "Rockfall Vale");
    assert!(!entered_tapped(&t, vale));
}

#[test]
fn a_basic_land_entering_at_the_same_time_isnt_counted() {
    cr!("614.12", "614.1d");
    ruling!(
        "Abandoned Air Temple",
        "If one of these lands enters at the same time as any number of basic lands, those other lands are not counted when determining if this land enters tapped or untapped."
    );
    supported("Abandoned Air Temple");
    // "This land enters tapped unless you control a basic land."
    let mut t = TestGame::new(2);
    let cards = wave(&mut t, &["Abandoned Air Temple", "Plains"]);
    assert!(t.on_battlefield(cards[1]));
    assert!(entered_tapped(&t, cards[0]));
    let temple = t.enter(P0, "Abandoned Air Temple");
    assert!(!entered_tapped(&t, temple));
}

#[test]
fn a_battle_land_doesnt_count_basic_lands_entering_at_the_same_time() {
    cr!("614.12", "614.1d");
    ruling!(
        "Sunken Hollow",
        "If one of these lands enters the battlefield at the same time as any number of basic lands, those other lands are not counted when determining if this land enters the battlefield tapped or untapped."
    );
    ruling!(
        "Scorched Geyser",
        "If this land enters the battlefield at the same time as any number of basic lands, those other lands are not counted when determining if this land enters the battlefield tapped or untapped."
    );
    supported("Sunken Hollow");
    supported("Scorched Geyser");
    // "This land enters tapped unless you control two or more basic lands."
    for name in ["Sunken Hollow", "Scorched Geyser"] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Island", 1);
        let cards = wave(&mut t, &[name, "Mountain"]);
        assert!(t.on_battlefield(cards[1]));
        assert!(entered_tapped(&t, cards[0]), "{name}");
        let other = t.enter(P0, name);
        assert!(!entered_tapped(&t, other), "{name}");
    }
}

#[test]
fn a_snarl_may_reveal_a_land_entering_from_the_hand_at_the_same_time() {
    cr!("614.12", "614.12a", "614.13a");
    ruling!(
        "Frostboil Snarl",
        "If a land card with an appropriate subtype is entering the battlefield from your hand at the same time as one of these lands, you may reveal the other land to have the \"Snarl\" enter untapped."
    );
    supported("Frostboil Snarl");
    // "As this land enters, you may reveal an Island or Mountain card from your hand. If
    // you don't, this land enters tapped."
    let mut t = TestGame::new(2);
    let snarl = t.hand(P0, "Frostboil Snarl");
    let island = t.hand(P0, "Island");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(island)]);
    let ids = put_together(&mut t, &[snarl, island]);
    assert!(ids.iter().all(|i| i.is_some()));
    assert_eq!(t.zone(island), Zone::Battlefield);
    assert!(!entered_tapped(&t, snarl));
    // Without such a card, it enters tapped.
    let lone = t.hand(P0, "Frostboil Snarl");
    put_together(&mut t, &[lone]);
    assert!(entered_tapped(&t, lone));
}

#[test]
fn a_snarl_put_onto_the_battlefield_tapped_enters_tapped_even_if_you_reveal() {
    cr!("614.12", "614.1c");
    ruling!(
        "Frostboil Snarl",
        "If an effect instructs you to put one of these lands onto the battlefield tapped, it will still enter the battlefield tapped even if you reveal a land card from your hand."
    );
    supported("Frostboil Snarl");
    supported("Hour of Promise");
    // Hour of Promise: "Search your library for up to two land cards, put them onto the
    // battlefield tapped, then shuffle. ..."
    let mut t = TestGame::new(2);
    let snarl = t.library_top(P0, "Frostboil Snarl");
    let island = t.hand(P0, "Island");
    lands_for_cost(&mut t, P0, "Hour of Promise");
    let hour = t.hand(P0, "Hour of Promise");
    t.answer_choose(P0, &[Entity::Object(snarl)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(island)]);
    t.cast(P0, hour).go();
    t.resolve_all();
    // The Island was revealed, and the Snarl still entered tapped.
    assert!(t.in_hand(P0, "Island"));
    assert!(revealed_cards(&t).contains(&island));
    assert!(entered_tapped(&t, snarl));
}

#[test]
fn karoos_return_is_a_triggered_ability_players_can_respond_to() {
    cr!("603.2", "603.3", "614.1d");
    ruling!(
        "Karoo",
        "This has a triggered ability when it enters, not a replacement effect, as previously worded."
    );
    supported("Karoo");
    // "This land enters tapped. When this land enters, sacrifice it unless you return an
    // untapped Plains you control to its owner's hand."
    let mut t = TestGame::new(2);
    let plains = t.battlefield(P0, "Plains");
    let karoo = t.hand(P0, "Karoo");
    t.play_land(P0, karoo).expect("play Karoo");
    t.settle();
    let karoo = t.g.current(karoo);
    assert!(entered_tapped(&t, karoo));
    // The return happens as a triggered ability resolves: Karoo is on the battlefield with
    // the trigger on the stack, and P0 may tap the Plains for mana in response.
    assert_eq!(triggers_from(&t, karoo), 1);
    assert!(t.on_battlefield(plains));
    assert!(tap_for_mana(&mut t, P0, plains, "Add"));
    t.resolve_all();
    // No untapped Plains to return: Karoo is sacrificed.
    assert!(t.in_graveyard(P0, "Karoo"));
    assert!(t.on_battlefield(plains));

    // Letting it resolve with an untapped Plains: P0 returns it and keeps Karoo.
    let mut t = TestGame::new(2);
    let plains = t.battlefield(P0, "Plains");
    let karoo = t.hand(P0, "Karoo");
    t.play_land(P0, karoo).expect("play Karoo");
    t.settle();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(plains)]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Plains"));
    assert!(t.on_battlefield(karoo));
}

#[test]
fn opponents_are_counted_as_the_land_enters_not_as_the_game_began() {
    cr!("614.1d", "800.4a", "102.2");
    ruling!(
        "Morphic Pool",
        "If you began the game with two or more opponents but now only have one opponent left, these lands enter the battlefield tapped."
    );
    ruling!(
        "Rejuvenating Springs",
        "Count the number of opponents you currently have, not how many you started with. If your four-player game is down to you and a single opponent, the land enters the battlefield tapped."
    );
    supported("Morphic Pool");
    supported("Rejuvenating Springs");
    // "This land enters tapped unless you have two or more opponents."
    let mut t = TestGame::new(3);
    let pool = t.enter(P0, "Morphic Pool");
    assert!(!entered_tapped(&t, pool));
    t.g.lose_game(P2);
    t.settle();
    assert!(!t.g.player(P2).in_game());
    let pool = t.enter(P0, "Morphic Pool");
    assert!(entered_tapped(&t, pool));

    let mut t = TestGame::new(4);
    let springs = t.enter(P0, "Rejuvenating Springs");
    assert!(!entered_tapped(&t, springs));
    t.g.lose_game(P1);
    t.settle();
    // Two opponents left: still untapped.
    let springs = t.enter(P0, "Rejuvenating Springs");
    assert!(!entered_tapped(&t, springs));
    t.g.lose_game(P3);
    t.settle();
    let springs = t.enter(P0, "Rejuvenating Springs");
    assert!(entered_tapped(&t, springs));
}

#[test]
fn untapping_a_land_that_entered_tapped_doesnt_make_it_enter_untapped() {
    cr!("603.6a", "614.1d");
    ruling!(
        "Castle Ardenvale",
        "Once the common lands (such as Mystic Sanctuary) enter the battlefield tapped, there's no way to untap them with a spell or ability to make their last ability trigger."
    );
    supported("Castle Ardenvale");
    supported("Mystic Sanctuary");
    let mut t = TestGame::new(2);
    // Castle Ardenvale: "This land enters tapped unless you control a Plains."
    let castle = t.enter(P0, "Castle Ardenvale");
    assert!(entered_tapped(&t, castle));
    // Mystic Sanctuary with only two other Islands enters tapped.
    t.lands(P0, "Island", 2);
    t.graveyard(P0, "Lightning Bolt");
    let sanctuary = t.enter(P0, "Mystic Sanctuary");
    t.settle();
    assert!(entered_tapped(&t, sanctuary));
    assert_eq!(t.stack_len(), 0);
    // Untapping it later doesn't make "When this land enters untapped" trigger.
    t.g.untap(sanctuary);
    t.g.flush_events();
    t.settle();
    assert!(!t.obj_now(sanctuary).tapped);
    assert_eq!(t.stack_len(), 0);
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
}
