//! Rulings batch P052 — artifacts that exile cards and later return them: "exiled with"
//! links to one object (CR 400.7, 607.2a), "until [this] leaves the battlefield"
//! (CR 610.3), tokens leaving the battlefield (CR 111.7, 704.5d), Auras, Equipment and
//! counters of an exiled permanent (CR 704.5m, 704.5n, 122.2), face-down exiled cards
//! (CR 406.3, 708.5), players leaving a multiplayer game (CR 800.4a), and a cost of {0}
//! with the sacrifice in the effect (CR 701.21a).

use crate::r_p006_common::top;
use crate::r_p046_common::*;
use crate::r_p052_common::*;
use crate::r_s01_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::facedown::can_look_at;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, StackKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Boomerangs `target` (P0's instant).
fn boomerang(t: &mut TestGame, target: ObjectId) {
    let b = t.hand(P0, "Boomerang");
    fill_pool(t, P0, 2);
    t.cast(P0, b).target(obj(target)).go();
    t.resolve_all();
}

/// Casts `card` from P0's hand and resolves it; returns the permanent.
fn recast(t: &mut TestGame, card: ObjectId) -> ObjectId {
    let card = t.g.current(card);
    fill_pool(t, P0, 4);
    t.cast(P0, card).go();
    t.resolve_all();
    t.g.current(card)
}

#[test]
fn pinnacle_starcage_exiles_low_mana_values_dropping_auras_equipment_and_counters() {
    cr!("704.5m", "704.5n", "122.2", "610.3", "202.3e");
    supported("Pinnacle Starcage");
    ruling!(
        "Pinnacle Starcage",
        "Auras attached to an exiled permanent will be put into their owners’ graveyards. Equipment attached to an exiled permanent (if they’re not also being exiled) will become unattached and remain on the battlefield. Any counters on the exiled permanents will cease to exist."
    );
    ruling!(
        "Pinnacle Starcage",
        "If an artifact or creature has {X} in its mana cost, X is 0 for the purpose of determining its mana value."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let pacifism = t.battlefield(P1, "Pacifism");
    t.g.attach(pacifism, obj(bears));
    let hammer = t.battlefield(P1, "Loxodon Warhammer");
    t.g.attach(hammer, obj(bears));
    t.g.objects[bears.0 as usize]
        .counters
        .insert("+1/+1".into(), 2);
    let walker = t.battlefield(P1, "Hangarback Walker");
    t.g.objects[walker.0 as usize]
        .counters
        .insert("+1/+1".into(), 3);
    let krasis = t.battlefield(P1, "Hydroid Krasis");
    t.g.objects[krasis.0 as usize]
        .counters
        .insert("+1/+1".into(), 4);
    let giant = t.battlefield(P1, "Hill Giant");
    let cage = t.enter(P0, "Pinnacle Starcage");
    t.resolve_all();
    for id in [bears, walker, krasis] {
        assert_eq!(t.zone(id), Zone::Exile, "{}", t.obj_now(id).chars.name);
    }
    assert!(t.on_battlefield(giant) && t.on_battlefield(hammer));
    assert_eq!(t.obj_now(hammer).attached_to, None);
    assert!(t.in_graveyard(P1, "Pacifism"));
    // Back on the battlefield: no counters.
    t.g.destroy(cage, None);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.counters(bears, "+1/+1"), 0);
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn pinnacle_starcage_leaving_before_its_trigger_resolves_exiles_nothing() {
    cr!("610.3b");
    ruling!(
        "Pinnacle Starcage",
        "If Pinnacle Starcage leaves the battlefield before its first ability resolves, no artifacts or creatures will be exiled."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let cage = t.enter(P0, "Pinnacle Starcage");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.g.destroy(cage, None);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert!(t.g.exile.is_empty());
}

#[test]
fn pinnacle_starcage_returns_the_cards_when_its_owner_leaves_the_game() {
    cr!("800.4a", "610.3");
    ruling!(
        "Pinnacle Starcage",
        "In a multiplayer game, if Pinnacle Starcage’s owner leaves the game, the exiled cards will return to the battlefield. Because the one-shot effect that returns the cards isn’t an ability that goes on the stack, it won’t cease to exist along with the leaving player’s spells and abilities on the stack."
    );
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let elves = t.battlefield(P2, "Llanowar Elves");
    t.enter(P0, "Pinnacle Starcage");
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    t.g.lose_game(P0);
    t.settle();
    assert!(t.has_lost(P0));
    assert!(t.on_battlefield(bears) && t.on_battlefield(elves));
    assert_eq!(t.obj_now(bears).controller, P1);
    assert_eq!(t.obj_now(elves).controller, P2);
}

#[test]
fn realmbreakers_lands_are_exiled_when_their_controller_leaves() {
    cr!("800.4a", "614.1a");
    ruling!(
        "Realmbreaker, the Invasion Tree",
        "In a multiplayer game, if a player leaves the game, all cards that player owns leave as well. If you leave the game, any lands you control from Realmbreaker's ability are exiled."
    );
    supported("Realmbreaker, the Invasion Tree");
    let mut t = TestGame::new(3);
    let tree = t.battlefield(P0, "Realmbreaker, the Invasion Tree");
    stack_library(&mut t, P1, &["Forest", "Grizzly Bears", "Hill Giant"]);
    activate_resolve(&mut t, P0, tree, 0, &[pl(P1)]);
    let forest = t.named_on_battlefield("Forest");
    assert_eq!(forest.len(), 1);
    assert_eq!(t.obj_now(forest[0]).controller, P0);
    t.g.lose_game(P0);
    t.settle();
    assert!(t.named_on_battlefield("Forest").is_empty());
    assert!(t.in_exile("Forest"));
    assert!(!t.in_graveyard(P1, "Forest"));
}

#[test]
fn cold_storage_returns_only_creature_cards_and_never_tokens() {
    cr!("111.7", "704.5d", "607.2a", "400.7");
    supported("Cold Storage");
    supported("Mishra's Groundbreaker");
    ruling!(
        "Cold Storage",
        "For a time, the second ability returned all cards exiled by it to the battlefield, regardless of what types they had while exiled. It has been restored to its original functionality, so only cards that actually have the type Creature will get returned."
    );
    ruling!(
        "Cold Storage",
        "If it exiles a token creature, the token ceases to exist and will not return."
    );
    let mut t = TestGame::new(2);
    let storage = t.battlefield(P0, "Cold Storage");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // A land made a creature by Mishra's Groundbreaker: a land card in exile.
    let forest = t.battlefield(P0, "Forest");
    let breaker = t.battlefield(P0, "Mishra's Groundbreaker");
    activate_resolve(&mut t, P0, breaker, 0, &[obj(forest)]);
    let alarm = t.hand(P0, "Raise the Alarm");
    fill_pool(&mut t, P0, 2);
    t.cast(P0, alarm).go();
    t.resolve_all();
    let soldier = with_subtype(&t, P0, "Soldier")[0];
    for c in [bears, forest, soldier] {
        activate_resolve(&mut t, P0, storage, 0, &[obj(c)]);
    }
    assert_eq!(t.zone(bears), Zone::Exile);
    assert_eq!(t.zone(forest), Zone::Exile);
    assert_eq!(with_subtype(&t, P0, "Soldier").len(), 1);
    activate_resolve(&mut t, P0, storage, 1, &[]);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.zone(forest), Zone::Exile);
    assert_eq!(with_subtype(&t, P0, "Soldier").len(), 1);
}

#[test]
fn cold_storage_leaving_leaves_its_cards_exiled() {
    cr!("607.2a", "400.7");
    ruling!(
        "Cold Storage",
        "If this card leaves the battlefield, the cards it exiled remain exiled."
    );
    let mut t = TestGame::new(2);
    let storage = t.battlefield(P0, "Cold Storage");
    let bears = t.battlefield(P0, "Grizzly Bears");
    activate_resolve(&mut t, P0, storage, 0, &[obj(bears)]);
    t.g.destroy(storage, None);
    t.resolve_all();
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.zone(bears), Zone::Exile);
}

#[test]
fn voyager_staff_doesnt_return_an_exiled_token() {
    cr!("111.7", "704.5d", "603.7c");
    ruling!(
        "Voyager Staff",
        "If a creature token is exiled by Voyager Staff's ability, it will cease to exist. It won't return to the battlefield."
    );
    supported("Voyager Staff");
    for token in [true, false] {
        let mut t = TestGame::new(2);
        let staff = t.battlefield(P0, "Voyager Staff");
        let target = if token {
            let alarm = t.hand(P1, "Raise the Alarm");
            t.set_step(P1, Step::PrecombatMain);
            fill_pool(&mut t, P1, 2);
            t.cast(P1, alarm).go();
            t.resolve_all();
            t.set_step(P0, Step::PrecombatMain);
            with_subtype(&t, P1, "Soldier")[0]
        } else {
            t.battlefield(P1, "Grizzly Bears")
        };
        activate_resolve(&mut t, P0, staff, 0, &[obj(target)]);
        t.advance_to(P0, Step::End);
        t.resolve_all();
        assert_eq!(t.on_battlefield(target), !token, "token: {token}");
        if token {
            assert!(t.g.exile.is_empty());
        }
    }
}

#[test]
fn bag_of_holding_cards_belong_to_the_bag_that_exiled_them() {
    cr!("607.2a", "400.7", "603.3b");
    supported("Bag of Holding");
    ruling!(
        "Bag of Holding",
        "If you control more than one Bag of Holding, you choose which one will hold the discarded card. Other Bags of Holding can't return that card."
    );
    for first in [0usize, 1] {
        let mut t = TestGame::new(2);
        let bags = [
            t.battlefield(P0, "Bag of Holding"),
            t.battlefield(P0, "Bag of Holding"),
        ];
        let card = t.hand(P0, "Hill Giant");
        t.answer(P0, DecisionKind::Order, Answer::Indices(vec![first, 1 - first]));
        t.g.discard(P0, card, None);
        t.settle();
        assert_eq!(t.stack_len(), 2);
        let holder = match &t.g.obj(top(&t)).stack.as_ref().unwrap().kind {
            StackKind::Triggered { source, .. } => *source,
            k => panic!("{k:?}"),
        };
        t.resolve_all();
        assert_eq!(t.zone(card), Zone::Exile);
        let other = if holder == bags[0] { bags[1] } else { bags[0] };
        activate_resolve(&mut t, P0, other, 1, &[]);
        assert_eq!(t.zone(card), Zone::Exile);
        activate_resolve(&mut t, P0, holder, 1, &[]);
        assert_eq!(t.zone(card), Zone::Hand(P0));
    }
}

#[test]
fn bag_of_holding_exiled_by_leyline_stays_in_exile() {
    cr!("614.1a", "607.2a");
    ruling!(
        "Bag of Holding",
        "If Bag of Holding is moved to exile when you sacrifice it (most likely due to Leyline of the Void's effect), it remains in exile. It won't be returned to your hand."
    );
    supported("Leyline of the Void");
    let mut t = TestGame::new(2);
    let bag = t.battlefield(P0, "Bag of Holding");
    let card = t.hand(P0, "Hill Giant");
    t.g.discard(P0, card, None);
    t.resolve_all();
    assert_eq!(t.zone(card), Zone::Exile);
    t.battlefield(P1, "Leyline of the Void");
    activate_resolve(&mut t, P0, bag, 1, &[]);
    assert_eq!(t.zone(card), Zone::Hand(P0));
    assert_eq!(t.zone(bag), Zone::Exile);
    assert!(!t.in_hand(P0, "Bag of Holding"));
}

#[test]
fn a_returned_bag_of_holding_is_a_new_object() {
    cr!("400.7", "607.2a");
    ruling!(
        "Bag of Holding",
        "If Bag of Holding leaves the battlefield, the items it contained are exiled forever (and, perhaps, scattered throughout the Astral Plane). If the same Bag of Holding card returns to the battlefield, it's considered a new object without access to the cards stored by the old object."
    );
    let mut t = TestGame::new(2);
    let bag = t.battlefield(P0, "Bag of Holding");
    let card = t.hand(P0, "Hill Giant");
    t.g.discard(P0, card, None);
    t.resolve_all();
    boomerang(&mut t, bag);
    let bag = recast(&mut t, bag);
    activate_resolve(&mut t, P0, bag, 1, &[]);
    assert_eq!(t.zone(card), Zone::Exile);
}

#[test]
fn bag_of_holding_doesnt_chase_a_card_that_left_the_graveyard() {
    cr!("400.7", "608.2b");
    ruling!(
        "Bag of Holding",
        "If you discard a card but that card is not in your graveyard as Bag of Holding's first ability resolves, that card remains wherever it has moved."
    );
    supported("Tormod's Crypt");
    let mut t = TestGame::new(2);
    let bag = t.battlefield(P0, "Bag of Holding");
    let crypt = t.battlefield(P1, "Tormod's Crypt");
    let card = t.hand(P0, "Hill Giant");
    t.g.discard(P0, card, None);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // In response, P1's Tormod's Crypt exiles P0's graveyard.
    t.activate(P1, crypt, 0, &[pl(P0)]).expect("crypt");
    t.resolve_all();
    assert_eq!(t.zone(card), Zone::Exile);
    activate_resolve(&mut t, P0, bag, 1, &[]);
    assert_eq!(t.zone(card), Zone::Exile);
}

#[test]
fn bomat_couriers_cards_stay_with_the_old_object() {
    cr!("400.7", "607.2a", "406.3");
    ruling!(
        "Bomat Courier",
        "If Bomat Courier leaves the battlefield before you activate its last ability, any cards exiled by its triggered ability remain exiled face down for the rest of the game. If you somehow return the same Bomat Courier card to the battlefield, it will be a different object with no connection to those face-down cards."
    );
    supported("Bomat Courier");
    let mut t = TestGame::new(2);
    let courier = t.battlefield(P0, "Bomat Courier");
    let first = *t.g.player(P0).library.last().unwrap();
    attack_with(&mut t, &[(courier, pl(P1))]);
    t.resolve_all();
    assert_eq!(t.zone(first), Zone::Exile);
    t.advance_to(P0, Step::PostcombatMain);
    boomerang(&mut t, courier);
    let courier = recast(&mut t, courier);
    // Next turn it attacks again (haste isn't needed) and exiles another card.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    let second = *t.g.player(P0).library.last().unwrap();
    attack_with(&mut t, &[(courier, pl(P1))]);
    t.resolve_all();
    t.advance_to(P0, Step::PostcombatMain);
    assert_eq!(t.zone(second), Zone::Exile);
    activate_resolve(&mut t, P0, courier, 0, &[]);
    assert_eq!(t.zone(second), Zone::Hand(P0));
    assert_eq!(t.zone(first), Zone::Exile);
    assert!(t.obj_now(first).face_down);
}

#[test]
fn pyxis_of_pandemonium_cards_are_hidden_and_tied_to_that_pyxis() {
    cr!("406.3", "400.7", "607.2a");
    supported("Pyxis of Pandemonium");
    ruling!(
        "Pyxis of Pandemonium",
        "Neither player can look at cards exiled with Pyxis of Pandemonium until those cards are turned face up."
    );
    ruling!(
        "Pyxis of Pandemonium",
        "If Pyxis of Pandemonium leaves the battlefield, and later another Pyxis of Pandemonium enters the battlefield, it is a new object (even if the two were represented the same card). Cards exiled by the original one can't be turned face up or put onto the battlefield by the second one."
    );
    let mut t = TestGame::new(2);
    let pyxis = t.battlefield(P0, "Pyxis of Pandemonium");
    t.library_top(P0, "Grizzly Bears");
    t.library_top(P1, "Hill Giant");
    activate_resolve(&mut t, P0, pyxis, 0, &[]);
    let exiled = t.g.exile.clone();
    assert_eq!(exiled.len(), 2);
    for id in &exiled {
        assert!(t.g.obj(*id).face_down);
        for p in [P0, P1] {
            assert!(!can_look_at(&t.g, p, *id), "{p:?}");
        }
    }
    // The same card, back on the battlefield: a new object.
    boomerang(&mut t, pyxis);
    let pyxis = recast(&mut t, pyxis);
    activate_resolve(&mut t, P0, pyxis, 1, &[]);
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    assert!(t.named_on_battlefield("Hill Giant").is_empty());
    for id in &exiled {
        assert!(t.g.obj(*id).face_down);
    }
    // The original Pyxis would have put them onto the battlefield.
    let mut t = TestGame::new(2);
    let pyxis = t.battlefield(P0, "Pyxis of Pandemonium");
    t.library_top(P0, "Grizzly Bears");
    t.library_top(P1, "Hill Giant");
    activate_resolve(&mut t, P0, pyxis, 0, &[]);
    t.g.objects[pyxis.0 as usize].tapped = false;
    activate_resolve(&mut t, P0, pyxis, 1, &[]);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
}

#[test]
fn knowledge_vault_sacrifices_itself_on_resolution_or_does_nothing() {
    cr!("701.21a", "608.2c", "118.5");
    ruling!(
        "Knowledge Vault",
        "If you activate Knowledge Vault's second ability, all that happens at that time is that you pay a cost of {0}. As the ability resolves, you must sacrifice Knowledge Vault. This is mandatory. If you are able to do so, you discard your hand and put all cards exiled with Knowledge Vault into their owners' hands. If you aren't able to do so because Knowledge Vault left the battlefield or changed controllers before the ability resolves, nothing happens."
    );
    supported("Knowledge Vault");
    for destroyed in [false, true] {
        let mut t = TestGame::new(2);
        let vault = t.battlefield(P0, "Knowledge Vault");
        for _ in 0..2 {
            t.g.objects[vault.0 as usize].tapped = false;
            activate_resolve(&mut t, P0, vault, 0, &[]);
        }
        let exiled = t.g.exile.clone();
        assert_eq!(exiled.len(), 2);
        let card = t.hand(P0, "Hill Giant");
        t.activate(P0, vault, 1, &[]).expect("activate {0}");
        assert!(t.on_battlefield(vault));
        if destroyed {
            t.g.destroy(vault, None);
        }
        t.resolve_all();
        assert!(!t.on_battlefield(vault));
        if destroyed {
            // Nothing happened; its leaves-the-battlefield trigger put the cards into
            // the graveyard.
            assert_eq!(t.zone(card), Zone::Hand(P0));
            for id in &exiled {
                assert_eq!(t.zone(*id), Zone::Graveyard(P0));
            }
        } else {
            assert_eq!(t.zone(card), Zone::Graveyard(P0));
            for id in &exiled {
                assert_eq!(t.zone(*id), Zone::Hand(P0));
            }
        }
    }
}

#[test]
fn bag_of_devouring_exiles_what_you_sacrifice_but_needs_a_sacrifice_outlet() {
    cr!("701.21a", "603.10a");
    ruling!(
        "Bag of Devouring",
        "Bag of Devouring's first ability is a triggered ability that happens when you sacrifice permanents, but you can't just sacrifice permanents whenever you want. You need to have some other means to do so, such as activating Bag of Devouring's second ability."
    );
    supported("Bag of Devouring");
    let mut t = TestGame::new(2);
    let bag = t.battlefield(P0, "Bag of Devouring");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    // Destroyed, not sacrificed: not exiled.
    t.g.destroy(giant, None);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hill Giant"));
    t.answer_choose(P0, &[obj(bears)]);
    let hand = t.hand_size(P0);
    activate_resolve(&mut t, P0, bag, 0, &[]);
    assert_eq!(t.zone(bears), Zone::Exile);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn found_footage_shows_face_down_creatures_not_face_down_spells() {
    cr!("708.5", "708.4");
    supported("Found Footage");
    ruling!(
        "Found Footage",
        "Found Footage's first ability doesn't let you look at face-down spells you don't control on the stack."
    );
    ruling!(
        "Found Footage",
        "Found Footage's first ability lets you look at face-down creatures whenever you want, even if you don't have priority. This action doesn't use the stack. Knowing what those cards are becomes part of the information you have access to, just like you can look at the cards in your hand."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Found Footage");
    let angel = t.hand(P1, "Exalted Angel");
    t.set_step(P1, Step::PrecombatMain);
    fill_pool(&mut t, P1, 3);
    let spell = t
        .cast(P1, angel)
        .method(CastMethod::FaceDown(KeywordKind::Morph))
        .go();
    assert!(t.g.obj(spell).face_down);
    assert!(!can_look_at(&t.g, P0, spell));
    assert!(can_look_at(&t.g, P1, spell));
    t.resolve_all();
    let creature = t.g.current(spell);
    assert!(t.obj_now(creature).face_down);
    // P1 is the active player with priority; P0 can still look at it.
    assert_eq!(t.g.turn.priority, Some(P1));
    assert!(can_look_at(&t.g, P0, creature));
    assert_eq!(t.stack_len(), 0);
}
