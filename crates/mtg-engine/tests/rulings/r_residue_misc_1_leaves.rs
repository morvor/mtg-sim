//! Item `rulings-residue-misc-1`, pass B — rulings about permanents entering and leaving,
//! life and drawing (batches P073, P076, P092, P094): leaves-the-battlefield triggers and
//! the delayed triggers left behind, linked exiles of spells that don't resolve, entering
//! with counters from a graveyard, "drawn this turn", replacement draws, life payments and
//! targets that become illegal.

use crate::r_residue_misc_1_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn aid_from_the_cowl_a_permanent_put_onto_the_battlefield_misses_this_end_step() {
    cr!("603.2", "513.1a");
    ruling!(
        "Aid from the Cowl",
        "If a permanent put onto the battlefield this way has an ability that triggers at the beginning of your end step, it won't trigger during this end step."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Aid from the Cowl");
    let bears = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, bears);
    // Below the second Aid from the Cowl, a permanent card it would reveal.
    let next = t.library_top(P0, "Hill Giant");
    let second = t.library_top(P0, "Aid from the Cowl");
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    through_end_step(&mut t, P0);
    assert!(t.on_battlefield(t.g.current(second)));
    assert_eq!(t.zone(t.g.current(next)), Zone::Library(P0));
}

#[test]
fn elemental_expressionist_a_token_is_exiled_before_it_ceases_to_exist() {
    cr!("111.7", "614.1a", "603.6c");
    ruling!(
        "Elemental Expressionist",
        "If you target a creature token with Elemental Expressionist's ability, and that token would leave the battlefield, it is still exiled (very briefly) before it ceases to exist, and you will still create an Elemental token."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Elemental Expressionist");
    let soldier = create_token(&mut t, P0, "Soldier");
    lands_for_cost(&mut t, P0, "Lightning Bolt");
    let bolt = t.hand(P0, "Lightning Bolt");
    t.answer_targets(P0, &[P1.into()]);
    t.answer_targets(P0, &[obj(soldier)]);
    t.cast(P0, bolt).go();
    t.resolve_all();
    destroy(&mut t, soldier);
    t.resolve_all();
    assert!(!t.on_battlefield(soldier));
    assert_eq!(tokens_of(&t, P0, "Elemental"), 1);
}

#[test]
fn three_tree_scribe_sees_any_zone_but_the_graveyard() {
    cr!("603.6c", "603.10a");
    ruling!(
        "Three Tree Scribe",
        "Three Tree Scribe’s ability triggers whenever a creature you control goes to any zone other than the graveyard from the battlefield."
    );
    let mut t = TestGame::new(2);
    let scribe = t.battlefield(P0, "Three Tree Scribe");
    let bears = t.battlefield(P0, "Grizzly Bears");
    lands_for_cost(&mut t, P0, "Unsummon");
    let unsummon = t.hand(P0, "Unsummon");
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_targets(P0, &[obj(scribe)]);
    t.cast(P0, unsummon).go();
    t.resolve_all();
    assert_eq!(p1(&t, scribe), 1);
    // Dying doesn't count.
    let giant = t.battlefield(P0, "Hill Giant");
    destroy(&mut t, giant);
    t.resolve_all();
    assert_eq!(p1(&t, scribe), 1);
}

#[test]
fn xiras_delayed_trigger_works_without_xira() {
    cr!("603.7", "603.7c");
    ruling!(
        "Xira, the Golden Sting",
        "The delayed triggered ability created as Xira's ability resolves will trigger even if Xira is no longer on the battlefield when that creature dies."
    );
    let mut t = TestGame::new(2);
    let xira = t.battlefield(P0, "Xira, the Golden Sting");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    t.attack(&[(xira, P1.into())], &[]);
    assert_eq!(t.counters(bears, "egg"), 1);
    destroy(&mut t, xira);
    t.resolve_all();
    let hand = t.hand_size(P0);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(tokens_of(&t, P0, "Insect"), 1);
}

#[test]
fn a_countered_champion_leaves_the_beheld_card_in_exile() {
    cr!("701.4a", "607.2a", "603.6c");
    ruling!(
        "Champion of the Clachan",
        "If Champion of the Clachan is countered or otherwise fails to resolve, the beheld card will remain in exile indefinitely."
    );
    ruling!(
        "Champion of the Path",
        "If Champion of the Path is countered or otherwise fails to resolve, the beheld card will remain in exile indefinitely."
    );
    ruling!(
        "Champions of the Perfect",
        "If Champions of the Perfect is countered or otherwise fails to resolve, the beheld card will remain in exile indefinitely."
    );
    for (name, beheld) in [
        ("Champion of the Clachan", "Goldmeadow Harrier"),
        ("Champion of the Path", "Flamekin Harbinger"),
        ("Champions of the Perfect", "Llanowar Elves"),
    ] {
        let mut t = TestGame::new(2);
        let card = t.hand(P0, beheld);
        t.answer_choose(P0, &[obj(card)]);
        let champ = cast_new(&mut t, P0, name, &[]);
        let card = t.g.current(card);
        assert_eq!(t.zone(card), Zone::Exile, "{name}");
        t.lands(P1, "Island", 2);
        let cs = t.hand(P1, "Counterspell");
        t.g.turn.priority = Some(P1);
        t.cast(P1, cs).target(champ).go();
        t.resolve_all();
        assert_eq!(t.zone(champ), Zone::Graveyard(P0), "{name}");
        assert_eq!(t.zone(card), Zone::Exile, "{name}");
        t.advance_to(P1, Step::End);
        assert_eq!(t.zone(card), Zone::Exile, "{name}");
    }
}

#[test]
fn mysterio_leaving_before_the_tokens_are_created() {
    cr!("603.7a", "603.10a");
    ruling!(
        "Mysterio, Master of Illusion",
        "If Mysterio leaves the battlefield before the tokens are created, the ability that removes them will never trigger."
    );
    let mut t = TestGame::new(2);
    // A.I.M. Synthoids: a nontoken Villain that stays.
    t.battlefield(P0, "A.I.M. Synthoids");
    let mysterio = t.enter(P0, "Mysterio, Master of Illusion");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    exile_card(&mut t, mysterio);
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0, "Illusion"), 1);
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0, "Illusion"), 1, "never exiled");
}

