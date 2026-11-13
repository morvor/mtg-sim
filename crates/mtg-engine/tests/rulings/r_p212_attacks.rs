//! Rulings batch P212 — Equipment (CR 301.5, 702.6) with "whenever equipped creature
//! attacks" triggers: tokens put onto the battlefield attacking (CR 506.3, 508.4),
//! myriad (CR 702.116), additional combat phases (CR 500.8), intervening "if" clauses
//! (CR 603.4), untap restrictions (CR 502.3), attack/block requirements (CR 508.1d,
//! 509.1c) and "you may play those cards this turn" (CR 305.1, 601.3).

use crate::r_p212_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::{can_cast, can_play_land, destroy};
use crate::r_s04_common::next_upkeep;
use crate::r_s06_common::*;
use crate::r_s09_common::{legal_attack, to_combat};
use crate::r_s12_common::attack_target;
use crate::r_s21_common::legal_blocks;
use crate::r_s30_common::set_life;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn spirits(t: &TestGame, p: PlayerId) -> Vec<ObjectId> {
    with_subtype(t, p, "Spirit")
}

// ---------------------------------------------------------------------------------------
// Andúril, Flame of the West
// ---------------------------------------------------------------------------------------

#[test]
fn anduril_on_an_opponents_creature_makes_tapped_spirits_that_dont_attack() {
    cr!("506.3b", "603.3a", "301.5d");
    ruling!(
        "Andúril, Flame of the West",
        "If Andúril is equipped to a creature an opponent controls, Andúril's controller will create two tapped Spirit tokens each time that creature attacks. Even if that creature is legendary, the Spirits would not enter the battlefield attacking."
    );
    supported("Andúril, Flame of the West");
    for legendary in ["Grizzly Bears", "Isamaru, Hound of Konda"] {
        let mut t = TestGame::new(2);
        t.set_step(P1, Step::PrecombatMain);
        let c = t.battlefield(P1, legendary);
        attach_new(&mut t, P0, "Andúril, Flame of the West", c);
        attack_with(&mut t, &[(c, Entity::Player(P0))]);
        t.resolve_all();
        let s = spirits(&t, P0);
        assert_eq!(s.len(), 2, "{legendary}");
        assert!(spirits(&t, P1).is_empty());
        for id in s {
            assert!(t.obj_now(id).tapped);
            assert_eq!(attack_target(&t, id), None, "{legendary}");
        }
    }
}

#[test]
fn anduril_legendary_creature_each_spirit_chooses_what_it_attacks() {
    cr!("508.4", "506.2");
    ruling!(
        "Andúril, Flame of the West",
        "If the equipped creature is legendary, Andúril's controller chooses which player, planeswalker, or battle the Spirit tokens are attacking. Each Spirit token can enter attacking a different player, planeswalker, or battle, and they don't need to be the same player, planeswalker, or battle that the equipped creature is attacking."
    );
    let mut t = TestGame::new(3);
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    attach_new(&mut t, P0, "Andúril, Flame of the West", isamaru);
    let jace = t.battlefield(P2, "Jace, the Mind Sculptor");
    attack_with(&mut t, &[(isamaru, Entity::Player(P1))]);
    t.answer_choose(P0, &[Entity::Object(jace)]);
    t.answer_choose(P0, &[Entity::Player(P2)]);
    t.resolve_all();
    let s = spirits(&t, P0);
    assert_eq!(s.len(), 2);
    let mut got: Vec<_> = s.iter().filter_map(|id| attack_target(&t, *id)).collect();
    got.sort();
    let mut want = vec![Entity::Object(jace), Entity::Player(P2)];
    want.sort();
    assert_eq!(got, want);
    assert_eq!(attack_target(&t, isamaru), Some(Entity::Player(P1)));
}

// ---------------------------------------------------------------------------------------
// Blade of Selves (myriad)
// ---------------------------------------------------------------------------------------

