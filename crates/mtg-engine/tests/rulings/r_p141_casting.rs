//! Rulings batch P141 — casting from engines: cascade granted by effects (Smoldering
//! Stagecoach, Yidris, Maelstrom Nexus), casting during a resolution (Svella, The Tale of
//! Tamiyo, Possibility Storm), triggers on casting that resolve first (Arjun, Mindmoil),
//! rebound on a self-exiling spell (Narset), playing cards from graveyards and exile
//! (Coram, Grolnok, Intet), adventurers (Kellan), and costs (Gorex, Ajani).

use crate::r_p141_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn can_cast(t: &mut TestGame, p: PlayerId, card: ObjectId, m: CastMethod) -> bool {
    crate::r_s02_common::can_cast(t, p, card, m)
}

/// P0 attacks with a crewed Smoldering Stagecoach (it then has its attack trigger
/// resolved) and moves to the postcombat main phase.
fn stagecoach_attacked(t: &mut TestGame) {
    let s = t.battlefield(P0, "Smoldering Stagecoach");
    let crew = t.battlefield(P0, "Hill Giant");
    assert!(crate::r_s04_common::crew(t, P0, s, &[crew]));
    t.resolve_all();
    attack_with(t, &[(s, Entity::Player(P1))]);
    t.resolve_all();
    t.advance_to(P0, Step::PostcombatMain);
}

#[test]
fn smoldering_stagecoach_cascade_casts_without_paying_but_with_additional_costs() {
    cr!("702.85a", "118.9", "601.2b");
    ruling!(
        "Smoldering Stagecoach",
        "\"Cascade\" means \"When you cast this spell, exile cards from the top of your library until you exile a nonland card whose mana value is less than this spell's mana value."
    );
    ruling!(
        "Smoldering Stagecoach",
        "If you cast a spell \"without paying its mana cost,\" you can't choose to cast it for any alternative costs. You can, however, pay additional costs. If the spell has any mandatory additional costs, you must pay those to cast the card."
    );
    supported("Smoldering Stagecoach");
    supported("Village Rites");
    // The next sorcery (Divination, mana value 3) cascades into Village Rites (1): its
    // mandatory sacrifice is paid.
    let mut t = TestGame::new(2);
    stagecoach_attacked(&mut t);
    let bears = t.battlefield(P0, "Grizzly Bears");
    stack_library(&mut t, P0, &["Forest", "Village Rites", "Opt", "Opt", "Opt", "Opt"]);
    yes(&mut t, P0);
    t.answer_choose(P0, &[obj(bears)]);
    let hand = t.hand_size(P0);
    cast_new(&mut t, P0, "Divination", &[]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "ascade"), 1);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Village Rites"));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    // Divination drew 2, Village Rites 2.
    assert_eq!(t.hand_size(P0), hand + 4);
    // The Forest was exiled and put on the bottom (Village Rites, the next instant this
    // turn, cascaded too: its cascade put the Opts on the bottom as well).
    assert!(!t.in_exile("Forest") && !t.in_hand(P0, "Forest"));
    assert_eq!(t.library_size(P0), 30 + 6 - 2 - 4);
}

#[test]
fn yidris_twice_gives_cascade_twice_and_nexus_adds_a_second_cascade() {
    cr!("702.85c", "603.2");
    ruling!(
        "Yidris, Maelstrom Wielder",
        "If Yidris's triggered ability resolves more than once in one turn, spells you cast from your hand will gain cascade that many times. Each instance of cascade triggers separately"
    );
    ruling!(
        "Maelstrom Nexus",
        "If the first spell you cast in a turn already has cascade, both cascade abilities will trigger separately."
    );
    supported("Yidris, Maelstrom Wielder");
    supported("Maelstrom Nexus");
    // Yidris with double strike deals combat damage twice.
    let mut t = TestGame::new(2);
    let y = t.battlefield(P0, "Yidris, Maelstrom Wielder");
    grant(&mut t, y, KeywordKind::DoubleStrike);
    t.attack(&[(y, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 10);
    t.advance_to(P0, Step::PostcombatMain);
    stack_library(&mut t, P0, &["Opt", "Opt", "Opt", "Opt"]);
    cast_new(&mut t, P0, "Divination", &[]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "ascade"), 2);
    t.resolve_all();
    assert_eq!(t.g.player(P0).graveyard.len(), 3, "two Opts and Divination");
    // Maelstrom Nexus and Bloodbraid Elf.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Maelstrom Nexus");
    stack_library(&mut t, P0, &["Opt", "Opt", "Opt", "Opt"]);
    cast_new(&mut t, P0, "Bloodbraid Elf", &[]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "ascade"), 2);
}

