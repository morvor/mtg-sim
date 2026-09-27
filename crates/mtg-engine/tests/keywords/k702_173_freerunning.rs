//! CR 702.173 Freerunning.

use crate::common_k702_011_017::assert_supported;
use crate::common_k702_052_066::destroy;
use crate::common_k702_140_152::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const FREERUNNING: CastMethod = CastMethod::Keyword(KeywordKind::Freerunning);

/// P0 attacks P1 with `attackers` (P1 blocks with `blocks`) and goes to the postcombat
/// main phase.
fn attack_then_main(t: &mut TestGame, attackers: &[ObjectId], blocks: &[(ObjectId, ObjectId)]) {
    let decl: Vec<(ObjectId, Entity)> =
        attackers.iter().map(|a| (*a, Entity::Player(P1))).collect();
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&decl, blocks);
    t.advance_to(P0, Step::PostcombatMain);
}

#[test]
fn freerunning_after_an_assassin_dealt_combat_damage_to_a_player() {
    cr!("702.173", "702.173a");
    assert_supported("Eagle Vision");
    // Eagle Vision: {4}{U} sorcery, freerunning {1}{U}, "Draw three cards."
    let mut t = TestGame::new(2);
    let vision = t.hand(P0, "Eagle Vision");
    let assassin = t.battlefield(P0, "Royal Assassin");
    add_mana(&mut t, P0, ManaType::U, 2);
    assert!(!castable(&mut t, P0, vision, FREERUNNING));
    attack_then_main(&mut t, &[assassin], &[]);
    assert_eq!(t.life(P1), 19);
    add_mana(&mut t, P0, ManaType::U, 2);
    assert!(castable(&mut t, P0, vision, FREERUNNING));
    let hand = t.hand_size(P0);
    let spell = t.cast(P0, vision).method(FREERUNNING).go();
    // {1}{U} rather than {4}{U}; its mana value is unchanged.
    assert_eq!(pool(&t, P0), 0);
    assert_eq!(t.g.mana_value_of(spell), 5);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand - 1 + 3);
    // Only this turn.
    let vision = t.hand(P0, "Eagle Vision");
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    add_mana(&mut t, P0, ManaType::U, 2);
    assert!(!castable(&mut t, P0, vision, FREERUNNING));
}

#[test]
fn a_commander_counts_even_if_it_isnt_an_assassin() {
    cr!("702.173a");
    let mut t = TestGame::new(2);
    let vision = t.hand(P0, "Eagle Vision");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Another (non-commander, non-Assassin) creature isn't enough.
    attack_then_main(&mut t, &[bears], &[]);
    add_mana(&mut t, P0, ManaType::U, 2);
    assert!(!castable(&mut t, P0, vision, FREERUNNING));
    let mut t = TestGame::new(2);
    let vision = t.hand(P0, "Eagle Vision");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.objects[bears.0 as usize].is_commander = true;
    attack_then_main(&mut t, &[bears], &[]);
    add_mana(&mut t, P0, ManaType::U, 2);
    assert!(castable(&mut t, P0, vision, FREERUNNING));
}

#[test]
fn only_combat_damage_to_a_player_by_a_creature_you_control_counts() {
    cr!("702.173a");
    // Blocked: the Assassin dealt combat damage only to a creature.
    let mut t = TestGame::new(2);
    let vision = t.hand(P0, "Eagle Vision");
    let assassin = t.battlefield(P0, "Royal Assassin");
    let wall = t.battlefield(P1, "Wall of Stone");
    attack_then_main(&mut t, &[assassin], &[(wall, assassin)]);
    add_mana(&mut t, P0, ManaType::U, 2);
    assert!(!castable(&mut t, P0, vision, FREERUNNING));
    // An opponent's Assassin dealing combat damage doesn't let you freerun.
    let mut t = TestGame::new(2);
    let vision = t.hand(P0, "Eagle Vision");
    let assassin = t.battlefield(P1, "Royal Assassin");
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(&[(assassin, Entity::Player(P0))], &[]);
    assert_eq!(t.life(P0), 19);
    add_mana(&mut t, P0, ManaType::U, 2);
    t.g.turn.priority = Some(P0);
    assert!(!t
        .cast_options(P0, vision)
        .iter()
        .any(|o| o.method == FREERUNNING));
    // Noncombat damage from an Assassin doesn't count either.
    let mut t = TestGame::new(2);
    let vision = t.hand(P0, "Eagle Vision");
    t.battlefield(P0, "Royal Assassin");
    let shock = t.hand(P0, "Shock");
    add_mana(&mut t, P0, ManaType::R, 1);
    t.cast(P0, shock).target(Entity::Player(P1)).go();
    t.resolve_all();
    add_mana(&mut t, P0, ManaType::U, 2);
    assert!(!castable(&mut t, P0, vision, FREERUNNING));
}

#[test]
fn what_happens_to_the_assassin_afterwards_doesnt_matter() {
    cr!("702.173a");
    ruling!(
        "Eagle Vision",
        "Once an Assassin creature or commander you control deals combat damage to a player in a turn, you can cast spells for their freerunning costs for the rest of that turn. It doesn’t matter what happens to that Assassin or commander after that."
    );
    let mut t = TestGame::new(2);
    let vision = t.hand(P0, "Eagle Vision");
    let assassin = t.battlefield(P0, "Royal Assassin");
    attack_then_main(&mut t, &[assassin], &[]);
    destroy(&mut t, assassin);
    assert!(t.in_graveyard(P0, "Royal Assassin"));
    add_mana(&mut t, P0, ManaType::U, 2);
    assert!(castable(&mut t, P0, vision, FREERUNNING));
}

