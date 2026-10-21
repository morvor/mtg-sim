//! Choices each player makes for the same object, read by who chose what (CR 607.2d;
//! `oracle/patterns/choice_grammar_per_player.rs`): "each player chooses war or peace"
//! with "creatures controlled by players who chose war", "for each player who chose
//! money", "each opponent who chose silence", "if each opponent chose snitch".

use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn compiles(name: &str) {
    let c = mtg_engine::card::card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name}: {:?}",
        c.unsupported_text()
    );
}

/// Queues `p`'s answer to the next "choose one" decision.
fn pick(t: &mut TestGame, p: PlayerId, i: usize) {
    t.answer(p, DecisionKind::Option, Answer::Index(i));
}

#[test]
fn archangel_of_strife_each_players_creatures_get_the_bonus_that_player_chose() {
    cr!("607.2d", "101.4", "614.12");
    ruling!(
        "Archangel of Strife",
        "The active player chooses war or peace first, followed by each other player in turn order"
    );
    ruling!(
        "Archangel of Strife",
        "depends only on the choice that player made when Archangel of Strife entered"
    );
    compiles("Archangel of Strife");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    pick(&mut t, P0, 0); // war
    pick(&mut t, P1, 1); // peace
    let angel = t.enter(P0, "Archangel of Strife");
    t.settle();
    let order: Vec<PlayerId> = t
        .asked()
        .into_iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseOption { .. }))
        .map(|(p, _)| p)
        .collect();
    assert_eq!(order, vec![P0, P1], "the active player chooses first");
    assert_eq!(t.pt(mine), (5, 2), "war: +3/+0");
    assert_eq!(t.pt(theirs), (2, 5), "peace: +0/+3");
    assert_eq!(
        t.pt(angel),
        (9, 6),
        "it enters as a 9/6 with its controller's war"
    );
}

#[test]
fn archangel_of_strife_two_archangels_with_different_choices_give_both_bonuses() {
    cr!("607.2d");
    ruling!(
        "Archangel of Strife",
        "you choose war for one and peace for the other, your creatures get +3/+3"
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    pick(&mut t, P0, 0); // war
    pick(&mut t, P1, 0); // war
    t.enter(P0, "Archangel of Strife");
    pick(&mut t, P0, 1); // peace
    pick(&mut t, P1, 0); // war
    t.enter(P0, "Archangel of Strife");
    t.settle();
    assert_eq!(t.pt(bears), (5, 5));
}

#[test]
fn archangel_of_strife_leaving_ends_the_bonus_and_a_new_object_chooses_again() {
    cr!("400.7", "607.2d");
    ruling!(
        "Archangel of Strife",
        "As soon as Archangel of Strife leaves the battlefield, the bonus stops applying"
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    pick(&mut t, P0, 0); // war
    pick(&mut t, P1, 0);
    let angel = t.enter(P0, "Archangel of Strife");
    t.settle();
    assert_eq!(t.pt(bears), (5, 2));
    t.g.destroy(angel, None);
    t.settle();
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn master_of_ceremonies_each_opponents_choice_gives_both_of_you_its_reward() {
    cr!("101.4", "608.2c");
    compiles("Master of Ceremonies");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Master of Ceremonies");
    pick(&mut t, P1, 0); // money
    pick(&mut t, P2, 2); // secrets
    t.advance_to(P2, Step::End);
    let hands = (t.hand_size(P0), t.hand_size(P1), t.hand_size(P2));
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    let treasures = |t: &TestGame, p: PlayerId| {
        t.g.battlefield
            .iter()
            .filter(|id| {
                t.g.obj(**id).controller == p && t.g.obj(**id).chars.has_subtype("Treasure")
            })
            .count()
    };
    let citizens =
        t.g.battlefield
            .iter()
            .filter(|id| t.g.obj(**id).chars.has_subtype("Citizen"))
            .count();
    assert_eq!(treasures(&t, P0), 1, "P1 chose money");
    assert_eq!(treasures(&t, P1), 1);
    assert_eq!(treasures(&t, P2), 0);
    assert_eq!(citizens, 0, "no one chose friends");
    assert_eq!(t.hand_size(P0), hands.0 + 1, "P2 chose secrets");
    assert_eq!(t.hand_size(P2), hands.2 + 1);
    assert_eq!(t.hand_size(P1), hands.1);
}

/// Prisoner's Dilemma cast by P0 against two opponents choosing `p1` and `p2` (0 silence,
/// 1 snitch): their life totals after it resolves.
fn dilemma(p1: usize, p2: usize) -> (i32, i32) {
    let mut t = TestGame::new(3);
    t.lands(P0, "Mountain", 5);
    let spell = t.hand(P0, "Prisoner's Dilemma");
    t.advance_to(P0, Step::PrecombatMain);
    pick(&mut t, P1, p1);
    pick(&mut t, P2, p2);
    t.cast(P0, spell).go();
    t.resolve_all();
    (t.life(P1), t.life(P2))
}

#[test]
fn prisoners_dilemma_deals_damage_by_what_the_opponents_secretly_chose() {
    cr!("608.2c", "608.2e");
    compiles("Prisoner's Dilemma");
    assert_eq!(dilemma(0, 0), (16, 16), "each chose silence: 4 each");
    assert_eq!(dilemma(1, 1), (12, 12), "each chose snitch: 8 each");
    assert_eq!(dilemma(0, 1), (8, 20), "otherwise: 12 to who chose silence");
}

/// Chooses `a` and `b` for the next "choose two players".
fn choose_two(t: &mut TestGame, p: PlayerId, a: PlayerId, b: PlayerId) {
    t.answer_choose(p, &[Entity::Player(a), Entity::Player(b)]);
}

#[test]
fn sower_of_discord_damage_to_one_chosen_player_makes_the_other_lose_life() {
    cr!("607.2d", "119.3");
    ruling!("Sower of Discord", "You can be one of those players");
    ruling!(
        "Sower of Discord",
        "causes the second player to lose life. This isn"
    );
    compiles("Sower of Discord");
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::PrecombatMain);
    choose_two(&mut t, P0, P1, P2);
    t.enter(P0, "Sower of Discord");
    t.settle();
    t.lands(P0, "Mountain", 2);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1), t.life(P2)), (20, 17, 17));
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(Entity::Player(P0)).go();
    t.resolve_all();
    assert_eq!(
        (t.life(P0), t.life(P1), t.life(P2)),
        (18, 17, 17),
        "P0 wasn't chosen"
    );
}

