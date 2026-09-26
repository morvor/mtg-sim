//! Plural pronouns (patterns in `src/oracle/patterns/pronoun_groups.rs`): "they", "them",
//! "those creatures", "each of them" after an instruction that affected a group ("Untap
//! all creatures you control. They gain ..."), several targets ("They each get +2/+2"),
//! or the targets of a triggering spell; "them" meaning a player; and "it" after a group
//! instruction, which still means what it meant before.

use mtg_engine::card::{CardDef, Layout};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{Characteristics, Zone};
use mtg_engine::oracle::{self, CompileContext};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;
use smol_str::SmolStr;
use std::sync::Arc;

fn assert_supported(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

fn has(t: &TestGame, id: ObjectId, k: KeywordKind) -> bool {
    t.obj_now(id).has_keyword(k)
}

fn tapped(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).tapped
}

fn tap(t: &mut TestGame, id: ObjectId) {
    t.g.tap(id);
    t.g.flush_events();
}

/// A card compiled from oracle text with the real compiler; `Err` holds the text it
/// doesn't understand.
fn compile_card(name: &str, type_line: &str, cost: &str, text: &str) -> Result<CardDef, Vec<String>> {
    let tl = TypeLine::parse(type_line);
    let keywords = ["Kicker".to_string()];
    let ctx = CompileContext {
        card_name: name,
        full_name: name,
        type_line: &tl,
        layout: Layout::Normal,
        face_index: 0,
        keywords: &keywords,
        power: None,
        toughness: None,
    };
    let compiled = oracle::compile(text, &ctx);
    if !compiled.unsupported.is_empty() {
        return Err(compiled.unsupported);
    }
    let m = mtg_engine::mana::ManaCost::parse(cost);
    Ok(CardDef::custom(Characteristics {
        name: SmolStr::new(name),
        colors: m.as_ref().map_or(ColorSet::NONE, |m| m.colors()),
        mana_cost: m,
        card_types: tl.card_types,
        abilities: compiled.abilities,
        rules_text: Arc::from(text),
        ..Default::default()
    }))
}

// ---------------------------------------------------------------------------
// Groups: "Untap them. They gain haste until end of turn."
// ---------------------------------------------------------------------------

#[test]
fn them_are_the_artifacts_whose_control_was_gained() {
    cr!("611.2c", "608.2c", "701.26b");
    assert_supported(&[
        "Broadcast Takeover",
        "Rowan, Fearless Sparkmage",
        "Mob Rule",
    ]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 5);
    // "Artifacts your opponents control" matches nothing once you control them: "them"
    // are the artifacts the control change affected.
    let theirs = t.battlefield(P1, "Ornithopter");
    tap(&mut t, theirs);
    let mine = t.battlefield(P0, "Ornithopter");
    tap(&mut t, mine);
    let spell = t.hand(P0, "Broadcast Takeover");
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(t.obj_now(theirs).controller, P0);
    assert!(!tapped(&t, theirs));
    assert!(has(&t, theirs, KeywordKind::Haste));
    // Not one of them.
    assert!(tapped(&t, mine));
    assert!(!has(&t, mine, KeywordKind::Haste));
}

#[test]
fn untap_all_creatures_and_gain_control_of_them() {
    cr!("611.2c", "608.2c");
    ruling!(
        "Insurrection",
        "You untap all creatures, control all creatures, and give all creatures haste."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 8);
    let bears = t.battlefield(P1, "Grizzly Bears");
    tap(&mut t, bears);
    let giant = t.battlefield(P0, "Hill Giant");
    tap(&mut t, giant);
    let spell = t.hand(P0, "Insurrection");
    t.cast(P0, spell).go();
    t.resolve_all();
    for id in [bears, giant] {
        assert_eq!(t.obj_now(id).controller, P0);
        assert!(!tapped(&t, id));
        assert!(has(&t, id, KeywordKind::Haste));
    }
}