#[test]
fn blade_of_selves_on_a_legendary_creature_the_legend_rule_applies_to_the_tokens() {
    cr!("702.116a", "704.5j", "603.6c");
    ruling!(
        "Blade of Selves",
        "If the equipped creature is legendary, the tokens will all enter the battlefield. Then you'll choose one of them to remain on the battlefield. The others will be put into your graveyard. Any enters-the-battlefield or dies triggers of the tokens will work."
    );
    supported("Kokusho, the Evening Star");
    // Four players: two token Kokushos enter; P0 keeps the original. Each token that dies
    // triggers: each opponent loses 5 life twice.
    let mut t = TestGame::new(4);
    let kokusho = t.battlefield(P0, "Kokusho, the Evening Star");
    attach_new(&mut t, P0, "Blade of Selves", kokusho);
    attack_with(&mut t, &[(kokusho, Entity::Player(P1))]);
    t.answer_choose(P0, &[Entity::Object(kokusho)]);
    t.resolve_all();
    assert!(t.on_battlefield(kokusho));
    assert!(tokens(&t, P0).is_empty());
    for p in [P1, P2, P3] {
        assert_eq!(t.life(p), 10);
    }
    assert_eq!(t.life(P0), 50);
}

#[test]
fn blade_of_selves_noncreature_tokens_dont_attack_but_are_still_exiled() {
    cr!("702.116a", "506.3a");
    ruling!(
        "Blade of Selves",
        "If the tokens aren't creatures (perhaps because the equipped creature was an animated land), they'll enter the battlefield but they won't be attacking. You'll still exile those tokens at end of combat."
    );
    // Mishra's Factory, animated ({1}: it becomes a 2/2 artifact creature until end of
    // turn): the token copies are just lands.
    let mut t = TestGame::new(3);
    let factory = t.battlefield(P0, "Mishra's Factory");
    crate::r_s04_common::add_mana(&mut t, P0, mtg_engine::mana::ManaType::C, 1);
    activate_containing(&mut t, P0, factory, "becomes").unwrap();
    t.resolve_all();
    attach_new(&mut t, P0, "Blade of Selves", factory);
    attack_with(&mut t, &[(factory, Entity::Player(P1))]);
    assert_eq!(attack_target(&t, factory), Some(Entity::Player(P1)));
    t.resolve_all();
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 1);
    let tok = toks[0];
    assert!(t.on_battlefield(tok));
    assert!(!t.obj_now(tok).chars.is_creature());
    assert_eq!(attack_target(&t, tok), None);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert!(!t.on_battlefield(tok));
    assert!(tokens(&t, P0).is_empty());
}

// ---------------------------------------------------------------------------------------
// Seraphic Greatsword: "Whenever equipped creature attacks the player with the most life
// or tied for most life, create a 4/4 white Angel ... tapped and attacking that player."
// ---------------------------------------------------------------------------------------

#[test]
fn seraphic_greatsword_condition_is_checked_only_as_it_triggers() {
    cr!("603.2", "508.4");
    ruling!(
        "Seraphic Greatsword",
        "Once the triggered ability triggers, it doesn't matter what happens to players' life totals before the ability resolves. You'll create an Angel token even if the player the equipped creature attacked doesn't have the most life as the ability resolves."
    );
    ruling!(
        "Seraphic Greatsword",
        "Seraphic Greatsword's triggered ability won't trigger if the equipped creature attacks a planeswalker controlled by the player with the most life."
    );
    supported("Seraphic Greatsword");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Seraphic Greatsword", bears);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 1);
    set_life(&mut t, P1, 5);
    t.resolve_all();
    let angels = with_subtype(&t, P0, "Angel");
    assert_eq!(angels.len(), 1);
    assert_eq!(attack_target(&t, angels[0]), Some(Entity::Player(P1)));
    // Attacking a planeswalker of the player with the most life: no trigger.
    let mut t = TestGame::new(2);
    set_life(&mut t, P1, 30);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Seraphic Greatsword", bears);
    let jace = t.battlefield(P1, "Jace, the Mind Sculptor");
    attack_with(&mut t, &[(bears, Entity::Object(jace))]);
    assert_eq!(t.stack_len(), 0);
    t.resolve_all();
    assert!(with_subtype(&t, P0, "Angel").is_empty());
}