#[test]
fn drakestown_forgotten_counts_itself_from_a_graveyard() {
    cr!("614.1c", "614.12");
    ruling!(
        "Drakestown Forgotten",
        "If Drakestown Forgotten enters the battlefield from a graveyard, it counts itself when determining the value of X."
    );
    let mut t = TestGame::new(2);
    let dk = t.graveyard(P0, "Drakestown Forgotten");
    t.graveyard(P1, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Zombify", &[obj(dk)]);
    let dk = t.g.current(dk);
    assert!(t.on_battlefield(dk));
    assert_eq!(p1(&t, dk), 2);
}

#[test]
fn shanid_doesnt_see_a_legendary_land_put_onto_the_battlefield() {
    cr!("305.1", "305.4");
    ruling!(
        "Shanid, Sleepers' Scourge",
        "A legendary land entering the battlefield without being played does not cause Shanid’s last ability to trigger."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Shanid, Sleepers' Scourge");
    let hand = t.hand_size(P0);
    t.enter(P0, "Urborg, Tomb of Yawgmoth");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    let land = t.hand(P0, "Hammerheim");
    t.play_land(P0, land).unwrap();
    t.resolve_all();
    // Played: P0 draws a card (the land left the hand) and loses 1 life.
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.life(P0), 19);
}

#[test]
fn greven_counts_damage_and_paid_life() {
    cr!("119.3", "118.3b", "120.3a");
    ruling!(
        "Greven, Predator Captain",
        "A player loses life if they're dealt damage or if they pay life."
    );
    let mut t = TestGame::new(2);
    let greven = t.battlefield(P0, "Greven, Predator Captain");
    let base = pt_now(&mut t, greven);
    cast_resolve(&mut t, P1, "Shock", &[P0.into()]);
    // Street Wraith: "Cycling—Pay 2 life."
    let wraith = t.hand(P0, "Street Wraith");
    activate_containing(&mut t, P0, wraith, "Cycling").unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 16);
    assert_eq!(pt_now(&mut t, greven), (base.0 + 4, base.1));
}

