//! Rulings batch P209 — enchant (CR 303.4, 702.5): Aura triggered abilities — enters
//! triggers whose enchanted permanent leaves, upkeep and draw triggers of Curses, abilities
//! that keep working after the Aura leaves, and Auras in multiplayer games.

use crate::r_p209_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s03_common::*;
use crate::r_s04_common::*;
use crate::r_s05_common::*;
use crate::r_s06_common::*;
use mtg_engine::decision::Action;
use mtg_engine::designations::become_monarch;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Exiles cards from the top of `p`'s library until it has `n` left.
fn library_down_to(t: &mut TestGame, p: PlayerId, n: usize) {
    while t.library_size(p) > n {
        let top = *t.g.player(p).library.last().unwrap();
        t.g.move_object(top, Zone::Exile, mtg_engine::events::MoveCause::Effect, None);
    }
    t.g.flush_events();
}

fn concede(t: &mut TestGame, p: PlayerId) {
    t.g.take_action(p, Action::Concede);
    t.g.flush_events();
}

// ---------------------------------------------------------------------------------------
// Enters triggers
// ---------------------------------------------------------------------------------------

#[test]
fn new_horizons_with_an_illegal_target_doesnt_enter() {
    cr!("608.3b", "303.4j");
    ruling!(
        "New Horizons",
        "If the land this Aura would enchant is an illegal target by the time New Horizons resolves, the entire spell doesn't resolve."
    );
    supported("New Horizons");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let forest = t.battlefield(P0, "Forest");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let a = in_hand_with_mana(&mut t, P0, "New Horizons");
    t.cast(P0, a).target(forest).go();
    t.answer_targets(P0, &[Entity::Object(bears)]);
    move_to(&mut t, forest, Zone::Graveyard(P0));
    t.resolve_all();
    assert!(t.in_graveyard(P0, "New Horizons"));
    assert_eq!(t.counters(bears, "+1/+1"), 0);
}

#[test]
fn nurturing_presence_spirit_triggers_the_granted_ability() {
    cr!("603.2", "603.3");
    ruling!(
        "Nurturing Presence",
        "Assuming that Nurturing Presence is still on the battlefield as that first triggered ability resolves, the Spirit will cause the ability the enchanted creature has to trigger"
    );
    supported("Nurturing Presence");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_aura(&mut t, P0, "Nurturing Presence", bears).unwrap();
    assert_eq!(with_subtype(&t, P0, "Spirit").len(), 1);
    assert_eq!(t.pt(bears), (3, 3));
}

// ---------------------------------------------------------------------------------------
// Upkeep triggers and Curses
// ---------------------------------------------------------------------------------------

#[test]
fn curse_of_oblivion_with_one_card_exiles_it() {
    cr!("701.13a", "107.1c");
    ruling!(
        "Curse of Oblivion",
        "If the enchanted player has only one card in their graveyard, they exile that card."
    );
    supported("Curse of Oblivion");
    let mut t = TestGame::new(2);
    attach_new(&mut t, P0, "Curse of Oblivion", P1);
    t.graveyard(P1, "Forest");
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    t.resolve_all();
    assert_eq!(t.graveyard_size(P1), 0);
    assert!(t.in_exile("Forest"));
}

#[test]
fn curse_of_the_bloody_tome_with_one_card_mills_it() {
    cr!("701.17a", "701.17b");
    ruling!(
        "Curse of the Bloody Tome",
        "If the enchanted player has only one card in their library, they put that card into their graveyard."
    );
    supported("Curse of the Bloody Tome");
    let mut t = TestGame::new(2);
    attach_new(&mut t, P0, "Curse of the Bloody Tome", P1);
    t.set_step(P1, Step::Untap);
    library_down_to(&mut t, P1, 1);
    let g = t.graveyard_size(P1);
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    t.resolve_all();
    assert_eq!(t.library_size(P1), 0);
    assert_eq!(t.graveyard_size(P1), g + 1);
}

#[test]
fn shattered_ego_with_a_short_library_puts_the_creature_on_the_bottom() {
    cr!("401.7", "401.4");
    ruling!(
        "Shattered Ego",
        "If the enchanted creature's owner has two or fewer cards in their library, the enchanted creature is put on the bottom of their library"
    );
    supported("Shattered Ego");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let ego = attach_new(&mut t, P0, "Shattered Ego", bears);
    library_down_to(&mut t, P1, 2);
    add_mana(&mut t, P0, ManaType::U, 2);
    add_mana(&mut t, P0, ManaType::C, 3);
    t.activate(P0, ego, 0, &[]).unwrap();
    t.resolve_all();
    let lib = &t.g.player(P1).library;
    assert_eq!(lib.len(), 3);
    // The top is the end of the list: the Bears are at the bottom.
    assert_eq!(t.g.obj(lib[0]).chars.name, "Grizzly Bears");
}

