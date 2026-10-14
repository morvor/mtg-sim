//! Control changes involving several players (CR 101.4, 608.2f, 701.12): Aminatou, the
//! Fateshifter; Order of Succession; Inniaz, the Gale Force; Scrambleverse; Cultural
//! Exchange; Perplexing Chimera; Sudden Substitution.

use crate::basic_effects_common::assert_supported;
use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const LEFT: usize = 0;
const RIGHT: usize = 1;

fn loyal(t: &mut TestGame, p: PlayerId, name: &str, n: u32) -> ObjectId {
    let pw = t.battlefield(p, name);
    t.g.objects[pw.0 as usize]
        .counters
        .insert("loyalty".into(), n);
    pw
}

fn controller(t: &TestGame, o: ObjectId) -> PlayerId {
    t.obj_now(o).controller
}

/// Aminatou's −6 with the direction answered `dir`.
fn aminatou_ultimate(t: &mut TestGame, aminatou: ObjectId, dir: usize) {
    t.answer(P0, DecisionKind::Option, Answer::Index(dir));
    t.activate(P0, aminatou, 2, &[]).unwrap();
    t.resolve_all();
}

#[test]
fn aminatou_passes_every_nonland_permanent_to_the_next_player_at_once() {
    cr!("608.2f", "101.4");
    ruling!(
        "Aminatou, the Fateshifter",
        "each player takes control of their new permanents at the same time"
    );
    assert_supported("Aminatou, the Fateshifter");
    for (dir, expect) in [(LEFT, [P2, P0, P1]), (RIGHT, [P1, P2, P0])] {
        let mut t = TestGame::new(3);
        let aminatou = loyal(&mut t, P0, "Aminatou, the Fateshifter", 6);
        let a = t.battlefield(P0, "Grizzly Bears");
        let b = t.battlefield(P1, "Hill Giant");
        let c = t.battlefield(P2, "Craw Wurm");
        let land = t.battlefield(P1, "Forest");
        aminatou_ultimate(&mut t, aminatou, dir);
        // To the left: P0 takes P1's, P1 takes P2's, P2 takes P0's (and the reverse).
        assert_eq!(
            [controller(&t, a), controller(&t, b), controller(&t, c)],
            expect,
            "direction {dir}"
        );
        // Not Aminatou, not lands.
        assert_eq!(controller(&t, aminatou), P0);
        assert_eq!(controller(&t, land), P1);
    }
}

#[test]
fn aminatou_in_a_two_player_game_exchanges_everything_but_herself() {
    cr!("608.2f");
    ruling!("Aminatou, the Fateshifter", "no matter which direction you choose");
    for dir in [LEFT, RIGHT] {
        let mut t = TestGame::new(2);
        let aminatou = loyal(&mut t, P0, "Aminatou, the Fateshifter", 6);
        let a = t.battlefield(P0, "Grizzly Bears");
        let b = t.battlefield(P1, "Hill Giant");
        aminatou_ultimate(&mut t, aminatou, dir);
        assert_eq!(controller(&t, a), P1);
        assert_eq!(controller(&t, b), P0);
        assert_eq!(controller(&t, aminatou), P0);
    }
}

/// The players who were asked to choose objects since `from`.
fn choosers_since(t: &TestGame, from: usize) -> Vec<PlayerId> {
    t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .map(|(p, _)| *p)
        .collect()
}

#[test]
fn order_of_succession_goes_around_the_table_in_the_chosen_direction() {
    cr!("101.4b", "608.2f");
    ruling!(
        "Order of Succession",
        "All players will gain control of the creatures they chose simultaneously"
    );
    assert_supported("Order of Succession");
    let mut t = TestGame::new(4);
    t.set_step(P0, Step::PrecombatMain);
    let c: Vec<ObjectId> = [P0, P1, P2, P3]
        .iter()
        .map(|p| t.battlefield(*p, "Grizzly Bears"))
        .collect();
    let spell = t.hand(P0, "Order of Succession");
    t.lands(P0, "Island", 4);
    // To the right: P0 (from P3), then P3 (from P2), P2 (from P1), P1 (from P0).
    t.answer(P0, DecisionKind::Option, Answer::Index(RIGHT));
    let from = t.asked().len();
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(choosers_since(&t, from), vec![P0, P3, P2, P1]);
    assert_eq!(controller(&t, c[3]), P0);
    assert_eq!(controller(&t, c[2]), P3);
    assert_eq!(controller(&t, c[1]), P2);
    assert_eq!(controller(&t, c[0]), P1);
}

