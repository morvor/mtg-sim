//! Rulings batch P154 — mana values of cards and permanents off the stack: {X} is 0
//! (CR 202.3e), a card with no mana cost has mana value 0 (CR 202.3a), and a split card
//! off the stack has the mana value of its combined mana cost (CR 202.3d, 709.4b).

use crate::r_p154_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Discarding to Mercurial Chemister, Volcanic Vision returning a card, Hoard-Smelter
/// Dragon destroying an artifact, Razor Hippogriff returning an artifact card: each uses
/// the card's or permanent's mana value with X as 0.
#[test]
fn x_in_a_mana_cost_is_0_for_cards_and_permanents_off_the_stack() {
    cr!("202.3e", "107.3g");
    ruling!(
        "Mercurial Chemister",
        "If the discarded card has {X} in its mana cost, X is 0."
    );
    ruling!(
        "Hoard-Smelter Dragon",
        "If the mana cost of a permanent includes {X}, that X is considered to be 0."
    );
    ruling!(
        "Razor Hippogriff",
        "If the mana cost of the targeted card includes {X}, X is considered to be 0."
    );
    ruling!("Volcanic Vision", "If the target has {X} in its mana cost, X is 0.");
    supported("Mercurial Chemister");
    supported("Hoard-Smelter Dragon");
    supported("Razor Hippogriff");
    supported("Volcanic Vision");
    // Mercurial Chemister discards Fireball ({X}{R}, mana value 1): 1 damage.
    let mut t = TestGame::new(2);
    let chem = t.battlefield(P0, "Mercurial Chemister");
    let fireball = t.hand(P0, "Fireball");
    let giant = t.battlefield(P1, "Hill Giant");
    mana(&mut t, P0, "R");
    t.answer_choose(P0, &[obj(fireball)]);
    act(&mut t, P0, chem, "Discard a card", &[obj(giant)]).expect("Chemister");
    t.resolve_all();
    assert_eq!(dmg(&t, giant), 1);
    // Hoard-Smelter Dragon destroys a Walking Ballista ({X}{X}) with two counters: +0/+0.
    let mut t = TestGame::new(2);
    let dragon = t.battlefield(P0, "Hoard-Smelter Dragon");
    let ballista = endless_artifact(&mut t, P1);
    mana(&mut t, P0, "3R");
    act(&mut t, P0, dragon, "Destroy target artifact", &[obj(ballista)]).expect("Dragon");
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Walking Ballista"));
    assert_eq!(t.pt(dragon).0, 5, "+0/+0");
    // The Dragon destroying a Mind Stone (mana value 2): +2/+0.
    let mut t = TestGame::new(2);
    let dragon = t.battlefield(P0, "Hoard-Smelter Dragon");
    let stone = t.battlefield(P1, "Mind Stone");
    mana(&mut t, P0, "3R");
    act(&mut t, P0, dragon, "Destroy target artifact", &[obj(stone)]).expect("Dragon");
    t.resolve_all();
    assert_eq!(t.pt(dragon).0, 7);
    // Razor Hippogriff returns a Walking Ballista card: no life gained; a Mind Stone: 2.
    for (card, gained) in [("Walking Ballista", 0), ("Mind Stone", 2)] {
        let mut t = TestGame::new(2);
        let c = t.graveyard(P0, card);
        t.answer_targets(P0, &[obj(c)]);
        t.enter(P0, "Razor Hippogriff");
        t.resolve_all();
        assert!(t.in_hand(P0, card), "{card}");
        assert_eq!(t.life(P0), 20 + gained, "{card}");
    }
    // Volcanic Vision returns a Fireball card: 1 damage to each creature opponents control.
    let mut t = TestGame::new(2);
    let fireball = t.graveyard(P0, "Fireball");
    let giant = t.battlefield(P1, "Hill Giant");
    cast_resolve(&mut t, P0, "Volcanic Vision", "5RR", &[obj(fireball)]);
    assert!(t.in_hand(P0, "Fireball"));
    assert_eq!(dmg(&t, giant), 1);
    assert!(t.in_exile("Volcanic Vision"));
}

