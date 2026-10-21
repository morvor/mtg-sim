//! Rulings batch P143 — impulse draw: cards exiled with "you may play that card this
//! turn" (CR 601.3, 305.2a), casting from the graveyard as an Adventure, and abilities
//! that reveal until a card is found.

use crate::r_p107_common::{act, mana};
use crate::r_p119_common::counter_spell;
use crate::r_p143_common::*;
use crate::r_s01_common::with_subtype;
use crate::r_s02_common::{can_cast, can_play_land, destroy};
use crate::r_s06_common::attach_new;
use crate::r_s09_common::declare;
use mtg_engine::decision::Decision;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn exiled(t: &TestGame, id: ObjectId) -> bool {
    is_in(t, id, Zone::Exile)
}

#[test]
fn ardent_dustspeaker_cards_are_playable_only_that_turn() {
    cr!("611.2a", "400.7");
    supported("Ardent Dustspeaker");
    ruling!(
        "Ardent Dustspeaker",
        "Any cards exiled with Ardent Dustspeaker that you don’t play remain exiled. You will not be able to play them on later turns, even if Ardent Dustspeaker’s ability triggers again."
    );
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Ardent Dustspeaker");
    t.graveyard(P0, "Lightning Bolt");
    t.graveyard(P0, "Shock");
    let lib = set_library(&mut t, P0, &["Lightning Bolt", "Island", "Swamp", "Plains", "Forest"]);
    t.answer_yes(P0, true);
    declare(&mut t, P0, &[(d, Entity::Player(P1))]);
    t.resolve_all();
    assert!(exiled(&t, lib[0]) && exiled(&t, lib[1]));
    mana(&mut t, P0, ManaType::R, 1);
    assert!(can_cast(&mut t, P0, lib[0], CastMethod::Normal));
    // P0's next turn: it attacks again (another trigger), but the old cards stay unplayable.
    t.advance_to(P1, Step::PrecombatMain);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.answer_yes(P0, true);
    declare(&mut t, P0, &[(d, Entity::Player(P1))]);
    t.resolve_all();

    assert!(exiled(&t, lib[0]) && exiled(&t, lib[1]));
    mana(&mut t, P0, ManaType::R, 1);
    assert!(!can_cast(&mut t, P0, lib[0], CastMethod::Normal));
    assert!(!can_play_land(&mut t, P0, lib[1]));
    // The newly exiled ones are playable (the Swamp was drawn this turn).
    assert!(is_in(&t, lib[2], Zone::Hand(P0)));
    assert!(exiled(&t, lib[3]));
    t.set_step(P0, Step::PostcombatMain);
    assert!(can_play_land(&mut t, P0, lib[3]));
}

#[test]
fn hellcarver_demon_leaves_uncast_cards_in_exile() {
    cr!("608.2g", "118.9");
    supported("Hellcarver Demon");
    ruling!(
        "Hellcarver Demon",
        "Any cards you can’t cast (because they’re lands or they have no legal targets, perhaps) or choose not to cast simply remain exiled."
    );
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Hellcarver Demon");
    let lib = set_library(
        &mut t,
        P0,
        &["Island", "Memnite", "Giant Growth", "Swamp", "Ornithopter", "Forest", "Plains"],
    );
    // Cast nothing (Giant Growth has no legal target anyway: P0 controls no creature).
    t.answer_choose(P0, &[]);
    declare(&mut t, P0, &[(h, Entity::Player(P1))]);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    for c in &lib[..6] {
        assert!(exiled(&t, *c));
    }
    assert!(!exiled(&t, lib[6]));
    // Later this turn they can't be played.
    t.set_step(P0, Step::PostcombatMain);
    assert!(!can_play_land(&mut t, P0, lib[0]));
    assert!(!can_cast(&mut t, P0, lib[1], CastMethod::Normal));
}

