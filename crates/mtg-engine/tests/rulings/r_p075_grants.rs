//! Rulings batch P075 — spells and abilities that grant reach, rebound, shroud, evasion,
//! trample and tap abilities: affected sets fixed on resolution (CR 611.2c), putting a land
//! onto the battlefield isn't playing it, rebound needing its source as the spell finishes
//! resolving, shroud and Auras, timestamps against later flying grants, "until your next
//! turn" when its controller leaves the game (CR 800.4m), type-changing before
//! ability-removing (CR 613.1d/f), and granted tap abilities.

use crate::r_p075_common::*;
use crate::r_s17_common::token_copy;
use crate::r_s21_common::legal_blocks;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Whether the permanent has an activated ability whose text contains `needle` (ignoring
/// case: granted abilities quoted in Oracle text keep their normalized lowercase text).
fn has_ability(t: &mut TestGame, id: ObjectId, needle: &str) -> bool {
    t.g.recompute();
    t.obj_now(id).chars.abilities.iter().any(|a| {
        matches!(a.kind, mtg_engine::ability::AbilityKind::Activated(_))
            && a.text.to_lowercase().contains(&needle.to_lowercase())
    })
}

#[test]
fn atarkas_command_land_isnt_played_and_earlier_life_stays() {
    cr!("305.4", "305.2");
    ruling!(
        "Atarka's Command",
        "If you choose the third mode and put a land card onto the battlefield, that doesn't count as playing a land."
    );
    ruling!(
        "Atarka's Command",
        "The first mode won't affect life that was gained earlier in the turn."
    );
    supported("Atarka's Command");
    let mut t = TestGame::new(2);
    gain(&mut t, P1, 3);
    let forest = t.hand(P0, "Forest");
    let mountain = t.hand(P0, "Mountain");
    t.answer_choose(P0, &[obj(forest)]);
    t.answer_yes(P0, true);
    give_mana_for(&mut t, P0, "Atarka's Command");
    let c = t.hand(P0, "Atarka's Command");
    t.cast(P0, c).modes(&[0, 2]).go();
    t.resolve_all();
    assert_eq!(
        t.named_on_battlefield("Forest").len(),
        2,
        "one put onto the battlefield"
    );
    assert!(
        t.play_land(P0, mountain).is_ok(),
        "the land play is still available"
    );
    assert_eq!(t.life(P1), 23, "life gained earlier stays");
    gain(&mut t, P1, 3);
    assert_eq!(t.life(P1), 23, "can't gain life now");
}

#[test]
fn group_grants_affect_only_creatures_controlled_on_resolution() {
    cr!("611.2c");
    ruling!(
        "Tower Defense",
        "Only creatures you control when Tower Defense resolves will get the bonuses."
    );
    ruling!(
        "Crash Through",
        "Crash Through affects only creatures you control at the time it resolves."
    );
    for (name, kw) in [
        ("Tower Defense", KeywordKind::Reach),
        ("Crash Through", KeywordKind::Trample),
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        let old = t.battlefield(P0, "Grizzly Bears");
        cast_card(&mut t, P0, name, &[]);
        t.resolve_all();
        let late = t.battlefield(P0, "Grizzly Bears");
        assert!(has(&t, old, kw), "{name}");
        assert!(!has(&t, late, kw), "{name}");
    }
}

#[test]
fn steady_aim_can_target_an_untapped_creature() {
    cr!("701.26b", "115.1");
    ruling!(
        "Steady Aim",
        "Steady Aim can target a creature that’s already untapped. It will still get +1/+4 and gain reach."
    );
    supported("Steady Aim");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_card(&mut t, P0, "Steady Aim", &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 6));
    assert!(has(&t, bears, KeywordKind::Reach));
}