#[test]
fn svella_casts_during_the_resolution_ignoring_timing() {
    cr!("608.2g", "601.2", "307.1");
    ruling!(
        "Svella, Ice Shaper",
        "If you cast a spell using Svella’s ability, you do so as part of the resolution of that ability. You can’t wait to cast it later in the turn. Timing permissions based on the card’s type are ignored."
    );
    supported("Svella, Ice Shaper");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Svella, Ice Shaper");
    t.lands(P0, "Mountain", 4);
    t.lands(P0, "Forest", 4);
    t.set_step(P0, Step::BeginningOfCombat);
    let cards = stack_library(&mut t, P0, &["Divination", "Forest", "Forest", "Forest"]);
    t.answer_choose_moved(P0, &[obj(cards[0])]);
    yes(&mut t, P0);
    let hand = t.hand_size(P0);
    t.activate(P0, s, 1, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Divination"));
    assert_eq!(t.hand_size(P0), hand + 2);
    // Declined: it goes to the bottom with the rest.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Svella, Ice Shaper");
    t.lands(P0, "Mountain", 4);
    t.lands(P0, "Forest", 4);
    stack_library(&mut t, P0, &["Divination", "Forest", "Forest", "Forest"]);
    t.answer_yes(P0, false);
    t.activate(P0, s, 1, &[]).unwrap();
    t.resolve_all();
    assert!(!t.in_graveyard(P0, "Divination") && !t.in_hand(P0, "Divination"));
    let lib = &t.g.player(P0).library;
    assert!(lib[..4]
        .iter()
        .any(|id| t.g.obj(*id).chars.name == "Divination"));
}

#[test]
fn casting_in_response_retriggers_arjun_and_mindmoil() {
    cr!("603.3", "405.5", "601.2i");
    ruling!(
        "Arjun, the Shifting Flame",
        "After Arjun's ability triggers, players may cast instants and activate activated abilities."
    );
    ruling!(
        "Arjun, the Shifting Flame",
        "Arjun's triggered ability will resolve before the spell that caused it to trigger."
    );
    ruling!(
        "Mindmoil",
        "After Mindmoil’s ability resolves, but before the spell that triggered it does, you can cast instants from your new hand of cards. Each time you do, Mindmoil’s ability triggers again."
    );
    ruling!(
        "Mindmoil",
        "After Mindmoil’s ability triggers, players may cast instants and activate activated abilities. Each time you cast an instant spell “in response,” Mindmoil’s ability triggers again."
    );
    supported("Arjun, the Shifting Flame");
    supported("Mindmoil");
    for engine in ["Arjun, the Shifting Flame", "Mindmoil"] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, engine);
        t.lands(P0, "Island", 3);
        let opt = t.hand(P0, "Opt");
        let bolt = t.hand(P0, "Lightning Bolt");
        t.hand(P0, "Grizzly Bears");
        stack_library(&mut t, P0, &["Opt", "Opt", "Island", "Island"]);
        let spell = t.cast(P0, opt).go();
        t.settle();
        assert_eq!(t.stack_len(), 2, "{engine}");
        // Before the trigger resolves: cast an instant in response; it triggers again.
        t.lands(P0, "Mountain", 1);
        t.cast(P0, bolt).target(Entity::Player(P1)).go();
        t.settle();
        assert_eq!(t.stack_len(), 4, "{engine}");
        // Resolve the latest trigger: the hand (one Bears) is replaced by one card (the
        // top Opt); Bolt and the first trigger and Opt still wait.
        t.resolve();
        assert_eq!(hand_names(&t, P0), vec!["Opt"], "{engine}");
        assert!(t.g.stack.contains(&spell));
        assert_eq!(t.stack_len(), 3, "{engine}");
        // Cast the new Opt from the new hand: triggers again.
        let new_opt = t.g.player(P0).hand[0];
        t.cast(P0, new_opt).go();
        t.settle();
        assert_eq!(t.stack_len(), 5, "{engine}");
        t.resolve_all();
        assert_eq!(t.life(P1), 17);
    }
}