#[test]
fn untap_all_creatures_you_control_they_gain_hexproof_and_indestructible() {
    cr!("611.2c", "701.26b");
    ruling!(
        "Join Shields",
        "Untapped creatures you control can’t be untapped again, but those creatures still gain hexproof and indestructible."
    );
    assert_supported(&["Join Shields", "Flying Crane Technique"]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    t.lands(P0, "Plains", 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    tap(&mut t, bears);
    let giant = t.battlefield(P0, "Hill Giant");
    let angel = t.battlefield(P1, "Serra Angel");
    tap(&mut t, angel);
    let spell = t.hand(P0, "Join Shields");
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(!tapped(&t, bears));
    for id in [bears, giant] {
        assert!(has(&t, id, KeywordKind::Hexproof));
        assert!(has(&t, id, KeywordKind::Indestructible));
    }
    assert!(tapped(&t, angel));
    assert!(!has(&t, angel, KeywordKind::Hexproof));
}

#[test]
fn untap_them_after_a_pump_untaps_the_creatures_not_the_spell() {
    cr!("611.2c", "608.2c", "701.26b");
    assert_supported(&[
        "Rallying Roar",
        "Gleam of Resistance",
        "Jeskai Ascendancy",
        "War Flare",
        "Tenacity",
        "Great Oak Guardian",
    ]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    tap(&mut t, bears);
    let angel = t.battlefield(P1, "Serra Angel");
    tap(&mut t, angel);
    let spell = t.hand(P0, "Rallying Roar");
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 3));
    assert!(!tapped(&t, bears));
    assert!(tapped(&t, angel));
    assert_eq!(t.pt(angel), (4, 4));
}

#[test]
fn those_creatures_get_more_instead_if_kicked() {
    cr!("608.2c", "611.2c", "702.33d");
    for (kicked, pt) in [(false, (3, 3)), (true, (4, 3))] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Plains", 4);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let spell = t.hand(P0, "Dauntless Unity");
        t.cast(P0, spell).kicked(kicked).go();
        t.resolve_all();
        assert_eq!(t.pt(bears), pt, "kicked: {kicked}");
    }
}

#[test]
fn if_kicked_they_get_more() {
    cr!("608.2c", "702.33d");
    for (kicked, pt) in [(false, (2, 2)), (true, (3, 3))] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Mountain", 2);
        t.lands(P0, "Forest", 1);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let spell = t.hand(P0, "Savage Offensive");
        t.cast(P0, spell).kicked(kicked).go();
        t.resolve_all();
        assert_eq!(t.pt(bears), pt, "kicked: {kicked}");
        assert!(has(&t, bears, KeywordKind::FirstStrike));
    }
}

#[test]
fn it_after_a_group_is_still_the_spell() {
    cr!("608.2c", "702.16e");
    // "~ deals 1 damage to each creature. If it was kicked, it deals 2 damage to each
    // creature instead.": the damage is still the red spell's, so protection from red
    // prevents it.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    t.battlefield(P0, "Grizzly Bears");
    let falcon = t.battlefield(P1, "Freewind Falcon");
    let giant = t.battlefield(P1, "Hill Giant");
    let spell = t.hand(P0, "Cinderclasm");
    t.cast(P0, spell).kicked(true).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"), "{}", t.dump_log());
    assert!(t.on_battlefield(falcon));
    assert_eq!(t.obj_now(falcon).damage, 0);
    assert_eq!(t.obj_now(giant).damage, 2);
}

#[test]
fn tap_those_creatures_after_damaging_each_of_them() {
    cr!("608.2c", "702.16e", "701.26a");
    ruling!(
        "Thundermaw Hellkite",
        "whether those creatures are tapped or untapped"
    );
    ruling!(
        "Thundermaw Hellkite",
        "If the damage that would be dealt to a creature with flying is prevented, that creature will still be tapped."
    );
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P1, "Serra Angel");
    let drake = t.battlefield(P1, "Wind Drake");
    tap(&mut t, drake);
    let falcon = t.battlefield(P1, "Freewind Falcon");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let own = t.battlefield(P0, "Wind Drake");
    t.enter(P0, "Thundermaw Hellkite");
    t.resolve_all();
    assert_eq!(t.obj_now(angel).damage, 1);
    assert_eq!(t.obj_now(drake).damage, 1);
    assert!(tapped(&t, angel) && tapped(&t, drake));
    // Protection from red prevents the damage; it's still tapped.
    assert_eq!(t.obj_now(falcon).damage, 0);
    assert!(tapped(&t, falcon));
    assert!(!tapped(&t, bears));
    assert!(!tapped(&t, own));
    assert_eq!(t.obj_now(own).damage, 0);
}