/// A Walking Ballista ({X}{X}) with two +1/+1 counters on `p`'s battlefield: an artifact
/// creature whose X was 2 as it was cast and is 0 now.
fn endless_artifact(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let b = t.battlefield(p, "Walking Ballista");
    put(t, b, counters::PLUS1, 2);
    b
}

#[test]
fn soul_tithe_on_a_permanent_with_x_in_its_cost_asks_for_0_mana() {
    cr!("202.3e", "107.3h");
    ruling!("Soul Tithe", "If the permanent has {X} in its mana cost, X is 0.");
    supported("Soul Tithe");
    // Endless One with four counters, no lands: paying {0} keeps it.
    let mut t = TestGame::new(2);
    let e = endless_one(&mut t, P1, 4);
    crate::r_s06_common::attach_new(&mut t, P0, "Soul Tithe", e);
    t.answer_yes(P1, true);
    crate::r_s04_common::next_upkeep(&mut t, P1);
    t.resolve_all();
    assert!(t.on_battlefield(e), "paid {{0}}");
    // Not paying: sacrificed.
    let mut t = TestGame::new(2);
    let e = endless_one(&mut t, P1, 4);
    crate::r_s06_common::attach_new(&mut t, P0, "Soul Tithe", e);
    t.answer_yes(P1, false);
    crate::r_s04_common::next_upkeep(&mut t, P1);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Endless One"));
}

#[test]
fn yunas_whistle_puts_0_counters_for_a_creature_card_whose_x_is_0() {
    cr!("202.3e", "603.12");
    ruling!(
        "Yuna's Whistle",
        "If the revealed card has {X} in its mana cost, X is 0 for the purpose of determining its mana value."
    );
    supported("Yuna's Whistle");
    for (top, n) in [("Endless One", 0), ("Hill Giant", 4)] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        stack_library(&mut t, P0, &["Forest", top, "Island"]);
        t.answer_targets(P0, &[obj(bears)]);
        cast_resolve(&mut t, P0, "Yuna's Whistle", "1GG", &[]);
        assert!(t.in_hand(P0, top), "{top}");
        assert_eq!(t.counters(bears, counters::PLUS1), n, "{top}");
    }
}

/// Caustic Bronco (unsaddled) attacks P1 with `top` on top of P0's library; the attack
/// trigger resolves. Returns the life P0 lost.
fn bronco_reveals(top: &str) -> i32 {
    let mut t = TestGame::new(2);
    let bronco = t.battlefield(P0, "Caustic Bronco");
    stack_library(&mut t, P0, &[top, "Island"]);
    attack_with(&mut t, &[(bronco, pl(P1))]);
    t.resolve_all();
    assert!(t.in_hand(P0, top), "{top}");
    20 - t.life(P0)
}

#[test]
fn caustic_bronco_mana_values_of_the_revealed_card() {
    cr!("202.3a", "202.3d", "202.3e", "709.4b");
    ruling!(
        "Caustic Bronco",
        "If the revealed card doesn’t have a mana cost (because it’s a land card, for example), its mana value is 0."
    );
    ruling!(
        "Caustic Bronco",
        "If the revealed card is a split card, that card’s mana value is based on the combined mana cost of its two halves."
    );
    ruling!(
        "Caustic Bronco",
        "If the revealed card’s mana cost includes {X}, X is 0 for the purpose of determining its mana value."
    );
    supported("Caustic Bronco");
    supported("Fire // Ice");
    assert_eq!(bronco_reveals("Forest"), 0, "a land");
    assert_eq!(bronco_reveals("Fire // Ice"), 4, "{{1}}{{R}} + {{1}}{{U}}");
    assert_eq!(bronco_reveals("Fireball"), 1, "{{X}}{{R}}");
    assert_eq!(bronco_reveals("Hill Giant"), 4);
}

