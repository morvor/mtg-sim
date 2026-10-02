//! Rulings batch P161 — permanents that become copies of other permanents for a while:
//! Crystalline Resonance ("Whenever you cycle a card, you may have this enchantment become
//! a copy of another target permanent until your next turn, except it has this ability.")
//! and Mizzium Transreliquat ("{3}: This artifact becomes a copy of target artifact until
//! end of turn." / "{1}{U}{R}: This artifact becomes a copy of target artifact, except it
//! has this ability."). Copiable values (CR 707.2), copy effects in layer 1 (CR 613.2),
//! durations (CR 611.2), linked abilities (CR 607).

use crate::r_p160_common::*;
use crate::r_s04_common::cycle;
use mtg_engine::ability::AbilityKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const RESONANCE: &str = "Crystalline Resonance";
const RELIQUAT: &str = "Mizzium Transreliquat";

fn name_of(t: &TestGame, id: ObjectId) -> String {
    t.obj_now(id).chars.name.to_string()
}

fn activated_count(t: &TestGame, id: ObjectId) -> usize {
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .count()
}

fn triggered_count(t: &TestGame, id: ObjectId) -> usize {
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Triggered(_)))
        .count()
}

/// Cycles a Barren Moor and puts the Resonance trigger (targeting `target`) on the stack.
fn cycle_targeting(t: &mut TestGame, target: ObjectId) {
    t.lands(P0, "Swamp", 1);
    let moor = t.hand(P0, "Barren Moor");
    t.answer_targets(P0, &[Entity::Object(target)]);
    cycle(t, P0, moor, 0).expect("cycling");
    t.settle();
}

/// Cycles and has Crystalline Resonance become a copy of `target`.
fn resonate(t: &mut TestGame, target: ObjectId) {
    cycle_targeting(t, target);
    yes(t, P0);
    t.resolve_all();
}

/// Passes to the opponent's turn, then to the start of P0's next turn.
fn next_own_turn(t: &mut TestGame) {
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    // State-based actions before P0 gets priority (CR 117.5).
    t.settle();
}

#[test]
fn resonance_copies_only_copiable_values() {
    cr!("707.2", "613.2a", "122.1");
    ruling!(
        "Crystalline Resonance",
        "Crystalline Resonance copies the printed values of the target permanent, plus any copy effects that have been applied to it. It won’t copy counters on that permanent or effects that have changed its power, toughness, types, color, or so on. Notably, it won’t copy effects that made the target permanent become a creature."
    );
    supported(RESONANCE);
    supported("Barren Moor");
    supported("Majestic Metamorphosis");
    // A pumped Grizzly Bears with a counter: just a 2/2 Grizzly Bears.
    let mut t = TestGame::new(2);
    let res = t.battlefield(P0, RESONANCE);
    let bears = t.battlefield(P0, "Grizzly Bears");
    plus_counters(&mut t, bears, 1);
    cast_resolve(&mut t, P0, "Giant Growth", &[Entity::Object(bears)]);
    assert_eq!(t.pt(bears), (6, 6));
    resonate(&mut t, bears);
    assert_eq!(name_of(&t, res), "Grizzly Bears");
    assert_eq!(t.pt(res), (2, 2));
    assert_eq!(t.counters(t.g.current(res), counters::PLUS1), 0);
    // A Sol Ring made a creature: just a noncreature Sol Ring.
    let mut t = TestGame::new(2);
    let res = t.battlefield(P0, RESONANCE);
    let ring = t.battlefield(P0, "Sol Ring");
    cast_resolve(&mut t, P0, "Majestic Metamorphosis", &[Entity::Object(ring)]);
    assert!(t.obj(ring).is(CardType::Creature));
    resonate(&mut t, ring);
    assert_eq!(name_of(&t, res), "Sol Ring");
    assert!(t.obj_now(res).is(CardType::Artifact));
    assert!(!t.obj_now(res).is(CardType::Creature));
    assert!(!t.obj_now(res).is(CardType::Enchantment));
}

