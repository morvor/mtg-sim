//! Rulings batch P104 — milling (and exiling from) an opponent's library: Sméagol,
//! Thought Harvester, Undead Alchemist, The Water Crystal, Trepanation Blade, Thieves'
//! Guild Enforcer, Umbris, Dimensional Infiltrator, Teferi's Tutelage, Foreshadow, ...

use crate::r_p104_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::{can_cast, destroy};
use crate::r_s06_common::attach_new;
use crate::r_p189_common::set_library;
use mtg_engine::decision::Decision;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn smeagol_end_step_trigger_once_and_only_if_a_creature_already_died() {
    cr!("603.4");
    ruling!(
        "Sméagol, Helpful Guide",
        "Sméagol, Helpful Guide's first ability will trigger only once during your end step, no matter how many creatures died under your control this turn."
    );
    supported("Sméagol, Helpful Guide");
    // Two creatures died: one trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sméagol, Helpful Guide");
    for _ in 0..2 {
        let b = t.battlefield(P0, "Grizzly Bears");
        destroy(&mut t, b);
    }
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "died under your control"), 1);
    // None died before the end step: no trigger, and a death during it is too late.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sméagol, Helpful Guide");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "died under your control"), 0);
    destroy(&mut t, b);
    assert_eq!(triggers_on_stack(&t, "died under your control"), 0);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn soaring_thought_thief_pumps_itself() {
    cr!("611.3a", "613.4c");
    ruling!(
        "Soaring Thought-Thief",
        "Soaring Thought-Thief’s third ability applies to itself as well as other Rogues you control."
    );
    supported("Soaring Thought-Thief");
    let mut t = TestGame::new(2);
    let thief = t.battlefield(P0, "Soaring Thought-Thief");
    assert_eq!(t.pt(thief), (1, 3));
    graveyard_of(&mut t, P1, 8);
    assert_eq!(t.pt(thief), (2, 3));
}

#[test]
fn thieves_guild_enforcer_bonus_applies_once() {
    cr!("611.3a", "613.4c");
    ruling!(
        "Thieves' Guild Enforcer",
        "Thieves' Guild Enforcer gets +2/+1 just once, no matter how many opponents have eight or more cards in their graveyard and no matter how many cards they have past the eighth."
    );
    supported("Thieves' Guild Enforcer");
    let mut t = TestGame::new(3);
    let e = t.battlefield(P0, "Thieves' Guild Enforcer");
    assert_eq!(t.pt(e), (1, 1));
    graveyard_of(&mut t, P1, 12);
    graveyard_of(&mut t, P2, 9);
    assert_eq!(t.pt(e), (3, 2));
}

/// P0 casts Ornithopter (a colorless spell) with Thought Harvester on the battlefield,
/// and the Harvester's ability is on the stack above it.
fn harvester_trigger(t: &mut TestGame) -> ObjectId {
    supported("Thought Harvester");
    t.battlefield(P0, "Thought Harvester");
    let top = t.library_top(P1, "Grizzly Bears");
    let thopter = t.hand(P0, "Ornithopter");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let spell = t.cast(P0, thopter).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    assert_eq!(*t.g.stack.first().unwrap(), spell);
    top
}

#[test]
fn thought_harvester_trigger_resolves_first_and_exiles_face_up() {
    cr!("603.3", "405.5", "406.3");
    ruling!(
        "Thought Harvester",
        "The triggered ability will resolve before the colorless spell that caused it to trigger."
    );
    ruling!("Thought Harvester", "The card is exiled face up.");
    let mut t = TestGame::new(2);
    let top = harvester_trigger(&mut t);
    t.resolve();
    // The trigger resolved; Ornithopter is still on the stack.
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.zone(top), Zone::Exile);
    assert!(!face_down(&t, top));
    assert!(t.named_on_battlefield("Ornithopter").is_empty());
    t.resolve();
    assert_eq!(t.named_on_battlefield("Ornithopter").len(), 1);
}