#[test]
fn sylvan_library_can_put_back_a_card_drawn_in_the_upkeep() {
    cr!("121.1", "608.2c");
    ruling!(
        "Sylvan Library",
        "Any cards drawn prior to Sylvan Library’s ability resolving, including in your upkeep or in response to Sylvan Library’s triggered ability, can be chosen to be put back using this effect."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sylvan Library");
    let old = t.hand(P0, "Lightning Bolt");
    let upkeep_card = t.library_top(P0, "Grizzly Bears");
    // Not the first turn of the game, whose draw step is skipped (CR 103.8a).
    t.g.turn.number = 3;
    t.set_step(P0, Step::Upkeep);
    run_from(
        &mut t,
        P0,
        None,
        Effect::Draw {
            who: PlayerRef::You,
            n: Value::c(1),
        },
        &[],
    );
    assert_eq!(t.zone(upkeep_card), Zone::Hand(P0));
    let upkeep_card = t.g.current(upkeep_card);
    // The draw step's three cards.
    let drawn = t.library_top(P0, "Hill Giant");
    t.library_top(P0, "Craw Wurm");
    t.library_top(P0, "Centaur Courser");
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_choose_moved(P0, &[obj(upkeep_card), obj(drawn)]);
    t.advance_to(P0, Step::Draw);
    t.resolve_all();
    let offered: Vec<Entity> = t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { candidates, .. } if *p == P0 => Some(candidates.clone()),
            _ => None,
        })
        .flatten()
        .collect();
    assert!(offered.contains(&obj(upkeep_card)), "{offered:?}");
    assert!(!offered.contains(&obj(old)), "not drawn this turn");
    // Put back on top of the library.
    assert_eq!(t.zone(upkeep_card), Zone::Library(P0));
    assert_eq!(t.zone(drawn), Zone::Library(P0));
}

