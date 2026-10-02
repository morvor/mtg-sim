//! Conditions about referents (`src/oracle/patterns/conditions_referents.rs`): a subject
//! the effect parser resolves ("it", "that creature", "that player", "the sacrificed
//! creature", "the discarded card", enchanted creature, X) and a state predicate, in
//! leading "If ..., " sentences, trailing " if ..." conditions, "instead" replacements and
//! intervening-if clauses (CR 603.4, 608.2c).

use mtg_engine::ability::Effect;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

fn destroy(t: &mut TestGame, id: ObjectId) {
    t.g.destroy_all(vec![id], None, false);
    t.settle();
}

#[test]
fn destroy_target_creature_if_its_white() {
    cr!("608.2c");
    // Soul Rend: "Destroy target creature if it's white. ..."
    assert_supported(&["Soul Rend"]);
    for (victim, dies) in [("Savannah Lions", true), ("Grizzly Bears", false)] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Swamp", 4);
        let c = t.battlefield(P1, victim);
        let s = t.hand(P0, "Soul Rend");
        t.cast(P0, s).target(c).go();
        t.resolve_all();
        assert_eq!(!t.on_battlefield(c), dies, "{victim}");
    }
}

#[test]
fn if_that_creature_was_a_human_uses_last_known_information() {
    cr!("608.2c", "608.2h");
    ruling!(
        "Death's Caress",
        "Check the creature as it last existed on the battlefield to determine if it was a Human"
    );
    // Death's Caress: "Destroy target creature. If that creature was a Human, you gain
    // life equal to its toughness."
    assert_supported(&["Death's Caress"]);
    for (victim, gain) in [("Elite Vanguard", 1), ("Grizzly Bears", 0)] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Swamp", 5);
        let c = t.battlefield(P1, victim);
        let s = t.hand(P0, "Death's Caress");
        t.cast(P0, s).target(c).go();
        t.resolve_all();
        assert!(!t.on_battlefield(c));
        assert_eq!(t.life(P0), 20 + gain, "{victim}");
    }
}

