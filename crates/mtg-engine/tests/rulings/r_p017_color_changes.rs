//! Rulings batch P017 — effects that change colors and types (CR 105.3, 205.1, 613):
//! which objects an effect from a resolved spell or ability affects (CR 611.2c), how
//! later effects overwrite or add to it (CR 613.7), the layers that change types,
//! colors, abilities, and power/toughness (CR 613.1, 613.4), and lands that become
//! creatures (CR 302.6).

use crate::r_p017_common::*;
use crate::r_s01_common::supported;
use crate::r_s04_common::add_mana;
use crate::r_s17_common::token_copy;
use crate::r_s20_common::tap_for_mana;
use crate::r_s26_common::modify_until_eot;
use mtg_engine::ability::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn colors(t: &mut TestGame, id: ObjectId) -> ColorSet {
    t.g.recompute();
    t.obj_now(id).chars.colors
}

fn of(cs: &[Color]) -> ColorSet {
    cs.iter()
        .fold(ColorSet::NONE, |a, c| a.union(ColorSet::single(*c)))
}

fn subtypes(t: &TestGame, id: ObjectId) -> Vec<String> {
    t.obj_now(id)
        .chars
        .subtypes
        .iter()
        .map(|s| s.to_string())
        .collect()
}

/// Queues `p`'s answer to "Choose a color or colors".
fn choose_colors(t: &mut TestGame, p: PlayerId, cs: ColorSet) {
    t.answer(p, DecisionKind::Option, Answer::Index(cs.0 as usize - 1));
}

/// Ends the turn: cleanup's "until end of turn" effects end (CR 514.2).
fn next_turn(t: &mut TestGame) {
    let other = if t.g.turn.active == P0 { P1 } else { P0 };
    t.advance_to(other, Step::Upkeep);
}

// ---------------------------------------------------------------------------
// Which objects are affected
// ---------------------------------------------------------------------------

#[test]
fn nightcreep_doesnt_affect_what_enters_later() {
    cr!("611.2c");
    ruling!(
        "Nightcreep",
        "A land or creature that enters after Nightcreep resolves won't be affected by it."
    );
    supported("Nightcreep");
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P1, "Llanowar Elves");
    let forest = t.battlefield(P1, "Forest");
    t.lands(P0, "Swamp", 2);
    let spell = t.hand(P0, "Nightcreep");
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(colors(&mut t, elves), of(&[Color::Black]));
    assert_eq!(subtypes(&t, forest), vec!["Swamp"]);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let island = t.battlefield(P1, "Island");
    assert_eq!(colors(&mut t, bears), of(&[Color::Green]));
    assert_eq!(subtypes(&t, island), vec!["Island"]);
}

#[test]
fn nightcreep_on_a_creature_land_replaces_land_types_and_its_rules_text_abilities() {
    cr!("305.7", "613.1d", "613.1f");
    ruling!(
        "Nightcreep",
        "If a permanent is both a creature and a land, Nightcreep replaces all of its land types with Swamp, but it doesn't affect its creature types."
    );
    supported("Blinkmoth Nexus");
    let mut t = TestGame::new(2);
    let nexus = t.battlefield(P0, "Blinkmoth Nexus");
    t.lands(P0, "Swamp", 2);
    add_mana(&mut t, P0, ManaType::C, 1);
    t.activate(P0, nexus, 1, &[]).expect("animate");
    t.resolve_all();
    assert!(t.obj_now(nexus).is(CardType::Creature));
    let spell = t.hand(P0, "Nightcreep");
    t.cast(P0, spell).go();
    t.resolve_all();
    let o = t.obj_now(nexus);
    assert!(o.is(CardType::Creature) && o.is(CardType::Land));
    assert_eq!(o.chars.colors, of(&[Color::Black]));
    assert!(o.chars.has_subtype("Swamp") && o.chars.has_subtype("Blinkmoth"));
    // Flying (from the animation effect) stays; the rules-text abilities are gone; its
    // only mana ability is the Swamp's.
    assert!(o.has_keyword(KeywordKind::Flying));
    let texts: Vec<String> = o
        .chars
        .abilities
        .iter()
        .map(|a| a.text.to_string())
        .collect();
    assert!(
        texts
            .iter()
            .all(|x| !x.contains("{C}") && !x.contains("becomes")),
        "{texts:?}"
    );
    assert!(tap_for_mana(&mut t, P0, nexus, "{B}"));
}

