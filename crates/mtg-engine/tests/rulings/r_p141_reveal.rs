//! Rulings batch P141 — "reveal cards until" and "look at the top card" engines: the
//! creature types compared are the creature's last known information (Heirloom Blade,
//! Descendants' Fury), a library without a match is revealed and put back (Atla Palani,
//! Vivien, Dreamshaper Shaman, Abundance, Bag of Tricks), {X} is 0 in a library or
//! graveyard (Part in Friendship, Puca's Covenant, Meren), and creatures put onto the
//! battlefield attacking (Fireflux Squad, Hans Eriksson).

use crate::r_p141_common::*;
use mtg_engine::ability::Modification;
use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The names of the cards in `p`'s library, top first.
fn library_names(t: &TestGame, p: PlayerId) -> Vec<String> {
    t.g.player(p)
        .library
        .iter()
        .rev()
        .map(|id| t.g.obj(*id).chars.name.to_string())
        .collect()
}

/// Gives the permanent the creature type `ty` too (an effect, until end of turn).
fn add_type(t: &mut TestGame, id: ObjectId, ty: &str) {
    crate::r_s26_common::modify_until_eot(t, id, vec![Modification::AddSubtypes(vec![ty.into()])]);
}

#[test]
fn heirloom_blade_compares_the_creature_as_it_last_existed() {
    cr!("608.2h", "603.10a", "205.3m");
    ruling!(
        "Heirloom Blade",
        "Compare the revealed cards to the creature as it last existed before it died, not to the creature card as it exists in its owner's graveyard"
    );
    ruling!("Heirloom Blade", "\"Artifact\" isn't a creature type.");
    supported("Heirloom Blade");
    // A Bears that an effect made a Goblin too finds the Goblin.
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Heirloom Blade", b);
    add_type(&mut t, b, "Goblin");
    stack_library(&mut t, P0, &["Llanowar Elves", "Raging Goblin"]);
    yes(&mut t, P0);
    destroy(&mut t, b);
    t.resolve_all();
    assert!(t.in_hand(P0, "Raging Goblin"));
    assert!(!t.in_hand(P0, "Llanowar Elves"));
    // An Ornithopter (an artifact Thopter) skips another artifact creature that isn't a
    // Thopter.
    let mut t = TestGame::new(2);
    let o = t.battlefield(P0, "Ornithopter");
    attach_new(&mut t, P0, "Heirloom Blade", o);
    stack_library(&mut t, P0, &["Steel Overseer", "Ornithopter"]);
    yes(&mut t, P0);
    destroy(&mut t, o);
    t.resolve_all();
    assert!(t.in_hand(P0, "Ornithopter"));
    assert!(!t.in_hand(P0, "Steel Overseer"));
    assert_eq!(library_names(&t, P0).last().unwrap(), "Steel Overseer");
}

/// P0 attacks with `attacker` alone and (with Descendants' Fury) sacrifices it as the
/// trigger resolves.
fn fury_attack(t: &mut TestGame, attacker: ObjectId) {
    yes(t, P0);
    t.answer_choose(P0, &[obj(attacker)]);
    t.attack(&[(attacker, Entity::Player(P1))], &[]);
    t.resolve_all();
}

#[test]
fn descendants_fury_compares_the_sacrificed_creature_as_it_last_existed() {
    cr!("608.2h", "701.21a");
    ruling!(
        "Descendants' Fury",
        "Compare the revealed cards to the creature as it last existed before it was sacrificed"
    );
    ruling!(
        "Descendants' Fury",
        "If the sacrificed creature has no creature type, no card can share a creature type with it."
    );
    supported("Descendants' Fury");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Descendants' Fury");
    let b = t.battlefield(P0, "Grizzly Bears");
    add_type(&mut t, b, "Goblin");
    stack_library(&mut t, P0, &["Llanowar Elves", "Raging Goblin"]);
    fury_attack(&mut t, b);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.named_on_battlefield("Raging Goblin").len(), 1);
    assert!(t.named_on_battlefield("Llanowar Elves").is_empty());
    // No creature types: the whole library is revealed and nothing is found.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Descendants' Fury");
    let b = t.battlefield(P0, "Grizzly Bears");
    crate::r_s26_common::modify_until_eot(&mut t, b, vec![Modification::RemoveAllCreatureTypes]);
    let lib = t.library_size(P0);
    stack_library(&mut t, P0, &["Grizzly Bears", "Raging Goblin"]);
    fury_attack(&mut t, b);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(creatures(&t, P0).len(), 0);
    assert_eq!(t.library_size(P0), lib + 2);
}