#[test]
fn order_of_succession_swaps_in_a_two_player_game() {
    cr!("608.2f");
    // Determined one at a time, the second player would see both creatures under the
    // first player's control.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let spell = t.hand(P0, "Order of Succession");
    t.lands(P0, "Island", 4);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(controller(&t, a), P1);
    assert_eq!(controller(&t, b), P0);
}

#[test]
fn order_of_succession_gives_nothing_from_a_player_without_creatures() {
    cr!("101.4b");
    ruling!(
        "Order of Succession",
        "If the next player in the chosen direction controls no creatures"
    );
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::PrecombatMain);
    let a = t.battlefield(P0, "Grizzly Bears");
    let c = t.battlefield(P2, "Craw Wurm");
    let spell = t.hand(P0, "Order of Succession");
    t.lands(P0, "Island", 4);
    t.answer(P0, DecisionKind::Option, Answer::Index(LEFT));
    t.cast(P0, spell).go();
    t.resolve_all();
    // P0 chooses from P1 (nothing), P1 from P2, P2 from P0.
    assert_eq!(controller(&t, c), P1);
    assert_eq!(controller(&t, a), P2);
    assert!(t
        .g
        .permanents()
        .all(|o| o.controller != P0 || !o.is_creature()));
}

#[test]
fn order_of_succession_counts_a_teammate_as_the_next_player() {
    cr!("101.4b");
    ruling!(
        "Order of Succession",
        "whether the next player is a teammate or an opponent"
    );
    let config = mtg_engine::game::GameConfig {
        teams: Some(vec![0, 0, 1, 1]),
        ..Default::default()
    };
    let mut t = TestGame::with_config(4, config);
    t.set_step(P0, Step::PrecombatMain);
    let c: Vec<ObjectId> = [P0, P1, P2, P3]
        .iter()
        .map(|p| t.battlefield(*p, "Grizzly Bears"))
        .collect();
    let spell = t.hand(P0, "Order of Succession");
    t.lands(P0, "Island", 4);
    t.answer(P0, DecisionKind::Option, Answer::Index(LEFT));
    t.cast(P0, spell).go();
    t.resolve_all();
    // P0 takes its teammate P1's creature.
    assert_eq!(controller(&t, c[1]), P0);
    assert_eq!(controller(&t, c[0]), P3);
}

/// Inniaz attacking with two other flyers: its trigger resolves.
fn inniaz_attacks(t: &mut TestGame) -> ObjectId {
    let inniaz = t.battlefield(P0, "Inniaz, the Gale Force");
    let crows: Vec<ObjectId> = (0..2).map(|_| t.battlefield(P0, "Storm Crow")).collect();
    t.advance_to(P0, Step::BeginningOfCombat);
    let defender = Entity::Player(P1);
    t.attack(
        &[(inniaz, defender), (crows[0], defender), (crows[1], defender)],
        &[],
    );
    t.resolve_all();
    inniaz
}

#[test]
fn inniaz_exchanges_two_permanents_of_your_choice_with_your_opponent() {
    cr!("608.2f", "101.4");
    ruling!(
        "Inniaz, the Gale Force",
        "you exchange control of two permanents of your choice with your opponent"
    );
    ruling!(
        "Inniaz, the Gale Force",
        "No player may take actions between the time you make the choices and the time control changes"
    );
    assert_supported("Inniaz, the Gale Force");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    t.battlefield(P1, "Craw Wurm");
    // P0's part first (from P1), then P1's (from P0): both chosen by P0.
    t.answer_choose(P0, &[Entity::Object(theirs)]);
    t.answer_choose(P0, &[Entity::Object(mine)]);
    let from = t.asked().len();
    inniaz_attacks(&mut t);
    assert_eq!(choosers_since(&t, from), vec![P0, P0]);
    assert_eq!(controller(&t, theirs), P0);
    assert_eq!(controller(&t, mine), P1);
}

