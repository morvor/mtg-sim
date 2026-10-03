//! Rulings batch P037 — lands and artifacts that become creatures: what they keep (CR
//! 205.1b, 613.1d), durations (CR 611.2a/b, 610.3), values of X fixed on resolution (CR
//! 608.2h), base power and toughness and later effects (CR 613.4b, 613.7), Equipment that
//! become creatures (CR 301.5c), summoning sickness (CR 302.6), abilities whose source
//! left the battlefield (CR 610.3c, 611.2b).

use crate::r_p037_common::*;
use crate::r_s01_common::supported;
use crate::r_s06_common::{activate_containing, attach_new, attached_to, give_control};
use crate::r_s25_common::cast_new;
use crate::r_s26_common::modify_until_eot;
use mtg_engine::ability::{Modification, Value};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Activates the ability of `source` containing `needle` at `target`, paying with `lands`
/// Wastes plus the listed basic lands, and resolves the stack.
fn animate(
    t: &mut TestGame,
    source: ObjectId,
    needle: &str,
    target: ObjectId,
    wastes: usize,
    basics: &[&str],
) {
    t.lands(P0, "Wastes", wastes);
    for b in basics {
        t.lands(P0, b, 1);
    }
    t.answer_targets(P0, &[obj(target)]);
    activate_containing(t, P0, source, needle).expect("activate");
    t.resolve_all();
}

fn tapped(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).tapped
}

fn kw(t: &TestGame, id: ObjectId, k: KeywordKind) -> bool {
    t.obj_now(id).has_keyword(k)
}

fn next_turn(t: &mut TestGame) {
    t.advance_to(P1, Step::Upkeep);
}

// --- What the animated permanent keeps -------------------------------------------------------

#[test]
fn avalanche_caller_s_land_keeps_its_types_and_abilities_and_tapped_state() {
    cr!("205.1b", "613.1d", "110.5");
    ruling!(
        "Avalanche Caller",
        "A land that becomes a creature because of Avalanche Caller’s activated ability will retain any other supertypes, card types, subtypes, and abilities it had. In particular, it will be a snow creature land."
    );
    ruling!(
        "Avalanche Caller",
        "The activated ability doesn’t cause the target snow land to become tapped or untapped."
    );
    ruling!(
        "Avalanche Caller",
        "In most cases, the land that becomes a creature will remain colorless. If another effect caused the land to have one or more colors, the resulting creature land will retain those colors."
    );
    supported("Avalanche Caller");
    for tap in [false, true] {
        let mut t = TestGame::new(2);
        let caller = t.battlefield(P0, "Avalanche Caller");
        let land = t.battlefield(P0, "Snow-Covered Island");
        if tap {
            t.g.tap(land);
        }
        pool(&mut t, P0, mtg_engine::mana::ManaType::C, 2);
        animate(&mut t, caller, "snow land", land, 0, &[]);
        let o = t.obj_now(land);
        assert!(o.is(CardType::Creature) && o.is(CardType::Land));
        assert!(o.chars.supertypes.contains(Supertype::Snow));
        assert!(o.chars.supertypes.contains(Supertype::Basic));
        assert!(o.chars.has_subtype("Island") && o.chars.has_subtype("Elemental"));
        assert!(has_ability_text(&t, land, "{U}"));
        assert!(o.chars.colors.is_colorless());
        assert_eq!(t.pt(land), (4, 4));
        assert_eq!(tapped(&t, land), tap);
    }
    // A land made red by another effect stays red.
    let mut t = TestGame::new(2);
    let caller = t.battlefield(P0, "Avalanche Caller");
    let land = t.battlefield(P0, "Snow-Covered Island");
    modify_until_eot(
        &mut t,
        land,
        vec![Modification::SetColors(ColorSet::single(Color::Red))],
    );
    animate(&mut t, caller, "snow land", land, 2, &[]);
    assert!(t.obj_now(land).chars.colors.contains(Color::Red));
}