#[test]
fn reveal_until_a_creature_with_none_puts_the_library_back() {
    cr!("701.20a", "701.24a");
    ruling!(
        "Atla Palani, Nest Tender",
        "If there are no creature cards in your library as Atla Palani's last ability resolves, you reveal your library then put it back in a random order."
    );
    ruling!(
        "Vivien, Nature's Avenger",
        "If there are no creature cards in your library as Vivien’s second ability resolves, you reveal your library then put it back in a random order."
    );
    ruling!(
        "Dreamshaper Shaman",
        "If there are no nonland cards remaining in your library, you just randomize your library and continue playing."
    );
    supported("Atla Palani, Nest Tender");
    supported("Vivien, Nature's Avenger");
    supported("Dreamshaper Shaman");
    // Atla Palani: an Egg dies.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Atla Palani, Nest Tender");
    let egg = create_token(&mut t, P0, "Egg");
    let lib = t.library_size(P0);
    destroy(&mut t, egg);
    assert_eq!(stacked_triggers(&t), 1);
    t.resolve_all();
    assert_eq!(t.library_size(P0), lib);
    assert_eq!(creatures(&t, P0).len(), 1);
    // Positive control: a creature is found.
    let egg = create_token(&mut t, P0, "Egg");
    t.library_top(P0, "Grizzly Bears");
    destroy(&mut t, egg);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.library_size(P0), lib);
    // Vivien's −1.
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Vivien, Nature's Avenger");
    let (lib, hand) = (t.library_size(P0), t.hand_size(P0));
    t.activate(P0, v, 1, &[]).unwrap();
    t.resolve_all();
    assert_eq!((t.library_size(P0), t.hand_size(P0)), (lib, hand));
    // Dreamshaper Shaman, with only lands left: sacrifices, finds nothing.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dreamshaper Shaman");
    let target = t.battlefield(P0, "Ornithopter");
    t.lands(P0, "Mountain", 3);
    let lib = t.library_size(P0);
    stack_library(&mut t, P0, &["Forest", "Island"]);
    yes(&mut t, P0);
    t.answer_choose(P0, &[obj(target)]);
    end_step(&mut t, P0);
    assert!(t.in_graveyard(P0, "Ornithopter"));
    assert_eq!(t.library_size(P0), lib + 2);
    assert_eq!(t.g.permanents().filter(|o| o.controller == P0).count(), 4);
}

#[test]
fn dreamshaper_shaman_can_sacrifice_itself() {
    cr!("118.12", "603.5");
    ruling!(
        "Dreamshaper Shaman",
        "Dreamshaper Shaman can be sacrificed to pay for the cost of its own triggered ability."
    );
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Dreamshaper Shaman");
    t.lands(P0, "Mountain", 3);
    stack_library(&mut t, P0, &["Forest", "Hill Giant"]);
    yes(&mut t, P0);
    t.answer_choose(P0, &[obj(d)]);
    end_step(&mut t, P0);
    assert!(t.in_graveyard(P0, "Dreamshaper Shaman"));
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    assert_eq!(library_names(&t, P0).last().unwrap(), "Forest");
}

