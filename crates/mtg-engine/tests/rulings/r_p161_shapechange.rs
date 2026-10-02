//! Rulings batch P161 — "shapechange" effects: what they leave alone (abilities, colors,
//! types), what Trickster's Elk overwrites, values and affected sets locked in as the
//! effect is created (CR 608.2h, 611.2c), and exchanges of power (CR 701.12).

use crate::r_p160_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn ability_count(t: &TestGame, id: ObjectId) -> usize {
    t.obj_now(id).chars.abilities.len()
}

fn subtypes(t: &TestGame, id: ObjectId) -> Vec<String> {
    let mut v: Vec<String> = t
        .obj_now(id)
        .chars
        .subtypes
        .iter()
        .map(|s| s.to_string())
        .collect();
    v.sort();
    v
}

fn flying(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id)
        .has_keyword(mtg_engine::keywords::KeywordKind::Flying)
}

#[test]
fn allosaurus_shepherd_elves_keep_their_abilities() {
    cr!("613.1d", "613.4b");
    ruling!(
        "Allosaurus Shepherd",
        "The affected creatures don't lose any abilities when they become Dinosaurs."
    );
    supported("Allosaurus Shepherd");
    let mut t = TestGame::new(2);
    let shepherd = t.battlefield(P0, "Allosaurus Shepherd");
    let elf = t.battlefield(P0, "Llanowar Elves");
    t.g.recompute();
    let before = ability_count(&t, elf);
    assert!(before > 0);
    t.lands(P0, "Forest", 6);
    activate_resolve(&mut t, P0, shepherd, 0, &[]);
    assert_eq!(t.pt(elf), (5, 5));
    let subs = subtypes(&t, elf);
    assert!(subs.contains(&"Dinosaur".to_string()) && subs.contains(&"Elf".to_string()));
    assert_eq!(ability_count(&t, elf), before);
    // The shepherd keeps its own abilities too.
    assert!(ability_count(&t, shepherd) >= 3);
}

#[test]
fn becoming_an_artifact_creature_keeps_abilities_and_colors() {
    cr!("613.1d", "613.4b", "205.1b");
    ruling!(
        "Relic's Roar",
        "The artifact or creature retains its abilities and colors, if any."
    );
    ruling!(
        "Majestic Metamorphosis",
        "The target artifact or creature retains its original abilities and colors, if any."
    );
    supported("Relic's Roar");
    supported("Majestic Metamorphosis");
    for name in ["Relic's Roar", "Majestic Metamorphosis"] {
        // A creature: Llanowar Elves keeps its mana ability and stays green.
        let mut t = TestGame::new(2);
        let elf = t.battlefield(P0, "Llanowar Elves");
        t.g.recompute();
        let before = ability_count(&t, elf);
        cast_resolve(&mut t, P0, name, &[Entity::Object(elf)]);
        assert!(t.obj(elf).is(CardType::Artifact), "{name}");
        // Majestic Metamorphosis adds flying; nothing is lost.
        let gained = usize::from(name == "Majestic Metamorphosis");
        assert_eq!(ability_count(&t, elf), before + gained, "{name}");
        assert!(t.obj_now(elf).chars.abilities.iter().any(|a| a.text.contains("{T}: Add {G}")), "{name}");
        assert_eq!(t.obj(elf).chars.colors, ColorSet::single(Color::Green), "{name}");
        // A noncreature artifact: Sol Ring keeps its mana ability and stays colorless.
        let mut t = TestGame::new(2);
        let ring = t.battlefield(P0, "Sol Ring");
        t.g.recompute();
        let before = ability_count(&t, ring);
        cast_resolve(&mut t, P0, name, &[Entity::Object(ring)]);
        assert!(t.obj(ring).is(CardType::Creature), "{name}");
        assert_eq!(ability_count(&t, ring), before + gained, "{name}");
        assert!(t.obj(ring).chars.colors.is_colorless(), "{name}");
    }
}