#[test]
fn blood_scrivener_replaces_only_the_first_of_two_draws() {
    cr!("121.2", "614.11");
    ruling!(
        "Blood Scrivener",
        "In total, you’ll draw three cards and lose 1 life."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Blood Scrivener");
    lands_for_cost(&mut t, P0, "Divination");
    let div = t.hand(P0, "Divination");
    t.cast(P0, div).go();
    assert_eq!(t.hand_size(P0), 0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 3);
    assert_eq!(t.life(P0), 19);
}

#[test]
fn aether_storm_stops_artifact_creature_spells() {
    cr!("101.2", "302.1");
    ruling!(
        "Aether Storm",
        "Affects any spell with the type creature, including those with other types such as artifact or enchantment."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Aether Storm");
    let orni = t.hand(P0, "Ornithopter");
    assert!(t.cast(P0, orni).try_go().is_err());
    let splitter = t.hand(P0, "Bonesplitter");
    t.lands(P0, "Plains", 1);
    assert!(t.cast(P0, splitter).try_go().is_ok());
}

#[test]
fn fumarole_needs_both_targets_and_one_creature_land_can_be_both() {
    cr!("115.3", "601.2c");
    ruling!(
        "Fumarole",
        "Fumarole can’t be cast unless you can target both a creature and a land. If there’s a permanent on the battlefield that’s both a creature and a land, it can be both targets."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    let spell = t.hand(P0, "Fumarole");
    fill_pool(&mut t, P0, 2);
    tap_lands(&mut t, P0);
    assert!(!crate::r_s02_common::can_cast(
        &mut t,
        P0,
        spell,
        CastMethod::Normal
    ));
    let mut t = TestGame::new(2);
    let arbor = t.battlefield(P1, "Dryad Arbor");
    let spell = t.hand(P0, "Fumarole");
    fill_pool(&mut t, P0, 2);
    tap_lands(&mut t, P0);
    t.cast(P0, spell).target(arbor).target(arbor).go();
    t.resolve_all();
    assert_eq!(t.zone(arbor), Zone::Graveyard(P1));
    assert_eq!(t.life(P0), 17);
}

#[test]
fn ashes_to_ashes_with_one_or_two_illegal_targets() {
    cr!("608.2b", "115.10");
    ruling!(
        "Ashes to Ashes",
        "If both of the targeted creatures are illegal targets by the time Ashes to Ashes resolves, it’s countered. It doesn’t deal 5 damage to you."
    );
    ruling!(
        "Ashes to Ashes",
        "If one of the targeted creatures is an illegal target by the time Ashes to Ashes resolves, it still exiles the other one and deals 5 damage to you."
    );
    for gone in [1, 2] {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P1, "Grizzly Bears");
        let b = t.battlefield(P1, "Hill Giant");
        cast_new(&mut t, P0, "Ashes to Ashes", &[obj(a), obj(b)]);
        bounce_now(&mut t, a);
        if gone == 2 {
            bounce_now(&mut t, b);
        }
        t.resolve_all();
        if gone == 1 {
            assert_eq!(t.zone(b), Zone::Exile);
            assert_eq!(t.life(P0), 15);
        } else {
            assert_eq!(t.life(P0), 20);
        }
    }
}

/// Returns the permanent to its owner's hand and settles.
fn bounce_now(t: &mut TestGame, id: ObjectId) {
    let o = t.obj_now(id).owner;
    move_to(t, id, Zone::Hand(o));
}

#[test]
fn lurking_evil_with_no_life_costs_nothing() {
    cr!("119.4", "107.1a");
    ruling!(
        "Lurking Evil",
        "If you have zero or negative life, half your life is zero, so it costs nothing to use the ability."
    );
    for life in [0, -3] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Platinum Angel");
        let evil = t.battlefield(P0, "Lurking Evil");
        t.g.players[0].life = life;
        t.activate(P0, evil, 0, &[]).unwrap();
        t.resolve_all();
        assert_eq!(t.life(P0), life);
        assert!(t.obj_now(evil).is(CardType::Creature));
        assert!(!t.has_lost(P0));
    }
}

#[test]
fn liesas_trigger_resolves_before_the_spell() {
    cr!("603.3", "405.5");
    ruling!(
        "Liesa, Shroud of Dusk",
        "Liesa's last ability will resolve before the spell that caused it to trigger."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Liesa, Shroud of Dusk");
    let bolt = cast_new(&mut t, P0, "Lightning Bolt", &[P1.into()]);
    t.settle();
    t.resolve();
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.zone(bolt), Zone::Stack);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn liesa_changes_only_the_commander_tax() {
    cr!("903.8", "601.2f");
    ruling!(
        "Liesa, Shroud of Dusk",
        "Liesa's first ability applies only to the \"commander tax.\" If there are any other additional costs to cast Liesa, those costs must be paid as normal."
    );
    let mut t = TestGame::with_config(
        2,
        GameConfig {
            variant: Variant::Commander,
            ..Default::default()
        },
    );
    let liesa = t.command(P0, "Liesa, Shroud of Dusk");
    t.g.objects[liesa.0 as usize].is_commander = true;
    t.g.players[0]
        .commander_names
        .push("Liesa, Shroud of Dusk".into());
    t.lands(P0, "Plains", 2);
    t.lands(P0, "Swamp", 3);
    t.cast(P0, liesa).go();
    t.resolve_all();
    t.answer_yes(P0, true);
    t.g.destroy(t.g.current(liesa), None);
    t.settle();
    // Chorus of the Conclave: "As an additional cost to cast creature spells, you may pay
    // any amount of mana. If you do, that creature enters with that many additional +1/+1
    // counters on it." Two mana for it, paid as normal; the tax is 2 life.
    t.battlefield(P0, "Chorus of the Conclave");
    t.lands(P0, "Plains", 4);
    t.lands(P0, "Swamp", 3);
    let life = t.life(P0);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(2));
    let liesa = t.g.current(liesa);
    let tapped = crate::r_s01_common::tapped_lands(&t, P0);
    t.cast(P0, liesa).go();
    assert_eq!(t.life(P0), life - 2);
    // {2}{W}{W}{B} and the {2} for Chorus: seven lands.
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0) - tapped, 7);
    t.resolve_all();
    let liesa = t.g.current(liesa);
    assert!(t.on_battlefield(liesa));
    assert_eq!(p1(&t, liesa), 2);
}