#[test]
fn rakdos_the_muscle_exiles_cards_equal_to_the_sacrificed_creatures_mana_value_with_x_0() {
    cr!("202.3e", "603.10a");
    ruling!(
        "Rakdos, the Muscle",
        "If the sacrificed creature had {X} in its mana cost, X is 0 for the purpose of determining its mana value."
    );
    supported("Rakdos, the Muscle");
    for (name, exiled) in [("Endless One", 0), ("Grizzly Bears", 2)] {
        let mut t = TestGame::new(2);
        let rakdos = t.battlefield(P0, "Rakdos, the Muscle");
        let fodder = if name == "Endless One" {
            endless_one(&mut t, P0, 3)
        } else {
            t.battlefield(P0, name)
        };
        let lib = t.library_size(P1);
        t.answer_choose(P0, &[obj(fodder)]);
        t.answer_targets(P0, &[pl(P1)]);
        act(&mut t, P0, rakdos, "Sacrifice another creature", &[]).expect("Rakdos");
        t.resolve_all();
        assert!(t.in_graveyard(P0, name), "{name}");
        assert_eq!(lib - t.library_size(P1), exiled, "{name}");
    }
}

#[test]
fn rushed_rebirth_compares_mana_values_with_x_as_0() {
    cr!("202.3e", "603.7a", "704.5f");
    ruling!(
        "Rushed Rebirth",
        "If you choose a creature or search for a creature card with {X} in its mana cost, X is considered to be 0."
    );
    supported("Rushed Rebirth");
    // The chosen creature is an Endless One with counters (mana value 0): nothing has a
    // lesser mana value, so nothing is offered.
    let mut t = TestGame::new(2);
    let e = endless_one(&mut t, P0, 3);
    t.library_top(P0, "Grizzly Bears");
    t.library_top(P0, "Endless One");
    cast_resolve(&mut t, P0, "Rushed Rebirth", "BG", &[obj(e)]);
    let from = t.asked().len();
    kill(&mut t, e);
    t.resolve_all();
    assert_eq!(last_offered_names(&t, P0, from), Vec::<String>::new());
    assert_eq!(t.library_size(P0), 32);
    // Grizzly Bears (2) dies: an Endless One card in the library (0) is offered, a Hill
    // Giant isn't.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.library_top(P0, "Hill Giant");
    let card = t.library_top(P0, "Endless One");
    cast_resolve(&mut t, P0, "Rushed Rebirth", "BG", &[obj(bears)]);
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(card)]);
    kill(&mut t, bears);
    t.resolve_all();
    assert_eq!(last_offered_names(&t, P0, from), vec!["Endless One".to_string()]);
    // Put onto the battlefield rather than cast, the Endless One enters with no counters
    // (X is 0) and dies as a 0/0 (CR 704.5f).
    assert_eq!(t.library_size(P0), 31, "searched out");
    assert!(t.in_graveyard(P0, "Endless One"));
}

#[test]
fn sacrifice_of_an_animated_land_adds_no_mana() {
    cr!("202.3a", "202.3", "118.8");
    ruling!(
        "Sacrifice",
        "Sacrificing an animated land gives no mana since the converted mana cost was zero."
    );
    supported("Sacrifice");
    supported("Mutavault");
    for (land, added) in [(true, 0), (false, 2)] {
        let mut t = TestGame::new(2);
        let fodder = if land {
            let mv = t.battlefield(P0, "Mutavault");
            mana(&mut t, P0, "1");
            t.activate(P0, mv, 1, &[]).expect("animate");
            t.resolve_all();
            assert!(t.obj_now(mv).is(CardType::Creature));
            mv
        } else {
            t.battlefield(P0, "Grizzly Bears")
        };
        let sac = t.hand(P0, "Sacrifice");
        mana(&mut t, P0, "B");
        t.answer_choose(P0, &[obj(fodder)]);
        t.cast(P0, sac).go();
        t.resolve_all();
        assert_eq!(t.zone(fodder), Zone::Graveyard(P0));
        assert_eq!(pool(&t, P0), added, "land: {land}");
    }
}