#[test]
fn vengeant_earth_land_keeps_its_abilities() {
    cr!("613.1d", "613.4b");
    ruling!(
        "Vengeant Earth",
        "The target creature or land will keep any abilities it previously had."
    );
    supported("Vengeant Earth");
    let mut t = TestGame::new(2);
    let land = t.battlefield(P0, "Forest");
    t.g.recompute();
    let before = ability_count(&t, land);
    cast_resolve(&mut t, P0, "Vengeant Earth", &[Entity::Object(land)]);
    let land = t.g.current(land);
    assert!(t.obj(land).is(CardType::Creature));
    assert!(t.obj(land).is(CardType::Land));
    assert_eq!(t.pt(land), (4, 4));
    // The Forest still has its mana ability (plus haste).
    assert!(ability_count(&t, land) > before);
    assert!(t
        .obj(land)
        .has_keyword(mtg_engine::keywords::KeywordKind::Haste));
    // A creature keeps its abilities too.
    let mut t = TestGame::new(2);
    let bird = t.battlefield(P0, "Ornithopter");
    cast_resolve(&mut t, P0, "Vengeant Earth", &[Entity::Object(bird)]);
    assert!(flying(&t, bird));
    assert_eq!(t.pt(bird), (4, 4));
}

#[test]
fn zhalfirin_shapecraft_changes_only_power_and_toughness() {
    cr!("613.4b");
    ruling!(
        "Zhalfirin Shapecraft",
        "Zhalfirin Shapecraft doesn’t affect the creature’s card types, creature types, or abilities."
    );
    supported("Zhalfirin Shapecraft");
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    t.g.recompute();
    let types = t.obj(thopter).chars.card_types.clone();
    let subs = subtypes(&t, thopter);
    cast_resolve(&mut t, P0, "Zhalfirin Shapecraft", &[Entity::Object(thopter)]);
    assert_eq!(t.pt(thopter), (4, 3));
    assert_eq!(t.obj(thopter).chars.card_types, types);
    assert_eq!(subtypes(&t, thopter), subs);
    assert!(flying(&t, thopter));
}

fn bestow_elk(t: &mut TestGame, target: ObjectId) {
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 1);
    let elk = t.hand(P0, "Trickster's Elk");
    t.answer_targets(P0, &[Entity::Object(target)]);
    t.cast(P0, elk)
        .method(mtg_engine::object::CastMethod::Keyword(
            mtg_engine::keywords::KeywordKind::Bestow,
        ))
        .go();
    t.resolve_all();
}

#[test]
fn tricksters_elk_overwrites_colors_and_types_but_not_supertypes() {
    cr!("613.1d", "613.1e", "205.1a", "205.4a");
    ruling!(
        "Trickster's Elk",
        "Trickster's Elk overwrites all colors and creature types the enchanted creature has. It's just a green Elk. The creature keeps any supertypes (such as legendary) it has but loses any other card types it has (such as artifact)."
    );
    supported("Trickster's Elk");
    supported("Karn, Silver Golem");
    let mut t = TestGame::new(2);
    let karn = t.battlefield(P0, "Karn, Silver Golem");
    bestow_elk(&mut t, karn);
    let o = t.obj_now(karn);
    assert!(o.chars.supertypes.contains(Supertype::Legendary));
    assert!(o.is(CardType::Creature));
    assert!(!o.is(CardType::Artifact));
    assert_eq!(subtypes(&t, karn), vec!["Elk".to_string()]);
    assert_eq!(t.obj_now(karn).chars.colors, ColorSet::single(Color::Green));
    assert_eq!(t.pt(karn), (3, 3));
    assert_eq!(ability_count(&t, karn), 0);
}

#[test]
fn tricksters_elk_keeps_a_temporary_creature_a_creature() {
    cr!("613.1d", "611.2a");
    ruling!(
        "Trickster's Elk",
        "Trickster's Elk may enchant a permanent that is only temporarily a creature, such as a Vehicle. If this happens, Trickster's Elk's effect causes the enchanted permanent to remain a 3/3 green Elk creature even after the temporary effect making it a creature expires."
    );
    supported("Trickster's Elk");
    supported("Majestic Metamorphosis");
    // A Sol Ring that's a creature until end of turn.
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P0, "Sol Ring");
    cast_resolve(&mut t, P0, "Majestic Metamorphosis", &[Entity::Object(ring)]);
    assert!(t.obj(ring).is(CardType::Creature));
    bestow_elk(&mut t, ring);
    let elk = t.named_on_battlefield("Trickster's Elk")[0];
    assert_eq!(t.obj(elk).attached_to, Some(Entity::Object(ring)));
    t.advance_to(P1, Step::Upkeep);
    let ring = t.g.current(ring);
    assert!(t.on_battlefield(ring));
    assert!(t.obj(ring).is(CardType::Creature));
    assert_eq!(t.pt(ring), (3, 3));
    assert_eq!(subtypes(&t, ring), vec!["Elk".to_string()]);
    assert_eq!(t.obj(elk).attached_to, Some(Entity::Object(ring)));
}

