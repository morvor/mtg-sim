//! Rulings batch P003 — lands that become creatures: what they keep (CR 205.1b, 613.1d),
//! base power and toughness set in layer 7b under counters and modifications (CR 613.4),
//! summoning sickness (CR 302.6), durations (CR 611.2), names (CR 613.1d), tapped status,
//! Equipment that are creatures (CR 301.5c), and resolution order (CR 405.5).

use crate::r_p003_common::*;
use crate::r_s01_common::supported;
use crate::r_s02_common::can_attack;
use crate::r_s04_common::crew;
use crate::r_s20_common::tap_for_mana;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn kw(t: &TestGame, id: ObjectId, k: KeywordKind) -> bool {
    t.obj_now(id).has_keyword(k)
}

fn tapped(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).tapped
}

// --- Fendeep Summoner ------------------------------------------------------------------------

#[test]
fn fendeep_summoner_swamps_keep_their_types_and_colors_and_tap_for_black() {
    cr!("205.1b", "613.1d", "302.6", "115.1d");
    ruling!(
        "Fendeep Summoner",
        "A Swamp affected by this ability is still a land, still a Swamp, and may still be tapped for black mana. If it had any other card types or subtypes, it retains those as well. This ability doesn’t affect the permanent’s colors, if any."
    );
    ruling!(
        "Fendeep Summoner",
        "A Swamp affected by this ability is subject to the “summoning sickness” rule: If its controller hasn’t continuously controlled it since the beginning of their most recent turn, it can’t attack or be tapped for mana (or use any other {T} abilities)."
    );
    ruling!(
        "Fendeep Summoner",
        "You may target zero, one, or two Swamps with this ability."
    );
    supported("Fendeep Summoner");
    // "{T}: Up to two target Swamps each become 3/5 Treefolk Warrior creatures in addition
    // to their other types until end of turn."
    for n in 0..=2usize {
        let mut t = TestGame::new(2);
        let summoner = t.battlefield(P0, "Fendeep Summoner");
        // Watery Grave (Land — Island Swamp), controlled since the turn began, and a
        // Swamp that entered this turn.
        let grave = t.battlefield(P0, "Watery Grave");
        let swamp = t.battlefield_sick(P0, "Swamp");
        let targets: Vec<Entity> = [grave, swamp][..n].iter().map(|x| obj(*x)).collect();
        t.answer_targets(P0, &targets);
        crate::r_s06_common::activate_containing(&mut t, P0, summoner, "Treefolk").unwrap();
        t.resolve_all();
        assert_eq!(is_creature(&t, grave), n >= 1, "{n} targets");
        assert_eq!(is_creature(&t, swamp), n >= 2, "{n} targets");
        if n < 2 {
            continue;
        }
        for land in [grave, swamp] {
            let o = t.obj_now(land);
            assert!(o.is(CardType::Land));
            assert!(o.chars.has_subtype("Swamp"));
            assert!(o.chars.has_subtype("Treefolk") && o.chars.has_subtype("Warrior"));
            assert_eq!(o.chars.colors, ColorSet::default(), "still colorless");
            assert_eq!(t.pt(land), (3, 5));
        }
        assert!(has_subtype(&t, grave, "Island"));
        // The Grave may attack and tap for {B}; the new Swamp may do neither.
        t.set_step(P0, Step::DeclareAttackers);
        assert!(can_attack(&mut t, grave));
        assert!(!can_attack(&mut t, swamp));
        assert!(!tap_for_mana(&mut t, P0, swamp, "{B}"));
        assert!(tap_for_mana(&mut t, P0, grave, "{B}"));
        assert_eq!(t.g.player(P0).mana_pool.count(ManaType::B), 1);
    }
}

// --- Hydroform -------------------------------------------------------------------------------