#[test]
fn cast_through_time_must_be_there_as_the_spell_finishes_resolving() {
    cr!("702.88a", "608.2n");
    ruling!(
        "Cast Through Time",
        "For the rebound effect to happen, Cast Through Time needs to be on the battlefield as the spell _finishes_ resolving."
    );
    supported("Cast Through Time");
    // Replenish returns it to the battlefield: Replenish rebounds.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Cast Through Time");
    cast_card(&mut t, P0, "Replenish", &[]);
    t.resolve_all();
    assert!(!t.named_on_battlefield("Cast Through Time").is_empty());
    assert!(t.in_exile("Replenish"));
    // Disenchant destroys it: Disenchant doesn't rebound.
    let mut t = TestGame::new(2);
    let ctt = t.battlefield(P0, "Cast Through Time");
    cast_card(&mut t, P0, "Disenchant", &[obj(ctt)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Disenchant"));
    assert!(!t.in_exile("Disenchant"));
}

#[test]
fn rebound_doesnt_apply_to_a_spell_that_moves_itself() {
    cr!("702.88a", "608.2n");
    ruling!(
        "Cast Through Time",
        "If a spell moves itself into another zone as part of its resolution (as Arc Blade, All Suns’ Dawn, and Beacon of Unrest do), rebound won’t get a chance to apply."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cast Through Time");
    let bears = t.graveyard(P1, "Grizzly Bears");
    let lib = t.library_size(P0);
    cast_card(&mut t, P0, "Beacon of Unrest", &[obj(bears)]);
    t.resolve_all();
    assert!(!t.in_exile("Beacon of Unrest"));
    assert_eq!(t.library_size(P0), lib + 1, "shuffled into the library");
}

#[test]
fn shroud_doesnt_remove_auras_or_stop_their_abilities() {
    cr!("702.18a");
    ruling!(
        "Svyelunite Priest",
        "Does not cause Auras on the creature to be removed when the ability is activated."
    );
    ruling!(
        "Svyelunite Priest",
        "Auras which confer an ability (such as Lance or Firebreathing) are not prevented."
    );
    ruling!(
        "Spectral Cloak",
        "Does not destroy Auras already on the creature."
    );
    supported("Svyelunite Priest");
    supported("Spectral Cloak");
    for cloak in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let fb = t.battlefield(P0, "Firebreathing");
        assert!(t.g.attach(fb, obj(bears)));
        if cloak {
            let c = t.battlefield(P0, "Spectral Cloak");
            assert!(t.g.attach(c, obj(bears)));
        } else {
            let p = t.battlefield(P0, "Svyelunite Priest");
            t.set_step(P0, Step::Upkeep);
            t.lands(P0, "Island", 2);
            t.activate(P0, p, 0, &[obj(bears)]).expect("activates");
            t.resolve_all();
        }
        t.g.recompute();
        t.settle();
        assert!(has(&t, bears, KeywordKind::Shroud), "cloak={cloak}");
        assert_eq!(t.obj_now(fb).attached_to, Some(obj(bears)), "cloak={cloak}");
        t.lands(P0, "Mountain", 1);
        t.activate(P0, fb, 0, &[])
            .expect("Firebreathing still works");
        t.resolve_all();
        assert_eq!(t.pt(bears).0, 3, "cloak={cloak}");
    }
}

#[test]
fn familiar_ground_with_menace_means_unblockable() {
    cr!("702.111b", "509.1b");
    ruling!("Familiar Ground", "none of your creatures can be blocked");
    supported("Familiar Ground");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Familiar Ground");
    let bears = t.battlefield(P0, "Grizzly Bears");
    grant(&mut t, bears, KeywordKind::Menace);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[bears]));
    assert!(!legal_blocks(&mut t, P1, &[(a, bears)]));
    assert!(!legal_blocks(&mut t, P1, &[(a, bears), (b, bears)]));
}