#[test]
fn double_the_power_of_each_then_those_creatures_gain_vigilance() {
    cr!("611.2c", "608.2h");
    ruling!(
        "God-Eternal Rhonas",
        "Creatures you begin to control later in the turn won't have their power doubled or gain vigilance."
    );
    ruling!(
        "God-Eternal Rhonas",
        "that creature gets +X/+0, where X is its power as that effect begins to apply"
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Wind Drake");
    let rhonas = t.enter(P0, "God-Eternal Rhonas");
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 2));
    assert_eq!(t.pt(giant), (6, 3));
    assert!(has(&t, bears, KeywordKind::Vigilance) && has(&t, giant, KeywordKind::Vigilance));
    assert!(!has(&t, rhonas, KeywordKind::Vigilance));
    assert!(!has(&t, theirs, KeywordKind::Vigilance));
    let later = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(later), (2, 2));
    assert!(!has(&t, later, KeywordKind::Vigilance));
}

#[test]
fn counters_on_each_then_they_gain_vigilance() {
    cr!("611.2c", "122.1a");
    assert_supported(&[
        "Now for Wrath, Now for Ruin!",
        "Ardbert, Warrior of Darkness",
        "Captain America, Skybound",
        "Sandstorm Salvager",
    ]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 4);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let spell = t.hand(P0, "Now for Wrath, Now for Ruin!");
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    assert_eq!(t.pt(bears), (3, 3));
    assert!(has(&t, bears, KeywordKind::Vigilance));
    assert_eq!(t.counters(theirs, "+1/+1"), 0);
    assert!(!has(&t, theirs, KeywordKind::Vigilance));
}

#[test]
fn they_gain_haste_after_returning_them_to_the_battlefield() {
    cr!("400.7", "611.2c");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 6);
    t.lands(P0, "Plains", 1);
    let thopter = t.graveyard(P0, "Ornithopter");
    let spell = t.hand(P0, "Wake the Past");
    t.cast(P0, spell).go();
    t.resolve_all();
    // The permanent the card became (CR 400.7).
    assert!(t.on_battlefield(thopter));
    assert!(has(&t, thopter, KeywordKind::Haste));
}

#[test]
fn they_are_the_group_only_if_the_conditional_instruction_happened() {
    cr!("608.2c", "702.33d");
    // As Hunting Wilds: "If this spell was kicked, untap all Forests put onto the
    // battlefield this way. They become 3/3 green creatures with haste that are still
    // lands." (ruling: "If the Hunting Wilds is kicked, the Forests become creatures").
    let def = compile_card(
        "Rally the Flock",
        "Sorcery",
        "{G}",
        "Kicker {1}\nIf this spell was kicked, untap all creatures you control. They gain flying until end of turn.",
    )
    .expect("compiles");
    for kicked in [false, true] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Forest", 2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        tap(&mut t, bears);
        let spell = t.custom(P0, def.clone(), Zone::Hand(P0));
        t.cast(P0, spell).kicked(kicked).go();
        t.resolve_all();
        assert_eq!(tapped(&t, bears), !kicked, "kicked: {kicked}");
        assert_eq!(has(&t, bears, KeywordKind::Flying), kicked, "kicked: {kicked}");
    }
}

#[test]
fn those_creatures_after_a_qualified_spell_target_is_not_accepted() {
    cr!("115.1a", "603.2");
    // "Those creatures" are the targets that are creatures you control, which "the
    // creatures the spell targets" isn't: the compiler must not accept the text rather
    // than give flying to an opponent's targeted creature too.
    let r = compile_card(
        "Wind Guide",
        "Enchantment",
        "{U}",
        "Whenever you cast a spell that targets one or more creatures you control, those creatures gain flying until end of turn.",
    );
    assert!(r.is_err());
    // Unqualified, it's supported (as Storm, Windrider).
    compile_card(
        "Wind Guide",
        "Enchantment",
        "{U}",
        "Whenever you cast a spell that targets one or more creatures, those creatures gain flying until end of turn.",
    )
    .expect("compiles");
}

// ---------------------------------------------------------------------------
// Several targets: "They each get +2/+2", "Put a stun counter on each of them."
// ---------------------------------------------------------------------------