#[test]
fn hydroform_keeps_name_types_and_abilities_and_modifications_still_apply() {
    cr!("205.1b", "613.1d", "613.4b", "613.4c", "613.4d", "302.6");
    ruling!(
        "Hydroform",
        "Effects that modify power and/or toughness but don’t set them to a specific value (like the one created by Giant Growth), power/toughness changes from counters, and effects that switch a creature’s power and toughness will continue to apply. This may happen if the land was already a creature when Hydroform resolved."
    );
    ruling!(
        "Hydroform",
        "Hydroform doesn’t affect the land’s name or any other types, subtypes, or supertypes (such as basic or legendary) the land may have. The land will also keep any abilities it had."
    );
    ruling!(
        "Hydroform",
        "If the target land hasn’t been under its controller’s control continuously since the beginning of their most recent turn, that land won’t be able to attack and its {T} abilities won’t be able to be activated. In most cases, that means it will no longer be able to be tapped for mana that turn."
    );
    supported("Hydroform");
    // "Target land becomes a 3/3 Elemental creature with flying until end of turn. It's
    // still a land."
    let mut t = TestGame::new(2);
    // Dryad Arbor (a 1/1 Land Creature — Forest Dryad) with a +1/+1 counter and Giant
    // Growth: 5/5.
    let arbor = t.battlefield(P0, "Dryad Arbor");
    put(&mut t, arbor, counters::PLUS1, 1);
    giant_growth(&mut t, arbor);
    assert_eq!(t.pt(arbor), (5, 5));
    cast_resolve(&mut t, P0, "Hydroform", "GU", &[obj(arbor)]);
    assert_eq!(t.pt(arbor), (7, 7));
    switch_pt(&mut t, arbor);
    put(&mut t, arbor, counters::MINUS1, 2);
    assert_eq!(t.pt(arbor), (5, 5));
    assert!(has_subtype(&t, arbor, "Dryad") && has_subtype(&t, arbor, "Elemental"));
    // A basic Island that entered this turn: still a basic Island named Island with its
    // mana ability, a 3/3 flier that can't attack or tap for mana.
    let island = t.battlefield_sick(P0, "Island");
    cast_resolve(&mut t, P0, "Hydroform", "GU", &[obj(island)]);
    let o = t.obj_now(island);
    assert_eq!(o.chars.name, "Island");
    assert!(o.chars.supertypes.contains(Supertype::Basic));
    assert!(o.is(CardType::Land) && o.chars.has_subtype("Island"));
    assert!(o.chars.has_subtype("Elemental"));
    assert!(kw(&t, island, KeywordKind::Flying));
    assert_eq!(t.pt(island), (3, 3));
    assert_eq!(activated_with(&t, island, "{U}"), 1);
    t.set_step(P0, Step::DeclareAttackers);
    assert!(!can_attack(&mut t, island));
    assert!(!tap_for_mana(&mut t, P0, island, "{U}"));
    assert!(can_attack(&mut t, arbor), "control: the Arbor was there all turn");
}

// --- Three +1/+1 counters, a 0/0 Elemental ---------------------------------------------------