#[test]
fn narset_rebound_on_a_self_exiling_spell_does_nothing() {
    cr!("702.88a", "608.2n");
    ruling!(
        "Narset Transcendent",
        "If a spell that exiles itself as part of its own resolution, such as Volcanic Vision, gains rebound, the ability that lets you cast the spell again won’t be created. The spell will simply be exiled."
    );
    supported("Narset Transcendent");
    supported("Volcanic Vision");
    let mut t = TestGame::new(2);
    let n = t.battlefield(P0, "Narset Transcendent");
    t.activate(P0, n, 1, &[]).unwrap();
    t.resolve_all();
    let opt = t.graveyard(P0, "Opt");
    cast_new(&mut t, P0, "Volcanic Vision", &[obj(opt)]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Opt"));
    assert!(t.in_exile("Volcanic Vision"));
    t.advance_to(P1, Step::End);
    into_upkeep(&mut t, P0);
    assert_eq!(stacked_triggers(&t), 0);
    // Positive control: Opt gains rebound and is offered again next upkeep.
    let mut t = TestGame::new(2);
    let n = t.battlefield(P0, "Narset Transcendent");
    t.activate(P0, n, 1, &[]).unwrap();
    t.resolve_all();
    cast_new(&mut t, P0, "Opt", &[]);
    t.resolve_all();
    assert!(t.in_exile("Opt"));
    t.advance_to(P1, Step::End);
    into_upkeep(&mut t, P0);
    assert_eq!(stacked_triggers(&t), 1);
}

#[test]
fn tale_of_tamiyo_copies_choose_x() {
    cr!("707.12", "107.3a", "714.2b");
    ruling!(
        "The Tale of Tamiyo",
        "Because you're paying the spells' costs, if one of the copied spells has {X} in its mana cost, you may choose its value as normal."
    );
    supported("The Tale of Tamiyo");
    let mut t = TestGame::new(2);
    let tale = t.battlefield(P0, "The Tale of Tamiyo");
    put_counters(&mut t, tale, "lore", 3);
    t.resolve_all();
    let blaze = t.graveyard(P0, "Blaze");
    t.lands(P0, "Mountain", 4);
    t.answer_targets(P0, &[obj(blaze)]);
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    put_counters(&mut t, tale, "lore", 1);
    t.answer_choose(P0, &[]);
    t.resolve_all();
    let asked = t.asked();
    let x_asked = asked
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, decision::Decision::ChooseX { .. }));
    assert!(x_asked);
    assert_eq!(t.life(P1), 17);
    assert!(t.in_exile("Blaze"));
}

#[test]
fn possibility_storm_shares_a_card_type_x_is_zero_and_works_without_exiling() {
    cr!("205.2a", "107.3b", "608.2c");
    ruling!(
        "Possibility Storm",
        "Two cards share a card type if they have at least one card type in common."
    );
    ruling!(
        "Possibility Storm",
        "If the spell you cast from exile has X in its mana cost, you must choose 0 as its value."
    );
    ruling!(
        "Possibility Storm",
        "If Possibility Storm's ability doesn't exile the original spell (perhaps because another Possibility Storm already exiled it), you'll still exile cards from the top of your library until you exile a card that shares a card type with it and have the opportunity to cast that spell."
    );
    supported("Possibility Storm");
    // Ornithopter (artifact creature) finds Bonesplitter (artifact), skipping Opt.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Possibility Storm");
    stack_library(&mut t, P0, &["Opt", "Bonesplitter"]);
    yes(&mut t, P0);
    let o = t.hand(P0, "Ornithopter");
    t.cast(P0, o).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Bonesplitter").len(), 1);
    assert!(t.named_on_battlefield("Ornithopter").is_empty());
    // Endless One (X) is cast with X = 0: it dies as a 0/0.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Possibility Storm");
    stack_library(&mut t, P0, &["Endless One"]);
    yes(&mut t, P0);
    t.answer(P0, DecisionKind::X, Answer::Number(5));
    let o = t.hand(P0, "Ornithopter");
    t.cast(P0, o).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Endless One"));
    // Two Possibility Storms: the second can't exile the spell but still finds a card.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Possibility Storm");
    t.battlefield(P0, "Possibility Storm");
    stack_library(&mut t, P0, &["Bonesplitter", "Mind Stone"]);
    yes(&mut t, P0);
    yes(&mut t, P0);
    let o = t.hand(P0, "Ornithopter");
    t.cast(P0, o).go();
    t.settle();
    assert_eq!(stacked_triggers(&t), 2);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Bonesplitter").len(), 1);
    assert_eq!(t.named_on_battlefield("Mind Stone").len(), 1);
}