#[test]
fn polymorphists_jest_doesnt_affect_creatures_that_arrive_later() {
    cr!("611.2c");
    ruling!(
        "Polymorphist's Jest",
        "Creatures that enter the battlefield or come under the target player’s control after Polymorphist’s Jest resolves won’t be affected."
    );
    supported("Polymorphist's Jest");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    jest(&mut t, P1);
    assert_eq!(t.pt(bears), (1, 1));
    let giant = t.battlefield(P1, "Hill Giant");
    assert_eq!(t.pt(giant), (3, 3));
    // Gaining control of a creature afterwards: not affected either.
    let mine = t.battlefield(P0, "Serra Angel");
    modify_until_eot(
        &mut t,
        mine,
        vec![Modification::SetController(PlayerRef::Player(P1))],
    );
    assert_eq!(t.obj_now(mine).controller, P1);
    assert_eq!(t.pt(mine), (4, 4));
}

/// P0 casts Polymorphist's Jest targeting `p` and it resolves.
fn jest(t: &mut TestGame, p: PlayerId) {
    t.lands(P0, "Island", 3);
    let spell = t.hand(P0, "Polymorphist's Jest");
    t.cast(P0, spell).target(p).go();
    t.resolve_all();
}

#[test]
fn polymorphists_jest_leaves_power_changes_and_new_abilities() {
    cr!("613.4c", "613.4d", "613.1f", "613.7");
    ruling!(
        "Polymorphist's Jest",
        "Effects that modify a creature’s power and/or toughness, such as the effect of Titanic Growth, will apply to the creatures no matter when they started to take effect."
    );
    ruling!(
        "Polymorphist's Jest",
        "If one of the affected creatures gains an ability after Polymorphist’s Jest resolves, it will keep that ability."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    modify_until_eot(
        &mut t,
        bears,
        vec![Modification::ModifyPT(Value::c(4), Value::c(4))],
    );
    t.g.add_counters(Entity::Object(giant), counters::PLUS1, 1, None);
    jest(&mut t, P1);
    assert_eq!(t.pt(bears), (5, 5));
    assert_eq!(t.pt(giant), (2, 2));
    modify_until_eot(
        &mut t,
        giant,
        vec![Modification::AddKeyword(KeywordKind::Flying.into())],
    );
    assert!(t.obj_now(giant).has_keyword(KeywordKind::Flying));
}

#[test]
fn polymorphists_jest_and_other_power_setting_effects_by_timestamp() {
    cr!("613.4b", "613.7");
    ruling!(
        "Polymorphist's Jest",
        "Polymorphist’s Jest overwrites all previous effects that set the creatures’ base power and toughness to specific values. Any power- or toughness-setting effects that start to apply after Polymorphist’s Jest resolves will overwrite this effect."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    modify_until_eot(
        &mut t,
        bears,
        vec![Modification::SetPT(Some(Value::c(5)), Some(Value::c(5)))],
    );
    jest(&mut t, P1);
    assert_eq!(t.pt(bears), (1, 1));
    modify_until_eot(
        &mut t,
        bears,
        vec![Modification::SetPT(Some(Value::c(3)), Some(Value::c(3)))],
    );
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn polymorphists_jest_keeps_card_types_and_supertypes() {
    cr!("205.1b", "613.1d");
    ruling!(
        "Polymorphist's Jest",
        "The creatures will lose all other colors and creature types, but they will retain any other card types (such as artifact) or supertypes (such as legendary) they may have."
    );
    let mut t = TestGame::new(2);
    let isamaru = t.battlefield(P1, "Isamaru, Hound of Konda");
    let thopter = t.battlefield(P1, "Ornithopter");
    jest(&mut t, P1);
    let o = t.obj_now(isamaru);
    assert!(o.chars.supertypes.contains(Supertype::Legendary));
    assert_eq!(subtypes(&t, isamaru), vec!["Frog"]);
    assert_eq!(colors(&mut t, isamaru), of(&[Color::Blue]));
    let o = t.obj_now(thopter);
    assert!(o.is(CardType::Artifact) && o.is(CardType::Creature));
    assert_eq!(subtypes(&t, thopter), vec!["Frog"]);
    assert!(!o.has_keyword(KeywordKind::Flying));
}

#[test]
fn polymorphists_jest_doesnt_stop_abilities_that_already_triggered() {
    cr!("113.7a", "603.3");
    ruling!(
        "Polymorphist's Jest",
        "Polymorphist’s Jest doesn’t counter abilities that have already triggered or been activated."
    );
    supported("Elvish Visionary");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Forest", 2);
    let ev = t.hand(P1, "Elvish Visionary");
    t.cast(P1, ev).go();
    t.resolve();
    t.settle();
    assert_eq!(t.stack_len(), 1, "the enters trigger");
    let hand = t.hand_size(P1);
    jest(&mut t, P1);
    assert_eq!(t.hand_size(P1), hand + 1);
}

#[test]
fn polymorphists_jest_on_a_god_applies_types_before_removing_abilities() {
    cr!("613.1d", "613.1f", "613.6", "700.5");
    ruling!(
        "Polymorphist's Jest",
        "If one of the Theros block Gods is affected by Polymorphist’s Jest, it will be a legendary 1/1 blue Frog enchantment creature with no abilities."
    );
    supported("Thassa, God of the Sea");
    let mut t = TestGame::new(2);
    let thassa = t.battlefield(P1, "Thassa, God of the Sea");
    let faeries: Vec<ObjectId> = (0..4).map(|_| t.battlefield(P1, "Indigo Faerie")).collect();
    t.g.recompute();
    assert!(t.obj_now(thassa).is(CardType::Creature));
    jest(&mut t, P1);
    let o = t.obj_now(thassa);
    assert!(o.is(CardType::Creature) && o.is(CardType::Enchantment));
    assert!(o.chars.supertypes.contains(Supertype::Legendary));
    assert_eq!(t.pt(thassa), (1, 1));
    assert_eq!(subtypes(&t, thassa), vec!["Frog"]);
    assert!(!t.obj_now(thassa).has_keyword(KeywordKind::Indestructible));
    // Devotion drops below five: it stops being a creature.
    for f in &faeries[..2] {
        t.g.destroy(*f, None);
    }
    t.settle();
    let o = t.obj_now(thassa);
    assert!(!o.is(CardType::Creature) && o.is(CardType::Enchantment));
    assert!(o.chars.supertypes.contains(Supertype::Legendary));
    assert_eq!(colors(&mut t, thassa), of(&[Color::Blue]));
}

// ---------------------------------------------------------------------------
// Overwriting and adding colors
// ---------------------------------------------------------------------------

#[test]
fn darkest_hour_makes_creatures_mono_black() {
    cr!("105.3", "613.1e");
    ruling!(
        "Darkest Hour",
        "Affected creatures lose all other colors and are mono-black."
    );
    supported("Darkest Hour");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Darkest Hour");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let thopter = t.battlefield(P1, "Ornithopter");
    assert_eq!(colors(&mut t, elves), of(&[Color::Black]));
    assert_eq!(colors(&mut t, thopter), of(&[Color::Black]));
}

#[test]
fn indigo_faerie_adds_blue_and_later_color_setting_overwrites_it() {
    cr!("105.3", "613.7");
    ruling!(
        "Indigo Faerie",
        "After Indigo Faerie’s ability resolves, an effect that changes the affected permanent’s colors will overwrite Indigo Faerie’s effect."
    );
    ruling!(
        "Indigo Faerie",
        "This ability doesn’t overwrite any previous colors. Rather, it adds another color."
    );
    ruling!(
        "Indigo Faerie",
        "If something affected by Indigo Faerie’s ability is normally colorless, it will simply be blue. It won’t be both blue and colorless."
    );
    supported("Indigo Faerie");
    let mut t = TestGame::new(2);
    let faerie = t.battlefield(P0, "Indigo Faerie");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let ring = t.battlefield(P1, "Sol Ring");
    t.lands(P0, "Island", 6);
    t.activate(P0, faerie, 0, &[Entity::Object(elves)])
        .expect("activate");
    t.activate(P0, faerie, 0, &[Entity::Object(ring)])
        .expect("activate");
    t.resolve_all();
    assert_eq!(colors(&mut t, elves), of(&[Color::Blue, Color::Green]));
    assert_eq!(colors(&mut t, ring), of(&[Color::Blue]));
    assert!(!colors(&mut t, ring).is_colorless());
    // Aphotic Wisps: just black.
    t.lands(P0, "Swamp", 1);
    let wisps = t.hand(P0, "Aphotic Wisps");
    t.cast(P0, wisps).target(elves).go();
    t.resolve_all();
    assert_eq!(colors(&mut t, elves), of(&[Color::Black]));
}

#[test]
fn scuttlemutt_overwrites_previous_colors() {
    cr!("105.3", "613.7");
    ruling!(
        "Scuttlemutt",
        "The target creature has only the colors Scuttlemutt gives it—Scuttlemutt’s ability overwrites any previous colors the creature had."
    );
    supported("Scuttlemutt");
    let mut t = TestGame::new(2);
    let mutt = t.battlefield(P0, "Scuttlemutt");
    let elves = t.battlefield(P1, "Llanowar Elves");
    choose_colors(&mut t, P0, of(&[Color::Red, Color::White]));
    t.activate(P0, mutt, 1, &[Entity::Object(elves)])
        .expect("activate");
    t.resolve_all();
    assert_eq!(colors(&mut t, elves), of(&[Color::Red, Color::White]));
}

#[test]
fn thran_lens_doesnt_make_artifacts_or_stop_later_colors() {
    cr!("105.2c", "613.7");
    ruling!(
        "Thran Lens",
        "Does not make the permanents into artifacts. They are simply without color."
    );
    ruling!(
        "Thran Lens",
        "Does not prevent a spell or ability from adding color to permanents after this effect is applied."
    );
    supported("Thran Lens");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Thran Lens");
    let elves = t.battlefield(P1, "Llanowar Elves");
    assert!(colors(&mut t, elves).is_colorless());
    assert!(!t.obj_now(elves).is(CardType::Artifact));
    let faerie = t.battlefield(P0, "Indigo Faerie");
    t.lands(P0, "Island", 1);
    t.activate(P0, faerie, 0, &[Entity::Object(elves)])
        .expect("activate");
    t.resolve_all();
    assert_eq!(colors(&mut t, elves), of(&[Color::Blue]));
}

#[test]
fn quickchange_and_prismwake_merrow_dont_stop_artifacts_being_artifacts() {
    cr!("105.3", "205.1b");
    ruling!(
        "Quickchange",
        "Quickchange won’t make an artifact stop being an artifact. It’ll just be a colorful artifact."
    );
    ruling!(
        "Prismwake Merrow",
        "This ability won’t make an artifact stop being an artifact. It’ll just be a colorful artifact."
    );
    supported("Quickchange");
    supported("Prismwake Merrow");
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P1, "Ornithopter");
    t.lands(P0, "Island", 2);
    choose_colors(&mut t, P0, of(&[Color::Red]));
    let qc = t.hand(P0, "Quickchange");
    t.cast(P0, qc).target(thopter).go();
    t.resolve_all();
    assert!(t.obj_now(thopter).is(CardType::Artifact));
    assert_eq!(colors(&mut t, thopter), of(&[Color::Red]));
    let ring = t.battlefield(P1, "Sol Ring");
    choose_colors(&mut t, P0, of(&[Color::Green, Color::White]));
    t.answer_targets(P0, &[Entity::Object(ring)]);
    t.enter(P0, "Prismwake Merrow");
    t.resolve_all();
    assert!(t.obj_now(ring).is(CardType::Artifact));
    assert_eq!(colors(&mut t, ring), of(&[Color::Green, Color::White]));
}

#[test]
fn eight_and_a_half_tails_whitens_a_permanent_spell_through_the_turn() {
    cr!("400.7a", "611.2a");
    ruling!(
        "Eight-and-a-Half-Tails",
        "A permanent spell that becomes white this way will enter the battlefield and continue to be white until end of turn."
    );
    supported("Eight-and-a-Half-Tails");
    let mut t = TestGame::new(2);
    let tails = t.battlefield(P0, "Eight-and-a-Half-Tails");
    t.lands(P0, "Plains", 1);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Forest", 2);
    let bears = t.hand(P1, "Grizzly Bears");
    let spell = t.cast(P1, bears).go();
    t.activate(P0, tails, 1, &[Entity::Object(spell)])
        .expect("activate");
    t.resolve_all();
    let perm = t.named_on_battlefield("Grizzly Bears")[0];
    assert_eq!(colors(&mut t, perm), of(&[Color::White]));
    next_turn(&mut t);
    assert_eq!(colors(&mut t, perm), of(&[Color::Green]));
}

#[test]
fn ersatz_gnomes_makes_a_permanent_spell_colorless_for_good() {
    cr!("400.7a", "611.2a");
    ruling!(
        "Ersatz Gnomes",
        "If Ersatz Gnomes targets a spell that becomes a permanent, the permanent will enter the battlefield colorless and will remain colorless until it leaves the battlefield"
    );
    supported("Ersatz Gnomes");
    let mut t = TestGame::new(2);
    let gnomes = t.battlefield(P0, "Ersatz Gnomes");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Forest", 2);
    let bears = t.hand(P1, "Grizzly Bears");
    let spell = t.cast(P1, bears).go();
    t.activate(P0, gnomes, 0, &[Entity::Object(spell)])
        .expect("activate");
    t.resolve_all();
    let perm = t.named_on_battlefield("Grizzly Bears")[0];
    assert!(colors(&mut t, perm).is_colorless());
    next_turn(&mut t);
    next_turn(&mut t);
    assert!(colors(&mut t, perm).is_colorless());
    // Another effect can still change its color.
    let faerie = t.battlefield(P0, "Indigo Faerie");
    t.lands(P0, "Island", 1);
    t.set_step(P0, Step::PrecombatMain);
    t.activate(P0, faerie, 0, &[Entity::Object(perm)])
        .expect("activate");
    t.resolve_all();
    assert_eq!(colors(&mut t, perm), of(&[Color::Blue]));
}

#[test]
fn grave_servitudes_color_change_lasts_only_while_attached() {
    cr!("303.4", "611.3a");
    ruling!(
        "Grave Servitude",
        "The color change lasts only while this card is on the creature."
    );
    supported("Grave Servitude");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let aura = t.battlefield(P0, "Grave Servitude");
    assert!(t.g.attach(aura, Entity::Object(bears)));
    t.g.recompute();
    assert_eq!(colors(&mut t, bears), of(&[Color::Black]));
    assert_eq!(t.pt(bears), (5, 1));
    t.g.destroy(aura, None);
    t.settle();
    assert_eq!(colors(&mut t, bears), of(&[Color::Green]));
}

// ---------------------------------------------------------------------------
// Lands that become creatures
// ---------------------------------------------------------------------------

/// Ignition Team with a +1/+1 counter animates `land`.
fn ignite(t: &mut TestGame, land: ObjectId) {
    supported("Ignition Team");
    let team = t.battlefield(P0, "Ignition Team");
    t.g.add_counters(Entity::Object(team), counters::PLUS1, 1, None);
    add_mana(t, P0, ManaType::R, 3);
    t.activate(P0, team, 0, &[Entity::Object(land)])
        .expect("activate");
    t.resolve_all();
}

#[test]
fn ignition_team_keeps_the_lands_name_types_and_abilities() {
    cr!("205.1b", "613.1d");
    ruling!(
        "Ignition Team",
        "Ignition Team’s ability doesn’t affect the land’s name or any other types, subtypes, or supertypes (such as basic or legendary) the land may have. The land will also keep any abilities it had."
    );
    let mut t = TestGame::new(2);
    let mountain = t.battlefield(P0, "Mountain");
    ignite(&mut t, mountain);
    let o = t.obj_now(mountain);
    assert_eq!(o.chars.name, "Mountain");
    assert!(o.is(CardType::Land) && o.is(CardType::Creature));
    assert!(o.chars.supertypes.contains(Supertype::Basic));
    assert!(o.chars.has_subtype("Mountain") && o.chars.has_subtype("Elemental"));
    assert_eq!(t.pt(mountain), (4, 4));
    assert_eq!(colors(&mut t, mountain), of(&[Color::Red]));
    assert!(tap_for_mana(&mut t, P0, mountain, "{R}"));
}

#[test]
fn ignition_team_keeps_power_changes_and_counters() {
    cr!("613.4c", "613.4d");
    ruling!(
        "Ignition Team",
        "Effects that modify power and/or toughness without setting them to a specific value (like the one created by Giant Growth), power and toughness changes from counters, and effects that switch a creature’s power and toughness will continue to apply."
    );
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    modify_until_eot(
        &mut t,
        forest,
        vec![Modification::ModifyPT(Value::c(3), Value::c(3))],
    );
    t.g.add_counters(Entity::Object(forest), counters::PLUS1, 1, None);
    ignite(&mut t, forest);
    assert_eq!(t.pt(forest), (8, 8));
}

#[test]
fn ignition_teams_land_has_summoning_sickness_if_it_just_arrived() {
    cr!("302.6", "302.1");
    ruling!(
        "Ignition Team",
        "If the target land hasn’t been under its controller’s control continuously since the beginning of their most recent turn, that land won’t be able to attack and its {T} abilities can’t be activated."
    );
    let mut t = TestGame::new(2);
    let new_land = t.enter(P0, "Mountain");
    t.settle();
    ignite(&mut t, new_land);
    assert!(!tap_for_mana(&mut t, P0, new_land, "{R}"));
    // A land P0 has controlled since the turn began can be tapped.
    let old = t.battlefield(P0, "Forest");
    ignite(&mut t, old);
    assert!(tap_for_mana(&mut t, P0, old, "{G}"));
}

#[test]
fn copies_of_an_ignited_land_are_just_the_land() {
    cr!("707.2", "613.2");
    ruling!(
        "Ignition Team",
        "Copies of the affected land won’t be 4/4 red Elemental creatures. For example, if the target land is a basic Mountain, a copy of that creature would just be a basic Mountain."
    );
    let mut t = TestGame::new(2);
    let mountain = t.battlefield(P0, "Mountain");
    ignite(&mut t, mountain);
    let copy = token_copy(&mut t, P0, mountain)[0];
    let o = t.obj_now(copy);
    assert_eq!(o.chars.name, "Mountain");
    assert!(o.is(CardType::Land) && !o.is(CardType::Creature));
    assert!(!o.chars.has_subtype("Elemental"));
}

#[test]
fn woodwraith_corrupters_forest_stays_a_creature() {
    cr!("611.2a", "302.6");
    ruling!(
        "Woodwraith Corrupter",
        "The effect has no stated duration, so a land turned into a creature this way continues being a creature as long as the land is on the battlefield."
    );
    supported("Woodwraith Corrupter");
    let mut t = TestGame::new(2);
    let wraith = t.battlefield(P0, "Woodwraith Corrupter");
    let forest = t.battlefield(P1, "Forest");
    t.lands(P0, "Bayou", 3);
    t.activate(P0, wraith, 0, &[Entity::Object(forest)])
        .expect("activate");
    t.resolve_all();
    assert_eq!(t.pt(forest), (4, 4));
    next_turn(&mut t);
    next_turn(&mut t);
    let o = t.obj_now(forest);
    assert!(o.is(CardType::Creature) && o.is(CardType::Land));
    assert_eq!(colors(&mut t, forest), of(&[Color::Black, Color::Green]));
}

#[test]
fn kormus_bell_animates_every_players_swamps_which_stay_lands() {
    cr!("205.2a", "302.6");
    ruling!(
        "Kormus Bell",
        "It affects Swamps controlled by any and all players."
    );
    ruling!(
        "Kormus Bell",
        "The lands are both lands and creatures at the same time. They are affected by anything that affects either permanent type."
    );
    supported("Kormus Bell");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kormus Bell");
    let mine = t.battlefield(P0, "Swamp");
    let theirs = t.battlefield(P1, "Swamp");
    for s in [mine, theirs] {
        let o = t.obj_now(s);
        assert!(o.is(CardType::Land) && o.is(CardType::Creature));
        assert_eq!(t.pt(s), (1, 1));
    }
    // A creature spell's effect destroys it.
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(theirs).go();
    t.resolve_all();
    assert!(!t.on_battlefield(theirs));
}

#[test]
fn living_terrain_gives_any_land_a_color_and_creature_type() {
    cr!("303.4", "205.3d");
    ruling!(
        "Living Terrain",
        "It does give the land a color and creature type, which is unlike most other ways to animate a land."
    );
    ruling!(
        "Living Terrain",
        "This can be placed on any land, not just Forests."
    );
    supported("Living Terrain");
    let mut t = TestGame::new(2);
    let island = t.battlefield(P0, "Island");
    t.lands(P0, "Forest", 4);
    let aura = t.hand(P0, "Living Terrain");
    t.cast(P0, aura).target(island).go();
    t.resolve_all();
    let o = t.obj_now(island);
    assert!(o.is(CardType::Land) && o.is(CardType::Creature));
    assert!(o.chars.has_subtype("Treefolk") && o.chars.has_subtype("Island"));
    assert_eq!(colors(&mut t, island), of(&[Color::Green]));
    assert_eq!(t.pt(island), (5, 6));
}

#[test]
fn vodalian_mystic_targets_only_instant_and_sorcery_spells() {
    cr!("115.1", "702.8a");
    ruling!(
        "Vodalian Mystic",
        "It can target spells of type instant or sorcery, and not spells of other types that say they can be cast “any time you could cast” an instant or sorcery."
    );
    supported("Vodalian Mystic");
    let mut t = TestGame::new(2);
    let mystic = t.battlefield(P0, "Vodalian Mystic");
    t.lands(P1, "Forest", 2);
    let viper = t.hand(P1, "Ambush Viper");
    let viper = t.cast(P1, viper).go();
    let from = t.asked().len();
    let _ = t.activate(P0, mystic, 0, &[Entity::Object(viper)]);
    let offered = crate::r_s02_common::target_candidates(&t, P0, from);
    assert!(offered.iter().all(|c| !c.contains(&Entity::Object(viper))));
    t.g.stack.retain(|s| *s == viper);
    t.resolve_all();
    // An instant spell can be targeted.
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let bolt = t.cast(P1, bolt).target(P0).go();
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    let mystic = t.g.current(mystic);
    t.g.untap(mystic);
    assert!(t.activate(P0, mystic, 0, &[Entity::Object(bolt)]).is_ok());
    t.resolve();
    assert_eq!(colors(&mut t, bolt), of(&[Color::White]));
    let _ = Zone::Stack;
}

// ---------------------------------------------------------------------------
// Neurok Transmuter: "isn't an artifact"
// ---------------------------------------------------------------------------

/// P0's Neurok Transmuter makes `target` blue and not an artifact until end of turn.
fn transmute(t: &mut TestGame, target: ObjectId) {
    supported("Neurok Transmuter");
    let nt = t.battlefield(P0, "Neurok Transmuter");
    add_mana(t, P0, ManaType::U, 1);
    t.activate(P0, nt, 1, &[Entity::Object(target)])
        .expect("activate");
    t.resolve_all();
}

#[test]
fn neurok_transmuter_removes_only_artifact_and_its_subtypes() {
    cr!("205.1b", "613.1d", "105.3");
    ruling!(
        "Neurok Transmuter",
        "Neurok Transmuter’s second ability removes the type “artifact” — and any artifact subtypes — from the artifact creature it targets. It doesn’t remove any other types or any subtypes of other types, and it doesn’t remove supertypes."
    );
    supported("Karn, Silver Golem");
    let mut t = TestGame::new(2);
    let karn = t.battlefield(P1, "Karn, Silver Golem");
    transmute(&mut t, karn);
    let o = t.obj_now(karn);
    assert!(!o.is(CardType::Artifact) && o.is(CardType::Creature));
    assert!(o.chars.supertypes.contains(Supertype::Legendary));
    assert!(o.chars.has_subtype("Golem"));
    assert_eq!(colors(&mut t, karn), of(&[Color::Blue]));
    // Still legendary: the legend rule applies normally.
    t.battlefield(P1, "Karn, Silver Golem");
    t.settle();
    assert_eq!(t.named_on_battlefield("Karn, Silver Golem").len(), 1);
    // Until end of turn.
    next_turn(&mut t);
    let karn = t.named_on_battlefield("Karn, Silver Golem")[0];
    assert!(t.obj_now(karn).is(CardType::Artifact));
}

#[test]
fn equipment_that_stops_being_an_artifact_isnt_equipment() {
    cr!("205.1b", "301.5");
    ruling!(
        "Neurok Transmuter",
        "Equipment that stops being an artifact loses the subtype “Equipment.” Permanents without the subtype Equipment can’t equip creatures."
    );
    let mut t = TestGame::new(2);
    let splitter = t.battlefield(P0, "Bonesplitter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    animate(&mut t, splitter, 2);
    transmute(&mut t, splitter);
    let o = t.obj_now(splitter);
    assert!(!o.is(CardType::Artifact));
    assert!(!o.chars.has_subtype("Equipment"));
    add_mana(&mut t, P0, ManaType::C, 1);
    let _ = t.activate(P0, splitter, 0, &[Entity::Object(bears)]);
    t.resolve_all();
    assert_eq!(t.obj_now(splitter).attached_to, None);
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn neurok_transmuter_on_a_march_of_the_machines_creature_leaves_no_types() {
    cr!("613.8a", "613.8b", "613.1d");
    ruling!(
        "Neurok Transmuter",
        "If an artifact is an artifact creature only because March of the Machines is on the battlefield and you then activate Neurok Transmuter’s second ability on that artifact creature, the result is a permanent with no types whatsoever."
    );
    supported("March of the Machines");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "March of the Machines");
    let ring = t.battlefield(P1, "Sol Ring");
    t.g.recompute();
    assert!(t.obj_now(ring).is(CardType::Creature));
    transmute(&mut t, ring);
    let o = t.obj_now(ring);
    assert!(t.on_battlefield(ring));
    assert!(o.chars.card_types.is_empty(), "{:?}", o.chars.card_types);
}