#[test]
fn intervening_if_that_player_has_two_or_fewer_cards_in_hand() {
    cr!("603.4");
    ruling!(
        "Hellfire Mongrel",
        "two or fewer cards in hand as their upkeep starts"
    );
    // Hellfire Mongrel: "At the beginning of each opponent's upkeep, if that player has
    // two or fewer cards in hand, ~ deals 2 damage to that player."
    assert_supported(&["Hellfire Mongrel"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hellfire Mongrel");
    t.advance_to(P1, Step::Draw);
    assert_eq!(t.life(P1), 18);
    // Three cards in hand: no damage.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hellfire Mongrel");
    for _ in 0..3 {
        t.hand(P1, "Forest");
    }
    t.advance_to(P1, Step::Draw);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn intervening_if_it_had_a_counter_looks_back() {
    cr!("603.4", "603.10a");
    ruling!(
        "Basri's Lieutenant",
        "triggers just once if the dying creature has +1/+1 counters on it"
    );
    // Basri's Lieutenant: "Whenever ~ or another creature you control dies, if it had a
    // +1/+1 counter on it, create a 2/2 white Knight creature token with vigilance."
    assert_supported(&["Basri's Lieutenant"]);
    let mut t = TestGame::new(2);
    let lieutenant = t.battlefield(P0, "Basri's Lieutenant");
    let plain = t.battlefield(P0, "Grizzly Bears");
    let marked = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(Entity::Object(marked), "+1/+1", 1, None);
    destroy(&mut t, plain);
    t.resolve_all();
    let knights = |t: &TestGame| {
        t.g.battlefield
            .iter()
            .filter(|id| **id != lieutenant && t.g.obj(**id).chars.has_subtype("Knight"))
            .count()
    };
    assert_eq!(knights(&t), 0);
    destroy(&mut t, marked);
    t.resolve_all();
    assert_eq!(knights(&t), 1);
}

#[test]
fn if_enchanted_creature_is_untapped_tap_it() {
    cr!("603.4", "303.4");
    ruling!(
        "Narcolepsy",
        "the ability triggers only if the enchanted creature is untapped as any player"
    );
    // Narcolepsy: "At the beginning of each upkeep, if enchanted creature is untapped, tap
    // it."
    assert_supported(&["Narcolepsy"]);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let aura = t.battlefield(P0, "Narcolepsy");
    assert!(t.g.attach(aura, Entity::Object(bears)));
    t.advance_to(P1, Step::Draw);
    assert!(t.obj_now(bears).tapped, "the enchanted creature is tapped");
    assert!(!t.obj_now(aura).tapped, "not the Aura");
}

#[test]
fn if_x_is_5_or_more() {
    cr!("107.3", "608.2c");
    ruling!("Martial Coup", "Martial Coup checks the number you chose for X");
    // Martial Coup: "Create X 1/1 white Soldier creature tokens. If X is 5 or more,
    // destroy all other creatures."
    assert_supported(&["Martial Coup"]);
    for (x, survives) in [(4, true), (5, false)] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Plains", 7);
        let wurm = t.battlefield(P1, "Craw Wurm");
        let s = t.hand(P0, "Martial Coup");
        t.cast(P0, s).x(x).go();
        t.resolve_all();
        assert_eq!(t.on_battlefield(wurm), survives, "X = {x}");
        let soldiers = t
            .g
            .battlefield
            .iter()
            .filter(|id| t.g.obj(**id).chars.has_subtype("Soldier"))
            .count();
        assert_eq!(soldiers as i64, x);
    }
}

#[test]
fn counter_target_spell_if_its_controller_is_poisoned() {
    cr!("122.1f", "608.2c");
    // Corrupted Resolve: "Counter target spell if its controller is poisoned."
    assert_supported(&["Corrupted Resolve"]);
    for poisoned in [false, true] {
        let mut t = TestGame::new(2);
        if poisoned {
            t.g.add_counters(Entity::Player(P1), "poison", 1, None);
        }
        t.set_step(P1, Step::PrecombatMain);
        t.lands(P1, "Forest", 2);
        t.lands(P0, "Island", 2);
        let bears = t.hand(P1, "Grizzly Bears");
        let spell = t.cast(P1, bears).go();
        let cr = t.hand(P0, "Corrupted Resolve");
        t.cast(P0, cr).target(spell).go();
        t.resolve_all();
        assert_eq!(
            t.named_on_battlefield("Grizzly Bears").is_empty(),
            poisoned
        );
    }
}

#[test]
fn sacrifice_a_creature_if_you_cant_damage() {
    cr!("101.3", "608.2c");
    ruling!("Lord of the Pit", "you must sacrifice one");
    // Lord of the Pit: "At the beginning of your upkeep, sacrifice a creature other than ~.
    // If you can't, ~ deals 7 damage to you."
    assert_supported(&["Lord of the Pit"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lord of the Pit");
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Draw);
    assert_eq!(t.life(P0), 13);
    // With a creature to sacrifice: no damage.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lord of the Pit");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Draw);
    assert_eq!(t.life(P0), 20);
    assert!(!t.on_battlefield(bears));
}

#[test]
fn if_the_player_cant_sacrifice_sacrifice_this() {
    cr!("101.3", "608.2c");
    // Woebringer Demon: "At the beginning of each player's upkeep, that player sacrifices
    // a creature of their choice. If the player can't, sacrifice ~."
    assert_supported(&["Woebringer Demon"]);
    let mut t = TestGame::new(2);
    let demon = t.battlefield(P0, "Woebringer Demon");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P1, Step::Draw);
    assert!(!t.on_battlefield(bears));
    assert!(t.on_battlefield(demon));
    // Next upkeep of P1 (no creatures): the Demon's controller sacrifices it. (P0's own
    // upkeep in between makes P0 sacrifice the Demon itself, the only creature.)
    let mut t = TestGame::new(2);
    let demon = t.battlefield(P0, "Woebringer Demon");
    t.advance_to(P1, Step::Draw);
    assert!(!t.on_battlefield(demon));
}

#[test]
fn if_a_land_card_was_milled_this_way() {
    cr!("701.17a", "608.2c");
    // Lorehold Excavation: "At the beginning of your end step, mill a card. If a land card
    // was milled this way, you gain 1 life. Otherwise, ~ deals 1 damage to each opponent."
    assert_supported(&["Lorehold Excavation"]);
    for (top, land) in [("Forest", true), ("Grizzly Bears", false)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Lorehold Excavation");
        t.set_step(P0, Step::PostcombatMain);
        t.library_top(P0, top);
        t.advance_to(P1, Step::Upkeep);
        if land {
            assert_eq!((t.life(P0), t.life(P1)), (21, 20), "{top}");
        } else {
            assert_eq!((t.life(P0), t.life(P1)), (20, 19), "{top}");
        }
    }
}

#[test]
fn if_a_creature_is_dealt_damage_this_way_it_gets_bigger() {
    cr!("120.3", "608.2c");
    // Provoke the Trolls: "~ deals 3 damage to any target. If a creature is dealt damage
    // this way, it gets +5/+0 until end of turn."
    assert_supported(&["Provoke the Trolls"]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let s = t.hand(P0, "Provoke the Trolls");
    t.cast(P0, s).target(wurm).go();
    t.resolve_all();
    assert_eq!(t.pt(wurm), (11, 4));
    // A player: nothing else happens.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let s = t.hand(P0, "Provoke the Trolls");
    t.cast(P0, s).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.pt(wurm), (6, 4));
}

#[test]
fn if_a_creature_is_destroyed_this_way_gain_its_toughness() {
    cr!("608.2c", "608.2h");
    ruling!(
        "Noxious Gearhulk",
        "Use the toughness of the creature as it last existed on the battlefield"
    );
    // Noxious Gearhulk: "When ~ enters, you may destroy another target creature. If a
    // creature is destroyed this way, you gain life equal to its toughness."
    assert_supported(&["Noxious Gearhulk"]);
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    // A +1/+1 counter: as it last existed, it was 7/5.
    t.g.add_counters(Entity::Object(wurm), "+1/+1", 1, None);
    t.answer_targets(P0, &[Entity::Object(wurm)]);
    t.answer_yes(P0, true);
    t.enter(P0, "Noxious Gearhulk");
    t.resolve_all();
    assert!(!t.on_battlefield(wurm));
    assert_eq!(t.life(P0), 25);
    // Declined: nothing.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[Entity::Object(wurm)]);
    t.answer_yes(P0, false);
    t.enter(P0, "Noxious Gearhulk");
    t.resolve_all();
    assert!(t.on_battlefield(wurm));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn if_a_goblin_is_sacrificed_this_way() {
    cr!("701.21a", "608.2c");
    // Warren Weirding: "Target player sacrifices a creature of their choice. If a Goblin
    // is sacrificed this way, that player creates two 1/1 black Goblin Rogue creature
    // tokens, and those tokens gain haste until end of turn."
    assert_supported(&["Warren Weirding"]);
    for (victim, tokens) in [("Raging Goblin", 2), ("Grizzly Bears", 0)] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Swamp", 2);
        let c = t.battlefield(P1, victim);
        let s = t.hand(P0, "Warren Weirding");
        t.cast(P0, s).target(P1).go();
        t.resolve_all();
        assert!(!t.on_battlefield(c));
        let rogues: Vec<ObjectId> = t
            .g
            .battlefield
            .iter()
            .copied()
            .filter(|id| t.g.obj(*id).chars.has_subtype("Rogue"))
            .collect();
        assert_eq!(rogues.len(), tokens, "{victim}");
        for r in rogues {
            assert_eq!(t.g.obj(r).controller, P1);
        }
    }
}

#[test]
fn if_a_land_card_is_discarded_this_way() {
    cr!("701.9a", "608.2c");
    ruling!("Psychic Miasma", "Psychic Miasma is returned to its owner");
    // Psychic Miasma: "Target player discards a card. If a land card is discarded this
    // way, return ~ to its owner's hand."
    assert_supported(&["Psychic Miasma"]);
    for (card_in_hand, back) in [("Forest", true), ("Grizzly Bears", false)] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Swamp", 2);
        t.hand(P1, card_in_hand);
        let s = t.hand(P0, "Psychic Miasma");
        t.cast(P0, s).target(P1).go();
        t.resolve_all();
        assert_eq!(t.in_hand(P0, "Psychic Miasma"), back, "{card_in_hand}");
        assert!(t.in_graveyard(P1, card_in_hand));
    }
}

#[test]
fn if_the_discarded_card_was_a_zombie_card() {
    cr!("601.2h", "400.7j");
    // Necromancer's Stockpile: "{1}{B}, Discard a creature card: Draw a card. If the
    // discarded card was a Zombie card, create a tapped 2/2 black Zombie creature token."
    assert_supported(&["Necromancer's Stockpile"]);
    for (discard, zombie) in [("Walking Corpse", true), ("Grizzly Bears", false)] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Swamp", 2);
        let stockpile = t.battlefield(P0, "Necromancer's Stockpile");
        let c = t.hand(P0, discard);
        t.answer_choose(P0, &[Entity::Object(c)]);
        t.activate(P0, stockpile, 0, &[]).unwrap();
        t.resolve_all();
        assert!(t.in_graveyard(P0, discard));
        let zombies = t
            .g
            .battlefield
            .iter()
            .filter(|id| t.g.obj(**id).chars.has_subtype("Zombie") && t.g.obj(**id).tapped)
            .count();
        assert_eq!(zombies, usize::from(zombie), "{discard}");
    }
}