// ---------------------------------------------------------------------------------------
// Hexplate Wallbreaker: "Whenever equipped creature attacks, if it's the first combat
// phase of the turn, untap each attacking creature. After this phase, there is an
// additional combat phase."
// ---------------------------------------------------------------------------------------

fn combats_scheduled(t: &TestGame) -> usize {
    t.g.turn
        .schedule
        .iter()
        .filter(|s| **s == Step::BeginningOfCombat)
        .count()
}

#[test]
fn hexplate_wallbreakers_each_add_a_combat_but_untap_only_in_the_first() {
    cr!("500.8", "603.4");
    ruling!(
        "Hexplate Wallbreaker",
        "If you attack with multiple creatures equipped with Hexplate Wallbreakers in one combat phase (or one creature equipped with multiple Hexplate Wallbreakers), you'll have that many additional combat phases, but your attacking creatures are untapped only during the current combat phase."
    );
    supported("Hexplate Wallbreaker");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lion = t.battlefield(P0, "Silvercoat Lion");
    attach_new(&mut t, P0, "Hexplate Wallbreaker", bears);
    attach_new(&mut t, P0, "Hexplate Wallbreaker", lion);
    // Reach the first combat phase normally (so the turn counts its combat phases).
    t.set_step(P0, Step::PrecombatMain);
    t.advance_to(P0, Step::BeginningOfCombat);
    let p1 = Entity::Player(P1);
    attack_with(&mut t, &[(bears, p1), (lion, p1)]);
    assert_eq!(combats_scheduled(&t), 0);
    t.resolve_all();
    assert_eq!(combats_scheduled(&t), 2);
    assert!(!t.obj_now(bears).tapped && !t.obj_now(lion).tapped);
    // The second combat: they attack again; no more untapping or combats.
    t.advance_to(P0, Step::EndOfCombat);
    t.advance_to(P0, Step::BeginningOfCombat);
    assert_eq!(combats_scheduled(&t), 1);
    attack_with(&mut t, &[(bears, p1), (lion, p1)]);
    t.resolve_all();
    assert_eq!(combats_scheduled(&t), 1);
    assert!(t.obj_now(bears).tapped && t.obj_now(lion).tapped);
}