#[test]
fn skarrg_guildmage_s_land_keeps_name_types_and_abilities() {
    cr!("205.1b", "613.1d");
    ruling!(
        "Skarrg Guildmage",
        "Skarrg Guildmage’s second ability doesn’t affect the land’s name or any other types, subtypes, or supertypes (such as basic or legendary) the land may have. The land will also keep any abilities it had."
    );
    supported("Skarrg Guildmage");
    let mut t = TestGame::new(2);
    let gm = t.battlefield(P0, "Skarrg Guildmage");
    let land = t.battlefield(P0, "Forest");
    animate(&mut t, gm, "4/4", land, 1, &["Mountain", "Forest"]);
    let o = t.obj_now(land);
    assert_eq!(o.chars.name, "Forest");
    assert!(o.chars.supertypes.contains(Supertype::Basic));
    assert!(o.is(CardType::Land) && o.is(CardType::Creature));
    assert!(o.chars.has_subtype("Forest") && o.chars.has_subtype("Elemental"));
    assert!(has_ability_text(&t, land, "{G}"));
    assert_eq!(t.pt(land), (4, 4));
}

#[test]
fn artifacts_animated_keep_their_types_and_abilities() {
    cr!("205.1b", "613.1d");
    ruling!(
        "Alloy Animist",
        "As Alloy Animist's ability resolves, the target permanent keeps any other types and subtypes it had before it became an artifact creature."
    );
    ruling!(
        "Unctus's Retrofitter",
        "The target artifact retains any types, subtypes, and supertypes it has."
    );
    ruling!(
        "Unctus's Retrofitter",
        "Unctus's Retrofitter doesn't remove any abilities the target artifact has."
    );
    ruling!(
        "Skilled Animator",
        "Skilled Animator doesn't remove any abilities the target artifact has."
    );
    supported("Alloy Animist");
    supported("Unctus's Retrofitter");
    supported("Skilled Animator");
    // Alloy Animist on an Equipment: still an Equipment.
    let mut t = TestGame::new(2);
    let aa = t.battlefield(P0, "Alloy Animist");
    let jitte = t.battlefield(P0, "Umezawa's Jitte");
    animate(&mut t, aa, "4/4", jitte, 2, &["Forest"]);
    let o = t.obj_now(jitte);
    assert!(o.is(CardType::Creature) && o.is(CardType::Artifact));
    assert!(o.chars.has_subtype("Equipment"));
    assert_eq!(t.pt(jitte), (4, 4));
    // Retrofitter and Animator: types, supertypes and abilities stay.
    for (card, pt) in [("Unctus's Retrofitter", 4), ("Skilled Animator", 5)] {
        let mut t = TestGame::new(2);
        let jitte = t.battlefield(P0, "Umezawa's Jitte");
        let stone = t.battlefield(P0, "Mind Stone");
        for target in [jitte, stone] {
            t.answer_targets(P0, &[obj(target)]);
            t.enter(P0, card);
            t.resolve_all();
        }
        let o = t.obj_now(jitte);
        assert!(o.is(CardType::Creature) && o.is(CardType::Artifact), "{card}");
        assert!(o.chars.supertypes.contains(Supertype::Legendary));
        assert!(o.chars.has_subtype("Equipment"));
        assert_eq!(t.pt(jitte), (pt, pt));
        assert!(is_creature(&t, stone));
        assert!(has_ability_text(&t, stone, "{C}"), "{card}");
        assert!(has_ability_text(&t, stone, "Draw a card"), "{card}");
    }
}

// --- Durations ---------------------------------------------------------------------------------