#[test]
fn if_the_sacrificed_creature_was_a_human() {
    cr!("601.2h", "608.2c");
    ruling!(
        "Falkenrath Aristocrat",
        "checks whether the sacrificed creature was a Human as it last existed on the battlefield"
    );
    // Falkenrath Aristocrat: "Sacrifice a creature: ~ gains indestructible until end of
    // turn. If the sacrificed creature was a Human, put a +1/+1 counter on ~."
    assert_supported(&["Falkenrath Aristocrat"]);
    for (victim, counters) in [("Elite Vanguard", 1), ("Grizzly Bears", 0)] {
        let mut t = TestGame::new(2);
        let fa = t.battlefield(P0, "Falkenrath Aristocrat");
        let c = t.battlefield(P0, victim);
        t.answer_choose(P0, &[Entity::Object(c)]);
        t.activate(P0, fa, 0, &[]).unwrap();
        t.resolve_all();
        assert!(!t.on_battlefield(c));
        assert_eq!(t.counters(fa, "+1/+1"), counters, "{victim}");
    }
}

#[test]
fn if_you_do_after_a_conditional_you_may() {
    cr!("608.2c");
    // Taborax, Hope's Demise: "Whenever another nontoken creature you control dies, put a
    // +1/+1 counter on ~. If that creature was a Cleric, you may draw a card. If you do,
    // you lose 1 life."
    assert_supported(&["Taborax, Hope's Demise"]);
    for (victim, cleric) in [("Grizzly Bears", false), ("Cleric of the Forward Order", true)] {
        let mut t = TestGame::new(2);
        let tab = t.battlefield(P0, "Taborax, Hope's Demise");
        let c = t.battlefield(P0, victim);
        t.answer_yes(P0, true);
        let hand = t.hand_size(P0);
        destroy(&mut t, c);
        t.resolve_all();
        assert_eq!(t.counters(tab, "+1/+1"), 1);
        assert_eq!(t.hand_size(P0), hand + usize::from(cleric), "{victim}");
        assert_eq!(t.life(P0), 20 - i32::from(cleric), "{victim}");
    }
}