#[test]
fn indestructible_permanents_still_go_to_the_graveyard_other_ways() {
    cr!("702.12b", "704.5j", "704.5f", "704.5m", "701.21a");
    ruling!(
        "Indestructibility",
        "The most likely reasons are if it’s sacrificed, if it’s legendary and another legendary permanent with the same name is controlled by the same player"
    );
    ruling!(
        "Indestructibility",
        "if it’s a creature with toughness 0 or less, or if it’s an Aura that’s either unattached or attached to something illegal."
    );
    // Lethal damage and "destroy" don't do it.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Indestructibility", bears);
    cast_resolve(&mut t, P1, "Lightning Bolt", &[obj(bears)]);
    destroy(&mut t, bears);
    assert!(t.on_battlefield(bears));
    // Sacrificed.
    cast_resolve(&mut t, P1, "Diabolic Edict", &[P0.into()]);
    assert!(!t.on_battlefield(bears));
    // The legend rule.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Urborg, Tomb of Yawgmoth");
    attach_new(&mut t, P0, "Indestructibility", a);
    t.answer_choose(P0, &[]);
    let b = t.battlefield(P0, "Urborg, Tomb of Yawgmoth");
    t.settle();
    assert_eq!(
        [a, b].iter().filter(|x| t.on_battlefield(**x)).count(),
        1,
        "one of the two is put into the graveyard"
    );
    // Toughness 0.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Indestructibility", bears);
    run_from(
        &mut t,
        P1,
        None,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::ModifyPT(Value::Const(-2), Value::Const(-2))],
            duration: Duration::EndOfTurn,
        },
        &[obj(bears)],
    );
    assert_eq!(t.zone(bears), Zone::Graveyard(P0));
    // An Aura whose creature is gone.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let pacifism = attach_new(&mut t, P0, "Pacifism", bears);
    attach_new(&mut t, P0, "Indestructibility", pacifism);
    assert!(has(&mut t, pacifism, KeywordKind::Indestructible));
    bounce_now(&mut t, bears);
    assert_eq!(t.zone(pacifism), Zone::Graveyard(P0));
}

/// Indestructibility's ruling also names "if it’s a planeswalker and another planeswalker
/// with the same subtype is controlled by the same player": the planeswalker uniqueness
/// rule, removed from the CR effective 2017-09-29 (Ixalan; 306.4 there: "This rule has been
/// removed"). Exempt in part (rulings-P073.tsv) and logged; two Jaces now both stay.
#[test]
fn two_planeswalkers_with_the_same_subtype_both_stay() {
    cr!("306.4", "704.5j");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Jace Beleren");
    let b = t.battlefield(P0, "Jace, the Mind Sculptor");
    t.settle();
    assert!(t.on_battlefield(a) && t.on_battlefield(b));
}

