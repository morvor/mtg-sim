//! Rulings batch P137 — what edicts target: "target player sacrifices a creature" targets
//! only the player, so a creature with hexproof or protection can be sacrificed (CR 115.1,
//! 702.11, 702.16); "target artifact creature's controller sacrifices it" targets only the
//! creature; and partially illegal targets as a spell resolves (CR 608.2b).

use crate::r_p137_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

fn add_mana(t: &mut TestGame, p: PlayerId, ty: ManaType, n: u32) {
    t.g.players[p.idx()].mana_pool.add_type(ty, n);
}

/// The targets P0 was offered for the first target choice since `from`.
fn first_targets(t: &TestGame, from: usize) -> Vec<Entity> {
    target_candidates(t, P0, from)
        .into_iter()
        .next()
        .unwrap_or_default()
}

#[test]
fn player_edicts_can_take_hexproof_and_protected_creatures() {
    cr!("115.1", "702.11b", "702.16b", "701.21a");
    ruling!(
        "Far // Away",
        "Away targets only a player. It doesn't target any creatures. The target player chooses which creature to sacrifice as the spell resolves."
    );
    ruling!(
        "Dead Drop",
        "Dead Drop doesn't target any creature. The target player could sacrifice a creature with hexproof, for example."
    );
    ruling!(
        "Devour Flesh",
        "Devour Flesh targets only a player. It doesn’t target any creatures. The target player chooses which creature to sacrifice when the spell resolves. That player can sacrifice only a creature they control."
    );
    ruling!("Urborg Justice", "Does not target the creatures.");
    // (card, how to cast it)
    type Cast = fn(&mut TestGame) -> ObjectId;
    let cases: [(&str, Cast); 4] = [
        ("Far // Away", |t| {
            let c = t.hand(P0, "Far // Away");
            add_mana(t, P0, ManaType::B, 1);
            add_mana(t, P0, ManaType::C, 2);
            t.cast(P0, c)
                .method(CastMethod::Half(1))
                .target(Entity::Player(P1))
                .go()
        }),
        ("Dead Drop", |t| {
            let c = t.hand(P0, "Dead Drop");
            add_mana(t, P0, ManaType::B, 10);
            t.cast(P0, c).target(Entity::Player(P1)).go()
        }),
        ("Devour Flesh", |t| {
            let c = t.hand(P0, "Devour Flesh");
            add_mana(t, P0, ManaType::B, 2);
            t.cast(P0, c).target(Entity::Player(P1)).go()
        }),
        ("Urborg Justice", |t| {
            // One creature of P0's died this turn.
            let mine = t.battlefield(P0, "Grizzly Bears");
            destroy(t, mine);
            let c = t.hand(P0, "Urborg Justice");
            add_mana(t, P0, ManaType::B, 2);
            t.cast(P0, c).target(Entity::Player(P1)).go()
        }),
    ];
    for (card, cast) in cases {
        supported(card);
        let mut t = TestGame::new(2);
        let scout = t.battlefield(P1, "Gladecover Scout");
        let knight = t.battlefield(P1, "White Knight");
        let n = if card == "Dead Drop" { 2 } else { 1 };
        let chosen = if n == 2 {
            vec![obj(scout), obj(knight)]
        } else {
            vec![obj(scout)]
        };
        t.answer_choose(P1, &chosen);
        let from = t.asked().len();
        cast(&mut t);
        // Only players could be targeted.
        assert!(
            first_targets(&t, from).iter().all(|e| e.player().is_some()),
            "{card}"
        );
        t.resolve_all();
        assert!(!t.on_battlefield(scout), "{card}: hexproof creature sacrificed");
        assert_eq!(t.on_battlefield(knight), n == 1, "{card}");
        // The target player chose among their own creatures only.
        for offered in offered_to(&t, P1, from) {
            assert!(offered.iter().all(|o| t.g.obj(*o).controller == P1), "{card}");
        }
        if card == "Devour Flesh" {
            assert_eq!(t.life(P1), 21);
        }
    }
}

#[test]
fn bearer_of_silence_targets_only_the_opponent() {
    cr!("115.1", "702.11b", "603.2");
    ruling!(
        "Bearer of Silence",
        "Bearer of Silence's triggered ability targets only the player, not any creature. For example, the player may sacrifice a creature with hexproof."
    );
    supported("Bearer of Silence");
    let mut t = TestGame::new(2);
    let scout = t.battlefield(P1, "Gladecover Scout");
    let bearer = t.hand(P0, "Bearer of Silence");
    add_mana(&mut t, P0, ManaType::B, 2);
    add_mana(&mut t, P0, ManaType::C, 2);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, bearer).go();
    t.resolve_all();
    assert!(!t.on_battlefield(scout));
    assert!(t.in_graveyard(P1, "Gladecover Scout"));
}