#[test]
fn coram_power_only_on_the_battlefield_and_priority_order() {
    cr!("611.3a", "117.3b", "604.1");
    ruling!(
        "Coram, the Undertaker",
        "Coram's first ability applies only while it is on the battlefield. In all other zones, its power is 0."
    );
    ruling!(
        "Coram, the Undertaker",
        "whoever gets priority sooner will have the first chance to play that card. In most cases, this will be you"
    );
    supported("Coram, the Undertaker");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Coram, the Undertaker");
    t.graveyard(P1, "Hill Giant");
    assert_eq!(t.pt(c), (3, 5));
    let g = t.graveyard(P0, "Coram, the Undertaker");
    assert_eq!(t.pt(g).0, 0);
    // P1's Think Twice is milled this turn: P0, active, may cast it first.
    t.library_top(P1, "Think Twice");
    mill(&mut t, P1, 1);
    let tt = t.g.player(P1).graveyard.last().copied().unwrap();
    t.lands(P0, "Island", 2);
    t.lands(P1, "Island", 3);
    assert_eq!(t.g.turn.priority, Some(P0));
    assert!(can_cast(&mut t, P0, tt, CastMethod::Normal));
    t.cast(P0, tt).go();
    t.resolve_all();
    // It went to its owner's graveyard; P1 may still flash it back (a new object).
    assert!(t.in_graveyard(P1, "Think Twice"));
}

