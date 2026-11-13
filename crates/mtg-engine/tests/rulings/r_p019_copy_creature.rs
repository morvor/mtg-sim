//! Rulings batch P019 — creatures becoming copies (CR 707.2, 707.4): Lazav, Dimir
//! Mastermind, Permeating Mass, Cytoshape, Dimir Doppelganger and Faerie Artisans; names
//! of face-down creatures (Mirror Box, CR 201.2a); the mana value of Phyrexian hybrid
//! symbols (Nahiri, CR 202.3f-g); and crimes (CR 700.13).

use crate::r_s01_common::{attack_with, block_and_finish, supported, triggers_on_stack};
use crate::r_s02_common::destroy;
use crate::r_s06_common::{activate_containing, damage};
use crate::r_s25_common::cast_new;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn lazav_becoming_a_copy_doesnt_trigger_enters_abilities() {
    cr!("707.4", "603.6a");
    ruling!(
        "Lazav, Dimir Mastermind",
        "\"Enters\" abilities of the creature card Lazav is copying won't trigger as Lazav is already on the battlefield when it becomes a copy of that creature card."
    );
    supported("Lazav, Dimir Mastermind");
    // "Whenever a creature card is put into an opponent's graveyard from anywhere, you may
    // have Lazav become a copy of that card, except its name is Lazav, Dimir Mastermind,
    // ..." Elvish Visionary: "When this creature enters, draw a card."
    let mut t = TestGame::new(2);
    let lazav = t.battlefield(P0, "Lazav, Dimir Mastermind");
    let visionary = t.battlefield(P1, "Elvish Visionary");
    destroy(&mut t, visionary);
    t.answer_yes(P0, true);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.pt(lazav), (1, 1));
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.hand_size(P0), hand);
}

/// P0's Permeating Mass ("Whenever this creature deals combat damage to a creature, that
/// creature becomes a copy of this creature.", 1/3) attacks and P1 blocks with `blocker`.
fn mass_blocked_by(t: &mut TestGame, blocker: ObjectId) -> ObjectId {
    let mass = t.battlefield(P0, "Permeating Mass");
    attack_with(t, &[(mass, Entity::Player(P1))]);
    block_and_finish(t, P1, &[(blocker, mass)]);
    t.resolve_all();
    mass
}

#[test]
fn permeating_mass_copies_survivors_even_if_it_dies() {
    cr!("510.2", "704.5g", "707.2");
    ruling!(
        "Permeating Mass",
        "A creature dealt lethal damage during combat won’t survive long enough to become a copy of Permeating Mass. If Permeating Mass is dealt lethal damage during combat, the creature Permeating Mass dealt damage to still becomes a copy of it."
    );
    supported("Permeating Mass");
    // Llanowar Elves (1/1) dies.
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P1, "Llanowar Elves");
    mass_blocked_by(&mut t, elves);
    assert!(t.in_graveyard(P1, "Llanowar Elves"));
    assert!(t.named_on_battlefield("Permeating Mass").len() == 1);
    // Hill Giant (3/3) kills the Mass and becomes a copy of it.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let mass = mass_blocked_by(&mut t, giant);
    assert!(!t.on_battlefield(mass));
    assert!(t.on_battlefield(giant));
    assert_eq!(t.obj_now(giant).chars.name, "Permeating Mass");
    assert_eq!(t.pt(giant), (1, 3));
}