#[test]
fn crackling_doom_sacrifices_a_creature_with_protection_from_black() {
    cr!("702.16b", "701.21a");
    ruling!(
        "Crackling Doom",
        "Crackling Doom doesn't target any player or creature. For example, a creature with protection from black could be sacrificed."
    );
    ruling!(
        "Crackling Doom",
        "If a player controls two or more creatures tied for the greatest power among creatures they control, that player chooses one of them to sacrifice."
    );
    supported("Crackling Doom");
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P1, "White Knight");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let elf = t.battlefield(P1, "Elvish Mystic");
    // P1 chooses the White Knight among its two creatures with power 2.
    t.answer_choose(P1, &[obj(knight)]);
    give_mana_for(&mut t, P0, "Crackling Doom");
    let spell = t.hand(P0, "Crackling Doom");
    let from = t.asked().len();
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(!t.on_battlefield(knight));
    assert!(t.on_battlefield(bears) && t.on_battlefield(elf));
    assert_eq!(t.life(P1), 18);
    let offered = offered_to(&t, P1, from);
    assert_eq!(offered.len(), 1);
    assert_eq!(offered[0].len(), 2);
}

#[test]
fn arcum_dagsson_targets_the_creature_not_its_controller() {
    cr!("115.1", "702.11b");
    ruling!(
        "Arcum Dagsson",
        "Arcum Dagsson's ability targets an artifact creature, not the player who controls it."
    );
    ruling!(
        "Arcum Dagsson",
        "Any triggered abilities that trigger when the creature is sacrificed will be put onto the stack at the same time as any triggered abilities that trigger when the noncreature artifact enters the battlefield."
    );
    supported("Arcum Dagsson");
    let mut t = TestGame::new(2);
    let arcum = t.battlefield(P0, "Arcum Dagsson");
    // P1 has hexproof; its Ornithopter can still be targeted.
    t.battlefield(P1, "Leyline of Sanctity");
    let thopter = t.battlefield(P1, "Ornithopter");
    // A "dies" trigger and an "enters" trigger, both P0's.
    t.battlefield(P0, "Moonlit Wake");
    t.battlefield(P0, "Vermiculos");
    let relic = t.library_top(P1, "Darksteel Relic");
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[obj(relic)]);
    t.activate(P0, arcum, 0, &[obj(thopter)]).unwrap();
    t.resolve();
    assert!(t.in_graveyard(P1, "Ornithopter"));
    assert_eq!(t.named_on_battlefield("Darksteel Relic").len(), 1);
    // Both triggers waited until the ability finished resolving, then went on the stack
    // together.
    assert_eq!(t.stack_len(), 2);
}

