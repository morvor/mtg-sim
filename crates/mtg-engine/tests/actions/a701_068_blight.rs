//! CR 701.68: blight.

use crate::a701_028_071_common::*;
use mtg_engine::ability::*;
use mtg_engine::events::MoveCause;
use mtg_engine::kwa::blight::BLIGHTED_EVENT;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn blight_puts_n_minus_one_counters_on_a_creature_you_control() {
    cr!("701.68a");
    ruling!(
        "Sting-Slinger",
        "All of the -1/-1 counters must be put on a single creature."
    );
    supported("Sting-Slinger");
    // "{1}{R}, {T}, Blight 1: This creature deals 2 damage to each opponent."
    let mut t = TestGame::new(2);
    let slinger = t.battlefield(P0, "Sting-Slinger");
    let giant = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    choose(&mut t, P0, &[giant]);
    t.activate(P0, slinger, 0, &[]).unwrap();
    // The cost was paid: the counter is on the chosen creature before the ability resolves.
    assert_eq!(t.counters(giant, "-1/-1"), 1);
    assert_eq!(t.counters(slinger, "-1/-1"), 0);
    let cands: Vec<Vec<Entity>> = t
        .asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates),
            _ => None,
        })
        .collect();
    assert_eq!(
        cands,
        vec![vec![Entity::Object(slinger), Entity::Object(giant)]]
    );
    assert!(!cands[0].contains(&Entity::Object(theirs)));
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.pt(giant), (2, 2));
}

#[test]
fn the_blighted_creature_needs_no_toughness_to_survive() {
    cr!("701.68a");
    ruling!(
        "Sting-Slinger",
        "The creature you choose to put -1/-1 counters on doesn't have to have enough toughness to survive the process."
    );
    let mut t = TestGame::new(2);
    let slinger = t.battlefield(P0, "Sting-Slinger");
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.lands(P0, "Mountain", 2);
    choose(&mut t, P0, &[elves]);
    t.activate(P0, slinger, 0, &[]).unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(elves));
    assert_eq!(t.life(P1), 18);
}

#[test]
fn a_player_who_controls_no_creatures_cant_choose_to_blight() {
    cr!("701.68b");
    ruling!(
        "Sting-Slinger",
        "If you can't place -1/-1 counters on any creatures you control (probably because you control no creatures), you can't choose to blight."
    );
    supported("Dream Seizer");
    // "When this creature enters, you may blight 1. If you do, each opponent discards a
    // card."
    let mut t = TestGame::new(2);
    t.hand(P1, "Grizzly Bears");
    let seizer = t.enter(P0, "Dream Seizer");
    t.settle();
    // It leaves before its ability resolves: P0 controls no creatures.
    t.g.move_object(seizer, Zone::Hand(P0), MoveCause::Effect, None);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 1);
    assert!(custom_events(&t, BLIGHTED_EVENT).is_empty());
    // With a creature, they can.
    let mut t = TestGame::new(2);
    t.hand(P1, "Grizzly Bears");
    let seizer = t.enter(P0, "Dream Seizer");
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 0);
    assert_eq!(t.counters(seizer, "-1/-1"), 1);
}

#[test]
fn the_blighted_creature_is_the_one_chosen() {
    cr!("701.68c");
    supported("Grub, Storied Matriarch");
    // Grub, Notorious Auntie: "Whenever Grub attacks, you may blight 1. If you do, create a
    // tapped and attacking token that's a copy of the blighted creature, except it has
    // 'At the beginning of the end step, sacrifice this token.'"
    let mut t = TestGame::new(2);
    let grub = t.battlefield(P0, "Grub, Storied Matriarch");
    mtg_engine::dfc::transform(&mut t.g, grub);
    t.g.recompute();
    assert_eq!(t.obj(grub).chars.name.as_str(), "Grub, Notorious Auntie");
    let giant = t.battlefield(P0, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer_yes(P0, true);
    choose(&mut t, P0, &[giant]);
    t.attack(&[(grub, Entity::Player(P1))], &[]);
    let copies: Vec<ObjectId> = t
        .named_on_battlefield("Hill Giant")
        .into_iter()
        .filter(|o| t.obj(*o).is_token())
        .collect();
    assert_eq!(copies.len(), 1);
    assert_eq!(t.counters(giant, "-1/-1"), 1);
    // A copy of the blighted creature (copiable values only: no counters), attacking.
    assert_eq!(t.pt(copies[0]), (3, 3));
    assert!(t.obj(copies[0]).tapped);
    assert_eq!(t.life(P1), 20 - 2 - 3);
}

#[test]
fn a_player_blights_whatever_events_actually_occurred() {
    cr!("701.68d");
    let mut t = TestGame::new(2);
    let watcher = text_card(
        "Blight Watcher",
        "Enchantment",
        "{0}",
        None,
        "Whenever you blight, you gain 1 life.",
    );
    t.custom(P0, watcher, Zone::Battlefield);
    let bears = t.battlefield(P0, "Grizzly Bears");
    // No counters can be put on creatures this turn.
    run(
        &mut t,
        P1,
        None,
        Effect::AddReplacement {
            def: ReplacementDef {
                event: ReplacementEvent::PutCounters {
                    on_objects: Some(Filter::creature()),
                    on_players: None,
                    kind: None,
                },
                action: ReplacementAction::Prevent,
                self_replacement: false,
                optional: false,
            },
            duration: Duration::EndOfTurn,
            uses: None,
        },
        &[],
    );
    run(&mut t, P0, None, ka(KeywordAction::Blight, Sel::None, 2), &[]);
    t.resolve_all();
    assert_eq!(t.counters(bears, "-1/-1"), 0);
    assert_eq!(
        custom_events(&t, BLIGHTED_EVENT),
        vec![(Some(P0), Some(bears), 2)]
    );
    assert_eq!(t.life(P0), 21);
}