#[test]
fn elemental_resonance_adds_the_mana_cost() {
    cr!("202.1", "107.3", "107.4e");
    ruling!(
        "Elemental Resonance",
        "If the enchanted permanent's mana cost is {2}{W}{U}, this Aura's controller adds {C}{C}{W}{U}."
    );
    supported("Elemental Resonance");
    for (name, expect) in [
        ("Thunderclap Wyvern", vec![(ManaType::C, 2), (ManaType::W, 1), (ManaType::U, 1)]),
        ("Jadelight Spelunker", vec![(ManaType::G, 1)]),
    ] {
        let mut t = TestGame::new(2);
        let p = t.battlefield(P1, name);
        attach_new(&mut t, P0, "Elemental Resonance", p);
        t.set_step(P0, Step::Upkeep);
        t.advance_to(P0, Step::PrecombatMain);
        t.settle();
        t.resolve_all();
        let pool = &t.g.player(P0).mana_pool;
        let total: usize = expect.iter().map(|(_, n)| n).sum();
        assert_eq!(pool.total(), total, "{name}");
        for (ty, n) in expect {
            assert_eq!(pool.count(ty), n, "{name} {ty:?}");
        }
    }
    // Hybrid: the controller chooses, each time.
    for (picks, w) in [([0, 0], 2), ([0, 1], 1), ([1, 1], 0)] {
        let mut t = TestGame::new(2);
        let p = t.battlefield(P1, "Azorius Guildmage");
        attach_new(&mut t, P0, "Elemental Resonance", p);
        t.set_step(P0, Step::Upkeep);
        for i in picks {
            t.answer(P0, DecisionKind::Option, Answer::Index(i));
        }
        t.advance_to(P0, Step::PrecombatMain);
        t.settle();
        t.resolve_all();
        let pool = &t.g.player(P0).mana_pool;
        assert_eq!(pool.total(), 2);
        assert_eq!(pool.count(ManaType::W), w, "{picks:?}");
        assert_eq!(pool.count(ManaType::U), 2 - w, "{picks:?}");
    }
}

#[test]
fn primal_cocoon_triggers_on_its_controllers_upkeep() {
    cr!("503.1a", "603.2");
    ruling!(
        "Primal Cocoon",
        "Primal Cocoon's first triggered ability triggers at the beginning of the upkeep of Primal Cocoon's controller"
    );
    supported("Primal Cocoon");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P0, "Primal Cocoon", bears);
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 0);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 1);
}

#[test]
fn paradox_haze_twice_gives_two_additional_upkeeps() {
    cr!("500.8", "603.2");
    ruling!(
        "Paradox Haze",
        "If two Paradox Hazes enchant the same player, they'll both trigger when that player's first upkeep of the turn begins."
    );
    supported("Paradox Haze");
    let mut t = TestGame::new(2);
    attach_new(&mut t, P0, "Paradox Haze", P1);
    attach_new(&mut t, P0, "Paradox Haze", P1);
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    let upkeeps = t.g.turn.schedule.iter().filter(|s| **s == Step::Upkeep).count();
    assert_eq!(upkeeps, 2);
    // The additional upkeeps aren't "first": no more triggers.
    t.advance_to_step(Step::Draw);
    let upkeeps = t.g.turn.schedule.iter().filter(|s| **s == Step::Upkeep).count();
    assert_eq!(upkeeps, 0);
    assert_eq!(t.g.turn.active, P1);
}

/// `p` draws a card, then triggers go on the stack.
fn draw(t: &mut TestGame, p: PlayerId) {
    t.g.draw_cards(p, 1);
    t.g.flush_events();
    t.settle();
}

#[test]
fn curse_of_fools_wisdom_on_yourself_doesnt_kill_between_loss_and_gain() {
    cr!("704.3", "608.2");
    ruling!(
        "Curse of Fool's Wisdom",
        "If you become enchanted by your own Curse of Fool's Wisdom, you'll lose 2 life and gain 2 life whenever you draw a card."
    );
    supported("Curse of Fool's Wisdom");
    let mut t = TestGame::new(2);
    attach_new(&mut t, P0, "Curse of Fool's Wisdom", P0);
    t.g.players[P0.idx()].life = 2;
    draw(&mut t, P0);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P0), 2);
    assert!(!t.has_lost(P0));
}