#[test]
fn nighthaze_illegal_target_no_draw() {
    cr!("608.2b");
    ruling!(
        "Nighthaze",
        "If the targeted creature is an illegal target by the time the ability resolves, the ability doesn’t resolve. You won’t draw a card."
    );
    supported("Nighthaze");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_card(&mut t, P0, "Nighthaze", &[obj(bears)]);
    let hand = t.hand_size(P0);
    exile_now(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn mu_yanling_loses_flying_until_a_later_grant() {
    cr!("613.7", "613.1f");
    ruling!(
        "Mu Yanling, Sky Dancer",
        "After Mu Yanling's first ability has resolved, the target creature will have flying again if another effect grants it flying."
    );
    ruling!(
        "Mu Yanling, Sky Dancer",
        "Mu Yanling's first ability can target a creature that doesn't have flying. The creature just gets -2/-0."
    );
    supported("Mu Yanling, Sky Dancer");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Mu Yanling, Sky Dancer");
    let angel = t.battlefield(P1, "Serra Angel");
    t.activate(P0, m, 0, &[obj(angel)]).expect("+2");
    t.resolve_all();
    assert!(!has(&t, angel, KeywordKind::Flying));
    assert_eq!(t.pt(angel), (2, 4));
    cast_card(&mut t, P0, "Jump", &[obj(angel)]);
    t.resolve_all();
    assert!(has(&t, angel, KeywordKind::Flying), "a later grant wins");
    // A creature without flying: just -2/-0.
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Mu Yanling, Sky Dancer");
    let giant = t.battlefield(P1, "Hill Giant");
    t.activate(P0, m, 0, &[obj(giant)]).expect("+2");
    t.resolve_all();
    assert_eq!(t.pt(giant), (1, 3));
}

#[test]
fn mu_yanling_effect_outlasts_its_controller_leaving() {
    cr!("800.4m", "611.2b");
    ruling!(
        "Mu Yanling, Sky Dancer",
        "its effect lasts until your next turn would have begun. It neither expires immediately nor lasts indefinitely."
    );
    let mut t = TestGame::new(3);
    let m = t.battlefield(P0, "Mu Yanling, Sky Dancer");
    let angel = t.battlefield(P2, "Serra Angel");
    t.activate(P0, m, 0, &[obj(angel)]).expect("+2");
    t.resolve_all();
    t.g.lose_game(P0);
    t.settle();
    t.advance_to(P1, Step::PrecombatMain);
    assert_eq!(t.pt(angel), (2, 4), "doesn't expire immediately");
    t.advance_to(P2, Step::PrecombatMain);
    assert_eq!(t.pt(angel), (2, 4));
    assert!(!has(&t, angel, KeywordKind::Flying));
    t.advance_to(P1, Step::PrecombatMain);
    assert_eq!(t.pt(angel), (4, 4), "ended as P0's turn would have begun");
    assert!(has(&t, angel, KeywordKind::Flying));
}

#[test]
fn mu_yanling_emblem_islands_by_subtype_and_one_tap() {
    cr!("205.3i", "602.5b", "114.4");
    ruling!(
        "Mu Yanling, Sky Dancer",
        "An Island is a land with the subtype Island, not any land with a mana ability that produces blue mana."
    );
    ruling!(
        "Mu Yanling, Sky Dancer",
        "You can't activate that ability and also activate the ability it gains from Mu Yanling's emblem."
    );
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Mu Yanling, Sky Dancer");
    t.g.objects[m.0 as usize]
        .counters
        .insert("loyalty".into(), 8);
    let trop = t.battlefield(P0, "Tropical Island");
    let aqueduct = t.battlefield(P0, "Dimir Aqueduct");
    t.activate(P0, m, 2, &[]).expect("-8");
    t.resolve_all();
    assert!(has_ability(&mut t, trop, "Draw a card"));
    assert!(!has_ability(&mut t, aqueduct, "Draw a card"));
    assert!(tap_for_mana(&mut t, P0, trop, "Add"));
    assert!(!can_activate(&mut t, P0, trop), "already tapped for mana");
}

#[test]
fn joiner_adept_and_great_divide_guide_grant_mana_abilities() {
    cr!("613.1f", "605.1a");
    ruling!("Joiner Adept", "Each land you control gets this ability.");
    ruling!(
        "Great Divide Guide",
        "The Great Divide Guide is an Ally and gives its granted ability to itself."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Joiner Adept");
    let w = t.battlefield(P0, "Wastes");
    assert!(tap_for_mana(&mut t, P0, w, "any color"));
    supported("Great Divide Guide");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Great Divide Guide");
    assert!(tap_for_mana(&mut t, P0, g, "any color"));
}

#[test]
fn toxicrene_type_changes_apply_before_abilities_are_removed() {
    cr!("613.1d", "613.1f", "613.8");
    ruling!(
        "Toxicrene",
        "type-changing effects are always applied before any effects that add or remove abilities"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Toxicrene");
    let a = t.battlefield(P0, "Ashaya, Soul of the Wild");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.recompute();
    for id in [a, bears] {
        let o = t.obj_now(id);
        assert!(o.is(CardType::Land) && o.chars.has_subtype("Forest"));
        let acts: Vec<_> = o
            .chars
            .abilities
            .iter()
            .filter(|x| !matches!(x.kind, mtg_engine::ability::AbilityKind::Keyword(_)))
            .map(|x| x.text.to_string())
            .collect();
        assert_eq!(acts.len(), 1, "{acts:?}");
        assert!(acts[0].contains("any color"), "{acts:?}");
    }
}

#[test]
fn insidious_roots_one_trigger_for_cards_leaving_together() {
    cr!("603.2c", "603.10a");
    ruling!(
        "Insidious Roots",
        "If multiple creature cards leave your graveyard at the same time, Insidious Roots's last ability will trigger only once."
    );
    supported("Insidious Roots");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Insidious Roots");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Hill Giant");
    let crypt = t.battlefield(P0, "Tormod's Crypt");
    t.activate(P0, crypt, 0, &[Entity::Player(P0)])
        .expect("activates");
    t.resolve_all();
    let plants = tokens(&t, P0);
    assert_eq!(plants.len(), 1);
    assert_eq!(plus(&t, plants[0]), 1);
}

#[test]
fn insidious_roots_creature_token_grants_itself_the_ability() {
    cr!("613.1f", "111.1");
    ruling!(
        "Insidious Roots",
        "In the rare case where Insidious Roots is a creature token, it will grant itself the ability"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Opalescence");
    let roots = t.battlefield(P0, "Insidious Roots");
    let tok = token_copy(&mut t, P0, roots)[0];
    t.g.recompute();
    assert!(t.obj_now(tok).is(CardType::Creature));
    assert!(has_ability(&mut t, tok, "any color"));
    assert!(!has_ability(&mut t, roots, "any color"), "not a token");
}

#[test]
fn fire_whip_tap_ability_taps_the_creature() {
    cr!("602.2b", "601.2h");
    ruling!(
        "Fire Whip",
        "if you have two Fire Whips on a creature activating the first one will tap the creature, so you can't use the second one"
    );
    supported("Fire Whip");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    for _ in 0..2 {
        let w = t.battlefield(P0, "Fire Whip");
        assert!(t.g.attach(w, obj(bears)));
    }
    t.g.recompute();
    let n = t
        .obj_now(bears)
        .chars
        .abilities
        .iter()
        .filter(|a| a.text.contains("deals 1 damage"))
        .count();
    assert_eq!(n, 2);
    t.activate(P0, bears, 0, &[Entity::Player(P1)])
        .expect("first");
    assert!(tapped(&t, bears));
    assert!(t.activate(P0, bears, 1, &[Entity::Player(P1)]).is_err());
}

#[test]
fn presence_of_gond_the_creatures_controller_activates() {
    cr!("602.2", "113.8");
    ruling!(
        "Presence of Gond",
        "The creature's controller, not Presence of Gond's controller, is the one who can activate the ability"
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let p = t.battlefield(P0, "Presence of Gond");
    assert!(t.g.attach(p, obj(bears)));
    t.g.recompute();
    assert!(t.activate(P0, bears, 0, &[]).is_err());
    t.activate(P1, bears, 0, &[]).expect("P1 activates");
    t.resolve_all();
    assert_eq!(tokens(&t, P1).len(), 1);
    assert!(tokens(&t, P0).is_empty());
}

#[test]
fn petrified_hamlet_names_a_land_card() {
    cr!("201.4", "201.4a", "602.5b");
    ruling!(
        "Petrified Hamlet",
        "You can choose the name of any land card. You can't choose the name of a token unless it's also the name of a card."
    );
    let mut t = TestGame::new(2);
    let factory = t.battlefield(P1, "Mishra's Factory");
    t.answer(
        P0,
        DecisionKind::Name,
        Answer::Text("Mishra's Factory".into()),
    );
    t.enter(P0, "Petrified Hamlet");
    t.resolve_all();
    t.lands(P1, "Wastes", 1);
    assert!(
        act(&mut t, P1, factory, "becomes", &[]).is_err(),
        "its non-mana ability can't be activated"
    );
    // A token's name that isn't a card's name can't be chosen.
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Name, Answer::Text("Treasure".into()));
    let h = t.enter(P0, "Petrified Hamlet");
    t.resolve_all();
    let chosen = t.obj_now(h).choices.card_name.clone();
    // The invalid answer names nothing: the choice stays undefined (CR 607.5a).
    assert!(chosen.as_deref().map_or(true, str::is_empty), "{chosen:?}");
    // A land card's name is accepted.
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Name, Answer::Text("Forest".into()));
    let h = t.enter(P0, "Petrified Hamlet");
    t.resolve_all();
    assert_eq!(t.obj_now(h).choices.card_name.as_deref(), Some("Forest"));
}

#[test]
fn call_of_the_full_moon_one_player_two_spells_last_turn() {
    cr!("603.4", "503.1a");
    ruling!(
        "Call of the Full Moon",
        "A single player must have cast two or more spells during the previous turn"
    );
    ruling!(
        "Call of the Full Moon",
        "Call of the Full Moon's last ability will look at the entire previous turn, even if Call of the Moon wasn't on the battlefield for some or all of that turn."
    );
    supported("Call of the Full Moon");
    // P0 casts Call of the Full Moon and another spell: sacrificed in the next upkeep.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_card(&mut t, P0, "Call of the Full Moon", &[obj(bears)]);
    t.resolve_all();
    cast_card(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.resolve_all();
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Call of the Full Moon"));
    // One spell each: no trigger.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_card(&mut t, P0, "Call of the Full Moon", &[obj(bears)]);
    t.resolve_all();
    cast_card(&mut t, P1, "Shock", &[Entity::Player(P0)]);
    t.resolve_all();
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    t.resolve_all();
    assert!(!t.named_on_battlefield("Call of the Full Moon").is_empty());
}

#[test]
fn rile_draw_happens_before_the_creature_dies() {
    cr!("608.2c", "704.3");
    ruling!(
        "Rile",
        "that creature won't be destroyed until after you've drawn a card. Its abilities may affect that draw or trigger on that draw"
    );
    supported("Rile");
    let mut t = TestGame::new(2);
    let skulker = t.battlefield(P0, "Chasm Skulker");
    cast_card(&mut t, P0, "Rile", &[obj(skulker)]);
    t.resolve();
    assert!(!t.on_battlefield(skulker));
    assert_eq!(
        triggers_on_stack(&t, "Whenever you draw a card"),
        1,
        "its draw trigger triggered"
    );
}