#[test]
fn sapling_of_colfenor_uses_characteristic_defining_pt_in_the_library() {
    cr!("604.3", "208.2a");
    ruling!(
        "Sapling of Colfenor",
        "If a creature card has “*” in its power or toughness, the ability that defines “*” works in all zones."
    );
    ruling!(
        "Sapling of Colfenor",
        "If the revealed card is not a creature card, it remains on top of your library."
    );
    supported("Sapling of Colfenor");
    // Tarmogoyf with an instant and a sorcery in graveyards is 2/3 in the library.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Sapling of Colfenor");
    t.graveyard(P1, "Lightning Bolt");
    t.graveyard(P1, "Divination");
    t.library_top(P0, "Tarmogoyf");
    t.attack(&[(s, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Tarmogoyf"));
    assert_eq!(t.life(P0), 21);
    // A land stays on top.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Sapling of Colfenor");
    t.library_top(P0, "Forest");
    t.attack(&[(s, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(library_names(&t, P0)[0], "Forest");
    assert_eq!(t.life(P0), 20);
}

#[test]
fn x_is_zero_for_mana_values_in_libraries_and_graveyards() {
    cr!("202.3e", "107.3b");
    ruling!(
        "Part in Friendship",
        "If the revealed creature card has {X} in its mana cost, X is 0 for the purposes of determining its mana value."
    );
    ruling!(
        "Puca's Covenant",
        "If a permanent card in a graveyard has {X} in its mana cost, X is 0 for the purpose of determining its mana value."
    );
    ruling!(
        "Meren of Clan Nel Toth",
        "If a creature card in your graveyard has {X} in its mana cost, X is 0."
    );
    supported("Part in Friendship");
    supported("Puca's Covenant");
    // Part in Friendship with no lands: Endless One (mana value 0) is put onto the
    // battlefield (and dies as a 0/0); a Grizzly Bears goes to the hand.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Part in Friendship");
    let b = t.battlefield(P0, "Hill Giant");
    t.library_top(P0, "Endless One");
    destroy(&mut t, b);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Endless One"));
    assert!(!t.in_hand(P0, "Endless One"));
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Part in Friendship");
    let b = t.battlefield(P0, "Hill Giant");
    t.library_top(P0, "Grizzly Bears");
    destroy(&mut t, b);
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    // Puca's Covenant: a creature with one counter dies; Endless One (mana value 0) can
    // be returned, Grizzly Bears (2) can't.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Puca's Covenant");
    let b = t.battlefield(P0, "Hill Giant");
    put_counters(&mut t, b, "+1/+1", 1);
    let e = t.graveyard(P0, "Endless One");
    let g = t.graveyard(P0, "Grizzly Bears");
    let from = n_asked(&t);
    yes(&mut t, P0);
    destroy(&mut t, b);
    assert!(offered_target(&t, P0, from, e));
    assert!(!offered_target(&t, P0, from, g));
    t.resolve_all();
    assert!(t.in_hand(P0, "Endless One"));
    // Meren with no experience counters returns Endless One to the battlefield.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Meren of Clan Nel Toth");
    t.graveyard(P0, "Endless One");
    end_step(&mut t, P0);
    assert!(!t.in_hand(P0, "Endless One"));
    assert!(t.in_graveyard(P0, "Endless One"), "returned, then died as a 0/0");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Meren of Clan Nel Toth");
    t.graveyard(P0, "Grizzly Bears");
    end_step(&mut t, P0);
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn bag_of_tricks_bottoming_the_library_isnt_shuffling() {
    cr!("701.24a", "701.20a");
    ruling!(
        "Bag of Tricks",
        "Even if you reveal the entire library this way, putting it on the bottom in a random order doesn't count as “shuffling” the library."
    );
    supported("Bag of Tricks");
    let mut t = TestGame::new(2);
    let bag = t.battlefield(P0, "Bag of Tricks");
    t.battlefield(P1, "Cosi's Trickster");
    t.lands(P0, "Forest", 5);
    let lib = t.library_size(P0);
    t.activate(P0, bag, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.library_size(P0), lib);
    let trickster = t.named_on_battlefield("Cosi's Trickster")[0];
    assert_eq!(t.counters(trickster, "+1/+1"), 0);
    // Positive control: a real shuffle triggers Cosi's Trickster.
    t.g.shuffle_library(P0);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.counters(trickster, "+1/+1"), 1);
}

#[test]
fn abundance_with_no_card_of_the_kind_gives_nothing() {
    cr!("614.11", "121.6");
    ruling!(
        "Abundance",
        "If no card of the chosen type is found before your library empties, you don't get a card, but you do get to order all the cards in your library any way you choose."
    );
    supported("Abundance");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Abundance");
    let lib = t.library_size(P0);
    let hand = t.hand_size(P0);
    // Choose "land": the library (fillers, nonland cards) has none.
    yes(&mut t, P0);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    let from = n_asked(&t);
    t.g.draw_cards(P0, 1);
    t.settle();
    let asked: Vec<_> = t.asked()[from..].to_vec();
    assert_eq!(t.hand_size(P0), hand, "{asked:?}");
    assert_eq!(t.library_size(P0), lib);
    assert!(!t.has_lost(P0));
    // Positive control: choosing "nonland" finds a card.
    yes(&mut t, P0);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.g.draw_cards(P0, 1);
    t.settle();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn scrying_sheets_reveals_only_snow_and_only_if_you_want() {
    cr!("701.20a", "205.4g");
    ruling!(
        "Scrying Sheets",
        "If the card isn’t snow, you can’t reveal it. If the card is snow, you don’t have to reveal it. An unrevealed card stays on top of your library."
    );
    supported("Scrying Sheets");
    for (top, take, in_hand) in [
        ("Snow-Covered Forest", true, true),
        ("Snow-Covered Forest", false, false),
        ("Forest", true, false),
    ] {
        let mut t = TestGame::new(2);
        let s = t.battlefield(P0, "Scrying Sheets");
        t.lands(P0, "Snow-Covered Forest", 2);
        t.library_top(P0, top);
        t.answer_yes(P0, take);
        t.activate(P0, s, 1, &[]).unwrap();
        t.resolve_all();
        assert_eq!(t.in_hand(P0, top), in_hand, "{top} {take}");
        if !in_hand {
            assert_eq!(library_names(&t, P0)[0], top);
        }
    }
}

#[test]
fn hans_eriksson_fight_needs_both_creatures() {
    cr!("603.12", "701.14b");
    ruling!(
        "Hans Eriksson",
        "If the creature card you put onto the battlefield isn't on the battlefield or isn't a creature as the reflexive triggered ability resolves, neither that creature nor Hans will deal or be dealt damage."
    );
    supported("Hans Eriksson");
    // Positive control: they fight.
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Hans Eriksson");
    t.library_top(P0, "Hill Giant");
    attack_with(&mut t, &[(h, Entity::Player(P1))]);
    t.resolve_all();
    let g = t.named_on_battlefield("Hill Giant")[0];
    assert_eq!(t.obj_now(h).damage, 3);
    assert_eq!(t.obj_now(g).damage, 1);
    // The Giant leaves before the reflexive trigger resolves.
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Hans Eriksson");
    t.library_top(P0, "Hill Giant");
    attack_with(&mut t, &[(h, Entity::Player(P1))]);
    t.resolve();
    let g = t.named_on_battlefield("Hill Giant")[0];
    assert_eq!(stacked_triggers(&t), 1, "the reflexive trigger");
    move_to(&mut t, g, Zone::Hand(P0));
    t.resolve_all();
    assert_eq!(t.obj_now(h).damage, 0);
}

#[test]
fn fireflux_squad_creature_enters_attacking_without_being_declared() {
    cr!("508.4", "508.3a");
    ruling!(
        "Fireflux Squad",
        "Although the new creature is an attacking creature, it was never declared as an attacking creature."
    );
    ruling!(
        "Fireflux Squad",
        "Any effects that say that the new creature can’t attack (such as that of Propaganda, or if you get a creature with defender) affect only the declaration of attackers."
    );
    supported("Fireflux Squad");
    for top in ["Hellrider", "Wall of Omens"] {
        let mut t = TestGame::new(2);
        let f = t.battlefield(P0, "Fireflux Squad");
        let b = t.battlefield(P0, "Grizzly Bears");
        t.battlefield(P1, "Propaganda");
        t.lands(P0, "Wastes", 4);
        t.library_top(P0, top);
        yes(&mut t, P0);
        t.answer_targets(P0, &[obj(b)]);
        attack_with(
            &mut t,
            &[(f, Entity::Player(P1)), (b, Entity::Player(P1))],
        );
        t.resolve_all();
        let c = t.named_on_battlefield(top);
        assert_eq!(c.len(), 1, "{top}");
        assert!(t.g.is_attacking(c[0]), "{top}");
        assert!(t.in_exile("Grizzly Bears"));
        if top == "Hellrider" {
            // Hellrider's "whenever a creature you control attacks" doesn't trigger for
            // itself.
            assert_eq!(t.life(P1), 20);
        }
        t.advance_to(P0, Step::EndOfCombat);
        let dmg = if top == "Hellrider" { 4 + 3 } else { 4 };
        assert_eq!(t.life(P1), 20 - dmg, "{top}");
    }
}

#[test]
fn amareth_compares_card_types_only() {
    cr!("205.2a", "205.4a");
    ruling!(
        "Amareth, the Lustrous",
        "Card types that can appear on permanents and cards in a library include artifact, creature, enchantment, land, planeswalker."
    );
    supported("Amareth, the Lustrous");
    // (entering permanent, top card, put into the hand)
    for (entering, top, found) in [
        ("Forest", "Island", true),
        ("Grizzly Bears", "Ornithopter", true),
        ("Ornithopter", "Bonesplitter", true),
        ("Grizzly Bears", "Forest", false),
        // Legendary isn't a card type: a legendary creature and a legendary artifact.
        ("Ravos, Soultender", "Aetherworks Marvel", false),
    ] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Amareth, the Lustrous");
        t.library_top(P0, top);
        t.answer_yes(P0, true);
        enter(&mut t, P0, entering);
        t.resolve_all();
        assert_eq!(t.in_hand(P0, top), found, "{entering} / {top}");
    }
}