#[test]
fn geths_verdict_loses_life_even_without_a_creature() {
    cr!("608.2c", "701.21a");
    ruling!(
        "Geth's Verdict",
        "As long as the player is a legal target when Geth’s Verdict resolves, that player loses 1 life even if they didn’t sacrifice a creature (most likely because they didn’t control any)."
    );
    supported("Geth's Verdict");
    let mut t = TestGame::new(2);
    give_mana_for(&mut t, P0, "Geth's Verdict");
    let spell = t.hand(P0, "Geth's Verdict");
    t.cast(P0, spell).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn grave_exchange_needs_both_targets_but_the_player_may_have_no_creatures() {
    cr!("601.2c", "115.1");
    ruling!(
        "Grave Exchange",
        "Grave Exchange targets both a creature card in your graveyard and a player. You can’t cast it unless there is a legal target to choose for both. You can target a player who controls no creatures."
    );
    supported("Grave Exchange");
    let mut t = TestGame::new(2);
    give_mana_for(&mut t, P0, "Grave Exchange");
    let spell = t.hand(P0, "Grave Exchange");
    // No creature card in P0's graveyard: it can't be cast.
    assert!(!can_cast(&mut t, P0, spell, CastMethod::Normal));
    let bears = t.graveyard(P0, "Grizzly Bears");
    assert!(can_cast(&mut t, P0, spell, CastMethod::Normal));
    // P1 controls no creatures and can still be targeted.
    t.cast(P0, spell)
        .target(obj(bears))
        .target(Entity::Player(P1))
        .go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn silverquill_command_with_all_targets_illegal_does_nothing() {
    cr!("608.2b");
    ruling!(
        "Silverquill Command",
        "If all of Silverquill Command’s targets are illegal as it tries to resolve, it will do nothing."
    );
    supported("Silverquill Command");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lions = t.graveyard(P0, "Savannah Lions");
    give_mana_for(&mut t, P0, "Silverquill Command");
    let spell = t.hand(P0, "Silverquill Command");
    t.cast(P0, spell)
        .modes(&[0, 1])
        .target(obj(bears))
        .target(obj(lions))
        .go();
    // Both targets become illegal.
    destroy(&mut t, bears);
    let lions_now = t.g.current(lions);
    t.g.move_object(
        lions_now,
        Zone::Exile,
        mtg_engine::events::MoveCause::Effect,
        None,
    );
    t.resolve_all();
    assert!(t.named_on_battlefield("Savannah Lions").is_empty());
    assert!(t.in_graveyard(P0, "Silverquill Command"));
    assert!(t.in_exile("Savannah Lions"));
}

#[test]
fn rootcast_apprenticeship_partial_and_all_illegal_targets() {
    cr!("608.2b", "700.2d");
    ruling!(
        "Rootcast Apprenticeship",
        "If all targets for the chosen modes become illegal before Rootcast Apprenticeship resolves, the spell will be countered and none of its effects will happen. If at least one target is still legal, the spell will resolve but will have no effect on any illegal targets."
    );
    supported("Rootcast Apprenticeship");
    // One target illegal: the rest of the spell still happens.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lions = t.battlefield(P0, "Savannah Lions");
    give_mana_for(&mut t, P0, "Rootcast Apprenticeship");
    let spell = t.hand(P0, "Rootcast Apprenticeship");
    // Two +1/+1 counters on the Bears, two on the Lions, and P0 creates a Squirrel.
    t.cast(P0, spell)
        .modes(&[0, 0, 2])
        .target(obj(bears))
        .target(obj(lions))
        .target(Entity::Player(P0))
        .go();
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.counters(lions, mtg_engine::types::counters::PLUS1), 2);
    assert_eq!(creatures(&t, P0).len(), 2, "the Lions and a Squirrel");
    // All targets illegal: countered, nothing happens.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lions = t.battlefield(P0, "Savannah Lions");
    give_mana_for(&mut t, P0, "Rootcast Apprenticeship");
    let spell = t.hand(P0, "Rootcast Apprenticeship");
    t.cast(P0, spell)
        .modes(&[0, 0, 0])
        .target(obj(bears))
        .target(obj(lions))
        .target(obj(lions))
        .go();
    destroy(&mut t, bears);
    destroy(&mut t, lions);
    t.resolve_all();
    assert!(creatures(&t, P0).is_empty());
    assert!(t.in_graveyard(P0, "Rootcast Apprenticeship"));
}

#[test]
fn rootcast_apprenticeship_copy_keeps_modes_with_new_targets() {
    cr!("707.10", "707.10c");
    ruling!(
        "Rootcast Apprenticeship",
        "If Rootcast Apprenticeship is copied, the effect that creates the copy will usually allow you to choose new targets, but you can't choose new modes."
    );
    supported("Rootcast Apprenticeship");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lions = t.battlefield(P0, "Savannah Lions");
    give_mana_for(&mut t, P0, "Rootcast Apprenticeship");
    let spell = t.hand(P0, "Rootcast Apprenticeship");
    t.cast(P0, spell)
        .modes(&[0, 0, 0])
        .target(obj(bears))
        .target(obj(bears))
        .target(obj(bears))
        .go();
    // Twincast the spell, changing every target to the Lions.
    let twincast = t.hand(P0, "Twincast");
    add_mana(&mut t, P0, ManaType::U, 2);
    t.cast(P0, twincast).target(obj(spell)).go();
    t.answer(P0, DecisionKind::YesNo, Answer::Bool(true));
    t.answer(P0, DecisionKind::YesNo, Answer::Bool(true));
    t.answer(P0, DecisionKind::YesNo, Answer::Bool(true));
    t.answer_targets(P0, &[obj(lions)]);
    t.answer_targets(P0, &[obj(lions)]);
    t.answer_targets(P0, &[obj(lions)]);
    t.resolve_all();
    // The copy kept the three "two +1/+1 counters" modes.
    assert_eq!(t.counters(bears, mtg_engine::types::counters::PLUS1), 6);
    assert_eq!(t.counters(lions, mtg_engine::types::counters::PLUS1), 6);
}