#[test]
fn a_revealed_land_has_mana_value_0() {
    cr!("202.3a");
    ruling!(
        "Dark Tutelage",
        "If the revealed card has no mana symbols in its upper right corner (because it's a land card, for example), its mana value is 0."
    );
    ruling!(
        "Cerebral Eruption",
        "If the revealed card has no mana symbols in its upper right corner (because it’s a land card, for example), its mana value is 0."
    );
    ruling!(
        "Interpret the Signs",
        "If you reveal a card with mana value 0, such as a land card, you won't draw any cards. Otherwise, the card you revealed will be the first card you draw."
    );
    ruling!(
        "Rashmi, Eternities Crafter",
        "If the revealed card is a land card, you put it into your hand."
    );
    supported("Dark Tutelage");
    supported("Cerebral Eruption");
    supported("Interpret the Signs");
    supported("Rashmi, Eternities Crafter");
    // Dark Tutelage: a Forest on top, no life lost; a Hill Giant: 4.
    for (top, lost) in [("Forest", 0), ("Hill Giant", 4)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Dark Tutelage");
        stack_library(&mut t, P0, &[top, "Island"]);
        crate::r_s04_common::next_upkeep(&mut t, P0);
        t.resolve_all();
        assert!(t.in_hand(P0, top), "{top}");
        assert_eq!(t.life(P0), 20 - lost, "{top}");
    }
    // Cerebral Eruption: P1's Forest on top deals nothing and returns the spell to hand;
    // a Hill Giant deals 4 to P1 and their creature.
    for (top, dealt) in [("Forest", 0), ("Hill Giant", 4)] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        stack_library(&mut t, P1, &[top, "Island"]);
        cast_resolve(&mut t, P0, "Cerebral Eruption", "2RR", &[pl(P1)]);
        assert_eq!(t.life(P1), 20 - dealt, "{top}");
        if dealt == 0 {
            assert!(t.on_battlefield(bears));
            assert!(t.in_hand(P0, "Cerebral Eruption"), "returned to hand");
        } else {
            assert!(t.in_graveyard(P1, "Grizzly Bears"));
            assert!(t.in_graveyard(P0, "Cerebral Eruption"));
        }
    }
    // Interpret the Signs: a Forest on top after the scry draws nothing; a Hill Giant
    // draws four, the Giant first.
    for (top, drawn) in [("Forest", 0), ("Hill Giant", 4)] {
        let mut t = TestGame::new(2);
        stack_library(&mut t, P0, &[top, "Island", "Swamp", "Mountain", "Plains"]);
        let hand = t.hand_size(P0);
        cast_resolve(&mut t, P0, "Interpret the Signs", "5U", &[]);
        assert_eq!(t.hand_size(P0), hand + drawn, "{top}");
        if drawn > 0 {
            assert!(t.in_hand(P0, "Hill Giant"));
            assert!(t.in_hand(P0, "Island") && t.in_hand(P0, "Swamp") && t.in_hand(P0, "Mountain"));
            assert!(!t.in_hand(P0, "Plains"));
        } else {
            assert_eq!(t.g.player(P0).library.len(), 30 + 5);
        }
    }
    // Rashmi: a Forest on top isn't a spell, so it's put into your hand.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rashmi, Eternities Crafter");
    stack_library(&mut t, P0, &["Forest", "Island"]);
    cast_resolve(&mut t, P0, "Hill Giant", "3R", &[]);
    assert!(t.in_hand(P0, "Forest"));
    assert_eq!(count_named(&t, "Hill Giant"), 1);
}

#[test]
fn interpret_the_signs_scry_happens_before_the_reveal() {
    cr!("701.22a", "608.2c");
    supported("Interpret the Signs");
    // Scrying the Forest and Island to the bottom reveals the Hill Giant.
    let mut t = TestGame::new(2);
    let lib = stack_library(&mut t, P0, &["Forest", "Island", "Hill Giant", "Swamp", "Mountain", "Plains"]);
    t.answer(
        P0,
        DecisionKind::Scry,
        Answer::Split(vec![lib[2]], vec![lib[0], lib[1]]),
    );
    let hand = t.hand_size(P0);
    cast_resolve(&mut t, P0, "Interpret the Signs", "5U", &[]);
    assert_eq!(t.hand_size(P0), hand + 4);
    assert!(t.in_hand(P0, "Hill Giant") && t.in_hand(P0, "Plains"));
}