#[test]
fn cyclone_sire_and_wall_of_resurgence_set_an_animated_land_to_base_0_0() {
    cr!("613.4b", "613.4c", "613.7a");
    ruling!(
        "Cyclone Sire",
        "If you target a land that’s already a creature with Cyclone Sire’s triggered ability, that land creature’s base power and toughness will become 0/0, overwriting its previous base power and toughness. Other effects that modify its power and/or toughness (including the new and any previously-existing +1/+1 counters) will continue to apply."
    );
    ruling!(
        "Wall of Resurgence",
        "If you target a land that’s already a creature with Wall of Resurgence’s triggered ability, that land creature’s base power and toughness will become 0/0, overwriting its previous base power and toughness. Other effects that modify its power and/or toughness (including the new and any previously-existing +1/+1 counters) will continue to apply."
    );
    supported("Cyclone Sire");
    supported("Wall of Resurgence");
    for name in ["Cyclone Sire", "Wall of Resurgence"] {
        let mut t = TestGame::new(2);
        // Dryad Arbor (1/1) made a 3/3 by Hydroform, with a +1/+1 counter and Giant
        // Growth: 7/7.
        let arbor = t.battlefield(P0, "Dryad Arbor");
        cast_resolve(&mut t, P0, "Hydroform", "GU", &[obj(arbor)]);
        put(&mut t, arbor, counters::PLUS1, 1);
        giant_growth(&mut t, arbor);
        assert_eq!(t.pt(arbor), (7, 7));
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[obj(arbor)]);
        if name == "Cyclone Sire" {
            let sire = add(&mut t, P0, name);
            kill(&mut t, sire);
        } else {
            enter(&mut t, P0, name);
        }
        t.resolve_all();
        assert_eq!(t.counters(arbor, counters::PLUS1), 4, "{name}");
        // Base 0/0, four counters, +3/+3.
        assert_eq!(t.pt(arbor), (7, 7), "{name}");
        assert!(kw(&t, arbor, KeywordKind::Haste), "{name}");
        // Hydroform's flying stays (only its base power and toughness was overwritten).
        assert!(kw(&t, arbor, KeywordKind::Flying), "{name}");
        cast_resolve(&mut t, P0, "Giant Growth", "G", &[obj(arbor)]);
        assert_eq!(t.pt(arbor), (10, 10), "{name}");
    }
}

#[test]
fn noyan_dar_resolves_first_and_isnt_awaken() {
    cr!("405.5", "603.3", "702.113a");
    ruling!(
        "Noyan Dar, Roil Shaper",
        "The ability will resolve before the instant or sorcery spell that caused it to trigger."
    );
    ruling!(
        "Noyan Dar, Roil Shaper",
        "Although Noyan Dar’s ability is very similar to awaken, it is not the same ability. Any card that refers to a “card with awaken” doesn’t include Noyan Dar."
    );
    supported("Noyan Dar, Roil Shaper");
    let mut t = TestGame::new(2);
    let noyan = t.battlefield(P0, "Noyan Dar, Roil Shaper");
    assert!(!kw(&t, noyan, KeywordKind::Awaken));
    let island = t.battlefield(P0, "Island");
    let bolt = t.hand(P0, "Lightning Bolt");
    mana(&mut t, P0, "R");
    let bolt = t.cast(P0, bolt).target(pl(P1)).go();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(island)]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    // The land is a 3/3 creature; the Bolt hasn't resolved yet.
    assert!(is_creature(&t, island));
    assert_eq!(t.pt(island), (3, 3));
    assert!(t.g.stack.contains(&bolt));
    assert_eq!(t.life(P1), 20);
    t.resolve();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn rootwise_survivor_land_remains_a_creature_indefinitely() {
    cr!("611.2b", "611.2a");
    ruling!(
        "Rootwise Survivor",
        "Rootwise Survivor's last ability doesn't have a duration, which means the land remains a creature indefinitely."
    );
    supported("Rootwise Survivor");
    // "Survival — At the beginning of your second main phase, if this creature is tapped,
    // put three +1/+1 counters on up to one target land you control. That land becomes a
    // 0/0 Elemental creature in addition to its other types. It gains haste until your
    // next turn."
    let mut t = TestGame::new(2);
    let survivor = t.battlefield(P0, "Rootwise Survivor");
    let forest = t.battlefield(P0, "Forest");
    t.g.tap(survivor);
    t.answer_targets(P0, &[obj(forest)]);
    t.advance_to(P0, Step::PostcombatMain);
    t.resolve_all();
    assert_eq!(t.pt(forest), (3, 3));
    assert!(kw(&t, forest, KeywordKind::Haste));
    t.advance_to(P1, Step::End);
    assert!(is_creature(&t, forest));
    assert!(kw(&t, forest, KeywordKind::Haste));
    t.advance_to(P0, Step::Upkeep);
    assert!(is_creature(&t, forest), "still a creature on your next turn");
    assert_eq!(t.pt(forest), (3, 3));
    assert!(!kw(&t, forest, KeywordKind::Haste), "the haste has ended");
}