#[test]
fn animations_without_a_duration_last_indefinitely() {
    cr!("611.2a", "514.2");
    ruling!(
        "Balduvian Frostwaker",
        "Balduvian Frostwaker’s effect has no duration. The snow land is a creature as long as it’s on the battlefield."
    );
    ruling!(
        "Tendril of the Mycotyrant",
        "Tendril of the Mycotyrant's ability doesn't have a duration. The land remains a creature until it leaves the battlefield."
    );
    ruling!(
        "Animating Faerie // Bring to Life",
        "Bring to Life's effect lasts indefinitely. It doesn't wear off during the cleanup step."
    );
    ruling!(
        "Waker of the Wilds",
        "The land-animation effect lasts indefinitely. It doesn’t wear off during the cleanup step or when you lose control of Waker of the Wilds."
    );
    supported("Balduvian Frostwaker");
    supported("Tendril of the Mycotyrant");
    supported("Animating Faerie // Bring to Life");
    supported("Waker of the Wilds");
    let mut t = TestGame::new(2);
    let bf = t.battlefield(P0, "Balduvian Frostwaker");
    let snow = t.battlefield(P0, "Snow-Covered Island");
    animate(&mut t, bf, "snow land", snow, 0, &["Island"]);
    let tendril = t.battlefield(P0, "Tendril of the Mycotyrant");
    let wastes = t.battlefield(P0, "Wastes");
    animate(&mut t, tendril, "seven", wastes, 5, &["Forest", "Forest"]);
    let waker = t.battlefield(P0, "Waker of the Wilds");
    let forest = t.battlefield(P0, "Forest");
    t.answer(
        P0,
        DecisionKind::X,
        mtg_engine::decision::Answer::Number(2),
    );
    animate(&mut t, waker, "counters", forest, 2, &["Forest", "Forest"]);
    let stone = t.battlefield(P0, "Mind Stone");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 2);
    let card = t.hand(P0, "Animating Faerie // Bring to Life");
    t.cast(P0, card)
        .method(CastMethod::Half(1))
        .target(stone)
        .go();
    t.resolve_all();
    give_control(&mut t, waker, P1);
    for id in [snow, wastes, forest, stone] {
        assert!(is_creature(&t, id));
    }
    next_turn(&mut t);
    assert_eq!(t.pt(snow), (2, 2));
    assert_eq!(t.pt(wastes), (7, 7));
    assert_eq!(t.pt(forest), (2, 2));
    assert_eq!(t.pt(stone), (4, 4));
    for id in [snow, wastes, forest, stone] {
        assert!(is_creature(&t, id));
    }
}

#[test]
fn balduvian_frostwaker_s_land_stays_a_creature_after_losing_snow() {
    cr!("611.2c", "608.2h");
    ruling!(
        "Balduvian Frostwaker",
        "If an affected land later loses the supertype snow, Balduvian Frostwaker’s effect doesn’t end. The land will remain a creature."
    );
    let mut t = TestGame::new(2);
    let bf = t.battlefield(P0, "Balduvian Frostwaker");
    let snow = t.battlefield(P0, "Snow-Covered Island");
    animate(&mut t, bf, "snow land", snow, 0, &["Island"]);
    modify_until_eot(
        &mut t,
        snow,
        vec![Modification::RemoveSupertypes(vec![Supertype::Snow])],
    );
    assert!(!t.obj_now(snow).chars.supertypes.contains(Supertype::Snow));
    assert!(is_creature(&t, snow));
    assert!(kw(&t, snow, KeywordKind::Flying));
}

// --- Tapped and untapped ------------------------------------------------------------------------

#[test]
fn animating_a_tapped_land_doesnt_untap_it() {
    cr!("110.5");
    ruling!(
        "Destiny Spinner",
        "Destiny Spinner's activated ability doesn't untap the land that becomes a creature."
    );
    ruling!(
        "Llanowar Loamspeaker",
        "Llanowar Loamspeaker's second ability doesn't untap the land that becomes a creature."
    );
    supported("Destiny Spinner");
    supported("Llanowar Loamspeaker");
    for (card, needle, wastes, basics) in [
        ("Destiny Spinner", "X/X", 3, &["Forest"][..]),
        ("Llanowar Loamspeaker", "3/3", 0, &[][..]),
    ] {
        let mut t = TestGame::new(2);
        let src = t.battlefield(P0, card);
        let land = t.battlefield(P0, "Forest");
        t.g.tap(land);
        animate(&mut t, src, needle, land, wastes, basics);
        assert!(is_creature(&t, land), "{card}");
        assert!(tapped(&t, land), "{card}");
    }
}

// --- X is fixed on resolution -------------------------------------------------------------------