#[test]
fn untap_up_to_two_target_creatures_they_each_get_bigger() {
    cr!("115.1a", "701.26b");
    ruling!(
        "Synchronized Strike",
        "Synchronized Strike can target an untapped creature. It will still get +2/+2."
    );
    assert_supported(&["Synchronized Strike", "Fancy Footwork", "Join Forces", "Hope and Glory"]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    tap(&mut t, bears);
    let giant = t.battlefield(P0, "Hill Giant");
    let spell = t.hand(P0, "Synchronized Strike");
    t.cast(P0, spell)
        .targets(&[Entity::Object(bears), Entity::Object(giant)])
        .go();
    t.resolve_all();
    assert!(!tapped(&t, bears));
    assert_eq!(t.pt(bears), (4, 4));
    assert_eq!(t.pt(giant), (5, 5));
}

#[test]
fn tap_targets_and_put_a_stun_counter_on_each_of_them() {
    cr!("115.1a", "122.1d", "701.26a");
    assert_supported(&[
        "Succumb to the Cold",
        "Out Cold",
        "Homesickness",
        "Twisted Riddlekeeper",
        "Donatello, Rad Scientist",
    ]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    let angel = t.battlefield(P1, "Serra Angel");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let spell = t.hand(P0, "Succumb to the Cold");
    t.cast(P0, spell)
        .targets(&[Entity::Object(angel), Entity::Object(bears)])
        .go();
    t.resolve_all();
    for id in [angel, bears] {
        assert!(tapped(&t, id));
        assert_eq!(t.counters(id, "stun"), 1);
    }
    assert!(!tapped(&t, giant));
    assert_eq!(t.counters(giant, "stun"), 0);
    // Their controller's untap step removes the stun counters instead of untapping them.
    t.advance_to(P1, Step::Upkeep);
    for id in [angel, bears] {
        assert!(tapped(&t, id));
        assert_eq!(t.counters(id, "stun"), 0);
    }
}

// ---------------------------------------------------------------------------
// The targets of a triggering spell; "them" as a player; a pair's own name
// ---------------------------------------------------------------------------

#[test]
fn those_creatures_are_the_spells_targets() {
    cr!("115.1a", "603.2", "611.2c");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    t.battlefield(P0, "Storm, Windrider");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let growth = t.hand(P0, "Giant Growth");
    t.cast(P0, growth).target(bears).go();
    // The trigger resolves first, while the spell is still on the stack.
    t.resolve();
    assert!(has(&t, bears, KeywordKind::Flying));
    assert!(!has(&t, giant, KeywordKind::Flying));
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5));
    assert!(has(&t, bears, KeywordKind::Flying));
}

#[test]
fn dack_fayden_emblem_gains_control_of_the_targeted_permanents() {
    cr!("114.1", "115.1a", "603.2");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    let dack = t.battlefield(P0, "Dack Fayden");
    t.g.objects[dack.0 as usize]
        .counters
        .insert("loyalty".into(), 6);
    t.activate(P0, dack, 2, &[]).expect("-6");
    t.resolve_all();
    let bears = t.battlefield(P1, "Grizzly Bears");
    let angel = t.battlefield(P1, "Serra Angel");
    let growth = t.hand(P0, "Giant Growth");
    t.cast(P0, growth).target(bears).go();
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P0);
    assert_eq!(t.obj_now(angel).controller, P1);
}

#[test]
fn them_is_the_opponent_who_drew() {
    cr!("603.2", "120.3a");
    ruling!(
        "Razorkin Needlehead",
        "If a spell or ability causes an opponent to put cards into their hand without specifically using the word"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Razorkin Needlehead");
    t.g.draw_cards(P1, 1);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.life(P0), 20);
    // Putting a card into their hand isn't drawing it.
    let bears = t.graveyard(P1, "Grizzly Bears");
    t.g.move_object(
        bears,
        mtg_engine::object::Zone::Hand(P1),
        mtg_engine::events::MoveCause::Effect,
        Some(P1),
    );
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn them_is_the_opponent_who_sacrificed() {
    cr!("603.2", "120.3a", "701.21a");
    assert_supported(&["Vengeful Tracker"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vengeful Tracker");
    let thopter = t.battlefield(P1, "Ornithopter");
    t.g.sacrifice(thopter, P1);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn a_pair_is_them() {
    cr!("603.2e", "701.26b");
    let mut t = TestGame::new(2);
    let tui = t.battlefield(P0, "Tui and La, Moon and Ocean");
    tap(&mut t, tui);
    t.resolve_all();
    t.g.untap(tui);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.counters(tui, "+1/+1"), 1);
}