#[test]
fn curse_of_fools_wisdom_trigger_resolves_without_the_curse() {
    cr!("113.7a");
    ruling!(
        "Curse of Fool's Wisdom",
        "Once the enchanted player has drawn one or more cards, they'll still lose life (and you'll still gain life) even if Curse of Fool's Wisdom leaves the battlefield"
    );
    let mut t = TestGame::new(2);
    let curse = attach_new(&mut t, P0, "Curse of Fool's Wisdom", P1);
    draw(&mut t, P1);
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, curse);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 22);
}

#[test]
fn curse_of_vengeance_controller_losing_at_the_same_time_gains_nothing() {
    cr!("104.4a", "704.5a", "800.4a");
    ruling!(
        "Curse of Vengeance",
        "If you and the enchanted player both reach 0 or less life at the same time, you'll lose the game before Curse of Vengeance's second triggered ability gives you more life."
    );
    supported("Curse of Vengeance");
    let mut t = TestGame::new(3);
    let curse = attach_new(&mut t, P0, "Curse of Vengeance", P1);
    t.g.add_counters(Entity::Object(curse), "spite", 3, None);
    t.g.players[P0.idx()].life = 0;
    t.g.players[P1.idx()].life = 0;
    t.settle();
    t.resolve_all();
    assert!(t.has_lost(P0));
    assert!(t.has_lost(P1));
    assert!(!t.has_lost(P2));
}

#[test]
fn quiet_disrepair_destroys_whatever_it_enchants_on_resolution() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Quiet Disrepair",
        "If you choose the “destroy” mode and Quiet Disrepair is moved to a different permanent while the ability is on the stack, the newly enchanted permanent will be destroyed when the ability resolves."
    );
    ruling!(
        "Quiet Disrepair",
        "If you choose the “destroy” mode and Quiet Disrepair leaves the battlefield while the ability is on the stack, the last permanent it enchanted will be destroyed."
    );
    supported("Quiet Disrepair");
    for moved in [false, true] {
        let mut t = TestGame::new(2);
        let first = t.battlefield(P1, "Mind Stone");
        let second = t.battlefield(P1, "Sol Ring");
        let qd = attach_new(&mut t, P0, "Quiet Disrepair", first);
        t.set_step(P1, Step::End);
        t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
        t.advance_to(P0, Step::Upkeep);
        t.settle();
        assert_eq!(t.stack_len(), 1);
        if moved {
            assert!(t.g.attach(qd, Entity::Object(second)));
            t.g.recompute();
        } else {
            destroy(&mut t, qd);
        }
        t.resolve_all();
        assert_eq!(t.on_battlefield(first), moved, "moved: {moved}");
        assert_eq!(t.on_battlefield(second), !moved, "moved: {moved}");
    }
}

#[test]
fn ghitu_firebreathing_pump_applies_after_it_returns_to_hand() {
    cr!("113.7a", "608.2h");
    ruling!(
        "Ghitu Firebreathing",
        "If you return Ghitu Firebreathing to its owner's hand while the +1/+0 ability is on the stack, that ability will still give the creature that was last enchanted by Ghitu Firebreathing +1/+0."
    );
    supported("Ghitu Firebreathing");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let g = attach_new(&mut t, P0, "Ghitu Firebreathing", bears);
    add_mana(&mut t, P0, ManaType::R, 2);
    activate_containing(&mut t, P0, g, "+1/+0").unwrap();
    activate_containing(&mut t, P0, g, "Return").unwrap();
    t.resolve();
    assert!(t.in_hand(P0, "Ghitu Firebreathing"));
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 2));
}

#[test]
fn illusionary_armor_sacrifices_only_itself() {
    cr!("701.21a", "603.2");
    ruling!(
        "Illusionary Armor",
        "Only Illusionary Armor is sacrificed because of its triggered ability; the enchanted creature is not."
    );
    supported("Illusionary Armor");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Illusionary Armor", bears);
    cast_spell(&mut t, P1, "Giant Growth", &[Entity::Object(bears)]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert!(t.in_graveyard(P0, "Illusionary Armor"));
    assert!(t.on_battlefield(bears));
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5));
}