#[test]
fn candlekeep_inspiration_locks_in_x_and_the_creatures() {
    cr!("608.2h", "611.2c", "613.4b");
    ruling!(
        "Candlekeep Inspiration",
        "The set of creatures that Candlekeep Inspiration applies to is locked in as it resolves. Creatures that enter the battlefield under your control after it resolves will not be affected."
    );
    ruling!(
        "Candlekeep Inspiration",
        "The value of X is locked in at the time that Candlekeep Inspiration resolves. The power and toughness of creatures affected by it will not change as cards enter and leave exile and/or your graveyard."
    );
    supported("Candlekeep Inspiration");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    t.graveyard(P0, "Lightning Bolt");
    t.exile(P0, "Shock");
    cast_resolve(&mut t, P0, "Candlekeep Inspiration", &[]);
    // Two instants (Candlekeep Inspiration itself was on the stack).
    assert_eq!(t.pt(giant), (2, 2));
    assert!(t.in_graveyard(P0, "Candlekeep Inspiration"));
    t.graveyard(P0, "Opt");
    t.g.recompute();
    assert_eq!(t.pt(giant), (2, 2));
    let later = t.battlefield(P0, "Hill Giant");
    t.g.recompute();
    assert_eq!(t.pt(later), (3, 3));
}

#[test]
fn jolrael_x_is_locked_in_as_the_ability_resolves() {
    cr!("608.2h", "613.4b");
    ruling!(
        "Jolrael, Mwonvuli Recluse",
        "The value of X is determined only as Jolrael's last ability resolves. Once that happens, the value of X won't change later in the turn even if the number of cards in your hand changes."
    );
    supported("Jolrael, Mwonvuli Recluse");
    let mut t = TestGame::new(2);
    let jolrael = t.battlefield(P0, "Jolrael, Mwonvuli Recluse");
    let giant = t.battlefield(P0, "Hill Giant");
    t.hand(P0, "Forest");
    t.hand(P0, "Forest");
    t.lands(P0, "Forest", 6);
    assert_eq!(t.hand_size(P0), 2);
    activate_resolve(&mut t, P0, jolrael, 0, &[]);
    assert_eq!(t.pt(giant), (2, 2));
    t.hand(P0, "Island");
    t.hand(P0, "Island");
    t.g.recompute();
    assert_eq!(t.pt(giant), (2, 2));
}

fn cats(t: &TestGame) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == P0 && o.is_token() && o.chars.subtypes.iter().any(|s| s == "Cat"))
        .count()
}

#[test]
fn jolrael_triggers_only_on_the_second_card_drawn() {
    cr!("603.2", "603.2c");
    ruling!(
        "Jolrael, Mwonvuli Recluse",
        "The triggered ability can trigger only once each turn. It doesn't matter if Jolrael was on the battlefield when the first card was drawn: if it's not on the battlefield when the second card is drawn, the ability can't trigger at all that turn."
    );
    supported("Jolrael, Mwonvuli Recluse");
    // Jolrael enters after the first draw: the second draw triggers it; the third doesn't.
    let mut t = TestGame::new(2);
    t.g.draw_cards(P0, 1);
    t.g.flush_events();
    t.battlefield(P0, "Jolrael, Mwonvuli Recluse");
    t.g.draw_cards(P0, 1);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(cats(&t), 1);
    t.g.draw_cards(P0, 1);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(cats(&t), 1);
    // Jolrael enters after the second draw: no Cat this turn, even on later draws.
    let mut t = TestGame::new(2);
    t.g.draw_cards(P0, 2);
    t.g.flush_events();
    t.battlefield(P0, "Jolrael, Mwonvuli Recluse");
    t.g.draw_cards(P0, 2);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(cats(&t), 0);
}