#[test]
fn trailing_condition_is_checked_before_the_instruction() {
    cr!("608.2c");
    ruling!("Agadeem Occultist", "You check whether the targeted card");
    // Agadeem Occultist: "{T}: Put target creature card from an opponent's graveyard onto
    // the battlefield under your control if its mana value is less than or equal to the
    // number of Allies you control."
    assert_supported(&["Agadeem Occultist"]);
    for (victim, returns) in [("Grizzly Bears", false), ("Savannah Lions", true)] {
        let mut t = TestGame::new(2);
        let occ = t.battlefield(P0, "Agadeem Occultist");
        let c = t.graveyard(P1, victim);
        t.activate(P0, occ, 0, &[Entity::Object(c)]).unwrap();
        t.resolve_all();
        // The Occultist is the only Ally: mana value 1 or less.
        assert_eq!(t.named_on_battlefield(victim).len(), usize::from(returns));
    }
}

#[test]
fn if_its_an_artifact_creature_more_damage_instead() {
    cr!("608.2c", "614.1a");
    // Electrostatic Bolt: "~ deals 2 damage to target creature. If it's an artifact
    // creature, ~ deals 4 damage to it instead."
    assert_supported(&["Electrostatic Bolt"]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let golem = t.battlefield(P1, "Bronze Sable");
    let s = t.hand(P0, "Electrostatic Bolt");
    t.cast(P0, s).target(golem).go();
    t.resolve_all();
    assert!(!t.on_battlefield(golem), "an artifact creature takes 4");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let s = t.hand(P0, "Electrostatic Bolt");
    t.cast(P0, s).target(wurm).go();
    t.resolve_all();
    assert_eq!(t.g.obj(t.g.current(wurm)).damage, 2);
}

#[test]
fn if_its_not_their_turn() {
    cr!("603.4");
    // Glademuse: "Whenever a player casts a spell, if it's not their turn, that player
    // draws a card."
    assert_supported(&["Glademuse"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Glademuse");
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let before = t.hand_size(P1);
    t.cast(P1, bolt).target(P0).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P1), before);
    // On its controller's own turn: no card.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Glademuse");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let before = t.hand_size(P1);
    t.cast(P1, bolt).target(P0).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P1), before - 1);
}

