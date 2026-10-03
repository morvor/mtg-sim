//! Rulings batch P166 — commander synergies (CR 903): the commander tax, "commanders you
//! control", "if you control a commander as you cast this spell", cost reductions and
//! copies keyed to commander casts, and what is counted as the spells resolve.

use crate::r_p146_common::counter;
use crate::r_s01_common::*;
use crate::r_s02_common::can_play_land;
use crate::r_s06_common::{activate_containing, give_control};
use crate::r_s13_common::{commander, commander_game};
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

/// Marks the real card `name` on the battlefield as `owner`'s commander, under P0's
/// control.
fn commander_for_p0(t: &mut TestGame, owner: PlayerId, name: &str) -> ObjectId {
    let id = t.battlefield(owner, name);
    t.g.objects[id.0 as usize].is_commander = true;
    t.g.players[owner.idx()].commander_names.push(name.into());
    t.g.dirty = true;
    if owner != P0 {
        give_control(t, id, P0);
    }
    t.settle();
    t.g.current(id)
}

fn hexproof(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).chars.has_keyword(KeywordKind::Hexproof)
}

#[test]
fn will_of_the_sultai_counts_lands_once_as_it_resolves() {
    cr!("608.2h", "700.2");
    ruling!(
        "Will of the Sultai",
        "The value of X is calculated only once, as Will of the Sultai resolves."
    );
    supported("Will of the Sultai");
    let mut t = TestGame::new(2);
    commander_for_p0(&mut t, P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Mill three lands, return them, then count lands: the returned lands count.
    stack_library(&mut t, P0, &["Forest", "Island", "Swamp"]);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 4);
    let will = t.hand(P0, "Will of the Sultai");
    t.cast(P0, will)
        .modes(&[0, 1])
        .target(Entity::Player(P0))
        .target(Entity::Object(bears))
        .go();
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 8);
    // More lands later don't change it.
    t.lands(P0, "Forest", 2);
    t.settle();
    assert_eq!(t.counters(bears, counters::PLUS1), 8);
}

#[test]
fn casting_a_commander_from_hand_has_no_tax_and_doesnt_add_to_it() {
    cr!("903.8");
    ruling!(
        "Netherborn Altar",
        "Casting a commander from your hand doesn’t require that additional cost, and it doesn’t increase what the cost will be the next time you cast that commander from the command zone."
    );
    supported("Netherborn Altar");
    let mut t = commander_game();
    let isamaru = commander(&mut t, P0, "Isamaru, Hound of Konda");
    let key = mtg_engine::kw::partner::commander_key(&t.g, isamaru);
    // Cast once from the command zone already: the tax is {2}.
    t.g.players[0].commander_casts.insert(key.clone(), 1);
    let altar = t.battlefield(P0, "Netherborn Altar");
    // (Not the command zone instead, CR 903.9b.)
    t.answer_yes(P0, false);
    activate_containing(&mut t, P0, altar, "soul").unwrap();
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(isamaru)), Zone::Hand(P0));
    assert_eq!(t.life(P0), 37);
    // From the hand: just {W}.
    t.lands(P0, "Plains", 1);
    let card = t.g.current(isamaru);
    t.cast(P0, card).go();
    t.resolve_all();
    let now = t.g.current(isamaru);
    assert!(t.on_battlefield(now));
    assert_eq!(t.g.player(P0).commander_casts.get(&key), Some(&1));
    // Back to the command zone: the tax is still {2}, not {4}.
    t.answer_yes(P0, true);
    t.g.destroy(now, None);
    t.settle();
    assert_eq!(t.zone(t.g.current(isamaru)), Zone::Command);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Wastes", 1);
    let card = t.g.current(isamaru);
    assert!(t.cast(P0, card).try_go().is_err());
    t.lands(P0, "Wastes", 1);
    let card = t.g.current(isamaru);
    t.cast(P0, card).go();
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(isamaru)));
    assert_eq!(t.g.player(P0).commander_casts.get(&key), Some(&2));
}

#[test]
fn guardian_augmenter_affects_any_commander_you_control() {
    cr!("903.3", "613.1f");
    ruling!(
        "Guardian Augmenter",
        "These effects apply to any commanders you control, not just your own commander."
    );
    supported("Guardian Augmenter");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Guardian Augmenter");
    let own = commander_for_p0(&mut t, P0, "Grizzly Bears");
    let theirs = commander_for_p0(&mut t, P1, "Hill Giant");
    let other = t.battlefield(P1, "Isamaru, Hound of Konda");
    t.g.objects[other.0 as usize].is_commander = true;
    t.g.dirty = true;
    t.settle();
    assert_eq!(t.pt(own), (4, 4));
    assert!(hexproof(&t, own));
    assert_eq!(t.pt(theirs), (5, 5));
    assert!(hexproof(&t, theirs));
    // A commander P1 controls doesn't get it.
    assert_eq!(t.pt(other), (2, 2));
    assert!(!hexproof(&t, other));
}