#[test]
fn x_of_animations_is_fixed_as_the_ability_resolves() {
    cr!("608.2h", "611.2c");
    ruling!(
        "Destiny Spinner",
        "The value of X is determined only as Destiny Spinner's activated ability resolves. Once that happens, the creature's base power and toughness won't change later in the turn even if the number of enchantments you control changes."
    );
    ruling!(
        "Vastwood Animist",
        "The value of X is determined as Vastwood Animist's activated ability resolves. The power and toughness of the Elemental won't change if the number of Allies you control changes later in the turn."
    );
    ruling!(
        "Elvish Branchbender",
        "The value of X is determined when the ability resolves. It won’t change later, even if the number of Elves you control changes."
    );
    ruling!(
        "Sage of the Maze",
        "The value of X is calculated only once, as Sage of the Maze's second ability resolves."
    );
    supported("Vastwood Animist");
    supported("Elvish Branchbender");
    supported("Sage of the Maze");
    // (card, ability text, Wastes, basics, extra permanent, value before, after)
    let cases: [(&str, &str, usize, &[&str], &str, i32); 4] = [
        ("Destiny Spinner", "X/X", 3, &["Forest"], "Ghostly Prison", 1),
        ("Vastwood Animist", "X/X", 0, &[], "Kazandu Blademaster", 1),
        ("Elvish Branchbender", "X/X", 0, &[], "Llanowar Elves", 1),
        ("Sage of the Maze", "Gates", 0, &[], "Azorius Guildgate", 2),
    ];
    for (card, needle, wastes, basics, extra, x) in cases {
        let mut t = TestGame::new(2);
        let src = t.battlefield(P0, card);
        if card == "Sage of the Maze" {
            t.battlefield(P0, "Azorius Guildgate");
        }
        let land = t.battlefield(P0, "Forest");
        animate(&mut t, src, needle, land, wastes, basics);
        assert_eq!(t.pt(land), (x, x), "{card}");
        t.battlefield(P0, extra);
        t.g.recompute();
        assert_eq!(t.pt(land), (x, x), "{card}: X doesn't change");
    }
}

#[test]
fn destiny_spinner_twice_on_one_land_the_latest_applies() {
    cr!("613.7", "608.2h");
    ruling!(
        "Destiny Spinner",
        "If you activate Destiny Spinner's ability more than once in a turn, the value of X may be different each time. If you choose the same land as the target more than once, the most recent effect (and most recent value of X) applies."
    );
    let mut t = TestGame::new(2);
    let ds = t.battlefield(P0, "Destiny Spinner");
    let land = t.battlefield(P0, "Forest");
    animate(&mut t, ds, "X/X", land, 3, &["Forest"]);
    assert_eq!(t.pt(land), (1, 1));
    t.battlefield(P0, "Ghostly Prison");
    animate(&mut t, ds, "X/X", land, 3, &["Forest"]);
    assert_eq!(t.pt(land), (2, 2));
}

#[test]
fn x_of_zero_makes_a_0_0_land_that_dies() {
    cr!("704.5f", "608.2h");
    ruling!(
        "Destiny Spinner",
        "If you control no enchantments at the time Destiny Spinner's activated ability resolves, the land becomes a 0/0 creature and is put into its owner's graveyard unless another effect is raising its toughness."
    );
    ruling!(
        "Vastwood Animist",
        "If you control no Allies when the ability resolves, the land becomes a 0/0 creature and is put into its owner's graveyard as a state-based action. However, since Vastwood Animist is itself an Ally, the land will usually be at least a 1/1."
    );
    ruling!(
        "Elvish Branchbender",
        "If you control no Elves when the ability resolves, the Forest will become a 0/0 creature and be put into the graveyard. However, since Elvish Branchbender is itself an Elf, the Forest will usually be at least a 1/1."
    );
    for (card, wastes, basics) in [
        ("Destiny Spinner", 3, &["Forest"][..]),
        ("Vastwood Animist", 0, &[][..]),
        ("Elvish Branchbender", 0, &[][..]),
    ] {
        let mut t = TestGame::new(2);
        let src = t.battlefield(P0, card);
        let land = t.battlefield(P0, "Forest");
        t.lands(P0, "Wastes", wastes);
        for b in basics {
            t.lands(P0, b, 1);
        }
        t.answer_targets(P0, &[obj(land)]);
        activate_containing(&mut t, P0, src, "X/X").unwrap();
        // The source (the only enchantment / Ally / Elf) leaves in response.
        t.g.destroy(src, None);
        t.resolve_all();
        assert!(t.in_graveyard(P0, "Forest"), "{card}");
    }
}