#[test]
fn inniaz_in_a_multiplayer_game_passes_to_the_left() {
    cr!("608.2f");
    ruling!(
        "Inniaz, the Gale Force",
        "If any player controls no nonland permanents, the player to their left doesn"
    );
    let mut t = TestGame::new(3);
    let b = t.battlefield(P1, "Hill Giant");
    t.battlefield(P1, "Forest");
    // P2 controls no nonland permanents: P0 (to P2's left) gets nothing.
    let inniaz = inniaz_attacks(&mut t);
    // P0's right is P2 (nothing); P1's right is P0; P2's right is P1.
    assert_eq!(controller(&t, b), P2);
    assert_eq!(
        t.g.permanents()
            .filter(|o| o.controller == P1 && !o.is(CardType::Land))
            .count(),
        1,
        "P1 took one of P0's permanents"
    );
    let _ = inniaz;
}

#[test]
fn scrambleverse_gives_each_nonland_permanent_to_a_random_player_and_untaps_it() {
    cr!("608.2f");
    ruling!("Scrambleverse", "including the permanent");
    ruling!("Scrambleverse", "Each nonland permanent will untap");
    assert_supported("Scrambleverse");
    let mut seen_stay = false;
    let mut seen_move = false;
    for seed in 0..4u64 {
        let mut t = TestGame::new(2);
        use rand::SeedableRng;
        t.g.rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
        t.set_step(P0, Step::PrecombatMain);
        let mut perms = Vec::new();
        for p in [P0, P1] {
            for _ in 0..3 {
                let o = t.battlefield(p, "Grizzly Bears");
                t.g.objects[o.0 as usize].tapped = true;
                perms.push((o, p));
            }
        }
        let land = t.battlefield(P1, "Forest");
        let spell = t.hand(P0, "Scrambleverse");
        t.lands(P0, "Mountain", 8);
        t.cast(P0, spell).go();
        t.resolve_all();
        for (o, before) in &perms {
            assert!(!t.obj_now(*o).tapped);
            if controller(&t, *o) == *before {
                seen_stay = true;
            } else {
                seen_move = true;
            }
        }
        assert_eq!(controller(&t, land), P1);
    }
    assert!(seen_stay && seen_move);
}

#[test]
fn cultural_exchange_swaps_the_same_number_of_creatures() {
    cr!("701.12a", "701.12b");
    assert_supported("Cultural Exchange");
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::PrecombatMain);
    let a1 = t.battlefield(P1, "Grizzly Bears");
    let a2 = t.battlefield(P1, "Hill Giant");
    let a3 = t.battlefield(P1, "Craw Wurm");
    let b1 = t.battlefield(P2, "Grizzly Bears");
    let b2 = t.battlefield(P2, "Hill Giant");
    let spell = t.hand(P0, "Cultural Exchange");
    t.lands(P0, "Island", 6);
    t.answer_choose(P0, &[Entity::Object(a1), Entity::Object(a2)]);
    t.answer_choose(P0, &[Entity::Object(b1), Entity::Object(b2)]);
    t.cast(P0, spell)
        .target(P1)
        .target(P2)
        .go();
    t.resolve_all();
    assert_eq!(controller(&t, a1), P2);
    assert_eq!(controller(&t, a2), P2);
    assert_eq!(controller(&t, a3), P1);
    assert_eq!(controller(&t, b1), P1);
    assert_eq!(controller(&t, b2), P1);
}

#[test]
fn cultural_exchange_does_nothing_if_the_numbers_differ() {
    cr!("701.12a");
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::PrecombatMain);
    let a1 = t.battlefield(P1, "Grizzly Bears");
    let a2 = t.battlefield(P1, "Hill Giant");
    let b1 = t.battlefield(P2, "Grizzly Bears");
    let spell = t.hand(P0, "Cultural Exchange");
    t.lands(P0, "Island", 6);
    // Two of P1's, but P2 has only one: the whole exchange can't happen.
    t.answer_choose(P0, &[Entity::Object(a1), Entity::Object(a2)]);
    t.cast(P0, spell)
        .target(P1)
        .target(P2)
        .go();
    t.resolve_all();
    assert_eq!(controller(&t, a1), P1);
    assert_eq!(controller(&t, a2), P1);
    assert_eq!(controller(&t, b1), P2);
}