fn drake_game() -> (TestGame, ObjectId) {
    supported("Thunderclap Drake");
    let mut t = TestGame::new(2);
    let drake = t.battlefield(P0, "Thunderclap Drake");
    (t, drake)
}

#[test]
fn thunderclap_drake_reduces_only_generic_mana() {
    cr!("601.2f", "118.7d");
    ruling!(
        "Thunderclap Drake",
        "Thunderclap Drake's second ability can't reduce the amount of colored mana you pay for a spell. It reduces only the generic mana component of that spell's cost."
    );
    // Lightning Bolt ({R}) with only an Island: still needs {R}.
    let (mut t, _) = drake_game();
    t.lands(P0, "Island", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    assert!(t
        .cast(P0, bolt)
        .target(Entity::Player(P1))
        .try_go()
        .is_err());
    // Counterspell ({U}{U}) with one Island: still two blue.
    let (mut t, _) = drake_game();
    t.lands(P0, "Island", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.lands(P0, "Mountain", 1);
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    let cs = t.hand(P0, "Counterspell");
    assert!(t.cast(P0, cs).target(bolt).try_go().is_err());
    // Think Twice ({1}{U}) with one Island: the {1} is reduced.
    let (mut t, _) = drake_game();
    t.lands(P0, "Island", 1);
    let tt = t.hand(P0, "Think Twice");
    t.cast(P0, tt).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Think Twice"));
}

#[test]
fn thunderclap_drake_doesnt_change_mana_value() {
    cr!("601.2f", "202.3", "118.7d");
    ruling!(
        "Thunderclap Drake",
        "Thunderclap Drake's second ability doesn't change the mana cost or mana value of any spell. It changes only the total cost you pay to cast instant and sorcery spells."
    );
    let (mut t, _) = drake_game();
    t.lands(P0, "Island", 1);
    let tt = t.hand(P0, "Think Twice");
    let spell = t.cast(P0, tt).go();
    assert_eq!(t.g.stack.last().copied(), Some(spell));
    assert_eq!(t.g.mana_value_of(spell), 2);
    assert_eq!(tapped_lands(&t, P0), 1);
}

#[test]
fn thunderclap_drake_copies_keep_the_kicker() {
    cr!("707.10", "702.33d", "903.8");
    ruling!(
        "Thunderclap Drake",
        "You can't choose to pay any additional costs for the copies. However, effects based on any additional costs that were paid for the original spell are copied as though those same costs were paid for the copies too."
    );
    let mut t = commander_game();
    let isamaru = commander(&mut t, P0, "Isamaru, Hound of Konda");
    let key = mtg_engine::kw::partner::commander_key(&t.g, isamaru);
    t.g.players[0].commander_casts.insert(key, 1);
    let drake = t.battlefield(P0, "Thunderclap Drake");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 2);
    activate_containing(&mut t, P0, drake, "Sacrifice").unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(drake));
    // Burst Lightning, kicked ({4}): 4 damage, and so does its copy.
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 4);
    let burst = t.hand(P0, "Burst Lightning");
    t.answer_yes(P0, false);
    t.cast(P0, burst)
        .kicked(true)
        .target(Entity::Player(P1))
        .go();
    t.resolve_all();
    // 40 - 4 - 4 (a Commander game).
    assert_eq!(t.life(P1), 32);
}

#[test]
fn moving_counters_puts_them_on_the_other_permanent() {
    cr!("122.5", "122.6");
    ruling!(
        "Nexus Mentality",
        "To move a counter from one permanent to another, the counter is removed from the first permanent and put onto the second. Any abilities that care about a counter being removed from or placed on a permanent will apply."
    );
    supported("Nexus Mentality");
    supported("Fathom Mage");
    // Fathom Mage: "Whenever a +1/+1 counter is put on this creature, you may draw a card."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(Entity::Object(bears), counters::PLUS1, 2, None);
    let mage = t.battlefield(P0, "Fathom Mage");
    t.settle();
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 3);
    let hand = t.hand_size(P0);
    let nm = t.hand(P0, "Nexus Mentality");
    t.answer(P0, DecisionKind::YesNo, Answer::Bool(true));
    t.answer(P0, DecisionKind::YesNo, Answer::Bool(true));
    t.cast(P0, nm)
        .modes(&[0])
        .targets(&[Entity::Object(bears), Entity::Object(mage)])
        .go();
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 0);
    assert_eq!(t.counters(mage, counters::PLUS1), 2);
    assert!(t.hand_size(P0) > hand, "Fathom Mage's trigger drew");
}