/// P1 or P0 casts Lightning Bolt at `target` with Bitter Feud(s) choosing P1 and P2.
fn feud(feuds: usize, caster: PlayerId, target: Entity) -> TestGame {
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::PrecombatMain);
    for _ in 0..feuds {
        choose_two(&mut t, P0, P1, P2);
        t.enter(P0, "Bitter Feud");
    }
    t.settle();
    t.lands(caster, "Mountain", 1);
    let bolt = t.hand(caster, "Lightning Bolt");
    t.cast(caster, bolt).target(target).go();
    t.resolve_all();
    t
}

#[test]
fn bitter_feud_doubles_damage_from_one_chosen_player_to_the_other() {
    cr!("614.1a", "120.3");
    ruling!(
        "Bitter Feud",
        "applies to damage dealt by any source controlled by one of the chosen players"
    );
    compiles("Bitter Feud");
    assert_eq!(feud(1, P1, Entity::Player(P2)).life(P2), 14, "doubled");
    assert_eq!(
        feud(1, P1, Entity::Player(P0)).life(P0),
        17,
        "P0 isn't chosen"
    );
    assert_eq!(feud(1, P0, Entity::Player(P1)).life(P1), 17, "P0's source");
    assert_eq!(
        feud(1, P1, Entity::Player(P1)).life(P1),
        17,
        "not the other player"
    );
}

#[test]
fn bitter_feud_doubles_damage_to_the_other_players_permanents_and_stacks() {
    cr!("614.1a", "616.1");
    ruling!(
        "Bitter Feud",
        "two Bitter Feuds will end up multiplying the damage by four"
    );
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.battlefield(P2, "Indomitable Ancients"); // 2/10
    choose_two(&mut t, P0, P1, P2);
    t.enter(P0, "Bitter Feud");
    t.settle();
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(giant).go();
    t.resolve_all();
    assert_eq!(t.g.obj(giant).damage, 6);
    assert_eq!(feud(2, P1, Entity::Player(P2)).life(P2), 8, "3 x 4");
}

#[test]
fn null_chamber_both_chosen_names_cant_be_cast_or_played() {
    cr!("101.4", "201.4");
    ruling!(
        "Null Chamber",
        "You choose which opponent makes the choice. Null Chamber does not target that opponent"
    );
    compiles("Null Chamber");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.answer(
        P0,
        DecisionKind::Name,
        Answer::Text("Lightning Bolt".into()),
    );
    t.answer(P1, DecisionKind::Name, Answer::Text("Mutavault".into()));
    t.enter(P0, "Null Chamber");
    t.settle();
    t.lands(P0, "Mountain", 2);
    let bolt = t.hand(P0, "Lightning Bolt");
    assert!(t
        .cast(P0, bolt)
        .target(Entity::Player(P1))
        .try_go()
        .is_err());
    let shock = t.hand(P0, "Shock");
    assert!(t
        .cast(P0, shock)
        .target(Entity::Player(P1))
        .try_go()
        .is_ok());
    let vault = t.hand(P0, "Mutavault");
    assert!(t.play_land(P0, vault).is_err(), "the opponent's name");
}