#[test]
fn invasion_of_kaldheim_cards_outlast_the_battle_but_not_the_duration() {
    cr!("610.3", "611.2a");
    supported("Invasion of Kaldheim // Pyre of the World Tree");
    ruling!(
        "Invasion of Kaldheim // Pyre of the World Tree",
        "You may play those cards during the specified duration even if the permanent with the ability leaves the battlefield or you lose control of it."
    );
    let mut t = TestGame::new(2);
    let bolt = t.hand(P0, "Lightning Bolt");
    let isl = t.hand(P0, "Island");
    let inv = t.enter(P0, "Invasion of Kaldheim");
    t.resolve_all();
    assert!(exiled(&t, bolt) && exiled(&t, isl));
    assert_eq!(t.hand_size(P0), 2);
    destroy(&mut t, inv);
    assert!(!t.on_battlefield(inv));
    mana(&mut t, P0, ManaType::R, 1);
    assert!(can_cast(&mut t, P0, bolt, CastMethod::Normal));

    ruling!(
        "Invasion of Kaldheim // Pyre of the World Tree",
        "Any cards you don’t play will remain exiled."
    );
    // Through P0's next turn they're playable; after it, they stay in exile.
    t.advance_to(P1, Step::PrecombatMain);
    t.advance_to(P0, Step::PrecombatMain);
    assert!(can_play_land(&mut t, P0, isl));
    t.advance_to(P1, Step::PrecombatMain);
    assert!(exiled(&t, bolt) && exiled(&t, isl));
    mana(&mut t, P0, ManaType::R, 1);
    assert!(!can_cast(&mut t, P0, bolt, CastMethod::Normal));
}

#[test]
fn bruse_tarl_makes_an_ox_for_a_land_it_doesnt_let_you_play() {
    cr!("305.1");
    supported("Bruse Tarl, Roving Rancher");
    ruling!(
        "Bruse Tarl, Roving Rancher",
        "Bruse Tarl's ability doesn't allow you to play land cards that it exiles."
    );
    let mut t = TestGame::new(2);
    let lib = set_library(&mut t, P0, &["Island", "Memnite"]);
    t.enter(P0, "Bruse Tarl, Roving Rancher");
    t.resolve_all();
    assert!(exiled(&t, lib[0]));
    assert_eq!(with_subtype(&t, P0, "Ox").len(), 1);
    t.set_step(P0, Step::PrecombatMain);
    assert!(!can_play_land(&mut t, P0, lib[0]));
}

#[test]
fn laelia_exiles_face_up() {
    cr!("406.3");
    supported("Laelia, the Blade Reforged");
    ruling!(
        "Laelia, the Blade Reforged",
        "Cards exiled by Laelia, the Blade Reforged are exiled face up."
    );
    let mut t = TestGame::new(2);
    let l = t.battlefield(P0, "Laelia, the Blade Reforged");
    let lib = set_library(&mut t, P0, &["Memnite", "Island"]);
    declare(&mut t, P0, &[(l, Entity::Player(P1))]);
    t.resolve_all();
    assert!(exiled(&t, lib[0]));
    assert!(!t.obj_now(lib[0]).face_down);
    assert_eq!(t.obj_now(lib[0]).chars.name.as_str(), "Memnite");
    // (And her last ability saw it: a +1/+1 counter.)
    assert_eq!(t.counters(l, mtg_engine::types::counters::PLUS1), 1);
}

#[test]
fn caves_of_chaos_adventurer_checks_the_dungeon_as_it_exiles() {
    cr!("608.2h", "309.7");
    supported("Caves of Chaos Adventurer");
    ruling!(
        "Caves of Chaos Adventurer",
        "Caves of Chaos Adventurer's last ability checks to see if you've completed a dungeon at the time you exile a card. If you haven't completed a dungeon at that time, you can't cast it without paying its mana cost, even if you complete a dungeon later during the same turn."
    );
    // Not completed when the card is exiled; completed afterwards.
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Caves of Chaos Adventurer");
    let lib = set_library(&mut t, P0, &["Lightning Bolt", "Island"]);
    declare(&mut t, P0, &[(c, Entity::Player(P1))]);
    t.resolve_all();
    assert!(exiled(&t, lib[0]));
    t.g.players[0].dungeons_completed = 1;
    assert!(!can_cast(&mut t, P0, lib[0], CastMethod::Free));
    assert!(!can_cast(&mut t, P0, lib[0], CastMethod::Normal));
    mana(&mut t, P0, ManaType::R, 1);
    assert!(can_cast(&mut t, P0, lib[0], CastMethod::Normal));
    // Completed when it's exiled: it may be cast without paying its mana cost.
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Caves of Chaos Adventurer");
    t.g.players[0].dungeons_completed = 1;
    let lib = set_library(&mut t, P0, &["Lightning Bolt", "Island"]);
    declare(&mut t, P0, &[(c, Entity::Player(P1))]);
    t.resolve_all();
    assert!(can_cast(&mut t, P0, lib[0], CastMethod::Free));
    ruling!(
        "Caves of Chaos Adventurer",
        "If you have completed a dungeon when Caves of Chaos Adventurer's last ability resolves, you can only play the exiled card without paying its mana cost. You can't choose to play it normally."
    );
    mana(&mut t, P0, ManaType::R, 1);
    assert!(!can_cast(&mut t, P0, lib[0], CastMethod::Normal));
}