#[test]
fn grolnok_plays_any_croaked_card() {
    cr!("400.7", "122.1");
    ruling!(
        "Grolnok, the Omnivore",
        "regardless of whether they were put there by the Grolnok that's currently on the battlefield or by an earlier incarnation."
    );
    supported("Grolnok, the Omnivore");
    let mut t = TestGame::new(2);
    let g1 = t.battlefield(P0, "Grolnok, the Omnivore");
    t.library_top(P0, "Grizzly Bears");
    mill(&mut t, P0, 1);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    destroy(&mut t, g1);
    t.battlefield(P0, "Grolnok, the Omnivore");
    let bears = t.g.exile.iter().copied().find(|id| t.g.obj(*id).chars.name == "Grizzly Bears").unwrap();
    assert_eq!(t.counters(bears, "croak"), 1);
    t.lands(P0, "Forest", 2);
    t.cast(P0, bears).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn intet_card_stays_visible_but_unplayable_after_intet_leaves() {
    cr!("406.3", "611.2b");
    ruling!(
        "Intet, the Dreamer",
        "After Intet leaves the battlefield, if a card exiled this way is still exiled, you may still look at it. You won't be able to play it, though."
    );
    supported("Intet, the Dreamer");
    let mut t = TestGame::new(2);
    let i = t.battlefield(P0, "Intet, the Dreamer");
    t.lands(P0, "Island", 3);
    t.library_top(P0, "Grizzly Bears");
    yes(&mut t, P0);
    t.attack(&[(i, Entity::Player(P1))], &[]);
    t.resolve_all();
    let card = t.g.exile.iter().copied().find(|id| t.g.obj(*id).face_down).expect("exiled");
    assert!(mtg_engine::player_control::can_see(&t.g, P0, card));
    assert!(!mtg_engine::player_control::can_see(&t.g, P1, card));
    t.advance_to(P0, Step::PostcombatMain);
    assert!(can_cast(&mut t, P0, card, CastMethod::Free));
    destroy(&mut t, i);
    assert!(mtg_engine::player_control::can_see(&t.g, P0, card));
    assert!(!can_cast(&mut t, P0, card, CastMethod::Free));
    assert!(!can_cast(&mut t, P0, card, CastMethod::Normal));
}

#[test]
fn kellan_is_a_creature_card_outside_the_stack_and_johann_casts_only_the_adventure() {
    cr!("715.4", "715.3a");
    ruling!(
        "Kellan, Daring Traveler // Journey On",
        "An adventurer card is a permanent card in every zone except the stack, as well as while on the stack if not cast as an Adventure."
    );
    ruling!(
        "Kellan, Daring Traveler // Journey On",
        "If you cast an adventurer card as an Adventure, use only its alternative characteristics to determine whether it's legal to cast that spell."
    );
    supported("Kellan, Daring Traveler // Journey On");
    supported("Johann, Apprentice Sorcerer");
    let mut t = TestGame::new(2);
    let k = t.graveyard(P0, "Kellan, Daring Traveler // Journey On");
    let o = t.obj_now(k);
    assert!(o.is(types::CardType::Creature));
    assert_eq!(t.g.mana_value_of(k), 2);
    assert!(o.chars.colors.contains(types::Color::White));
    let opt = t.graveyard(P0, "Opt");
    let from = n_asked(&t);
    enter(&mut t, P0, "Archaeomancer");
    assert!(offered_target(&t, P0, from, opt));
    assert!(!offered_target(&t, P0, from, k));
    // Johann: from the top of the library, only Journey On can be cast.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Johann, Apprentice Sorcerer");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Plains", 2);
    let k = t.library_top(P0, "Kellan, Daring Traveler // Journey On");
    assert!(t.cast(P0, k).try_go().is_err());
    assert!(t.cast(P0, k).method(CastMethod::Half(1)).try_go().is_ok());
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Map").len(), 1);
}

#[test]
fn gorex_cost_stops_at_bb() {
    cr!("601.2f", "118.7");
    ruling!(
        "Gorex, the Tombshell",
        "Gorex's first ability can't reduce its cost below {B}{B}. You can exile any number of creature cards from your graveyard, even if they won't reduce Gorex's cost any further."
    );
    supported("Gorex, the Tombshell");
    let mut t = TestGame::new(2);
    let cards: Vec<ObjectId> = (0..4).map(|_| t.graveyard(P0, "Grizzly Bears")).collect();
    let swamps = t.lands(P0, "Swamp", 2);
    let g = t.hand(P0, "Gorex, the Tombshell");
    // Exile all four (the fourth reduces nothing).
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(4));
    let es: Vec<Entity> = cards.iter().map(|c| obj(*c)).collect();
    t.answer_choose(P0, &es);
    t.cast(P0, g).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Gorex, the Tombshell").len(), 1);
    assert_eq!(t.g.player(P0).graveyard.len(), 0);
    assert!(cards.iter().all(|c| t.zone(*c) == Zone::Exile));
    // {B}{B} was still paid.
    assert!(swamps.iter().all(|l| t.obj_now(*l).tapped));
}

#[test]
fn ajani_mana_value_is_four_however_paid() {
    cr!("202.3", "107.4f");
    ruling!(
        "Ajani, Sleeper Agent",
        "Specifically, Ajani’s mana value is always 4, even if you pay {1}{G}{W} and 2 life to cast him."
    );
    supported("Ajani, Sleeper Agent");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Wastes", 1);
    let a = t.hand(P0, "Ajani, Sleeper Agent");
    assert_eq!(t.g.mana_value_of(a), 4);
    let spell = t.cast(P0, a).go();
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.g.mana_value_of(spell), 4);
    t.resolve_all();
    let p = t.named_on_battlefield("Ajani, Sleeper Agent")[0];
    assert_eq!(t.g.mana_value_of(p), 4);
    assert_eq!(t.counters(p, "loyalty"), 2);
}