#[test]
fn a_damaged_creature_becoming_a_copy_of_permeating_mass_may_die() {
    cr!("707.2", "704.5g");
    ruling!(
        "Permeating Mass",
        "A creature with 3 or more damage marked on it that becomes a copy of Permeating Mass will be destroyed immediately after becoming a copy."
    );
    // Craw Wurm (6/4) with 2 damage marked takes 1 more: as a 1/3 copy, it's destroyed.
    let mut t = TestGame::new(2);
    let source = t.battlefield(P0, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    damage(&mut t, source, 2, wurm);
    mass_blocked_by(&mut t, wurm);
    assert!(!t.on_battlefield(wurm));
    assert!(t.in_graveyard(P1, "Craw Wurm"));
}

#[test]
fn cytoshape_can_make_a_creature_a_copy_of_itself() {
    cr!("707.2");
    ruling!(
        "Cytoshape",
        "A creature may become a copy of itself this way. This generally won’t have any visible effect."
    );
    supported("Cytoshape");
    // "Choose a nonlegendary creature on the battlefield. Target creature becomes a copy
    // of that creature until end of turn."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    cast_new(&mut t, P0, "Cytoshape", &[Entity::Object(bears)]);
    t.resolve_all();
    assert_eq!(t.obj_now(bears).chars.name, "Grizzly Bears");
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn face_down_creatures_dont_share_a_name_for_mirror_box() {
    cr!("708.2", "201.2a");
    ruling!(
        "Mirror Box",
        "A face-down creature has no name, so it doesn't have the same name as anything else."
    );
    supported("Mirror Box");
    // Mirror Box: "Each nontoken creature you control gets +1/+1 for each other creature
    // you control with the same name as that creature." Two face-down creatures: no bonus;
    // two Grizzly Bears: +1/+1 each.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mirror Box");
    let a = crate::r_s12_common::morph(&mut t, P0, "Exalted Angel");
    let b = crate::r_s12_common::morph(&mut t, P0, "Exalted Angel");
    assert_eq!(t.pt(a), (2, 2));
    assert_eq!(t.pt(b), (2, 2));
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    t.g.recompute();
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn nahiris_mana_value_counts_the_phyrexian_hybrid_symbol_as_one() {
    cr!("202.3f", "202.3g");
    ruling!(
        "Nahiri, the Unforgiving",
        "A hybrid Phyrexian mana symbol contributes 1 toward the mana value of a card, even if life is paid for it. Specifically, Nahiri's mana value is always 4."
    );
    supported("Nahiri, the Unforgiving");
    // Cast for {1}{R}{W} and 2 life: still mana value 4 (and two fewer loyalty).
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Wastes", 1);
    let card = t.hand(P0, "Nahiri, the Unforgiving");
    assert_eq!(t.g.mana_value_of(card), 4);
    let spell = t.cast(P0, card).go();
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.g.mana_value_of(spell), 4);
    t.resolve_all();
    let nahiri = t.g.current(spell);
    assert!(t.on_battlefield(nahiri));
    assert_eq!(t.g.mana_value_of(nahiri), 4);
    assert_eq!(t.counters(nahiri, mtg_engine::types::counters::LOYALTY), 3);
}

#[test]
fn dimir_doppelganger_copying_sisters_cant_return_cards_its_own_ability_exiled() {
    cr!("607.2a", "707.2");
    ruling!(
        "Dimir Doppelganger",
        "A permanent's ability that refers to cards the creature exiled (such as Sisters of Stone Death's third ability) only affects cards exiled by other abilities intrinsic to that permanent (such as Sisters of Stone Death's second ability). Suppose that (a) Dimir Doppelganger copies Arc-Slogger, (b) its \"deal 2 damage\" ability is activated, and then (c) it copies Sisters of Stone Death. Creatures exiled by Arc-Slogger's ability can't be returned with Sisters of Stone Death's ability."
    );
    supported("Dimir Doppelganger");
    supported("Sisters of Stone Death");
    // "{1}{U}{B}: Exile target creature card from a graveyard. This creature becomes a copy
    // of that card, except it has this ability." It exiles the Sisters card itself; the
    // copied "{2}{B}: Put a creature card exiled with Sisters of Stone Death onto the
    // battlefield under your control." can't return it.
    let mut t = TestGame::new(2);
    let doppel = t.battlefield(P0, "Dimir Doppelganger");
    let sisters = t.graveyard(P1, "Sisters of Stone Death");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Wastes", 1);
    t.answer_targets(P0, &[Entity::Object(sisters)]);
    activate_containing(&mut t, P0, doppel, "Exile target creature card")
        .unwrap()
        .unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(doppel).chars.name, "Sisters of Stone Death");
    assert!(t.in_exile("Sisters of Stone Death"));
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Wastes", 2);
    activate_containing(&mut t, P0, doppel, "exiled with").unwrap();
    t.resolve_all();
    assert!(t.in_exile("Sisters of Stone Death"));
    assert_eq!(t.named_on_battlefield("Sisters of Stone Death").len(), 1);
}

#[test]
fn targeting_several_opponents_things_is_one_crime() {
    cr!("700.13");
    ruling!(
        "Forsaken Miner",
        "A player can commit only one crime per spell or ability they control. Targeting multiple opponents, permanents, spells, abilities, and/or cards with the same spell or ability doesn't constitute committing multiple crimes."
    );
    supported("Forsaken Miner");
    // Forsaken Miner (in P0's graveyard): "Whenever you commit a crime, you may pay {B}.
    // If you do, return this card from your graveyard to the battlefield." Arc Lightning
    // at P1 and two of P1's creatures: one crime, one trigger.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Forsaken Miner");
    let a = t.battlefield(P1, "Llanowar Elves");
    let b = t.battlefield(P1, "Llanowar Elves");
    t.answer(
        P0,
        DecisionKind::Divide,
        mtg_engine::decision::Answer::Numbers(vec![1, 1, 1]),
    );
    t.answer_targets(
        P0,
        &[Entity::Player(P1), Entity::Object(a), Entity::Object(b)],
    );
    crate::r_s25_common::lands_for_cost(&mut t, P0, "Arc Lightning");
    let card = t.hand(P0, "Arc Lightning");
    t.g.turn.priority = Some(P0);
    t.g.cast_spell(P0, card, mtg_engine::object::CastMethod::Normal)
        .unwrap();
    t.g.flush_events();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "commit a crime"), 1);
}

#[test]
fn a_token_copy_of_a_commander_isnt_a_commander() {
    cr!("903.3", "707.2");
    ruling!(
        "Faerie Artisans",
        "A token creature that’s a copy of a player’s commander isn’t a commander."
    );
    // P1's commander, Grizzly Bears, enters: Faerie Artisans makes a token copy of it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Faerie Artisans");
    let commander = t.enter(P1, "Grizzly Bears");
    t.g.objects[commander.0 as usize].is_commander = true;
    t.settle();
    t.resolve_all();
    let tokens = crate::r_s01_common::tokens(&t, P0);
    assert_eq!(tokens.len(), 1);
    let token = tokens[0];
    assert_eq!(t.obj_now(token).chars.name, "Grizzly Bears");
    assert!(t.obj_now(commander).is_commander);
    assert!(!t.obj_now(token).is_commander);
    // It dies as a token: it isn't put into the command zone.
    destroy(&mut t, token);
    t.resolve_all();
    assert_eq!(t.zone(token), Zone::Nowhere);
}