#[test]
fn thought_harvester_exile_can_be_processed_by_blight_herder() {
    cr!("603.3b");
    ruling!(
        "Thought Harvester",
        "If Thought Harvester’s ability resolves first (meaning it went on the stack last), the card exiled by that ability can be “processed” by the other ability."
    );
    supported("Thought Harvester");
    supported("Blight Herder");
    // Find the order that puts Thought Harvester's ability on top.
    for order in [vec![0, 1], vec![1, 0]] {
        let mut t = TestGame::new(2);
        let harvester = t.battlefield(P0, "Thought Harvester");
        let pre = t.exile(P1, "Island");
        let top = t.library_top(P1, "Grizzly Bears");
        t.lands(P0, "Wastes", 5);
        let herder = t.hand(P0, "Blight Herder");
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.answer(P0, DecisionKind::Order, Answer::Indices(order));
        t.cast(P0, herder).go();
        t.settle();
        assert_eq!(t.stack_len(), 3);
        let top_source = match &t.g.obj(*t.g.stack.last().unwrap()).stack {
            Some(si) => match &si.kind {
                mtg_engine::object::StackKind::Triggered { source, .. } => *source,
                _ => panic!(),
            },
            None => panic!(),
        };
        if top_source != harvester {
            continue;
        }
        t.resolve();
        assert_eq!(t.zone(top), Zone::Exile);
        // Blight Herder's ability processes both P1 cards in exile, including the new one.
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(pre), obj(t.g.current(top))]);
        t.resolve();
        assert_eq!(t.zone(top), Zone::Graveyard(P1));
        assert_eq!(t.zone(pre), Zone::Graveyard(P1));
        assert_eq!(tokens_with(&t, "Eldrazi"), 3);
        return;
    }
    panic!("no order put Thought Harvester's ability on top");
}

fn tokens_with(t: &TestGame, subtype: &str) -> usize {
    with_subtype(t, P0, subtype)
        .into_iter()
        .filter(|id| t.g.obj(*id).is_token())
        .count()
}

#[test]
fn exiled_from_library_face_up() {
    cr!("406.3");
    ruling!(
        "Sire of Stagnation",
        "The cards are exiled from the library face up."
    );
    ruling!(
        "Mindreaver",
        "The cards are exiled face up. All players can look at them."
    );
    supported("Sire of Stagnation");
    supported("Mindreaver");
    // Sire of Stagnation: P1's land enters.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sire of Stagnation");
    let cards = stack_library(&mut t, P1, &["Grizzly Bears", "Island"]);
    t.enter(P1, "Forest");
    t.g.flush_events();
    t.resolve_all();
    for c in cards {
        assert_eq!(t.zone(c), Zone::Exile);
        assert!(!face_down(&t, c));
    }
    // Mindreaver: heroic (a spell targeting it).
    let mut t = TestGame::new(2);
    let mr = t.battlefield(P0, "Mindreaver");
    let cards = stack_library(&mut t, P1, &["Grizzly Bears", "Island", "Hill Giant"]);
    t.lands(P0, "Forest", 1);
    let growth = t.hand(P0, "Giant Growth");
    t.cast(P0, growth).target(obj(mr)).go();
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    for c in cards {
        assert_eq!(t.zone(c), Zone::Exile);
        assert!(!face_down(&t, c));
    }
}

#[test]
fn necrogoyf_counts_itself_in_a_graveyard() {
    cr!("604.3");
    ruling!(
        "Mystic Redaction",
        "The ability that defines Necrogoyf's power works in all zones, not just the battlefield. If Necrogoyf is in a graveyard, that ability will count itself."
    );
    supported("Necrogoyf");
    let mut t = TestGame::new(2);
    let goyf = t.graveyard(P0, "Necrogoyf");
    t.graveyard(P1, "Grizzly Bears");
    t.g.recompute();
    assert_eq!(t.obj_now(goyf).power(), 2);
}

#[test]
fn water_crystal_reduces_only_generic_mana() {
    cr!("601.2f");
    ruling!(
        "The Water Crystal",
        "The cost reduction applies only to generic mana in the total cost of blue spells you cast."
    );
    supported("The Water Crystal");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Water Crystal");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 1);
    // Counterspell ({U}{U}) isn't reduced: one Island isn't enough.
    let cs = t.hand(P0, "Counterspell");
    assert!(!can_cast(&mut t, P0, cs, CastMethod::Normal));
    // Think Twice ({1}{U}) costs {U}.
    let tt = t.hand(P0, "Think Twice");
    t.cast(P0, tt).go();
    assert_eq!(tapped_lands(&t, P0), 1);
}