#[test]
fn invocation_of_saint_traft_token_is_exiled_even_without_the_aura() {
    cr!("603.7c", "603.7b");
    ruling!(
        "Invocation of Saint Traft",
        "Removing Invocation of Saint Traft or the enchanted creature from the battlefield won’t stop the delayed triggered ability from exiling the Angel token at end of combat."
    );
    supported("Invocation of Saint Traft");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let inv = attach_new(&mut t, P0, "Invocation of Saint Traft", bears);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(tokens_with_subtype(&t, P0, "Angel").len(), 1);
    destroy(&mut t, inv);
    destroy(&mut t, bears);
    block_and_finish(&mut t, P1, &[]);
    t.settle();
    t.resolve_all();
    assert_eq!(tokens_with_subtype(&t, P0, "Angel").len(), 0);
    assert_eq!(t.life(P1), 16);
}

// ---------------------------------------------------------------------------------------
// Multiplayer
// ---------------------------------------------------------------------------------------

#[test]
fn fealty_to_the_realm_new_monarch_gains_control() {
    cr!("725.4", "800.4a");
    ruling!(
        "Fealty to the Realm",
        "If the monarch leaves the game, and that player is not the owner of Fealty to the Realm, the active player or the next player in turn order becomes the monarch and thus gains control of the enchanted creature."
    );
    supported("Fealty to the Realm");
    // P2 owns Fealty to the Realm and the Bears; P1 is the monarch, then leaves the
    // game during P0's turn.
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P2, "Grizzly Bears");
    attach_new(&mut t, P2, "Fealty to the Realm", bears);
    become_monarch(&mut t.g, P1);
    t.g.recompute();
    assert_eq!(t.obj_now(bears).controller, P1);
    concede(&mut t, P1);
    t.settle();
    assert_eq!(t.g.monarch, Some(P0));
    assert_eq!(t.obj_now(bears).controller, P0);
}

#[test]
fn fealty_to_the_realm_owner_monarch_leaving_returns_control_to_the_owner() {
    cr!("725.4", "800.4a");
    ruling!(
        "Fealty to the Realm",
        "If the monarch leaves the game, and that player is the owner of Fealty to the Realm, the active player or the next player in turn order becomes the monarch."
    );
    let mut t = TestGame::new(3);
    t.set_step(P2, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P0, "Fealty to the Realm", bears);
    become_monarch(&mut t.g, P0);
    t.g.recompute();
    assert_eq!(t.obj_now(bears).controller, P0);
    concede(&mut t, P0);
    t.settle();
    assert_eq!(t.g.monarch, Some(P2));
    assert!(t.named_on_battlefield("Fealty to the Realm").is_empty());
    assert_eq!(t.obj_now(bears).controller, P1);
}

#[test]
fn psychic_possession_leaves_with_either_player() {
    cr!("704.5m", "800.4a", "303.4d");
    ruling!(
        "Psychic Possession",
        "In a multiplayer game, if the enchanted opponent leaves the game, Psychic Possession will be put into its owner's graveyard as a state-based action."
    );
    supported("Psychic Possession");
    let mut t = TestGame::new(3);
    attach_new(&mut t, P0, "Psychic Possession", P1);
    concede(&mut t, P1);
    t.settle();
    assert!(t.in_graveyard(P0, "Psychic Possession"));
    let mut t = TestGame::new(3);
    let pp = attach_new(&mut t, P0, "Psychic Possession", P1);
    concede(&mut t, P0);
    assert!(!t.g.is_live(pp));
    assert!(!t.in_graveyard(P0, "Psychic Possession"));
}

#[test]
fn psychic_possession_its_controller_skips_their_draw_step() {
    cr!("614.10", "504.1");
    ruling!(
        "Psychic Possession",
        "Psychic Possession's controller, not the enchanted player, skips their draw step."
    );
    let mut t = TestGame::new(2);
    attach_new(&mut t, P0, "Psychic Possession", P1);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    let h0 = t.hand_size(P0);
    t.advance_to(P0, Step::PrecombatMain);
    assert_eq!(t.hand_size(P0), h0);
    t.advance_to(P1, Step::Upkeep);
    let h1 = t.hand_size(P1);
    t.answer_yes(P0, false);
    t.advance_to(P1, Step::PrecombatMain);
    assert_eq!(t.hand_size(P1), h1 + 1);
}