#[test]
fn perplexing_chimera_exchanges_itself_and_the_spell_without_moving_them() {
    cr!("701.12a", "701.12b", "115.7d");
    ruling!(
        "Perplexing Chimera",
        "Neither Perplexing Chimera nor the spell changes zones"
    );
    assert_supported("Perplexing Chimera");
    let mut t = TestGame::new(2);
    let chimera = t.battlefield(P0, "Perplexing Chimera");
    t.set_step(P1, Step::PrecombatMain);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.lands(P1, "Mountain", 1);
    let spell = t.cast(P1, bolt).target(P0).go();
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.settle();
    t.resolve();
    // The trigger resolved: same objects, control exchanged.
    assert!(t.g.is_live(chimera) && t.zone(chimera) == Zone::Battlefield);
    assert!(t.g.is_live(spell) && t.zone(spell) == Zone::Stack);
    assert_eq!(controller(&t, chimera), P1);
    assert_eq!(controller(&t, spell), P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn sudden_substitution_gives_the_permanent_the_spell_becomes_to_its_new_controller() {
    cr!("701.12a", "608.3");
    ruling!(
        "Sudden Substitution",
        "its new controller will control the permanent it becomes"
    );
    assert_supported("Sudden Substitution");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    let stone = t.hand(P1, "Mind Stone");
    t.lands(P1, "Wastes", 2);
    let spell = t.cast(P1, stone).go();
    let sub = t.hand(P0, "Sudden Substitution");
    t.lands(P0, "Island", 4);
    t.g.turn.priority = Some(P0);
    t.cast(P0, sub)
        .targets(&[Entity::Object(spell), Entity::Object(bears)])
        .go();
    t.resolve();
    assert_eq!(controller(&t, bears), P1);
    assert_eq!(controller(&t, spell), P0);
    t.resolve_all();
    let stones = t.named_on_battlefield("Mind Stone");
    assert_eq!(stones.len(), 1);
    assert_eq!(controller(&t, stones[0]), P0);
}

#[test]
fn sudden_substitution_does_nothing_between_one_players_spell_and_creature() {
    cr!("701.12b");
    ruling!(
        "Sudden Substitution",
        "are both controlled by the same player, the exchange won"
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::PrecombatMain);
    let stone = t.hand(P0, "Mind Stone");
    t.lands(P0, "Wastes", 2);
    let spell = t.cast(P0, stone).go();
    let sub = t.hand(P0, "Sudden Substitution");
    t.lands(P0, "Island", 4);
    t.cast(P0, sub)
        .targets(&[Entity::Object(spell), Entity::Object(bears)])
        .go();
    t.resolve_all();
    assert_eq!(controller(&t, bears), P0);
    let stones = t.named_on_battlefield("Mind Stone");
    assert_eq!(controller(&t, stones[0]), P0);
}

#[test]
fn inniaz_in_two_headed_giant_ignores_teams() {
    cr!("608.2f", "101.4");
    ruling!(
        "Inniaz, the Gale Force",
        "The leftmost player on each team gains control of a teammate"
    );
    ruling!(
        "Inniaz, the Gale Force",
        "triggers just once, no matter how many creatures with flying attack beyond the third"
    );
    let mut t = TestGame::with_config(
        4,
        mtg_engine::game::GameConfig {
            variant: mtg_engine::game::Variant::TwoHeadedGiant,
            teams: Some(vec![0, 0, 1, 1]),
            ..Default::default()
        },
    );
    let inniaz = t.battlefield(P0, "Inniaz, the Gale Force");
    let crows: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P0, "Storm Crow")).collect();
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let wurm = t.battlefield(P2, "Craw Wurm");
    let stone = t.battlefield(P3, "Mind Stone");
    // P0 takes from P3 (to its right), P1 from its teammate P0, P2 from P1, P3 from its
    // teammate P2.
    t.answer_choose(P0, &[Entity::Object(stone)]);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.answer_choose(P0, &[Entity::Object(wurm)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    let d = Entity::Player(P2);
    let from = t.asked().len();
    t.attack(
        &[(inniaz, d), (crows[0], d), (crows[1], d), (crows[2], d)],
        &[],
    );
    assert_eq!(choosers_since(&t, from).len(), 4, "one trigger");
    assert_eq!(controller(&t, stone), P0);
    assert_eq!(controller(&t, bears), P1);
    assert_eq!(controller(&t, giant), P2);
    assert_eq!(controller(&t, wurm), P3);
}