#[test]
fn wakeroot_elemental_untaps_and_overwrites_setting_effects() {
    cr!("613.4b", "613.4c", "613.7a", "611.2b");
    ruling!(
        "Wakeroot Elemental",
        "Wakeroot Elemental’s ability can target a land that’s already a creature or a land that’s already untapped. The ability will untap it if possible and set its base power and toughness to 5/5. Any counters or effects modifying that creature’s power and toughness will continue to apply, and any effects that set the creature’s power and toughness will be overwritten."
    );
    supported("Wakeroot Elemental");
    let mut t = TestGame::new(2);
    let wakeroot = t.battlefield(P0, "Wakeroot Elemental");
    // Dryad Arbor: tapped, made 1/1 by Diminish, with a +1/+1 counter and Giant Growth.
    let arbor = t.battlefield(P0, "Dryad Arbor");
    put(&mut t, arbor, counters::PLUS1, 1);
    giant_growth(&mut t, arbor);
    cast_resolve(&mut t, P0, "Diminish", "U", &[obj(arbor)]);
    assert_eq!(t.pt(arbor), (5, 5));
    t.g.tap(arbor);
    activate_resolve(&mut t, P0, wakeroot, "Untap", "GGGGG", &[obj(arbor)]);
    assert!(!tapped(&t, arbor));
    assert_eq!(t.pt(arbor), (9, 9));
    assert!(kw(&t, arbor, KeywordKind::Haste));
    // An untapped land: it becomes a 5/5.
    let forest = t.battlefield(P0, "Forest");
    activate_resolve(&mut t, P0, wakeroot, "Untap", "GGGGG", &[obj(forest)]);
    assert!(!tapped(&t, forest));
    assert_eq!(t.pt(forest), (5, 5));
    // It lasts as long as the land remains on the battlefield.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.pt(forest), (5, 5));
}

#[test]
fn awakening_of_vitu_ghazi_names_the_land_doesnt_untap_it_and_lasts() {
    cr!("613.1d", "611.2b", "205.4");
    ruling!(
        "Awakening of Vitu-Ghazi",
        "Awakening of Vitu-Ghazi doesn’t untap the land that becomes a creature."
    );
    ruling!(
        "Awakening of Vitu-Ghazi",
        "Awakening of Vitu-Ghazi’s effect lasts indefinitely. It doesn’t wear off during the cleanup step."
    );
    ruling!(
        "Awakening of Vitu-Ghazi",
        "The target land loses the name it had, and its name is just Vitu-Ghazi. It keeps any types and abilities it had."
    );
    supported("Awakening of Vitu-Ghazi");
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    t.g.tap(forest);
    cast_resolve(&mut t, P0, "Awakening of Vitu-Ghazi", "3GG", &[obj(forest)]);
    let o = t.obj_now(forest);
    assert!(o.tapped);
    assert_eq!(o.chars.name, "Vitu-Ghazi");
    assert!(o.chars.supertypes.contains(Supertype::Legendary));
    assert!(o.chars.supertypes.contains(Supertype::Basic));
    assert!(o.is(CardType::Land) && o.chars.has_subtype("Forest"));
    assert!(o.chars.has_subtype("Elemental"));
    assert_eq!(t.pt(forest), (9, 9));
    assert_eq!(activated_with(&t, forest, "{G}"), 1);
    t.advance_to(P1, Step::Upkeep);
    assert!(is_creature(&t, forest));
    assert_eq!(t.obj_now(forest).chars.name, "Vitu-Ghazi");
}

// --- Relive the Past -------------------------------------------------------------------------