// --- Base power and toughness ----------------------------------------------------------------

#[test]
fn setting_base_power_and_toughness_of_an_artifact_creature() {
    cr!("613.4b", "613.7");
    ruling!(
        "Unctus's Retrofitter",
        "If the artifact was already a creature, its base power and toughness will become 4/4. This overwrites any previous effects that set its base power and/or toughness to specific values. Any power- or toughness-setting effects that start to apply after Unctus's Retrofitter's ability resolves will overwrite this effect."
    );
    ruling!(
        "Skilled Animator",
        "If the artifact was already a creature, its base power and toughness will each become 5. This overwrites any previous effects that set the creature's base power and toughness to specific values. Any power- or toughness-setting effects that start to apply after Skilled Animator's ability resolves will overwrite this effect."
    );
    ruling!(
        "Unctus's Retrofitter",
        "Effects that modify a creature's power and/or toughness, such as the one created by Blazing Crescendo or a +1/+1 counter, will apply to the creature no matter when they started to take effect."
    );
    for (card, n) in [("Unctus's Retrofitter", 4), ("Skilled Animator", 5)] {
        let mut t = TestGame::new(2);
        // Ornithopter: an 0/2 artifact creature; first set to 1/1.
        let orn = t.battlefield(P0, "Ornithopter");
        let set = |v: i32| Modification::SetPT(Some(Value::c(v)), Some(Value::c(v)));
        modify_until_eot(&mut t, orn, vec![set(1)]);
        modify_until_eot(&mut t, orn, vec![Modification::ModifyPT(Value::c(2), Value::c(2))]);
        t.g.add_counters(Entity::Object(orn), counters::PLUS1, 1, None);
        t.answer_targets(P0, &[obj(orn)]);
        t.enter(P0, card);
        t.resolve_all();
        // Base n/n, +2/+2 from earlier, +1/+1 counter.
        assert_eq!(t.pt(orn), (n + 3, n + 3), "{card}");
        modify_until_eot(&mut t, orn, vec![set(2)]);
        assert_eq!(t.pt(orn), (5, 5), "{card}: the later setting wins");
    }
}

#[test]
fn waker_of_the_wilds_on_a_land_creature_sets_base_0_0() {
    cr!("613.4b", "613.4c");
    ruling!(
        "Waker of the Wilds",
        "If the ability targets a land that’s already a creature, that land creature’s base power and toughness will become 0/0, overwriting its previous base power and toughness. Other effects that modify its power and/or toughness (including any +1/+1 counters that were on it) will continue to apply."
    );
    let mut t = TestGame::new(2);
    let waker = t.battlefield(P0, "Waker of the Wilds");
    // Dryad Arbor: a 1/1 land creature, with a +1/+1 counter and +1/+1 until end of turn.
    let arbor = t.battlefield(P0, "Dryad Arbor");
    t.g.add_counters(Entity::Object(arbor), counters::PLUS1, 1, None);
    modify_until_eot(&mut t, arbor, vec![Modification::ModifyPT(Value::c(1), Value::c(1))]);
    assert_eq!(t.pt(arbor), (3, 3));
    t.answer(
        P0,
        DecisionKind::X,
        mtg_engine::decision::Answer::Number(2),
    );
    animate(&mut t, waker, "counters", arbor, 2, &["Forest", "Forest"]);
    // 0/0 base, three +1/+1 counters, +1/+1.
    assert_eq!(t.pt(arbor), (4, 4));
}

// --- Equipment that become creatures --------------------------------------------------------------