#[test]
fn water_crystal_reduces_flashback_cost_but_not_mana_value() {
    cr!("601.2f", "202.3", "702.34a");
    ruling!(
        "The Water Crystal",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying (such as a flashback cost), add any cost increases (such as kicker costs), then apply any cost reductions (such as that of The Water Crystal's first ability). The mana value of the spell is determined by only its mana cost, no matter what the total cost to cast that spell was."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Water Crystal");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 1);
    let tt = t.graveyard(P0, "Think Twice");
    // Flashback {2}{U} - {1} = {1}{U}.
    let spell = t.cast(P0, tt).method(CastMethod::Keyword(mtg_engine::keywords::KeywordKind::Flashback)).go();
    assert_eq!(tapped_lands(&t, P0), 2);
    assert_eq!(t.g.mana_value_of(spell), 2);
}

#[test]
fn trepanation_blade_counts_the_land_and_mills_it_too() {
    cr!("701.20a", "508.1m");
    ruling!(
        "Trepanation Blade",
        "The land card is counted when calculating the bonus, and it will be put into the graveyard with the other revealed cards."
    );
    supported("Trepanation Blade");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Trepanation Blade", obj(bears));
    let cards = stack_library(&mut t, P1, &["Hill Giant", "Lightning Bolt", "Forest"]);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 2));
    for c in cards {
        assert_eq!(t.zone(c), Zone::Graveyard(P1));
    }
}

#[test]
fn umbris_counts_every_card_opponents_own_in_exile() {
    cr!("611.3a", "406.1");
    ruling!(
        "Umbris, Fear Manifest",
        "Umbris, Fear Manifest's first ability counts all cards your opponents own in exile, not just cards exiled by the second ability."
    );
    supported("Umbris, Fear Manifest");
    let mut t = TestGame::new(2);
    let u = t.battlefield(P0, "Umbris, Fear Manifest");
    let base = t.pt(u);
    t.exile(P1, "Grizzly Bears");
    t.exile(P1, "Island");
    // P0's own exiled card isn't counted.
    t.exile(P0, "Island");
    t.g.recompute();
    assert_eq!(t.pt(u), (base.0 + 2, base.1 + 2));
}

#[test]
fn two_undead_alchemists_each_make_a_zombie_but_exile_the_card_once() {
    cr!("603.2", "608.2b", "400.7");
    ruling!(
        "Undead Alchemist",
        "Whenever a creature card is put into an opponent's graveyard from their library, the triggered ability of each Undead Alchemist you control will trigger. The first such ability to resolve will exile that creature card and create a Zombie token. Subsequent abilities won't exile the creature card, but each will create another Zombie token."
    );
    supported("Undead Alchemist");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Undead Alchemist");
    t.battlefield(P0, "Undead Alchemist");
    let bears = t.library_top(P1, "Grizzly Bears");
    mill(&mut t, P1, 1);
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(t.zone(bears), Zone::Exile);
    assert_eq!(tokens_with(&t, "Zombie"), 1);
    t.resolve();
    assert_eq!(t.zone(bears), Zone::Exile);
    assert_eq!(tokens_with(&t, "Zombie"), 2);
}

#[test]
fn mind_sculpt_with_fewer_than_seven_cards() {
    cr!("701.17b");
    ruling!(
        "Mind Sculpt",
        "You can cast Mind Sculpt no matter how many cards are in the opponent’s library. If there are fewer than seven, the opponent will put all of them into their graveyard."
    );
    ruling!(
        "Tolarian Serpent",
        "If less than seven cards in the library, move all that are there."
    );
    supported("Mind Sculpt");
    supported("Tolarian Serpent");
    let mut t = TestGame::new(2);
    set_library(&mut t, P1, &["Island", "Forest", "Grizzly Bears"]);
    t.lands(P0, "Island", 2);
    let ms = t.hand(P0, "Mind Sculpt");
    t.cast(P0, ms).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.library_size(P1), 0);
    assert_eq!(t.graveyard_size(P1), 3);
    // Tolarian Serpent's upkeep mill with two cards left.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tolarian Serpent");
    t.advance_to(P1, Step::Upkeep);
    let lib = set_library(&mut t, P0, &["Island", "Forest"]);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    for c in lib {
        assert_eq!(t.zone(c), Zone::Graveyard(P0));
    }
}

/// Activates Dimensional Infiltrator with `top` on top of P1's library and resolves it,
/// answering `yes` to returning it; returns whether a yes/no question was asked.
fn infiltrate(top: &str, yes: bool) -> (TestGame, ObjectId, bool) {
    supported("Dimensional Infiltrator");
    let mut t = TestGame::new(2);
    let di = t.battlefield(P0, "Dimensional Infiltrator");
    t.lands(P0, "Wastes", 2);
    t.library_top(P1, top);
    t.activate(P0, di, 0, &[Entity::Player(P1)]).unwrap();
    let from = t.asked().len();
    assert_eq!(
        asked_kind(&t, P0, from, |d| matches!(d, Decision::YesNo { .. })),
        0
    );
    t.answer_yes(P0, yes);
    t.resolve_all();
    let asked = asked_kind(&t, P0, from, |d| matches!(d, Decision::YesNo { .. })) > 0;
    (t, di, asked)
}