#[test]
fn resonance_wears_off_as_your_next_turn_begins() {
    cr!("611.2a", "613.2a");
    ruling!(
        "Crystalline Resonance",
        "Crystalline Resonance’s copy effect wears off immediately before your next turn begins."
    );
    supported(RESONANCE);
    let mut t = TestGame::new(2);
    let res = t.battlefield(P0, RESONANCE);
    let ring = t.battlefield(P0, "Sol Ring");
    resonate(&mut t, ring);
    assert_eq!(name_of(&t, res), "Sol Ring");
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(name_of(&t, res), "Sol Ring");
    t.advance_to(P1, Step::End);
    assert_eq!(name_of(&t, res), "Sol Ring");
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(name_of(&t, res), RESONANCE);
}

#[test]
fn resonance_copying_a_legend_you_control_triggers_the_legend_rule() {
    cr!("704.5j", "707.2");
    ruling!(
        "Crystalline Resonance",
        "If Crystalline Resonance becomes a copy of a legendary permanent you control, you’ll put one of them into its owner’s graveyard."
    );
    supported(RESONANCE);
    supported("Karn, Silver Golem");
    let mut t = TestGame::new(2);
    t.battlefield(P0, RESONANCE);
    let karn = t.battlefield(P0, "Karn, Silver Golem");
    resonate(&mut t, karn);
    assert_eq!(t.named_on_battlefield("Karn, Silver Golem").len(), 1);
    let in_gy = t
        .g
        .player(P0)
        .graveyard
        .iter()
        .any(|id| matches!(t.obj(*id).chars.name.as_str(), "Karn, Silver Golem" | RESONANCE));
    assert!(in_gy);
}

#[test]
fn resonance_copying_a_planeswalker_has_no_loyalty() {
    cr!("704.5i", "306.5b", "707.2");
    ruling!(
        "Crystalline Resonance",
        "If Crystalline Resonance becomes a copy of a planeswalker, it won’t receive loyalty counters for that planeswalker’s starting loyalty. Unless it already has loyalty counters on it somehow, it will be put into its owner’s graveyard."
    );
    supported(RESONANCE);
    supported("Oko, Thief of Crowns");
    let mut t = TestGame::new(2);
    let res = t.battlefield(P0, RESONANCE);
    let oko = t.battlefield(P1, "Oko, Thief of Crowns");
    resonate(&mut t, oko);
    assert!(!t.on_battlefield(res));
    assert!(t.in_graveyard(P0, RESONANCE));
    assert!(t.on_battlefield(oko));
}