#[test]
fn heroic_trigger_resolves_even_if_the_spell_is_countered() {
    cr!("603.3");
    supported("Cleon, Merry Champion");
    ruling!(
        "Cleon, Merry Champion",
        "Cleon's second ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack without resolving."
    );
    let mut t = TestGame::new(2);
    let cleon = t.battlefield(P0, "Cleon, Merry Champion");
    let lib = set_library(&mut t, P0, &["Memnite", "Island"]);
    let gg = cast_new(&mut t, P0, "Giant Growth", &[obj(cleon)]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    counter_spell(&mut t, gg);
    t.resolve_all();
    assert!(exiled(&t, lib[0]));
    assert_eq!(t.pt(cleon), (2, 2));
}

#[test]
fn dark_dweller_oracle_follows_normal_timing() {
    cr!("305.2", "307.1", "601.3");
    supported("Dark-Dweller Oracle");
    ruling!(
        "Dark-Dweller Oracle",
        "Dark-Dweller Oracle doesn't change when you can cast the exiled card. For example, if you exile a sorcery card, you can cast it only during your main phase when the stack is empty. If you exile a land card, you can play it only during your main phase and only if you have an available land play remaining."
    );
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Dark-Dweller Oracle");
    let fodder = [t.battlefield(P0, "Grizzly Bears"), t.battlefield(P0, "Memnite")];
    let lib = set_library(&mut t, P0, &["Lava Spike", "Island", "Swamp"]);
    t.set_step(P0, Step::BeginningOfCombat);
    for f in fodder {
        mana(&mut t, P0, ManaType::C, 1);
        t.answer_choose(P0, &[obj(f)]);
        act(&mut t, P0, d, "Exile the top card", &[]).unwrap();
        t.resolve_all();
    }
    assert!(exiled(&t, lib[0]) && exiled(&t, lib[1]));
    mana(&mut t, P0, ManaType::R, 1);
    // During combat: neither the sorcery nor the land.
    assert!(!can_cast(&mut t, P0, lib[0], CastMethod::Normal));
    assert!(!can_play_land(&mut t, P0, lib[1]));
    // In a main phase with an empty stack: both — the land only with a land play left.
    t.set_step(P0, Step::PostcombatMain);
    mana(&mut t, P0, ManaType::R, 1);
    assert!(can_cast(&mut t, P0, lib[0], CastMethod::Normal));
    assert!(can_play_land(&mut t, P0, lib[1]));
    t.g.players[0].lands_played_this_turn = 1;
    assert!(!can_play_land(&mut t, P0, lib[1]));
}

#[test]
fn flame_channeler_counts_damage_from_spells_on_the_stack() {
    cr!("120.2", "608.2");
    supported("Flame Channeler // Embodiment of Flame");
    ruling!(
        "Flame Channeler // Embodiment of Flame",
        "\"A spell you control\" means a spell on the stack. Usually, spells on the stack can deal damage only as part of their own resolution."
    );
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Flame Channeler");
    cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.obj_now(f).chars.name.as_str(), "Embodiment of Flame");

    ruling!(
        "Flame Channeler // Embodiment of Flame",
        "A spell that becomes a permanent and then deals damage, such as one that has a triggered ability when it enters the battlefield, will not cause Flame Channeler's abilities to trigger."
    );
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Flame Channeler");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    cast_new(&mut t, P0, "Flametongue Kavu", &[]);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.obj_now(f).chars.name.as_str(), "Flame Channeler");
}