#[test]
fn dimensional_infiltrator_return_chosen_on_resolution() {
    cr!("608.2c", "608.2d");
    ruling!(
        "Dimensional Infiltrator",
        "You choose whether to return Dimensional Infiltrator to its owner’s hand as its ability resolves. If the exiled card is a land card, you can’t wait and return Dimensional Infiltrator at a later time."
    );
    let (t, di, asked) = infiltrate("Forest", true);
    assert!(asked);
    assert_eq!(t.zone(di), Zone::Hand(P0));
    let (t, di, asked) = infiltrate("Forest", false);
    assert!(asked);
    assert!(t.on_battlefield(di));
    let (t, di, asked) = infiltrate("Grizzly Bears", true);
    assert!(!asked);
    assert!(t.on_battlefield(di));
}

/// Number of Teferi's Tutelage "whenever you draw" triggers on the stack.
fn draw_triggers(g: &mtg_engine::game::Game) -> usize {
    g.stack
        .iter()
        .filter(|id| {
            g.obj(**id).stack.as_ref().is_some_and(|si| {
                matches!(&si.kind, mtg_engine::object::StackKind::Triggered { ability, .. }
                    if ability.text.contains("Whenever you draw"))
            })
        })
        .count()
}

#[test]
fn teferis_tutelage_draw_trigger_waits_for_the_discard() {
    cr!("603.3", "608.2");
    ruling!(
        "Teferi's Tutelage",
        "You draw a card and discard a card all while the first ability is resolving. Nothing can happen between the two, and no player may choose to take actions. This means that the second ability is put onto the stack after you've finished resolving the first ability."
    );
    supported("Teferi's Tutelage");
    let mut t = TestGame::new(2);
    t.hand(P0, "Island");
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseEntities { .. }),
        draw_triggers,
    );
    crate::r_s05_common::enter(&mut t, P0, "Teferi's Tutelage");
    assert_eq!(t.stack_len(), 1);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve();
    // During the discard, the draw trigger hadn't been put on the stack.
    assert_eq!(*seen.lock().unwrap(), vec![0]);
    assert_eq!(draw_triggers(&t.g), 1);
    assert_eq!(triggers_on_stack(&t, "Whenever you draw"), 1);
    assert_eq!(t.graveyard_size(P0), 1);
    t.resolve_all();
    assert_eq!(t.graveyard_size(P1), 2);
}

#[test]
fn foreshadow_names_the_card_on_resolution() {
    cr!("608.2", "201.4");
    ruling!("Foreshadow", "You name the card on resolution.");
    supported("Foreshadow");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    t.library_top(P1, "Grizzly Bears");
    let f = t.hand(P0, "Foreshadow");
    let from = t.asked().len();
    t.cast(P0, f).target(Entity::Player(P1)).go();
    let named = |d: &Decision| matches!(d, Decision::NameCard { .. });
    assert_eq!(asked_kind(&t, P0, from, named), 0);
    t.answer(P0, DecisionKind::Name, Answer::Text("Grizzly Bears".into()));
    let hand = t.hand_size(P0);
    t.resolve();
    assert_eq!(asked_kind(&t, P0, from, named), 1);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn jesters_scepter_cards_are_seen_only_by_its_controller() {
    cr!("406.3");
    ruling!(
        "Jester's Scepter",
        "You can see the cards that are exiled with Jester’s Scepter, but no one else can."
    );
    // (Only the Scepter's first ability is exercised: its second isn't supported yet.)
    let mut t = TestGame::new(2);
    stack_library(
        &mut t,
        P1,
        &["Grizzly Bears", "Island", "Hill Giant", "Shock", "Forest"],
    );
    t.answer_targets(P0, &[Entity::Player(P1)]);
    crate::r_s05_common::enter(&mut t, P0, "Jester's Scepter");
    t.resolve_all();
    let exiled = exiled_of(&t, P1);
    assert_eq!(exiled.len(), 5);
    for c in exiled {
        assert!(face_down(&t, c));
        assert!(mtg_engine::facedown::can_look_at(&t.g, P0, c));
        assert!(!mtg_engine::facedown::can_look_at(&t.g, P1, c));
    }
}