#[test]
fn resonance_copying_an_aura_or_equipment() {
    cr!("704.5m", "704.5n", "301.5c", "707.2");
    ruling!(
        "Crystalline Resonance",
        "If Crystalline Resonance becomes a copy of an Aura, it’s put into its owner’s graveyard unless it’s somehow attached to an appropriate object or player already. If it becomes a copy of an Equipment and is attached to a creature, it’ll become unattached when it becomes an enchantment again."
    );
    supported(RESONANCE);
    supported("Pacifism");
    supported("Bonesplitter");
    // An unattached Aura: it goes to the graveyard.
    let mut t = TestGame::new(2);
    let res = t.battlefield(P0, RESONANCE);
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Pacifism", &[Entity::Object(bears)]);
    let pacifism = t.named_on_battlefield("Pacifism")[0];
    resonate(&mut t, pacifism);
    assert!(!t.on_battlefield(res));
    assert!(t.in_graveyard(P0, RESONANCE));
    // An Equipment: equipped to a creature, then unattached when it stops being a copy.
    let mut t = TestGame::new(2);
    let res = t.battlefield(P0, RESONANCE);
    let splitter = t.battlefield(P0, "Bonesplitter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    resonate(&mut t, splitter);
    assert_eq!(name_of(&t, res), "Bonesplitter");
    t.lands(P0, "Wastes", 1);
    activate_resolve(&mut t, P0, res, 0, &[Entity::Object(bears)]);
    let res = t.g.current(res);
    assert_eq!(t.obj(res).attached_to, Some(Entity::Object(bears)));
    assert_eq!(t.pt(bears), (4, 2));
    next_own_turn(&mut t);
    let res = t.g.current(res);
    assert!(t.on_battlefield(res));
    assert_eq!(name_of(&t, res), RESONANCE);
    assert_eq!(t.obj(res).attached_to, None);
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn resonance_linked_abilities_last_only_while_copying() {
    cr!("607.2a", "607.1", "707.2");
    ruling!(
        "Crystalline Resonance",
        "If Crystalline Resonance becomes a copy of an object with a set of linked abilities (for example, one ability that exiles a card and another that refers to the card “exiled with” the object), that link only lasts as long as Crystalline Resonance is copying that object. If it stops being a copy of that object and then becomes a copy again later, the link is lost."
    );
    supported(RESONANCE);
    supported("Cold Storage");
    // Cold Storage: "{3}: Exile target creature you control." / "Sacrifice this artifact:
    // Return each creature card exiled with this artifact to the battlefield under your
    // control."
    // While still a copy, the link works.
    let mut t = TestGame::new(2);
    let res = t.battlefield(P0, RESONANCE);
    let storage = t.battlefield(P0, "Cold Storage");
    let bears = t.battlefield(P0, "Grizzly Bears");
    resonate(&mut t, storage);
    t.lands(P0, "Wastes", 3);
    activate_resolve(&mut t, P0, res, 0, &[Entity::Object(bears)]);
    assert!(t.in_exile("Grizzly Bears"));
    activate_resolve(&mut t, P0, res, 1, &[]);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    // Stopping being a copy and becoming one again loses the link.
    let mut t = TestGame::new(2);
    let res = t.battlefield(P0, RESONANCE);
    let storage = t.battlefield(P0, "Cold Storage");
    let bears = t.battlefield(P0, "Grizzly Bears");
    resonate(&mut t, storage);
    t.lands(P0, "Wastes", 3);
    activate_resolve(&mut t, P0, res, 0, &[Entity::Object(bears)]);
    assert!(t.in_exile("Grizzly Bears"));
    next_own_turn(&mut t);
    t.advance_to(P0, Step::PrecombatMain);
    assert_eq!(name_of(&t, res), RESONANCE);
    resonate(&mut t, storage);
    assert_eq!(name_of(&t, res), "Cold Storage");
    activate_resolve(&mut t, P0, res, 1, &[]);
    assert!(!t.on_battlefield(res));
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    assert!(t.in_exile("Grizzly Bears"));
}

#[test]
fn resonance_becoming_a_creature_the_turn_it_entered_is_summoning_sick() {
    cr!("302.6", "707.2");
    ruling!(
        "Crystalline Resonance",
        "If Crystalline Resonance becomes a creature the same turn it enters the battlefield, you can’t attack with it or use any of its {T} abilities (if it gains any) unless it has haste."
    );
    supported(RESONANCE);
    supported("Prodigal Sorcerer");
    // Prodigal Sorcerer: "{T}: This creature deals 1 damage to any target."
    let mut t = TestGame::new(2);
    let res = t.battlefield_sick(P0, RESONANCE);
    let sorcerer = t.battlefield(P0, "Prodigal Sorcerer");
    resonate(&mut t, sorcerer);
    assert_eq!(name_of(&t, res), "Prodigal Sorcerer");
    assert!(t.obj_now(res).is(CardType::Creature));
    let res = t.g.current(res);
    assert!(t.activate(P0, res, 0, &[Entity::Player(P1)]).is_err());
    // One that has been on the battlefield since the start of the turn can.
    let mut t = TestGame::new(2);
    let res = t.battlefield(P0, RESONANCE);
    let sorcerer = t.battlefield(P0, "Prodigal Sorcerer");
    resonate(&mut t, sorcerer);
    let res = t.g.current(res);
    t.activate(P0, res, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn resonance_copying_a_copy_becomes_what_it_copies() {
    cr!("707.3", "707.2");
    ruling!(
        "Crystalline Resonance",
        "If Crystalline Resonance copies a permanent that’s copying something else, it will become whatever the target is copying."
    );
    supported(RESONANCE);
    supported("Clone");
    let mut t = TestGame::new(2);
    let res = t.battlefield(P0, RESONANCE);
    let giant = t.battlefield(P1, "Hill Giant");
    yes(&mut t, P0);
    t.answer_choose(P0, &[Entity::Object(giant)]);
    cast_resolve(&mut t, P0, "Clone", &[]);
    let clone = t
        .named_on_battlefield("Hill Giant")
        .into_iter()
        .find(|id| *id != giant)
        .unwrap();
    resonate(&mut t, clone);
    assert_eq!(name_of(&t, res), "Hill Giant");
    assert_eq!(t.pt(res), (3, 3));
}

#[test]
fn resonance_triggering_twice_ends_as_the_last_copy_and_both_wear_off_together() {
    cr!("608.2", "613.7", "611.2a");
    ruling!(
        "Crystalline Resonance",
        "If Crystalline Resonance’s ability triggers multiple times in a turn, then each time one of those abilities resolves, it will overwrite whatever Crystalline Resonance is copying. Crystalline Resonance will wind up as a copy of the permanent targeted by the last ability to resolve. As your next turn begins, all instances of the ability will wear off at the same time."
    );
    supported(RESONANCE);
    let mut t = TestGame::new(2);
    let res = t.battlefield(P0, RESONANCE);
    let ring = t.battlefield(P0, "Sol Ring");
    let giant = t.battlefield(P0, "Hill Giant");
    // First cycle targets the Giant; in response, the second targets the Sol Ring.
    cycle_targeting(&mut t, giant);
    cycle_targeting(&mut t, ring);
    yes(&mut t, P0);
    yes(&mut t, P0);
    t.resolve_all();
    // The ability targeting the Giant resolved last.
    assert_eq!(name_of(&t, res), "Hill Giant");
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(name_of(&t, res), "Hill Giant");
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(name_of(&t, res), RESONANCE);
}

#[test]
fn effects_on_resonance_before_it_becomes_a_copy_continue_to_apply() {
    cr!("613.2a", "611.2c", "707.2");
    ruling!(
        "Crystalline Resonance",
        "If an effect begins to apply to Crystalline Resonance before it becomes a copy of another permanent, that effect will continue to apply."
    );
    supported(RESONANCE);
    supported("Heroic Intervention");
    let mut t = TestGame::new(2);
    let res = t.battlefield(P0, RESONANCE);
    let giant = t.battlefield(P1, "Hill Giant");
    cast_resolve(&mut t, P0, "Heroic Intervention", &[]);
    resonate(&mut t, giant);
    assert_eq!(name_of(&t, res), "Hill Giant");
    let o = t.obj_now(res);
    assert!(o.has_keyword(mtg_engine::keywords::KeywordKind::Indestructible));
    assert!(o.has_keyword(mtg_engine::keywords::KeywordKind::Hexproof));
    // The Hill Giant itself didn't get them.
    assert!(!t
        .obj(giant)
        .has_keyword(mtg_engine::keywords::KeywordKind::Indestructible));
}

#[test]
fn a_copy_of_resonance_copies_what_it_copies_and_its_ability() {
    cr!("707.3", "707.9b", "707.2");
    ruling!(
        "Crystalline Resonance",
        "If another permanent becomes a copy of Crystalline Resonance, it will become whatever Crystalline Resonance is copying and it will also have its ability."
    );
    supported(RESONANCE);
    supported("Clone");
    let mut t = TestGame::new(2);
    let res = t.battlefield(P0, RESONANCE);
    let giant = t.battlefield(P1, "Hill Giant");
    resonate(&mut t, giant);
    assert_eq!(name_of(&t, res), "Hill Giant");
    let res = t.g.current(res);
    yes(&mut t, P0);
    t.answer_choose(P0, &[Entity::Object(res)]);
    cast_resolve(&mut t, P0, "Clone", &[]);
    let clone = t
        .named_on_battlefield("Hill Giant")
        .into_iter()
        .find(|id| *id != giant && *id != res)
        .expect("the Clone");
    assert_eq!(t.pt(clone), (3, 3));
    assert_eq!(triggered_count(&t, clone), 1);
    // Cycling now triggers both (the Clone's copy targeting a Sol Ring).
    let ring = t.battlefield(P0, "Sol Ring");
    t.lands(P0, "Swamp", 1);
    let moor = t.hand(P0, "Barren Moor");
    t.answer_targets(P0, &[Entity::Object(ring)]);
    t.answer_targets(P0, &[Entity::Object(ring)]);
    cycle(&mut t, P0, moor, 0).unwrap();
    t.settle();
    assert_eq!(t.stack_len(), 3);
    yes(&mut t, P0);
    yes(&mut t, P0);
    t.resolve_all();
    assert_eq!(name_of(&t, clone), "Sol Ring");
}

// ---------------------------------------------------------------------------------------
// Mizzium Transreliquat
// ---------------------------------------------------------------------------------------

#[test]
fn reliquat_copies_only_copiable_values() {
    cr!("707.2", "613.2a");
    ruling!(
        "Mizzium Transreliquat",
        "An artifact's \"copiable values\" are those printed on it, as modified by other copy effects, plus any values set by \"enters as\" abilities. Counters and other effects aren't copied. For example, if you copy a Gruul War Plow that's been turned into an artifact creature, the result will be just an artifact."
    );
    supported(RELIQUAT);
    let mut t = TestGame::new(2);
    let rel = t.battlefield(P0, RELIQUAT);
    let ring = t.battlefield(P0, "Sol Ring");
    cast_resolve(&mut t, P0, "Majestic Metamorphosis", &[Entity::Object(ring)]);
    let ring = t.g.current(ring);
    t.g.add_counters(Entity::Object(ring), counters::CHARGE, 2, None);
    t.g.flush_events();
    assert!(t.obj(ring).is(CardType::Creature));
    t.lands(P0, "Wastes", 3);
    activate_resolve(&mut t, P0, rel, 0, &[Entity::Object(ring)]);
    assert_eq!(name_of(&t, rel), "Sol Ring");
    assert!(!t.obj_now(rel).is(CardType::Creature));
    assert_eq!(t.counters(t.g.current(rel), counters::CHARGE), 0);
}

fn reliquat_setup() -> (TestGame, ObjectId, ObjectId, ObjectId) {
    let mut t = TestGame::new(2);
    let rel = t.battlefield(P0, RELIQUAT);
    let ring = t.battlefield(P0, "Sol Ring");
    let storage = t.battlefield(P0, "Cold Storage");
    (t, rel, ring, storage)
}

#[test]
fn reliquat_activations_in_response_last_one_to_resolve_wins() {
    cr!("608.2", "613.7", "514.2");
    ruling!(
        "Mizzium Transreliquat",
        "If you use Mizzium Transreliquat's abilities multiple times in a turn in response to one another, then each time one of those abilities resolves, it will overwrite whatever the permanent was copying. The Transreliquat will wind up as a copy of the artifact targeted by the last ability to resolve. When the turn ends, all instances of its first ability will wear off at the same time. If one of those was the last copy ability to resolve, the Transreliquat will become what it was before those abilities resolved. This will likely be either the original Mizzium Transreliquat or whatever it copied with the last instance of its second ability to resolve."
    );
    supported(RELIQUAT);
    // {3} targeting Sol Ring, then in response {1}{U}{R} targeting Cold Storage: the {3}
    // resolves last (Sol Ring); at end of turn it's a Cold Storage with the {1}{U}{R}
    // ability.
    let (mut t, rel, ring, storage) = reliquat_setup();
    t.lands(P0, "Wastes", 4);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Mountain", 1);
    t.activate(P0, rel, 0, &[Entity::Object(ring)]).unwrap();
    t.activate(P0, rel, 1, &[Entity::Object(storage)]).unwrap();
    t.resolve_all();
    assert_eq!(name_of(&t, rel), "Sol Ring");
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(name_of(&t, rel), "Cold Storage");
    // Cold Storage's two abilities plus the {1}{U}{R} one.
    assert_eq!(activated_count(&t, rel), 3);

    // Two {3} activations: it's a copy of the one resolving last, then the original.
    let (mut t, rel, ring, storage) = reliquat_setup();
    t.lands(P0, "Wastes", 6);
    t.activate(P0, rel, 0, &[Entity::Object(ring)]).unwrap();
    t.activate(P0, rel, 0, &[Entity::Object(storage)]).unwrap();
    t.resolve_all();
    assert_eq!(name_of(&t, rel), "Sol Ring");
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(name_of(&t, rel), RELIQUAT);
}

#[test]
fn reliquat_first_ability_loses_its_copy_abilities_until_end_of_turn() {
    cr!("707.2", "514.2", "611.2a");
    ruling!(
        "Mizzium Transreliquat",
        "If you use the first ability, Mizzium Transreliquat loses both of its copy abilities until the effect wears off (unless it copied itself or another Mizzium Transreliquat). When it wears off, Mizzium Transreliquat loses all abilities it gained this way and goes back to being what it was before."
    );
    supported(RELIQUAT);
    let (mut t, rel, _ring, storage) = reliquat_setup();
    t.lands(P0, "Wastes", 3);
    activate_resolve(&mut t, P0, rel, 0, &[Entity::Object(storage)]);
    assert_eq!(name_of(&t, rel), "Cold Storage");
    // Only Cold Storage's two abilities: no copy abilities.
    assert_eq!(activated_count(&t, rel), 2);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(name_of(&t, rel), RELIQUAT);
    assert_eq!(activated_count(&t, rel), 2);
    let rel = t.g.current(rel);
    assert!(t
        .obj(rel)
        .chars
        .abilities
        .iter()
        .all(|a| !a.text.contains("Exile target creature")));
}

#[test]
fn a_copy_of_a_copying_reliquat_stays_what_it_copied() {
    cr!("707.3", "707.2");
    ruling!(
        "Mizzium Transreliquat",
        "The results of all copy effects are copied. If Mizzium Transreliquat is copied, the copy will be a Mizzium Transreliquat after applying all copy effects currently affecting the original. For example, Copy Artifact copying a Transreliquat that's using its first ability to copy an Izzet Signet will be an Izzet Signet. The effect won't wear off at the end of the turn; rather, the Copy Artifact will remain an Izzet Signet for the rest of the game."
    );
    supported(RELIQUAT);
    supported("Copy Artifact");
    let (mut t, rel, ring, _storage) = reliquat_setup();
    t.lands(P0, "Wastes", 3);
    activate_resolve(&mut t, P0, rel, 0, &[Entity::Object(ring)]);
    assert_eq!(name_of(&t, rel), "Sol Ring");
    let rel = t.g.current(rel);
    yes(&mut t, P0);
    t.answer_choose(P0, &[Entity::Object(rel)]);
    cast_resolve(&mut t, P0, "Copy Artifact", &[]);
    let copy = t
        .named_on_battlefield("Sol Ring")
        .into_iter()
        .find(|id| *id != ring && *id != rel)
        .expect("Copy Artifact");
    assert!(t.obj(copy).is(CardType::Enchantment));
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(name_of(&t, rel), RELIQUAT);
    assert_eq!(name_of(&t, copy), "Sol Ring");
}

#[test]
fn reliquat_second_ability_lasts_and_keeps_only_itself() {
    cr!("707.9b", "707.2", "611.2a");
    ruling!(
        "Mizzium Transreliquat",
        "The second ability doesn't wear off. If you use it, Mizzium Transreliquat becomes a copy of target artifact permanently and gains the {1}{U}{R} ability. It no longer has the {3} ability (unless it copied itself or another Mizzium Transreliquat)."
    );
    supported(RELIQUAT);
    let (mut t, rel, _ring, storage) = reliquat_setup();
    t.lands(P0, "Wastes", 1);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Mountain", 1);
    activate_resolve(&mut t, P0, rel, 1, &[Entity::Object(storage)]);
    assert_eq!(name_of(&t, rel), "Cold Storage");
    assert_eq!(activated_count(&t, rel), 3);
    next_own_turn(&mut t);
    assert_eq!(name_of(&t, rel), "Cold Storage");
    assert_eq!(activated_count(&t, rel), 3);
    // The {3} ability is gone: its first activated ability is Cold Storage's.
    let rel = t.g.current(rel);
    let texts: Vec<String> = t
        .obj(rel)
        .chars
        .abilities
        .iter()
        .map(|a| a.text.to_string())
        .collect();
    assert!(!texts.iter().any(|s| s.contains("until end of turn")), "{texts:?}");
}