#[test]
fn dread_whispers_from_the_graveyard_follows_timing_and_costs() {
    cr!("715.3", "601.3", "307.1");
    supported("Mosswood Dreadknight // Dread Whispers");
    ruling!(
        "Mosswood Dreadknight // Dread Whispers",
        "You must follow the normal timing permissions and restrictions for Dread Whispers when casting it with the permission of Mosswood Dreadknight's triggered ability. You must pay its mana cost"
    );
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Mosswood Dreadknight");
    t.set_step(P0, Step::BeginningOfCombat);
    destroy(&mut t, k);
    t.resolve_all();
    let card = t.g.current(k);
    assert_eq!(t.zone(card), Zone::Graveyard(P0));
    let adventure = CastMethod::Half(1);
    // Combat: no (it's a sorcery), even with mana.
    mana(&mut t, P0, ManaType::B, 2);
    assert!(!can_cast(&mut t, P0, card, adventure.clone()));
    // Main phase without mana: no; with mana: yes.
    t.set_step(P0, Step::PostcombatMain);
    t.g.players[0].mana_pool = Default::default();
    assert!(!can_cast(&mut t, P0, card, adventure.clone()));
    mana(&mut t, P0, ManaType::B, 2);
    assert!(can_cast(&mut t, P0, card, adventure.clone()));
    t.cast(P0, card).method(adventure.clone()).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.life(P0), 19);
}

#[test]
fn divining_witch_names_the_card_on_resolution() {
    cr!("608.2", "201.3");
    supported("Divining Witch");
    ruling!("Divining Witch", "You name a card during resolution of the ability.");
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Divining Witch");
    t.hand(P0, "Island");
    mana(&mut t, P0, ManaType::B, 2);
    let from = n_asked(&t);
    act(&mut t, P0, w, "Choose a card name", &[]).unwrap();
    assert!(!asked_of(&t, P0, from)
        .iter()
        .any(|d| matches!(d, Decision::NameCard { .. })));
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(asked_of(&t, P0, from)
        .iter()
        .any(|d| matches!(d, Decision::NameCard { .. })));
}

#[test]
fn heirloom_blade_finds_a_card_sharing_any_creature_type() {
    cr!("205.3m");
    supported("Heirloom Blade");
    ruling!(
        "Heirloom Blade",
        "You'll reveal cards until you find one that shares at least one creature type with the creature that died."
    );
    let mut t = TestGame::new(2);
    // Merfolk Thaumaturgist is a Merfolk Wizard; Prodigal Sorcerer is a Human Wizard.
    let m = t.battlefield(P0, "Merfolk Thaumaturgist");
    attach_new(&mut t, P0, "Heirloom Blade", obj(m));
    let lib = set_library(
        &mut t,
        P0,
        &["Grizzly Bears", "Island", "Prodigal Sorcerer", "Memnite"],
    );
    t.answer_yes(P0, true);
    destroy(&mut t, m);
    t.resolve_all();
    assert!(is_in(&t, lib[2], Zone::Hand(P0)));
    assert_eq!(library_top_first(&t, P0)[0], lib[3]);
    assert_eq!(t.library_size(P0), 3);
}

#[test]
fn svella_puts_the_rest_on_the_bottom_before_the_spell_resolves() {
    cr!("608.2g", "118.9");
    supported("Svella, Ice Shaper");
    ruling!(
        "Svella, Ice Shaper",
        "You’ll put the cards you didn’t cast on the bottom of your library before the spell you cast resolves."
    );
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Svella, Ice Shaper");
    let lib = set_library(
        &mut t,
        P0,
        &["Island", "Lightning Bolt", "Swamp", "Memnite", "Plains"],
    );
    mana(&mut t, P0, ManaType::C, 6);
    mana(&mut t, P0, ManaType::R, 1);
    mana(&mut t, P0, ManaType::G, 1);
    t.answer_choose(P0, &[obj(lib[1])]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    act(&mut t, P0, s, "Look at the top four", &[]).unwrap();
    t.resolve();
    // The Bolt is on the stack; the other three are already on the bottom.
    assert!(is_in(&t, lib[1], Zone::Stack));
    assert_eq!(library_top_first(&t, P0)[0], lib[4]);
    assert_eq!(t.life(P1), 20);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}