#[test]
fn null_chamber_names_must_not_be_basic_land_names() {
    cr!("201.4a");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.answer(P0, DecisionKind::Name, Answer::Text("Island".into()));
    t.answer(P1, DecisionKind::Name, Answer::Text("Forest".into()));
    t.enter(P0, "Null Chamber");
    t.settle();
    let island = t.hand(P0, "Island");
    assert!(
        t.play_land(P0, island).is_ok(),
        "an invalid name names nothing"
    );
}

#[test]
fn day_of_the_moon_goads_every_creature_with_a_name_chosen_for_it() {
    cr!("701.15a", "714.2b");
    compiles("Day of the Moon");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let lions = t.battlefield(P1, "Savannah Lions");
    let saga = t.battlefield(P0, "Day of the Moon");
    t.answer(P0, DecisionKind::Name, Answer::Text("Grizzly Bears".into()));
    t.g.add_counters(Entity::Object(saga), "lore", 1, None);
    t.settle();
    t.resolve_all();
    assert_eq!(t.obj_now(bears).goaded_by, vec![P0]);
    assert!(t.obj_now(lions).goaded_by.is_empty());
    t.answer(
        P0,
        DecisionKind::Name,
        Answer::Text("Savannah Lions".into()),
    );
    t.g.add_counters(Entity::Object(saga), "lore", 1, None);
    t.settle();
    t.resolve_all();
    assert_eq!(t.obj_now(lions).goaded_by, vec![P0]);
    assert_eq!(t.obj_now(bears).goaded_by, vec![P0], "still goaded");
    assert_eq!(
        t.g.obj(saga).choices.list.len(),
        2,
        "both names are chosen for it"
    );
}

#[test]
fn selective_obliteration_keeps_colorless_and_the_color_each_controller_chose() {
    cr!("101.4", "105.2");
    compiles("Selective Obliteration");
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 5);
    let lions = t.battlefield(P0, "Savannah Lions");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let goblin = t.battlefield(P1, "Raging Goblin");
    let thopter = t.battlefield(P1, "Ornithopter");
    let mage = t.battlefield(P1, "Boros Guildmage");
    let spell = t.hand(P0, "Selective Obliteration");
    t.advance_to(P0, Step::PrecombatMain);
    pick(&mut t, P0, 0); // white
    pick(&mut t, P1, 3); // red
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(t.on_battlefield(lions) && t.on_battlefield(goblin) && t.on_battlefield(thopter));
    assert!(!t.on_battlefield(bears), "green");
    assert!(!t.on_battlefield(mage), "red and white isn't only red");
}

#[test]
fn teferis_realm_the_upkeep_player_chooses_which_permanents_phase_out() {
    cr!("702.26b");
    ruling!("Teferi's Realm", "Can affect itself if you choose");
    compiles("Teferi's Realm");
    let mut t = TestGame::new(2);
    let realm = t.battlefield(P0, "Teferi's Realm");
    let anthem = t.battlefield(P0, "Glorious Anthem");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let aura = t.battlefield(P1, "Rancor");
    t.g.attach(aura, Entity::Object(bears));
    pick(&mut t, P1, 3); // non-Aura enchantment
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert!(t.obj_now(anthem).phased_out && t.obj_now(realm).phased_out);
    assert!(!t.obj_now(aura).phased_out, "an Aura");
    assert!(!t.obj_now(bears).phased_out);
}

#[test]
fn conundrum_sphinx_each_player_keeps_the_top_card_if_they_named_it() {
    cr!("101.4", "701.20a");
    ruling!(
        "Conundrum Sphinx",
        "First the player whose turn it is names a card, then each other player in turn order does the same"
    );
    compiles("Conundrum Sphinx");
    let mut t = TestGame::new(2);
    let sphinx = t.battlefield(P0, "Conundrum Sphinx");
    t.advance_to(P0, Step::BeginningOfCombat);
    let bolt = t.library_top(P0, "Lightning Bolt");
    let island = t.library_top(P1, "Island");
    t.answer(
        P0,
        DecisionKind::Name,
        Answer::Text("Lightning Bolt".into()),
    );
    t.answer(P1, DecisionKind::Name, Answer::Text("Forest".into()));
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(sphinx, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.resolve_all();
    assert!(
        matches!(t.zone(bolt), mtg_engine::object::Zone::Hand(_)),
        "named it"
    );
    let namers: Vec<PlayerId> = t
        .asked()
        .into_iter()
        .filter(|(_, d)| matches!(d, Decision::NameCard { .. }))
        .map(|(p, _)| p)
        .collect();
    assert_eq!(namers, vec![P0, P1]);
    let lib = &t.g.player(P1).library;
    assert_eq!(
        lib.first().copied().map(|c| t.g.current(c)),
        Some(t.g.current(island)),
        "on the bottom"
    );
}