#[test]
fn eldrazi_mimic_uses_the_creatures_pt_as_the_ability_resolves() {
    cr!("608.2h", "613.4b", "608.2b");
    ruling!(
        "Eldrazi Mimic",
        "Use the new creature’s power and toughness at the time the ability resolves to determine the base power and toughness of Eldrazi Mimic. If that creature is no longer on the battlefield at that time, use its power and toughness when it left the battlefield. (Keep in mind that those values may be negative, for example if a spell like Spatial Contortion giving it +3/-3 is what caused it to leave the battlefield.)"
    );
    supported("Eldrazi Mimic");
    // Pumped in response: 3/5.
    let mut t = TestGame::new(2);
    let mimic = t.battlefield(P0, "Eldrazi Mimic");
    t.enter(P0, "Ornithopter");
    t.settle();
    let thopter = t.named_on_battlefield("Ornithopter")[0];
    assert_eq!(t.stack_len(), 1);
    cast_new(&mut t, P0, "Giant Growth", &[Entity::Object(thopter)]);
    t.resolve();
    yes(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.pt(mimic), (3, 5));
    // Pumped and then destroyed in response: its last known 3/5.
    let mut t = TestGame::new(2);
    let mimic = t.battlefield(P0, "Eldrazi Mimic");
    t.enter(P0, "Ornithopter");
    t.settle();
    let thopter = t.named_on_battlefield("Ornithopter")[0];
    cast_new(&mut t, P0, "Giant Growth", &[Entity::Object(thopter)]);
    t.resolve();
    t.g.destroy(thopter, None);
    t.settle();
    assert!(!t.on_battlefield(thopter));
    yes(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.pt(mimic), (3, 5));
    // Killed by Spatial Contortion (+3/-3): its last known 3/-1, so the Mimic becomes a
    // 3/-1 and dies too.
    supported("Spatial Contortion");
    let mut t = TestGame::new(2);
    let mimic = t.battlefield(P0, "Eldrazi Mimic");
    t.enter(P0, "Ornithopter");
    t.settle();
    let thopter = t.named_on_battlefield("Ornithopter")[0];
    cast_new(&mut t, P0, "Spatial Contortion", &[Entity::Object(thopter)]);
    t.resolve();
    assert!(!t.on_battlefield(thopter));
    assert!(t.on_battlefield(mimic));
    yes(&mut t, P0);
    t.resolve_all();
    assert!(!t.on_battlefield(mimic));
    assert!(t.in_graveyard(P0, "Eldrazi Mimic"));
}

#[test]
fn pupu_ufo_counts_towns_as_the_ability_resolves() {
    cr!("608.2h", "613.4b");
    ruling!(
        "PuPu UFO",
        "Use the number of Towns you control when PuPu UFO's last ability resolves to determine PuPu UFO's new base power."
    );
    supported("PuPu UFO");
    let mut t = TestGame::new(2);
    let ufo = t.battlefield(P0, "PuPu UFO");
    t.battlefield(P0, "Capital City");
    t.lands(P0, "Wastes", 3);
    t.activate(P0, ufo, 1, &[]).unwrap();
    // A second Town arrives while the ability is on the stack.
    t.battlefield(P0, "Adventurer's Inn");
    t.resolve_all();
    assert_eq!(t.pt(ufo), (2, 4));
}

#[test]
fn bramblefort_fink_checks_for_oko_only_on_activation() {
    cr!("602.5", "613.4b");
    ruling!(
        "Bramblefort Fink",
        "Whether you control an Oko planeswalker is checked only as you begin to activate Bramblefort Fink's ability. It doesn't matter if Oko leaves the battlefield before the ability resolves or later in the turn."
    );
    supported("Bramblefort Fink");
    supported("Oko, Thief of Crowns");
    // Without Oko: can't activate.
    let mut t = TestGame::new(2);
    let fink = t.battlefield(P0, "Bramblefort Fink");
    t.lands(P0, "Wastes", 8);
    assert!(t.activate(P0, fink, 0, &[]).is_err());
    // With Oko, which then leaves before the ability resolves.
    let mut t = TestGame::new(2);
    let fink = t.battlefield(P0, "Bramblefort Fink");
    let oko = t.battlefield(P0, "Oko, Thief of Crowns");
    t.lands(P0, "Wastes", 8);
    t.activate(P0, fink, 0, &[]).unwrap();
    t.g.destroy(oko, None);
    t.settle();
    assert!(!t.on_battlefield(oko));
    t.resolve_all();
    assert_eq!(t.pt(fink), (10, 10));
}

#[test]
fn master_of_winds_chooses_as_the_ability_resolves() {
    cr!("608.2d", "613.4b");
    ruling!(
        "Master of Winds",
        "You don’t choose a power and toughness for Master of Winds as its ability triggers. Rather, as it resolves, you choose whether to have Master of Winds become 4/1, become 1/4, or not change its power and toughness."
    );
    supported("Master of Winds");
    supported("Opt");
    let mut t = TestGame::new(2);
    let master = t.battlefield(P0, "Master of Winds");
    cast_new(&mut t, P0, "Opt", &[]);
    t.settle();
    // Opt and the trigger are on the stack; no choice has been made yet.
    assert_eq!(t.stack_len(), 2);
    assert!(!t
        .asked()
        .iter()
        .any(|(p, d)| *p == P0
            && matches!(
                d,
                mtg_engine::decision::Decision::YesNo { .. }
                    | mtg_engine::decision::Decision::ChooseOption { .. }
            )));
    // As it resolves: decline — no change.
    t.answer_yes(P0, false);
    t.resolve();
    assert_eq!(t.pt(master), (1, 4));
    t.resolve_all();
    // Next time: choose 4/1 as it resolves.
    cast_new(&mut t, P0, "Opt", &[]);
    yes(&mut t, P0);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.resolve_all();
    assert_eq!(t.pt(master), (4, 1));
}