#[test]
fn lier_flashback_for_an_adventure_only() {
    cr!("601.3e", "715.3a", "702.34a", "715.4");
    ruling!(
        "Lier, Disciple of the Drowned",
        "if you control Lier and you have Bonecrusher Giant in your graveyard, you can cast Stomp with flashback, but not Bonecrusher Giant."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lier, Disciple of the Drowned");
    let giant = t.graveyard(P0, "Bonecrusher Giant // Stomp");
    // In the graveyard it's a creature card, with no flashback (CR 715.4).
    assert!(!t.obj_now(giant).chars.has_keyword(KeywordKind::Flashback));
    fill_pool(&mut t, P0, 3);
    t.g.turn.priority = Some(P0);
    t.g.recompute();
    let opts = t.cast_options(P0, giant);
    assert!(!opts.is_empty(), "Stomp can be cast with flashback");
    assert!(
        opts.iter()
            .all(|o| o.face == mtg_engine::object::FaceState::Half(1)
                && o.method == CastMethod::Keyword(KeywordKind::Flashback)),
        "only the Adventure, with flashback"
    );
    let s = t
        .cast(P0, giant)
        .method(CastMethod::Keyword(KeywordKind::Flashback))
        .target(P1)
        .go();
    assert_eq!(t.obj_now(s).chars.name, "Stomp");
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // Exiled as it resolves.
    assert_eq!(t.zone(t.g.current(giant)), Zone::Exile);
    // Without Lier, nothing.
    let mut t = TestGame::new(2);
    let giant = t.graveyard(P0, "Bonecrusher Giant // Stomp");
    fill_pool(&mut t, P0, 3);
    assert!(t.cast_options(P0, giant).is_empty());
}

#[test]
fn lier_flashback_for_an_omen() {
    cr!("601.3e", "720.3a", "720.3d", "702.34a", "400.7g", "616.1");
    ruling!(
        "Lier, Disciple of the Drowned",
        "Lier's last ability allows you to cast adventurer cards in your graveyard as Adventures and omen cards in your graveyard as Omens using the flashback it grants."
    );
    ruling!(
        "Lier, Disciple of the Drowned",
        "A spell cast using flashback will always be exiled afterward, whether it resolves, is countered, or leaves the stack in some other way."
    );
    // Dirgur Island Dragon // Skimming Strike ({1}{U} instant Omen: "Tap up to one target
    // creature. Draw a card.").
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lier, Disciple of the Drowned");
    let card = t.graveyard(P0, "Dirgur Island Dragon // Skimming Strike");
    let bears = t.battlefield(P1, "Grizzly Bears");
    fill_pool(&mut t, P0, 2);
    let hand = t.hand_size(P0);
    let s = t
        .cast(P0, card)
        .method(CastMethod::Keyword(KeywordKind::Flashback))
        .target(bears)
        .go();
    assert_eq!(t.obj_now(s).chars.name, "Skimming Strike");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert!(t.obj_now(bears).tapped);
    // Exiled, not shuffled into the library as an Omen otherwise is (CR 720.3d): flashback
    // exiles it "instead of putting it anywhere else" (CR 702.34a), whichever of the two
    // replacements is applied first (CR 616.1).
    assert_eq!(t.zone(t.g.current(card)), Zone::Exile);
    // Returned to its owner's hand from the stack by Venser, Shaper Savant ("When Venser
    // enters, return target spell or permanent to its owner's hand."), it's exiled too: the
    // spell keeps the flashback it was cast with (CR 400.7g), though Lier gives flashback
    // only to cards in a graveyard.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lier, Disciple of the Drowned");
    let card = t.graveyard(P0, "Dirgur Island Dragon // Skimming Strike");
    let bears = t.battlefield(P1, "Grizzly Bears");
    fill_pool(&mut t, P0, 2);
    let s = t
        .cast(P0, card)
        .method(CastMethod::Keyword(KeywordKind::Flashback))
        .target(bears)
        .go();
    assert!(has(&mut t, s, KeywordKind::Flashback));
    t.answer_targets(P1, &[obj(s)]);
    enter(&mut t, P1, "Venser, Shaper Savant");
    t.resolve();
    assert_eq!(t.zone(t.g.current(card)), Zone::Exile);
    assert!(!t.obj_now(bears).tapped);
}

#[test]
fn elspeth_resplendent_puts_both_counters_at_once() {
    cr!("122.1", "122.6");
    ruling!(
        "Elspeth Resplendent",
        "For Elspeth's first ability, its controller chooses flying, first strike, lifelink, or vigilance, then that counter and the +1/+1 counter are placed on the target creature at the same time."
    );
    let mut t = TestGame::new(2);
    let elspeth = t.battlefield(P0, "Elspeth Resplendent");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer(P0, DecisionKind::Option, Answer::Index(2));
    t.activate(P0, elspeth, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(p1(&t, bears), 1);
    assert_eq!(t.counters(bears, "lifelink"), 1);
    assert!(has(&mut t, bears, KeywordKind::Lifelink));
}