#[test]
fn a_freerunning_cost_may_be_a_non_mana_cost() {
    cr!("702.173a");
    assert_supported("Escape Detection");
    // Escape Detection: {1}{U}{U} instant, "Freerunning—Return a blue creature you control
    // to its owner's hand.", "Return target creature to its owner's hand. Draw a card."
    let mut t = TestGame::new(2);
    let escape = t.hand(P0, "Escape Detection");
    let assassin = t.battlefield(P0, "Royal Assassin");
    let crow = t.battlefield(P0, "Storm Crow");
    let giant = t.battlefield(P1, "Hill Giant");
    attack_then_main(&mut t, &[assassin], &[]);
    assert!(castable(&mut t, P0, escape, FREERUNNING));
    t.answer_choose(P0, &[Entity::Object(crow)]);
    t.cast(P0, escape).method(FREERUNNING).target(giant).go();
    assert!(t.in_hand(P0, "Storm Crow"));
    t.resolve_all();
    assert!(t.in_hand(P1, "Hill Giant"));
}

#[test]
fn if_this_spells_freerunning_cost_was_paid() {
    cr!("702.173a");
    assert_supported("Monastery Raid");
    // Monastery Raid: {2}{R} sorcery, "Freerunning {X}{R}", "Exile the top two cards of
    // your library. If this spell's freerunning cost was paid, exile the top X cards of
    // your library instead. You may play the exiled cards until the end of your next
    // turn."
    for freerunning in [false, true] {
        let mut t = TestGame::new(2);
        let raid = t.hand(P0, "Monastery Raid");
        let assassin = t.battlefield(P0, "Royal Assassin");
        let top: Vec<ObjectId> = (0..4).map(|_| t.library_top(P0, "Mountain")).collect();
        attack_then_main(&mut t, &[assassin], &[]);
        if freerunning {
            // X = 4: {4}{R}.
            add_mana(&mut t, P0, ManaType::R, 5);
            t.cast(P0, raid).method(FREERUNNING).x(4).go();
        } else {
            add_mana(&mut t, P0, ManaType::R, 3);
            t.cast(P0, raid).go();
        }
        assert_eq!(pool(&t, P0), 0);
        t.resolve_all();
        let exiled: Vec<ObjectId> = top
            .iter()
            .map(|c| t.g.current(*c))
            .filter(|c| t.zone(*c) == mtg_engine::object::Zone::Exile)
            .collect();
        assert_eq!(exiled.len(), if freerunning { 4 } else { 2 });
        // They may be played until the end of the next turn.
        t.g.turn.priority = Some(P0);
        assert!(t.g.legal_actions(P0).contains(
            &mtg_engine::decision::Action::PlayLand { card: exiled[0] }
        ));
    }
}

#[test]
fn assassin_spells_you_cast_have_freerunning() {
    cr!("702.173a", "610.5");
    assert_supported("Ezio Auditore da Firenze");
    ruling!(
        "Ezio Auditore da Firenze",
        "Once an Assassin creature or commander you control deals combat damage to a player in a turn, you can cast spells for their freerunning costs for the rest of that turn."
    );
    // Ezio Auditore da Firenze: 3/2 menace Human Assassin, "Assassin spells you cast have
    // freerunning {B}{B}."
    let mut t = TestGame::new(2);
    let ezio = t.battlefield(P0, "Ezio Auditore da Firenze");
    // Royal Assassin: {1}{B}{B} Human Assassin.
    let assassin = t.hand(P0, "Royal Assassin");
    let bears = t.hand(P0, "Grizzly Bears");
    add_mana(&mut t, P0, ManaType::B, 2);
    assert!(!castable(&mut t, P0, assassin, FREERUNNING));
    attack_then_main(&mut t, &[ezio], &[]);
    add_mana(&mut t, P0, ManaType::B, 2);
    assert!(castable(&mut t, P0, assassin, FREERUNNING));
    assert!(!castable(&mut t, P0, bears, FREERUNNING));
    let spell = t.cast(P0, assassin).method(FREERUNNING).go();
    assert_eq!(pool(&t, P0), 0);
    assert!(t.obj(spell).chars.has_keyword(KeywordKind::Freerunning));
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Royal Assassin").len(), 1);
}

#[test]
fn ezio_makes_a_player_with_10_or_less_life_lose() {
    cr!("603.12");
    ruling!(
        "Ezio Auditore da Firenze",
        "When Ezio’s triggered ability resolves, it checks to see if that player has 10 or less life at that time."
    );
    // "Whenever Ezio deals combat damage to a player, you may pay {W}{U}{B}{R}{G} if that
    // player has 10 or less life. When you do, that player loses the game."
    for life in [14, 13] {
        let mut t = TestGame::new(2);
        let ezio = t.battlefield(P0, "Ezio Auditore da Firenze");
        t.g.players[P1.idx()].life = life;
        for land in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
            t.lands(P0, land, 1);
        }
        t.answer_yes(P0, true);
        declare_attackers(&mut t, &[(ezio, Entity::Player(P1))]);
        // Until the end of combat, or the end of the game.
        t.g.run_until(10_000, |g| {
            g.result.is_some() || g.turn.step == Step::EndOfCombat
        });
        assert_eq!(t.life(P1), life - 3);
        assert_eq!(t.has_lost(P1), life - 3 <= 10);
    }
}