/// P1 attacks with `attacker`; Serene Master blocks it; the trigger targets it. Returns
/// with the trigger on the stack.
fn serene_block(t: &mut TestGame, master: ObjectId, attacker: ObjectId) {
    t.advance_to(P1, Step::BeginningOfCombat);
    t.answer(
        P1,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(attacker, Entity::Player(P0))]),
    );
    t.answer(
        P0,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(master, attacker)]),
    );
    t.answer_targets(P0, &[Entity::Object(attacker)]);
    t.advance_to(P1, Step::DeclareBlockers);
    t.settle();
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn serene_master_exchanges_powers_simultaneously() {
    cr!("701.12g", "613.4b");
    ruling!(
        "Serene Master",
        "When Serene Master’s ability resolves, its power becomes equal to the former power of the target creature. At the same time, that creature’s power will become equal to Serene Master’s former power."
    );
    supported("Serene Master");
    let mut t = TestGame::new(2);
    let master = t.battlefield(P0, "Serene Master");
    let giant = t.battlefield(P1, "Hill Giant");
    serene_block(&mut t, master, giant);
    t.resolve_all();
    assert_eq!(t.pt(master), (3, 2));
    assert_eq!(t.pt(giant), (0, 3));
}

#[test]
fn serene_master_exchange_needs_both_creatures_on_the_battlefield() {
    cr!("701.12a", "701.12g");
    ruling!(
        "Serene Master",
        "The exchange is made only if both Serene Master and the target creature are on the battlefield when the ability resolves."
    );
    supported("Serene Master");
    let mut t = TestGame::new(2);
    let master = t.battlefield(P0, "Serene Master");
    let giant = t.battlefield(P1, "Hill Giant");
    serene_block(&mut t, master, giant);
    // Serene Master leaves in response: the Giant keeps its power.
    t.g.destroy(master, None);
    t.settle();
    t.resolve_all();
    assert!(!t.on_battlefield(master));
    assert_eq!(t.pt(giant), (3, 3));
}

#[test]
fn urza_token_copiable_values_include_the_exception() {
    cr!("707.9b", "707.2", "111.4");
    ruling!(
        "Urza, Prince of Kroog",
        "The token is a Soldier creature in addition to its other types. Its base power and toughness is 1/1. These are the copiable values of the token's characteristics that other effects may copy."
    );
    supported("Urza, Prince of Kroog");
    supported("Clone");
    let mut t = TestGame::new(2);
    let urza = t.battlefield(P0, "Urza, Prince of Kroog");
    let ring = t.battlefield(P0, "Sol Ring");
    t.lands(P0, "Wastes", 6);
    activate_resolve(&mut t, P0, urza, 0, &[Entity::Object(ring)]);
    let token = tokens_of(&t, P0)[0];
    let o = t.obj(token);
    assert_eq!(o.chars.name, "Sol Ring");
    assert!(o.is(CardType::Artifact) && o.is(CardType::Creature));
    assert!(o.chars.subtypes.iter().any(|s| s == "Soldier"));
    // Base 1/1, +2/+2 from Urza.
    assert_eq!(t.pt(token), (3, 3));
    // Clone copies the exception too: a 1/1 Soldier artifact creature Sol Ring (+2/+2).
    yes(&mut t, P0);
    t.answer_choose(P0, &[Entity::Object(token)]);
    cast_resolve(&mut t, P0, "Clone", &[]);
    let clone = t
        .named_on_battlefield("Sol Ring")
        .into_iter()
        .find(|id| *id != ring && *id != token)
        .expect("the Clone");
    let o = t.obj(clone);
    assert!(!o.is_token());
    assert!(o.is(CardType::Artifact) && o.is(CardType::Creature));
    assert!(o.chars.subtypes.iter().any(|s| s == "Soldier"));
    assert_eq!(t.pt(clone), (3, 3));
}