#[test]
fn relive_the_past_returns_5_5_elementals_keeping_their_creature_types() {
    cr!("613.1d", "205.3d", "613.4b");
    ruling!(
        "Relive the Past",
        "An artifact creature, land creature, or enchantment creature returned this way will be a 5/5 creature and will be an Elemental in addition to its other creature types."
    );
    supported("Relive the Past");
    let mut t = TestGame::new(2);
    // Ornithopter (Artifact Creature — Thopter, 0/2), Dryad Arbor (Land Creature — Forest
    // Dryad, 1/1), Boon Satyr (Enchantment Creature — Satyr, 4/2).
    let thopter = t.graveyard(P0, "Ornithopter");
    let arbor = t.graveyard(P0, "Dryad Arbor");
    let satyr = t.graveyard(P0, "Boon Satyr");
    cast_resolve(
        &mut t,
        P0,
        "Relive the Past",
        "5GW",
        &[obj(thopter), obj(arbor), obj(satyr)],
    );
    for (id, ty) in [(thopter, "Thopter"), (arbor, "Dryad"), (satyr, "Satyr")] {
        assert!(t.on_battlefield(id), "{ty}");
        assert_eq!(t.pt(id), (5, 5), "{ty}");
        assert!(has_subtype(&t, id, ty) && has_subtype(&t, id, "Elemental"), "{ty}");
    }
    assert!(has_subtype(&t, arbor, "Forest"));
    assert!(t.obj_now(thopter).has_keyword(KeywordKind::Flying));
}

#[test]
fn relive_the_past_equipment_cant_attach_and_later_setters_win_but_crewing_doesnt() {
    cr!("301.5c", "702.6a", "613.7a", "702.122a");
    ruling!(
        "Relive the Past",
        "An Equipment without reconfigure that's also a creature can't be attached to anything. You can activate its equip ability, but it won't become attached."
    );
    ruling!(
        "Relive the Past",
        "If another effect causes one of the returned permanents to become a creature and sets its power and toughness as it does so, that creature will have that power and toughness; it won't be 5/5. Notably, crewing a Vehicle does not set its power and toughness, so a Vehicle will remain a 5/5 creature if you crew it."
    );
    supported("Mightstone's Animation");
    let mut t = TestGame::new(2);
    let splitter = t.graveyard(P0, "Bonesplitter");
    t.answer_targets(P0, &[obj(splitter)]);
    t.answer_targets(P0, &[]);
    t.answer_targets(P0, &[]);
    cast_resolve(&mut t, P0, "Relive the Past", "5GW", &[]);
    let splitter = t.g.current(splitter);
    assert!(is_creature(&t, splitter));
    assert_eq!(t.pt(splitter), (5, 5));
    let bear = t.battlefield(P0, "Grizzly Bears");
    activate_resolve(&mut t, P0, splitter, "Equip", "1", &[obj(bear)]);
    assert_eq!(t.obj_now(splitter).attached_to, None);
    assert_eq!(t.pt(bear), (2, 2));
    // Again with the Copter, crewed: still 5/5. Mightstone's Animation then makes it a 4/4.
    let mut t = TestGame::new(2);
    let copter2 = t.graveyard(P0, "Smuggler's Copter");
    t.answer_targets(P0, &[obj(copter2)]);
    t.answer_targets(P0, &[]);
    t.answer_targets(P0, &[]);
    cast_resolve(&mut t, P0, "Relive the Past", "5GW", &[]);
    let copter2 = t.g.current(copter2);
    assert_eq!(t.pt(copter2), (5, 5));
    let bear = t.battlefield(P0, "Grizzly Bears");
    assert!(crew(&mut t, P0, copter2, &[bear]));
    t.resolve_all();
    assert_eq!(t.pt(copter2), (5, 5));
    cast_resolve(&mut t, P0, "Mightstone's Animation", "3U", &[obj(copter2)]);
    assert_eq!(t.pt(copter2), (4, 4));
}