#[test]
fn if_you_control_the_creature_with_the_greatest_power() {
    cr!("603.4", "608.2c");
    ruling!(
        "Triumph of Ferocity",
        "The ability checks whether you'll draw a card when it resolves."
    );
    // Triumph of Ferocity: "At the beginning of your upkeep, draw a card if you control the
    // creature with the greatest power or tied for the greatest power."
    assert_supported(&["Triumph of Ferocity"]);
    for (mine, theirs, draws) in [
        ("Craw Wurm", "Grizzly Bears", true),
        ("Grizzly Bears", "Craw Wurm", false),
        ("Grizzly Bears", "Grizzly Bears", true),
    ] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Triumph of Ferocity");
        t.battlefield(P0, mine);
        t.battlefield(P1, theirs);
        t.advance_to(P1, Step::Upkeep);
        let before = t.hand_size(P0);
        t.advance_to(P0, Step::Draw);
        // The turn's draw happens after the upkeep: count only the trigger's card.
        let drawn = t.hand_size(P0) - before;
        assert_eq!(drawn, 1 + usize::from(draws), "{mine} vs {theirs}");
    }
    // No creatures at all: no card.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Triumph of Ferocity");
    t.advance_to(P1, Step::Upkeep);
    let before = t.hand_size(P0);
    t.advance_to(P0, Step::Draw);
    assert_eq!(t.hand_size(P0) - before, 1);
}

#[test]
fn if_you_dont_control_a_creature_named() {
    cr!("201.2", "608.2c");
    // Jiang Yanggu: "-1: If you don't control a creature named Mowu, create Mowu, a
    // legendary 3/3 green Dog creature token."
    assert_supported(&["Jiang Yanggu"]);
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let jiang = t.battlefield(P0, "Jiang Yanggu");
    t.activate(P0, jiang, 1, &[]).unwrap();
    t.resolve_all();
    let mowus = |t: &TestGame| {
        t.g.battlefield
            .iter()
            .filter(|id| t.g.obj(**id).chars.has_name("Mowu"))
            .count()
    };
    assert_eq!(mowus(&t), 1);
    // Next turn: Mowu is still there, so no second one.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    t.activate(P0, jiang, 1, &[]).unwrap();
    t.resolve_all();
    assert_eq!(mowus(&t), 1);
}

#[test]
fn counter_if_you_control_more_creatures_than_its_controller() {
    cr!("608.2c");
    ruling!(
        "Unified Will",
        "is checked only as Unified Will resolves"
    );
    // Unified Will: "Counter target spell if you control more creatures than that spell's
    // controller."
    assert_supported(&["Unified Will"]);
    for (mine, countered) in [(1, false), (2, true)] {
        let mut t = TestGame::new(2);
        t.set_step(P1, Step::PrecombatMain);
        for _ in 0..mine {
            t.battlefield(P0, "Grizzly Bears");
        }
        t.battlefield(P1, "Grizzly Bears");
        t.lands(P1, "Forest", 2);
        t.lands(P0, "Island", 2);
        let bears = t.hand(P1, "Grizzly Bears");
        let spell = t.cast(P1, bears).go();
        let uw = t.hand(P0, "Unified Will");
        t.cast(P0, uw).target(spell).go();
        t.resolve_all();
        let p1_bears = t
            .g
            .battlefield
            .iter()
            .filter(|id| t.g.obj(**id).controller == P1)
            .filter(|id| t.g.obj(**id).chars.has_name("Grizzly Bears"))
            .count();
        assert_eq!(p1_bears, if countered { 1 } else { 2 }, "{mine} creatures");
    }
}