#[test]
fn jeskas_will_counts_the_opponents_hand_as_it_resolves() {
    cr!("608.2h");
    ruling!(
        "Jeska's Will",
        "Use the number of cards in the target opponent's hand as Jeska's Will resolves to determine how much {R} to add."
    );
    supported("Jeska's Will");
    let mut t = TestGame::new(2);
    t.hand(P1, "Grizzly Bears");
    t.hand(P1, "Hill Giant");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 2);
    let will = t.hand(P0, "Jeska's Will");
    t.cast(P0, will).modes(&[0]).target(Entity::Player(P1)).go();
    // P1 gets another card with the spell on the stack.
    t.hand(P1, "Opt");
    let n = t.hand_size(P1);
    assert_eq!(n, 3);
    t.resolve();
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::R), n);
}

#[test]
fn jeskas_will_lands_follow_the_normal_timing_rules() {
    cr!("305.2", "305.1", "601.2");
    ruling!(
        "Jeska's Will",
        "You pay all costs and follow all timing rules for cards played with the permission from the last mode of Jeska's Will. For example, if one of the exiled cards is a land card, you may play it only during your main phase while the stack is empty and only if you have an available land play remaining."
    );
    let setup = || {
        let mut t = TestGame::new(2);
        stack_library(&mut t, P0, &["Forest", "Grizzly Bears", "Opt"]);
        t.lands(P0, "Mountain", 1);
        t.lands(P0, "Wastes", 2);
        let will = t.hand(P0, "Jeska's Will");
        t.cast(P0, will).modes(&[1]).go();
        t.resolve_all();
        assert!(t.in_exile("Forest"));
        let forest = t.g.find_in_zone(Zone::Exile, "Forest")[0];
        (t, forest)
    };
    let (mut t, forest) = setup();
    assert!(can_play_land(&mut t, P0, forest));
    // No land play left.
    let (mut t, forest) = setup();
    t.g.players[0].lands_played_this_turn = 1;
    assert!(!can_play_land(&mut t, P0, forest));
    // The stack isn't empty.
    let (mut t, forest) = setup();
    t.lands(P0, "Island", 1);
    let opt = t.hand(P0, "Opt");
    t.cast(P0, opt).go();
    assert!(!can_play_land(&mut t, P0, forest));
    // Not a main phase.
    let (mut t, forest) = setup();
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_play_land(&mut t, P0, forest));
    // A spell among them: its mana cost is paid. Grizzly Bears can't be cast without
    // mana.
    let (mut t, _) = setup();
    let bears = t.g.find_in_zone(Zone::Exile, "Grizzly Bears")[0];
    assert!(!crate::r_s02_common::can_cast(
        &mut t,
        P0,
        bears,
        CastMethod::Normal
    ));
    t.lands(P0, "Forest", 2);
    assert!(crate::r_s02_common::can_cast(
        &mut t,
        P0,
        bears,
        CastMethod::Normal
    ));
}

#[test]
fn vega_draws_before_the_spell_resolves_even_if_its_countered() {
    cr!("603.3", "701.6a");
    ruling!(
        "Vega, the Watcher",
        "Vega’s ability will resolve before the spell that caused it to trigger. It will resolve even if that spell is countered."
    );
    supported("Vega, the Watcher");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vega, the Watcher");
    let tt = t.graveyard(P0, "Think Twice");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 2);
    stack_library(&mut t, P0, &["Opt", "Hill Giant", "Grizzly Bears"]);
    let hand = t.hand_size(P0);
    t.cast(P0, tt)
        .method(CastMethod::Keyword(KeywordKind::Flashback))
        .go();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "draw a card"), 1);
    // The trigger is on top of the spell.
    let spell = t.g.stack[0];
    counter(&mut t, spell);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn will_modes_stay_chosen_if_the_commander_is_sacrificed_for_mana() {
    cr!("601.2b", "601.2g", "605.3a", "700.2");
    ruling!(
        "Will of the Abzan",
        "Whether or not you control a commander is determined only once, as you choose the modes for this spell. If you somehow lose control of that commander before you finish casting the spell (perhaps because you sacrifice it to activate a mana ability), it won’t change the number of modes chosen."
    );
    supported("Will of the Abzan");
    supported("Ashnod's Altar");
    let mut t = TestGame::new(2);
    let cmdr = commander_for_p0(&mut t, P0, "Hill Giant");
    t.battlefield(P0, "Ashnod's Altar");
    t.battlefield(P1, "Grizzly Bears");
    let dead = t.graveyard(P0, "Isamaru, Hound of Konda");
    // {3}{B}: a Swamp, a Wastes, and {C}{C} from sacrificing the commander.
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Wastes", 1);
    t.answer_choose(P0, &[Entity::Object(cmdr)]);
    let will = t.hand(P0, "Will of the Abzan");
    t.cast(P0, will)
        .modes(&[0, 1])
        .target(Entity::Player(P1))
        .target(Entity::Object(dead))
        .go();
    assert!(
        !t.on_battlefield(cmdr),
        "the commander was sacrificed for mana"
    );
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.life(P1), 17);
    assert!(t.on_battlefield(t.g.current(dead)));
}