#[test]
fn an_equipment_that_becomes_a_creature_becomes_unattached() {
    cr!("301.5c", "704.5n");
    ruling!(
        "Skilled Animator",
        "If an Equipment becomes an artifact creature, it usually can't be attached to another creature. If it was attached to a creature, it becomes unattached."
    );
    ruling!(
        "Animating Faerie // Bring to Life",
        "An Equipment that's also a creature can't be attached to anything. You can activate its equip ability, but it won't become attached. If it's attached as it becomes a creature, it becomes unattached."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let bs = attach_new(&mut t, P0, "Bonesplitter", obj(bears));
    t.answer_targets(P0, &[obj(bs)]);
    t.enter(P0, "Skilled Animator");
    t.resolve_all();
    assert!(is_creature(&t, bs));
    assert_eq!(attached_to(&t, bs), None);

    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let bs = attach_new(&mut t, P0, "Bonesplitter", obj(bears));
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 2);
    let card = t.hand(P0, "Animating Faerie // Bring to Life");
    t.cast(P0, card).method(CastMethod::Half(1)).target(bs).go();
    t.resolve_all();
    assert!(is_creature(&t, bs));
    assert_eq!(attached_to(&t, bs), None);
    // Equip can be activated, but it doesn't attach.
    t.lands(P0, "Wastes", 1);
    t.answer_targets(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, bs, "Equip").expect("equip can be activated");
    t.resolve_all();
    assert_eq!(attached_to(&t, bs), None);
}

#[test]
fn bring_to_life_s_artifact_needs_to_have_been_controlled_since_the_turn_began() {
    cr!("302.6");
    ruling!(
        "Animating Faerie // Bring to Life",
        "An artifact that becomes a creature due to Bring to Life can't attack unless you've controlled it continuously since your turn began."
    );
    let mut t = TestGame::new(2);
    let old = t.battlefield(P0, "Bonesplitter");
    let new = t.battlefield_sick(P0, "Bonesplitter");
    for a in [old, new] {
        t.lands(P0, "Island", 1);
        t.lands(P0, "Wastes", 2);
        let card = t.hand(P0, "Animating Faerie // Bring to Life");
        t.cast(P0, card).method(CastMethod::Half(1)).target(a).go();
        t.resolve_all();
    }
    assert!(t.g.can_attack(t.g.current(old)));
    assert!(!t.g.can_attack(t.g.current(new)));
}

// --- Awakener Druid -------------------------------------------------------------------------

/// The candidates of the last target decision asked.
fn last_target_candidates(t: &TestGame) -> Vec<Entity> {
    t.asked()
        .iter()
        .rev()
        .find_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseTargets { candidates, .. } => {
                Some(candidates.clone())
            }
            _ => None,
        })
        .unwrap_or_default()
}

#[test]
fn awakener_druid_targets_any_forest_and_must_target_one() {
    cr!("205.3i", "603.3d", "115.1");
    ruling!(
        "Awakener Druid",
        "Awakener Druid's ability affects a land with the land type Forest, not necessarily a land with the name Forest."
    );
    ruling!(
        "Awakener Druid",
        "Awakener Druid's ability is mandatory. When it enters, you must target a Forest (if there is one), even if you don't control that Forest."
    );
    supported("Awakener Druid");
    // Stomping Ground (Mountain Forest) is a Forest.
    let mut t = TestGame::new(2);
    let sg = t.battlefield(P0, "Stomping Ground");
    t.battlefield(P0, "Mountain");
    t.answer_targets(P0, &[obj(sg)]);
    t.enter(P0, "Awakener Druid");
    t.settle();
    assert_eq!(last_target_candidates(&t), vec![obj(sg)]);
    t.resolve_all();
    assert_eq!(t.pt(sg), (4, 5));
    // The only Forest is an opponent's: it's targeted (nothing chosen, still targeted).
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Forest");
    t.enter(P0, "Awakener Druid");
    t.resolve_all();
    assert!(is_creature(&t, theirs));
    assert_eq!(t.obj_now(theirs).controller, P1);
}

/// Awakener Druid enters with `forest` as its target; returns the Druid.
fn druid_on(t: &mut TestGame, forest: ObjectId) -> ObjectId {
    t.answer_targets(P0, &[obj(forest)]);
    let d = t.enter(P0, "Awakener Druid");
    t.settle();
    d
}