#[test]
fn if_a_graveyard_has_twenty_or_more_cards_draw_three_instead() {
    cr!("608.2c");
    ruling!(
        "Visions of Beyond",
        "You determine whether you'll draw one or three cards as Visions of Beyond begins to resolve"
    );
    // Visions of Beyond: "Draw a card. If a graveyard has twenty or more cards in it, draw
    // three cards instead."
    assert_supported(&["Visions of Beyond"]);
    for (cards, drawn) in [(19, 1), (20, 3)] {
        let mut t = TestGame::new(2);
        for _ in 0..cards {
            t.graveyard(P1, "Forest");
        }
        t.lands(P0, "Island", 1);
        let v = t.hand(P0, "Visions of Beyond");
        let before = t.hand_size(P0);
        t.cast(P0, v).go();
        t.resolve_all();
        assert_eq!(t.hand_size(P0) + 1 - before, drawn, "{cards} cards");
    }
}

#[test]
fn if_equipped_creature_is_a_vampire_two_counters_on_it_instead() {
    cr!("608.2c", "614.1a");
    // Blade of the Bloodchief: "Whenever a creature dies, put a +1/+1 counter on equipped
    // creature. If equipped creature is a Vampire, put two +1/+1 counters on it instead."
    assert_supported(&["Blade of the Bloodchief"]);
    for (holder, n) in [("Vampire Nighthawk", 2), ("Grizzly Bears", 1)] {
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, holder);
        let blade = t.battlefield(P0, "Blade of the Bloodchief");
        assert!(t.g.attach(blade, Entity::Object(c)));
        let victim = t.battlefield(P1, "Savannah Lions");
        destroy(&mut t, victim);
        t.resolve_all();
        assert_eq!(t.counters(c, "+1/+1"), n, "{holder}");
    }
}

#[test]
fn counter_that_spell_instead_if_its_controller_has_three_poison_counters() {
    cr!("608.2c", "122.1f");
    // Bring the Ending: "Counter target spell unless its controller pays {2}. Corrupted —
    // Counter that spell instead if its controller has three or more poison counters."
    assert_supported(&["Bring the Ending"]);
    for poison in [0, 3] {
        let mut t = TestGame::new(2);
        t.set_step(P1, Step::PrecombatMain);
        t.g.add_counters(Entity::Player(P1), "poison", poison, None);
        t.lands(P1, "Forest", 4);
        t.lands(P0, "Island", 2);
        let bears = t.hand(P1, "Grizzly Bears");
        let spell = t.cast(P1, bears).go();
        let s = t.hand(P0, "Bring the Ending");
        t.cast(P0, s).target(spell).go();
        // P1 would pay {2} if asked.
        t.answer_yes(P1, true);
        t.resolve_all();
        assert_eq!(
            t.named_on_battlefield("Grizzly Bears").len(),
            usize::from(poison == 0),
            "{poison} poison counters"
        );
    }
}

#[test]
fn if_you_return_a_nonland_card_this_way() {
    cr!("608.2c");
    // Vengeful Rebirth: "Return target card from your graveyard to your hand. If you
    // return a nonland card to your hand this way, ~ deals damage equal to that card's
    // mana value to any target."
    assert_supported(&["Vengeful Rebirth"]);
    for (card_back, damage) in [("Craw Wurm", 6), ("Forest", 0)] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Forest", 5);
        t.lands(P0, "Mountain", 1);
        let c = t.graveyard(P0, card_back);
        let s = t.hand(P0, "Vengeful Rebirth");
        t.cast(P0, s).target(c).target(P1).go();
        t.resolve_all();
        assert!(t.in_hand(P0, card_back));
        assert_eq!(t.life(P1), 20 - damage, "{card_back}");
    }
}

#[test]
fn if_the_sacrificed_creatures_toughness_was_four_two_food_instead() {
    cr!("601.2h", "608.2c");
    // Witch's Oven: "{T}, Sacrifice a creature: Create a Food token. If the sacrificed
    // creature's toughness was 4 or greater, create two Food tokens instead."
    assert_supported(&["Witch's Oven"]);
    for (victim, food) in [("Craw Wurm", 2), ("Grizzly Bears", 1)] {
        let mut t = TestGame::new(2);
        let oven = t.battlefield(P0, "Witch's Oven");
        let c = t.battlefield(P0, victim);
        t.answer_choose(P0, &[Entity::Object(c)]);
        t.activate(P0, oven, 0, &[]).unwrap();
        t.resolve_all();
        let foods = t
            .g
            .battlefield
            .iter()
            .filter(|id| t.g.obj(**id).chars.has_subtype("Food"))
            .count();
        assert_eq!(foods, food, "{victim}");
    }
}

#[test]
fn unused_effect_import() {
    // Keeps the `Effect` import used by helpers in this file.
    let _ = std::mem::size_of::<Effect>();
    let _ = Answer::Default;
}