#[test]
fn hexplate_wallbreaker_combat_is_added_even_if_the_creature_dies() {
    cr!("500.8", "603.10a");
    ruling!(
        "Hexplate Wallbreaker",
        "Once the equipped creature attacks and the triggered ability resolves, you get an additional combat phase even if the equipped creature doesn't survive the first combat phase."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Hexplate Wallbreaker", bears);
    let wurm = t.battlefield(P1, "Craw Wurm");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    block_and_finish(&mut t, P1, &[(wurm, bears)]);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    let turn = t.g.turn.number;
    t.advance_to(P0, Step::BeginningOfCombat);
    assert_eq!(t.g.turn.step, Step::BeginningOfCombat);
    assert_eq!(t.g.turn.number, turn);
}

// ---------------------------------------------------------------------------------------
// Stitcher's Graft
// ---------------------------------------------------------------------------------------

#[test]
fn stitchers_grafts_doesnt_untap_effects_share_one_untap_step() {
    cr!("502.3", "614.10");
    ruling!(
        "Stitcher's Graft",
        "If multiple effects say that a creature doesn’t untap during your next untap step, those effects all apply during one untap step. For example, a creature that attacks while equipped with two Stitcher’s Grafts will only spend one untap step without untapping."
    );
    supported("Stitcher's Graft");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Stitcher's Graft", bears);
    attach_new(&mut t, P0, "Stitcher's Graft", bears);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    block_and_finish(&mut t, P1, &[]);
    next_upkeep(&mut t, P0);
    assert!(t.obj_now(bears).tapped);
    next_upkeep(&mut t, P0);
    assert!(!t.obj_now(bears).tapped);
}

#[test]
fn stitchers_graft_becoming_unattached_sacrifices_the_permanent() {
    cr!("301.5c", "704.5n", "603.2");
    ruling!(
        "Stitcher's Graft",
        "Stitcher’s Graft becomes unattached from the creature it’s equipping if you equip it to a new creature, if Stitcher’s Graft leaves the battlefield, if the equipped creature ceases to be a creature, or if Stitcher’s Graft ceases to be an Equipment. (It also becomes unattached if the equipped creature leaves the battlefield, but the triggered ability won’t do anything in that case.)"
    );
    supported("Song of the Dryads");
    for case in 0..4 {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let graft = attach_new(&mut t, P0, "Stitcher's Graft", bears);
        let lion = t.battlefield(P0, "Silvercoat Lion");
        match case {
            // Equipped to a new creature.
            0 => {
                t.lands(P0, "Wastes", 2);
                crate::r_p213_common::equip(&mut t, P0, graft, lion).unwrap();
                t.resolve_all();
                assert!(on(&t, graft, lion));
            }
            // The Graft leaves the battlefield.
            1 => destroy(&mut t, graft),
            // The equipped creature stops being a creature (Song of the Dryads makes it
            // a Forest land).
            2 => {
                attach_new(&mut t, P1, "Song of the Dryads", bears);
                t.g.recompute();
                t.settle();
                assert!(!t.obj_now(bears).chars.is_creature());
                assert_eq!(attached_to(&t, graft), None);
            }
            // The Graft becomes a creature: an Equipment that's also a creature can't
            // equip (March of the Machines).
            _ => {
                t.battlefield(P1, "March of the Machines");
                t.g.recompute();
                t.settle();
                assert!(t.obj_now(graft).chars.is_creature());
                assert_eq!(attached_to(&t, graft), None);
            }
        }
        t.resolve_all();
        assert!(t.in_graveyard(P0, "Grizzly Bears"), "case {case}");
        assert!(t.on_battlefield(lion), "case {case}");
    }
    // The equipped creature leaves: the trigger does nothing to the card in the
    // graveyard (nor to anything else).
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Stitcher's Graft", bears);
    let lion = t.battlefield(P0, "Silvercoat Lion");
    crate::r_s05_common::move_to(&mut t, bears, Zone::Hand(P0));
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.on_battlefield(lion));
}

// ---------------------------------------------------------------------------------------
// Grappling Hook: "Whenever equipped creature attacks, you may have target creature
// block it this turn if able."
// ---------------------------------------------------------------------------------------

#[test]
fn grappling_hook_requirement_does_nothing_if_the_creature_cant_block_it() {
    cr!("509.1c");
    ruling!(
        "Grappling Hook",
        "If you choose to have the targeted creature block the creature that was equipped with Grappling Hook, but it isn’t able to do so as blockers are declared (for example, because the attacking creature has flying and the targeted creature doesn’t), the requirement to block does nothing. The targeted creature is free to block whichever creature its controller chooses, or block no creatures at all."
    );
    supported("Grappling Hook");
    for flying in [false, true] {
        let mut t = TestGame::new(2);
        let attacker = if flying {
            t.battlefield(P0, "Wind Drake")
        } else {
            t.battlefield(P0, "Grizzly Bears")
        };
        attach_new(&mut t, P0, "Grappling Hook", attacker);
        let lion = t.battlefield(P0, "Silvercoat Lion");
        let giant = t.battlefield(P1, "Hill Giant");
        t.answer_targets(P0, &[Entity::Object(giant)]);
        t.answer_yes(P0, true);
        let p1 = Entity::Player(P1);
        attack_with(&mut t, &[(attacker, p1), (lion, p1)]);
        t.resolve_all();
        assert_eq!(legal_blocks(&mut t, P1, &[]), flying);
        assert_eq!(legal_blocks(&mut t, P1, &[(giant, lion)]), flying);
        assert_eq!(legal_blocks(&mut t, P1, &[(giant, attacker)]), !flying);
    }
}