#[test]
fn awakener_druid_leaving_ends_or_prevents_the_animation() {
    cr!("611.2b", "610.3c", "704.5g");
    ruling!(
        "Awakener Druid",
        "If Awakener Druid leaves the battlefield before its \"enters\" ability resolves, nothing happens to the targeted Forest when that ability resolves. It won't become a creature."
    );
    ruling!(
        "Awakener Druid",
        "If Awakener Druid and the Treefolk are dealt lethal damage at the same time, both will be destroyed. However, if Awakener Druid leaves the battlefield before the Treefolk is dealt damage, it will immediately revert to being just a land and thus can't be dealt damage."
    );
    // Leaves before the ability resolves.
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    let d = druid_on(&mut t, forest);
    t.g.destroy(d, None);
    t.resolve_all();
    assert!(!is_creature(&t, forest));
    // Both dealt lethal damage at once (Blasphemous Act: 13 damage to each creature).
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    druid_on(&mut t, forest);
    t.resolve_all();
    assert!(is_creature(&t, forest));
    cast_new(&mut t, P0, "Blasphemous Act", &[]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Awakener Druid"));
    assert!(t.in_graveyard(P0, "Forest"));
    // The Druid leaves first: the Forest is just a land again; damage to each creature
    // misses it.
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    let d = druid_on(&mut t, forest);
    t.resolve_all();
    t.g.destroy(d, None);
    t.settle();
    assert!(!is_creature(&t, forest));
    cast_new(&mut t, P0, "Pyroclasm", &[]);
    t.resolve_all();
    assert_eq!(t.obj_now(forest).damage, 0);
    assert!(t.on_battlefield(forest));
}

#[test]
fn awakener_druid_s_new_forest_has_summoning_sickness() {
    cr!("302.6");
    ruling!(
        "Awakener Druid",
        "If the targeted Forest entered this turn, it will be affected by \"summoning sickness\" once it becomes a Treefolk. You won't be able to attack with it or use its activated abilities that have {T} in the cost (including its mana ability)."
    );
    let mut t = TestGame::new(2);
    let forest = t.battlefield_sick(P0, "Forest");
    druid_on(&mut t, forest);
    t.resolve_all();
    assert!(is_creature(&t, forest));
    assert!(!t.g.can_attack(t.g.current(forest)));
    // Its mana can't pay for Llanowar Elves.
    let elves = t.hand(P0, "Llanowar Elves");
    assert!(t.cast_with(P0, elves, &[]).is_err());
    assert!(!tapped(&t, forest));
}

// --- Skarrg Guildmage's trample ----------------------------------------------------------------

#[test]
fn skarrg_guildmage_s_trample_affects_only_creatures_there_on_resolution() {
    cr!("611.2c");
    ruling!(
        "Skarrg Guildmage",
        "Only creatures you control when the first ability resolves will gain trample. Creatures that come under your control later in the turn won’t have trample. Lands you control that aren’t creatures also won’t gain trample, even if you use the second ability to turn them into creatures later in the turn."
    );
    let mut t = TestGame::new(2);
    let gm = t.battlefield(P0, "Skarrg Guildmage");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let land = t.battlefield(P0, "Forest");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Forest", 1);
    activate_containing(&mut t, P0, gm, "trample").unwrap();
    t.resolve_all();
    assert!(kw(&t, bears, KeywordKind::Trample));
    let later = t.battlefield(P0, "Hill Giant");
    animate(&mut t, gm, "4/4", land, 1, &["Mountain", "Forest"]);
    assert!(is_creature(&t, land));
    assert!(!kw(&t, land, KeywordKind::Trample));
    assert!(!kw(&t, later, KeywordKind::Trample));
}

#[test]
fn destiny_spinner_can_be_countered_as_a_spell() {
    cr!("611.3b", "701.6a");
    ruling!(
        "Destiny Spinner",
        "Destiny Spinner's first ability applies only while it's on the battlefield. While it's a spell, it can be countered."
    );
    let mut t = TestGame::new(2);
    let spell = cast_new(&mut t, P0, "Destiny Spinner", &[]);
    cast_new(&mut t, P1, "Counterspell", &[obj(spell)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Destiny Spinner"));
}