#[test]
fn grappling_hook_the_creature_must_block_the_one_equipped_when_it_triggered() {
    cr!("509.1c", "608.2h");
    ruling!(
        "Grappling Hook",
        "If you choose to have the targeted creature block, the creature it must block is the one that was equipped by Grappling Hook at the time the ability triggered. It doesn’t matter if Grappling Hook leaves the battlefield or somehow becomes attached to another creature."
    );
    for leaves in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let hook = attach_new(&mut t, P0, "Grappling Hook", bears);
        let lion = t.battlefield(P0, "Silvercoat Lion");
        let giant = t.battlefield(P1, "Hill Giant");
        t.answer_targets(P0, &[Entity::Object(giant)]);
        t.answer_yes(P0, true);
        let p1 = Entity::Player(P1);
        attack_with(&mut t, &[(bears, p1), (lion, p1)]);
        if leaves {
            destroy(&mut t, hook);
        } else {
            move_eq(&mut t, hook, lion);
        }
        t.resolve_all();
        assert!(!legal_blocks(&mut t, P1, &[]));
        assert!(!legal_blocks(&mut t, P1, &[(giant, lion)]));
        assert!(legal_blocks(&mut t, P1, &[(giant, bears)]));
    }
}

// ---------------------------------------------------------------------------------------
// Tormentor's Trident: "Equipped creature gets +3/+0 and attacks each combat if able."
// ---------------------------------------------------------------------------------------

#[test]
fn tormentors_trident_doesnt_force_impossible_attacks_or_costs() {
    cr!("508.1d");
    ruling!(
        "Tormentor's Trident",
        "If, during your declare attackers step, the equipped creature is tapped, is affected by a spell or ability that says it can’t attack, or hasn’t been under your control continuously since the turn began (and doesn’t have haste), then it doesn’t attack. If there’s a cost associated with having a creature attack, you’re not forced to pay that cost, so it doesn’t have to attack in that case either."
    );
    supported("Tormentor's Trident");
    // 0: able (control); 1: tapped; 2: Pacifism; 3: summoning sick; 4: Ghostly Prison.
    for case in 0..5 {
        let mut t = TestGame::new(2);
        let bears = if case == 3 {
            t.battlefield_sick(P0, "Grizzly Bears")
        } else {
            t.battlefield(P0, "Grizzly Bears")
        };
        attach_new(&mut t, P0, "Tormentor's Trident", bears);
        match case {
            1 => t.g.objects[bears.0 as usize].tapped = true,
            2 => {
                attach_new(&mut t, P1, "Pacifism", bears);
            }
            4 => {
                t.battlefield(P1, "Ghostly Prison");
                t.lands(P0, "Wastes", 2);
            }
            _ => {}
        }
        to_combat(&mut t, P0);
        assert_eq!(legal_attack(&mut t, &[]), case != 0, "case {case}");
    }
}

// ---------------------------------------------------------------------------------------
// Sword of Forge and Frontier
// ---------------------------------------------------------------------------------------

#[test]
fn sword_of_forge_and_frontier_on_another_players_turn_no_land_play() {
    cr!("305.2", "307.1", "301.5d");
    ruling!(
        "Sword of Forge and Frontier",
        "If the equipped creature somehow deals combat damage to a player on an opponent's turn, the second ability will trigger. You'll be able to cast cards you exile if timing rules allow (so, instants, mostly), but you won't be able to play a land that turn."
    );
    supported("Sword of Forge and Frontier");
    // P0's Sword is on P1's creature, which attacks P2 on P1's turn.
    let mut t = TestGame::new(3);
    t.set_step(P1, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P0, "Sword of Forge and Frontier", bears);
    let top = stack_library(&mut t, P0, &["Forest", "Lightning Bolt"]);
    t.lands(P0, "Mountain", 1);
    attack_with(&mut t, &[(bears, Entity::Player(P2))]);
    block_and_finish(&mut t, P2, &[]);
    t.resolve_all();
    assert_eq!(t.life(P2), 16);
    let forest = t.g.current(top[0]);
    let bolt = t.g.current(top[1]);
    assert_eq!(t.zone(forest), Zone::Exile);
    assert_eq!(t.zone(bolt), Zone::Exile);
    assert!(can_cast(&mut t, P0, bolt, CastMethod::Normal));
    assert!(!can_play_land(&mut t, P0, forest));
}
