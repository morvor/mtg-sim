//! Rulings batch P200 — legendary-ish lands: City of Brass's tap trigger (CR 603.3,
//! 605.3), Command Tower (903.4, 106.5), Reflecting Pool (106.7), Cabal Coffers, Homeward Path's
//! owners (108.3, 111.2), Valakut's intervening "if" (603.4), Vesuva, The Mycosynth
//! Gardens and Adagia's copies (707.2), the enters-tapped-unless villages (614.12),
//! Magosi's extra and skipped turns (500.7, 614.10, 805.8), Sunhome's double strike
//! (702.4c–d), Phyrexia's Core's sacrifice cost (601.2h), Nivix's exiled card,
//! Island of Wak-Wak's base power (613.4b), Rix Maadi's simultaneous discards (101.4),
//! Svogthos's characteristic-defining power, and Diamond City's shield counters (122.1c).

use crate::r_p194_common::mana;
use crate::r_s01_common::*;
use crate::r_s02_common::create_token;
use crate::r_s04_common::*;
use crate::r_s05_common::move_to;
use crate::r_s06_common::{activate_containing, attached_to, give_control};
use crate::r_s20_common::tap_for_mana;
use crate::r_s31_common::wave;
use crate::r_s33_common::two_headed_giant;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// The mana type options offered when `p` taps `land` for mana with its ability whose
/// text contains "Add" (answering the first option), and whether it was activated.
fn mana_options(t: &mut TestGame, p: PlayerId, land: ObjectId) -> (Vec<String>, bool) {
    let from = t.asked().len();
    t.answer(p, DecisionKind::Option, Answer::Index(0));
    let ok = activate_containing(t, p, land, "Add").is_ok();
    let opts = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseOption { options, .. } => Some(options.clone()),
            _ => None,
        })
        .flatten()
        .collect();
    (opts, ok)
}

fn pool_total(t: &TestGame, p: PlayerId) -> usize {
    t.g.player(p).mana_pool.total() as usize
}

// ---------------------------------------------------------------------------------------
// City of Brass
// ---------------------------------------------------------------------------------------

#[test]
fn city_of_brass_tapped_while_casting_triggers_above_the_spell() {
    cr!("601.2g", "605.3a", "603.3", "601.2i");
    ruling!(
        "City of Brass",
        "If you tap City of Brass while you are casting a spell or activating an ability, its ability will trigger and wait. When you finish casting that spell or activating that ability, City of Brass's triggered ability is put on the stack on top of it. City of Brass's ability will resolve first."
    );
    let mut t = TestGame::new(2);
    let city = t.battlefield(P0, "City of Brass");
    let shock = t.hand(P0, "Shock");
    let spell = t.cast(P0, shock).target(Entity::Player(P1)).go();
    assert!(t.obj_now(city).tapped);
    // Paid for: the trigger waits until the spell is cast.
    assert_eq!(t.g.stack, vec![spell]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    assert_ne!(top_of_stack(&t), spell);
    t.resolve();
    assert_eq!((t.life(P0), t.life(P1)), (19, 20));
    assert_eq!(t.g.stack, vec![spell]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn city_of_brass_mana_can_be_spent_in_response_to_its_trigger() {
    cr!("605.3a", "603.3", "106.4");
    ruling!(
        "City of Brass",
        "On the other hand, you can tap City of Brass, put its triggered ability on the stack, and then respond to that ability by casting an instant or activating an ability using that mana. In that case, the instant spell or activated ability will resolve first."
    );
    let mut t = TestGame::new(2);
    let city = t.battlefield(P0, "City of Brass");
    // The first color offered is white; Healing Salve ("Choose one — Target player gains 3
    // life; or prevent the next 3 damage...") costs {W}.
    supported("Healing Salve");
    assert!(tap_for_mana(&mut t, P0, city, "Add"));
    t.settle();
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.life(P0), 20);
    let salve = t.hand(P0, "Healing Salve");
    t.cast(P0, salve)
        .modes(&[0])
        .target(Entity::Player(P0))
        .go();
    assert_eq!(pool_total(&t, P0), 0);
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    // The instant resolved first: 23, then the trigger's 1 damage.
    assert_eq!(t.life(P0), 23);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
}

#[test]
fn city_of_brass_triggers_however_it_becomes_tapped() {
    cr!("603.2", "701.26a");
    ruling!(
        "City of Brass",
        "The first ability triggers no matter how the land becomes tapped."
    );
    supported("Icy Manipulator");
    let mut t = TestGame::new(2);
    let city = t.battlefield(P0, "City of Brass");
    t.set_step(P1, Step::PrecombatMain);
    let icy = t.battlefield(P1, "Icy Manipulator");
    mana(&mut t, P1, ManaType::C, 1);
    t.activate(P1, icy, 0, &[obj(city)]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(city).tapped);
    assert_eq!(t.life(P0), 19);
    assert!(t.g.player(P0).mana_pool.total() == 0);
}

// ---------------------------------------------------------------------------------------
// Command Tower, Reflecting Pool, Cabal Coffers
// ---------------------------------------------------------------------------------------

#[test]
fn command_tower_with_a_colorless_commander_makes_no_mana() {
    cr!("903.4", "106.5");
    ruling!(
        "Command Tower",
        "If your commander is a card that has no colors in its color identity, Command Tower's ability produces no mana. It doesn't produce {C}."
    );
    // P0's commander is colorless (Karn), P1's is green (Llanowar Elves): the same Tower
    // makes {G} for P1 (so the commander is found) and nothing at all for P0.
    let mut t = TestGame::new(2);
    let karn = t.command(P0, "Karn, Legacy Reforged");
    t.g.objects[karn.0 as usize].is_commander = true;
    let elves = t.command(P1, "Llanowar Elves");
    t.g.objects[elves.0 as usize].is_commander = true;
    let theirs = t.battlefield(P1, "Command Tower");
    assert!(tap_for_mana(&mut t, P1, theirs, "Add"));
    assert_eq!(t.g.player(P1).mana_pool.count(ManaType::G), 1);
    let tower = t.battlefield(P0, "Command Tower");
    let (opts, ok) = mana_options(&mut t, P0, tower);
    assert!(ok, "the ability can still be activated");
    assert!(t.obj_now(tower).tapped);
    assert!(opts.is_empty(), "{opts:?}");
    assert_eq!(pool_total(&t, P0), 0);
}

#[test]
fn reflecting_pools_dont_help_each_other() {
    cr!("106.7");
    ruling!(
        "Reflecting Pool",
        "Multiple Reflecting Pools won't help each other produce mana. If you control a Reflecting Pool, and all other lands you control either lack mana abilities or are other Reflecting Pools, you may still activate Reflecting Pool's ability — it just won't produce any mana."
    );
    supported("Maze of Ith");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Reflecting Pool");
    t.battlefield(P0, "Reflecting Pool");
    t.battlefield(P0, "Maze of Ith");
    let (opts, ok) = mana_options(&mut t, P0, a);
    assert!(ok, "the ability can still be activated");
    assert!(opts.is_empty(), "{opts:?}");
    assert_eq!(pool_total(&t, P0), 0);
    // With a Forest too, each Pool can make {G}, and only {G}.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Reflecting Pool");
    let b = t.battlefield(P0, "Reflecting Pool");
    t.battlefield(P0, "Forest");
    for p in [a, b] {
        assert!(tap_for_mana(&mut t, P0, p, "Add"));
    }
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::G), 2);
}

#[test]
fn reflecting_pool_ignores_the_costs_of_other_lands_mana_abilities() {
    cr!("106.7");
    ruling!(
        "Reflecting Pool",
        "Reflecting Pool checks the effects of all mana-producing abilities of lands you control, but it doesn't check their costs."
    );
    supported("Vivid Crag");
    // Vivid Crag with no charge counter, and tapped: the Pool still makes any color.
    let mut t = TestGame::new(2);
    let pool = t.battlefield(P0, "Reflecting Pool");
    let crag = t.battlefield(P0, "Vivid Crag");
    assert_eq!(t.counters(crag, "charge"), 0);
    t.g.objects[crag.0 as usize].tapped = true;
    let (opts, ok) = mana_options(&mut t, P0, pool);
    assert!(ok);
    for c in ["W", "U", "B", "R", "G"] {
        assert!(opts.iter().any(|o| o == c), "{c}: {opts:?}");
    }
    assert_eq!(pool_total(&t, P0), 1);
}

#[test]
fn reflecting_pool_ignores_restrictions_on_other_lands_mana() {
    cr!("106.7", "106.6");
    ruling!(
        "Reflecting Pool",
        "Reflecting Pool doesn't care about any restrictions or riders your lands put on the mana they produce, such as Pillar of the Paruns and Hall of the Bandit Lord do. It just cares about types of mana."
    );
    supported("Pillar of the Paruns");
    let mut t = TestGame::new(2);
    let pool = t.battlefield(P0, "Reflecting Pool");
    let pillar = t.battlefield(P0, "Pillar of the Paruns");
    t.g.objects[pillar.0 as usize].tapped = true;
    // Red mana from the Pool pays for a monocolored Lightning Bolt.
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Option, Answer::Index(3));
    assert!(tap_for_mana(&mut t, P0, pool, "Add"));
    assert!(t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseOption { options, .. } if options.len() == 5)));
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::R), 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // Hall of the Bandit Lord: {C}, without its life payment.
    let mut t = TestGame::new(2);
    let pool = t.battlefield(P0, "Reflecting Pool");
    t.battlefield(P0, "Hall of the Bandit Lord");
    assert!(tap_for_mana(&mut t, P0, pool, "Add"));
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::C), 1);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn cabal_coffers_counts_lands_with_the_swamp_subtype() {
    cr!("305.6", "205.3i");
    ruling!(
        "Cabal Coffers",
        "The mana ability of Cabal Coffers counts each land you control with the subtype Swamp, not just ones named Swamp."
    );
    let mut t = TestGame::new(2);
    let coffers = t.battlefield(P0, "Cabal Coffers");
    t.battlefield(P0, "Swamp");
    t.battlefield(P0, "Underground Sea");
    t.battlefield(P1, "Swamp");
    mana(&mut t, P0, ManaType::C, 2);
    activate_containing(&mut t, P0, coffers, "Add").unwrap();
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::B), 2);
}

// ---------------------------------------------------------------------------------------
// Homeward Path
// ---------------------------------------------------------------------------------------

#[test]
fn homeward_path_returns_cards_and_tokens_to_their_owners() {
    cr!("108.3", "111.2");
    ruling!(
        "Homeward Path",
        "The owner of a card is the player who started the game with that card in their deck."
    );
    ruling!(
        "Homeward Path",
        "The owner of a token is the player under whose control the token was put onto the battlefield."
    );
    supported("Beast Within");
    let mut t = TestGame::new(2);
    let path = t.battlefield(P0, "Homeward Path");
    // P0's card stolen by P1, P1's card stolen by P0.
    let mine = t.battlefield(P0, "Grizzly Bears");
    give_control(&mut t, mine, P1);
    let theirs = t.battlefield(P1, "Hill Giant");
    give_control(&mut t, theirs, P0);
    // P0's Beast Within on P1's land: the Beast token is created under P1's control, so P1
    // owns it, though P0's spell made it. P0 then steals it.
    let land = t.battlefield(P1, "Forest");
    let bw = t.hand(P0, "Beast Within");
    mana(&mut t, P0, ManaType::G, 1);
    mana(&mut t, P0, ManaType::C, 2);
    t.cast(P0, bw).target(land).go();
    t.resolve_all();
    let beast = tokens(&t, P1)[0];
    assert_eq!(t.obj_now(beast).owner, P1);
    give_control(&mut t, beast, P0);
    // And a token P0 made, now controlled by P1.
    let soldier = create_token(&mut t, P0, "Soldier");
    give_control(&mut t, soldier, P1);
    activate_containing(&mut t, P0, path, "gains control").unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(mine).controller, P0);
    assert_eq!(t.obj_now(theirs).controller, P1);
    assert_eq!(t.obj_now(beast).controller, P1);
    assert_eq!(t.obj_now(soldier).controller, P0);
}

// ---------------------------------------------------------------------------------------
// Valakut
// ---------------------------------------------------------------------------------------

#[test]
fn valakut_intervening_if_checked_on_trigger_and_resolution() {
    cr!("603.4");
    ruling!(
        "Valakut, the Molten Pinnacle",
        "Valakut, the Molten Pinnacle's triggered ability has an \"intervening 'if' clause.\" That means (1) the ability won't trigger at all unless"
    );
    // (1) Four other Mountains: no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Valakut, the Molten Pinnacle");
    t.lands(P0, "Mountain", 4);
    t.enter(P0, "Mountain");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // (2) Five others, one of them gone by resolution (four left): nothing, though P0
    // would say yes.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Valakut, the Molten Pinnacle");
    let old = t.lands(P0, "Mountain", 5);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Mountain");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    move_to(&mut t, old[0], Zone::Hand(P0));
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
}

#[test]
fn valakut_targets_on_trigger_and_decides_on_resolution() {
    cr!("603.3d", "608.2d");
    ruling!(
        "Valakut, the Molten Pinnacle",
        "You target a permanent or player when the ability triggers. You decide whether to have Valakut deal damage to that permanent or player as the ability resolves."
    );
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Valakut, the Molten Pinnacle");
        t.lands(P0, "Mountain", 5);
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.answer_targets(P0, &[obj(bears)]);
        t.enter(P0, "Mountain");
        t.settle();
        // The target is chosen as it's put on the stack, before any yes/no is asked.
        let si = t.g.obj(top_of_stack(&t)).stack.clone().unwrap();
        assert!(si
            .chosen
            .iter()
            .flat_map(|c| c.targets.iter().flatten())
            .any(|e| *e == obj(bears)));
        assert!(!t
            .asked()
            .iter()
            .any(|(_, d)| matches!(d, Decision::YesNo { .. })));
        t.answer_yes(P0, yes);
        t.resolve_all();
        assert_eq!(t.on_battlefield(bears), !yes);
    }
}

// ---------------------------------------------------------------------------------------
// Copies: Vesuva, The Mycosynth Gardens, Adagia
// ---------------------------------------------------------------------------------------

#[test]
fn vesuva_copies_only_the_printed_land() {
    cr!("707.2", "613.2a");
    ruling!(
        "Vesuva",
        "Vesuva copies exactly what is printed on the land it's copying and nothing else (unless it's copying a land that's copying something else; see below). It doesn't copy whether a land is tapped or untapped, whether it has any counters on it or Auras attached to it, or any non-copy effects that changed its types, and so on."
    );
    supported("Spreading Seas");
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P1, "Forest");
    t.g.add_counters(obj(forest), "charge", 2, None);
    let seas = t.battlefield(P1, "Spreading Seas");
    assert!(t.g.attach(seas, obj(forest)));
    t.g.recompute();
    assert!(t.obj_now(forest).chars.has_subtype("Island"));
    t.answer_choose(P0, &[obj(forest)]);
    let v = t.enter(P0, "Vesuva");
    t.settle();
    let o = t.obj_now(v);
    assert_eq!(o.chars.name, "Forest");
    assert!(o.chars.has_subtype("Forest") && !o.chars.has_subtype("Island"));
    assert_eq!(t.counters(v, "charge"), 0);
    assert!(t.g.permanents().all(|p| p.attached_to != Some(obj(v))));
    // Untapped original, tapped Vesuva (its own "enter tapped").
    assert!(!t.obj_now(forest).tapped);
    assert!(t.obj_now(v).tapped);
}

/// P0's The Mycosynth Gardens becoming a copy of `target` (an artifact with mana value
/// `x`).
fn gardens_copy(t: &mut TestGame, target: ObjectId, x: i64) -> ObjectId {
    let gardens = t.battlefield(P0, "The Mycosynth Gardens");
    mana(t, P0, ManaType::C, x as u32);
    t.answer(P0, DecisionKind::X, Answer::Number(x));
    t.answer_targets(P0, &[obj(target)]);
    activate_containing(t, P0, gardens, "becomes a copy").unwrap();
    t.resolve_all();
    gardens
}

#[test]
fn mycosynth_gardens_copies_only_the_printed_artifact() {
    cr!("707.2");
    ruling!(
        "The Mycosynth Gardens",
        "The Mycosynth Gardens copies exactly what was printed on the original artifact and nothing else"
    );
    supported("Bonesplitter");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let split = t.battlefield(P0, "Bonesplitter");
    assert!(t.g.attach(split, obj(bears)));
    t.g.add_counters(obj(split), "charge", 1, None);
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 2));
    let gardens = gardens_copy(&mut t, split, 1);
    assert_eq!(t.obj_now(gardens).chars.name, "Bonesplitter");
    assert_eq!(t.counters(gardens, "charge"), 0);
    assert_eq!(attached_to(&t, gardens), None);
    assert_eq!(t.pt(bears), (4, 2));
}

#[test]
fn adagia_token_copies_only_the_printed_permanent() {
    cr!("707.2", "707.9b", "702.122c");
    ruling!(
        "Adagia, Windswept Bastion",
        "The token created by Adagia’s last ability copies exactly what was printed on the original permanent and nothing else"
    );
    supported("Smuggler's Copter");
    supported("Bonesplitter");
    // A crewed, tapped Copter with a counter: the token is an untapped, uncrewed,
    // counterless (legendary) Vehicle.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Adagia, Windswept Bastion");
    t.g.add_counters(obj(a), "charge", 12, None);
    let copter = t.battlefield(P0, "Smuggler's Copter");
    let crewman = t.battlefield(P0, "Grizzly Bears");
    assert!(crew(&mut t, P0, copter, &[crewman]));
    t.resolve_all();
    assert!(is_creature(&t, copter));
    t.g.objects[copter.0 as usize].tapped = true;
    t.g.add_counters(obj(copter), counters::PLUS1, 1, None);
    mana(&mut t, P0, ManaType::W, 1);
    mana(&mut t, P0, ManaType::C, 3);
    t.answer_targets(P0, &[obj(copter)]);
    activate_containing(&mut t, P0, a, "copy of target").unwrap();
    t.resolve_all();
    let tok = tokens(&t, P0)[0];
    let o = t.obj_now(tok);
    assert_eq!(o.chars.name, "Smuggler's Copter");
    assert!(o.chars.supertypes.contains(Supertype::Legendary));
    assert!(!o.is(CardType::Creature), "not crewed");
    assert!(!o.tapped);
    assert_eq!(t.counters(tok, counters::PLUS1), 0);
    // An attached Equipment: the copy isn't attached.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Adagia, Windswept Bastion");
    t.g.add_counters(obj(a), "charge", 12, None);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let split = t.battlefield(P0, "Bonesplitter");
    assert!(t.g.attach(split, obj(bears)));
    mana(&mut t, P0, ManaType::W, 1);
    mana(&mut t, P0, ManaType::C, 3);
    t.answer_targets(P0, &[obj(split)]);
    activate_containing(&mut t, P0, a, "copy of target").unwrap();
    t.resolve_all();
    let tok = tokens(&t, P0)[0];
    assert_eq!(attached_to(&t, tok), None);
    assert_eq!(t.pt(bears), (4, 2));
}

#[test]
fn planet_is_a_land_type_with_no_intrinsic_ability() {
    cr!("205.3i", "305.6");
    ruling!(
        "Adagia, Windswept Bastion",
        "Planet is a land subtype with no special meaning. It doesn’t grant the land any intrinsic abilities."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Adagia, Windswept Bastion");
    assert!(t.obj_now(a).chars.has_subtype("Planet"));
    // Its only mana ability is its own "{T}: Add {W}".
    let (opts, ok) = mana_options(&mut t, P0, a);
    assert!(ok && opts.is_empty());
    let p = &t.g.player(P0).mana_pool;
    assert_eq!((p.total(), p.count(ManaType::W)), (1, 1));
    let manas = t
        .obj_now(a)
        .chars
        .abilities
        .iter()
        .filter(|ab| ab.text.contains("Add"))
        .count();
    assert_eq!(manas, 1);
}

// ---------------------------------------------------------------------------------------
// Enters tapped unless you control ...
// ---------------------------------------------------------------------------------------

#[test]
fn villages_check_lands_already_on_the_battlefield() {
    cr!("614.12", "614.1d");
    ruling!(
        "Great Arashin City",
        "You must already control a Forest or Plains as Great Arashin City enters for it to enter untapped. If it enters at the same time as a Forest or Plains when you control no other Forests or Plains, it will enter tapped."
    );
    ruling!(
        "Mistrise Village",
        "You must already control a Mountain or Forest as Mistrise Village enters for it to enter untapped. If it enters at the same time as a Mountain or Forest when you control no other Mountains or Forests, it will enter tapped."
    );
    ruling!(
        "Cori Mountain Monastery",
        "You must already control a Plains or Island as Cori Mountain Monastery enters for it to enter untapped. If it enters at the same time as a Plains or Island when you control no other Plains or Islands, it will enter tapped."
    );
    ruling!(
        "Kishla Village",
        "You must already control an Island or a Swamp as Kishla Village enters for it to enter untapped. If it enters at the same time as an Island or Swamp when you control no other Islands or Swamps, it will enter tapped."
    );
    supported("Genesis Wave");
    let cases = [
        ("Great Arashin City", ["Forest", "Plains"]),
        ("Mistrise Village", ["Mountain", "Forest"]),
        ("Cori Mountain Monastery", ["Plains", "Island"]),
        ("Kishla Village", ["Island", "Swamp"]),
    ];
    for (village, basics) in cases {
        supported(village);
        for basic in basics {
            // Entering together with the basic: tapped.
            let mut t = TestGame::new(2);
            let cards = wave(&mut t, &[village, basic]);
            assert!(t.obj_now(cards[0]).tapped, "{village} with {basic}");
            assert!(!t.obj_now(cards[1]).tapped);
            // The basic already there: untapped.
            let mut t = TestGame::new(2);
            t.battlefield(P0, basic);
            let cards = wave(&mut t, &[village]);
            assert!(!t.obj_now(cards[0]).tapped, "{village} after {basic}");
        }
        // Neither: tapped.
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Wastes");
        let cards = wave(&mut t, &[village]);
        assert!(t.obj_now(cards[0]).tapped, "{village} alone");
    }
}

// ---------------------------------------------------------------------------------------
// Magosi, the Waterveil
// ---------------------------------------------------------------------------------------

/// Advances to the start of the next turn; returns its active player.
fn next_turn(t: &mut TestGame) -> PlayerId {
    let n = t.g.turn.number;
    assert!(t.g.run_until(10_000, |g| g.turn.number == n + 1));
    t.g.turn.active
}

#[test]
fn magosi_skip_and_extra_turn_cancel_out_in_either_order() {
    cr!("500.7", "614.10");
    ruling!(
        "Magosi, the Waterveil",
        "If you somehow activate Magosi’s third and fourth abilities during the same turn, the effects will cancel each other out. The turn you skip is the next one you would take, which is the turn created by Magosi’s fourth ability. It doesn’t matter in which order the abilities resolved."
    );
    for skip_first in [true, false] {
        let mut t = TestGame::new(2);
        let magosi = t.battlefield(P0, "Magosi, the Waterveil");
        if skip_first {
            mana(&mut t, P0, ManaType::U, 1);
            activate_containing(&mut t, P0, magosi, "Skip your next turn").unwrap();
            t.resolve_all();
            assert_eq!(t.counters(magosi, "eon"), 1);
            t.g.objects[magosi.0 as usize].tapped = false;
            activate_containing(&mut t, P0, magosi, "extra turn").unwrap();
            t.resolve_all();
        } else {
            // An eon counter from earlier: the skip goes on the stack first, then the
            // extra turn on top of it, which resolves first.
            t.g.add_counters(obj(magosi), "eon", 1, None);
            mana(&mut t, P0, ManaType::U, 1);
            activate_containing(&mut t, P0, magosi, "Skip your next turn").unwrap();
            t.g.objects[magosi.0 as usize].tapped = false;
            activate_containing(&mut t, P0, magosi, "extra turn").unwrap();
            assert_eq!(t.stack_len(), 2);
            t.resolve_all();
        }
        // P1's turn comes next (the extra turn was skipped), then P0's normal turn, then
        // P1's: nothing left over.
        assert_eq!(next_turn(&mut t), P1, "{skip_first}");
        assert_eq!(next_turn(&mut t), P0, "{skip_first}");
        assert_eq!(next_turn(&mut t), P1, "{skip_first}");
    }
}

#[test]
fn magosi_extra_turn_in_two_headed_giant_is_the_teams() {
    cr!("805.8", "500.7");
    ruling!(
        "Magosi, the Waterveil",
        "If you take an extra turn in a Two-Headed Giant game, you entire team takes the extra turn."
    );
    let mut t = two_headed_giant();
    let magosi = t.battlefield(P1, "Magosi, the Waterveil");
    t.g.add_counters(obj(magosi), "eon", 1, None);
    activate_containing(&mut t, P1, magosi, "extra turn").unwrap();
    t.resolve_all();
    let n = t.g.turn.number;
    next_turn(&mut t);
    assert_eq!(t.g.turn.number, n + 1);
    // The team of P0 and P1 takes the extra turn (and the other team the one after).
    let team = t.g.player(t.g.turn.active).team;
    assert_eq!(team, t.g.player(P1).team);
    assert_eq!(t.g.player(P0).team, team);
    let after = next_turn(&mut t);
    assert_ne!(t.g.player(after).team, team);
}

#[test]
fn magosi_extra_turn_needs_an_eon_counter_and_returns_it_as_a_cost() {
    cr!("602.2b", "118.3", "601.2h");
    ruling!(
        "Magosi, the Waterveil",
        "You can activate Magosi’s fourth ability only if it has an eon counter on it."
    );
    ruling!(
        "Magosi, the Waterveil",
        "You return Magosi to its owner’s hand as part of the cost to activate its fourth ability. If Magosi has more than one eon counter on it, you can’t activate the ability, untap it, then activate it again."
    );
    let mut t = TestGame::new(2);
    let magosi = t.battlefield(P0, "Magosi, the Waterveil");
    assert!(activate_containing(&mut t, P0, magosi, "extra turn").is_err());
    assert!(t.on_battlefield(magosi) && !t.obj_now(magosi).tapped);
    t.g.add_counters(obj(magosi), "eon", 2, None);
    activate_containing(&mut t, P0, magosi, "extra turn").unwrap();
    // Already in its owner's hand while the ability is on the stack.
    assert_eq!(t.zone(magosi), Zone::Hand(P0));
    assert_eq!(t.stack_len(), 1);
    assert!(!t.g.legal_actions(P0).iter().any(|a| matches!(
        a,
        mtg_engine::decision::Action::Activate { source, .. } if t.g.current(*source) == t.g.current(magosi)
    )));
    t.resolve_all();
    // One extra turn only.
    assert_eq!(next_turn(&mut t), P0);
    assert_eq!(next_turn(&mut t), P1);
}

// ---------------------------------------------------------------------------------------
// Sunhome
// ---------------------------------------------------------------------------------------

/// P0 attacks P1 with `name` alongside a Youthful Knight (first strike, so there is a
/// first-strike combat damage step), activates Sunhome targeting `name` in the step given,
/// and finishes combat. P1's life loss from `name`.
fn sunhome_damage(name: &str, step: Step) -> i32 {
    let mut t = TestGame::new(2);
    let sunhome = t.battlefield(P0, "Sunhome, Fortress of the Legion");
    let c = t.battlefield(P0, name);
    let knight = t.battlefield(P0, "Youthful Knight");
    attack_with(
        &mut t,
        &[(c, Entity::Player(P1)), (knight, Entity::Player(P1))],
    );
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P0, step);
    t.settle();
    mana(&mut t, P0, ManaType::R, 1);
    mana(&mut t, P0, ManaType::W, 1);
    mana(&mut t, P0, ManaType::C, 2);
    t.activate(P0, sunhome, 1, &[obj(c)]).unwrap();
    t.resolve_all();
    assert!(t
        .obj_now(c)
        .has_keyword(mtg_engine::keywords::KeywordKind::DoubleStrike));
    t.advance_to(P0, Step::PostcombatMain);
    // The Knight dealt 2 in the first-strike step.
    20 - 2 - t.life(P1)
}

#[test]
fn sunhome_double_strike_too_late_adds_nothing() {
    cr!("510.4", "702.4b");
    ruling!(
        "Sunhome, Fortress of the Legion",
        "If you give double strike to a creature after either first-strike or normal combat damage has been dealt, it won't help that creature deal any additional combat damage."
    );
    // After first-strike damage, to a creature without first strike: it deals its normal
    // damage only.
    assert_eq!(sunhome_damage("Grizzly Bears", Step::FirstStrikeDamage), 2);
    // After normal damage: nothing more.
    assert_eq!(sunhome_damage("Grizzly Bears", Step::CombatDamage), 2);
    assert_eq!(sunhome_damage("White Knight", Step::CombatDamage), 2);
}

#[test]
fn sunhome_double_strike_to_a_first_striker_after_first_strike_damage() {
    cr!("702.4d", "510.4");
    ruling!(
        "Sunhome, Fortress of the Legion",
        "If you give double strike to a creature with first strike after first-strike combat damage has been dealt, that creature will also deal damage during the normal combat damage step."
    );
    assert_eq!(sunhome_damage("White Knight", Step::FirstStrikeDamage), 4);
}

// ---------------------------------------------------------------------------------------
// Phyrexia's Core, Nivix, Island of Wak-Wak, Rix Maadi, Svogthos
// ---------------------------------------------------------------------------------------

#[test]
fn phyrexias_core_sacrifice_is_paid_before_anyone_can_respond() {
    cr!("601.2h", "602.2b");
    ruling!(
        "Phyrexia's Core",
        "Once you activate the ability and sacrifice an artifact, it’s too late for anyone to respond to try and prevent you from paying the cost."
    );
    let mut t = TestGame::new(2);
    let core = t.battlefield(P0, "Phyrexia's Core");
    let stone = t.battlefield(P0, "Mind Stone");
    mana(&mut t, P0, ManaType::C, 1);
    t.answer_choose(P0, &[obj(stone)]);
    activate_containing(&mut t, P0, core, "gain 1 life").unwrap();
    // On the stack, the artifact is already gone: nothing P1 does can save it.
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.zone(stone), Zone::Graveyard(P0));
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
}

/// P0 activates Nivix's second ability with the real card `top` on top of their library;
/// returns that card.
fn nivix_exile(t: &mut TestGame, top: &str) -> ObjectId {
    let nivix = t.battlefield(P0, "Nivix, Aerie of the Firemind");
    let c = t.library_top(P0, top);
    mana(t, P0, ManaType::U, 1);
    mana(t, P0, ManaType::R, 1);
    mana(t, P0, ManaType::C, 2);
    activate_containing(t, P0, nivix, "Exile the top card").unwrap();
    t.resolve_all();
    assert_eq!(t.zone(c), Zone::Exile);
    c
}

/// Whether `p` could cast the card (followed across zone changes) now.
fn castable(t: &mut TestGame, p: PlayerId, id: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.recompute();
    let c = t.g.current(id);
    t.g.legal_actions(p)
        .iter()
        .any(|a| matches!(a, mtg_engine::decision::Action::Cast { card, .. } if *card == c))
}

#[test]
fn nivix_card_not_cast_stays_exiled() {
    cr!("611.2a", "400.7");
    ruling!(
        "Nivix, Aerie of the Firemind",
        "If you don't cast the card, it stays exiled."
    );
    let mut t = TestGame::new(2);
    let bolt = nivix_exile(&mut t, "Lightning Bolt");
    t.lands(P0, "Mountain", 1);
    assert!(castable(&mut t, P0, bolt));
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    assert_eq!(t.zone(bolt), Zone::Exile);
    assert!(!castable(&mut t, P0, bolt));
    // A creature card exiled this way can't be cast, and stays exiled too.
    let mut t = TestGame::new(2);
    let bears = nivix_exile(&mut t, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    assert!(!castable(&mut t, P0, bears));
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.zone(bears), Zone::Exile);
}

#[test]
fn nivix_card_is_cast_normally() {
    cr!("601.2", "601.2f", "608.2n");
    ruling!(
        "Nivix, Aerie of the Firemind",
        "The card is cast using all normal rules and restrictions for casting spells. Casting it means you'll have to pay its mana cost, and it will go to the graveyard after it resolves or is countered. The only thing that's different is you're casting it from the Exile zone."
    );
    let mut t = TestGame::new(2);
    let bolt = nivix_exile(&mut t, "Lightning Bolt");
    // No mana: can't cast it.
    assert!(!castable(&mut t, P0, bolt));
    mana(&mut t, P0, ManaType::R, 1);
    assert!(castable(&mut t, P0, bolt));
    let c = t.g.current(bolt);
    t.cast(P0, c).target(Entity::Player(P1)).go();
    assert_eq!(pool_total(&t, P0), 0);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.zone(bolt), Zone::Graveyard(P0));
    // A sorcery exiled this way: only at sorcery speed (P1's turn: no).
    let mut t = TestGame::new(2);
    let rot = nivix_exile(&mut t, "Mind Rot");
    mana(&mut t, P0, ManaType::B, 1);
    mana(&mut t, P0, ManaType::C, 2);
    t.set_step(P1, Step::Upkeep);
    assert!(!castable(&mut t, P0, rot));
    // Countered: to the graveyard.
    let mut t = TestGame::new(2);
    let bolt = nivix_exile(&mut t, "Lightning Bolt");
    mana(&mut t, P0, ManaType::R, 1);
    let c = t.g.current(bolt);
    let spell = t.cast(P0, c).target(Entity::Player(P1)).go();
    let cancel = t.hand(P1, "Cancel");
    mana(&mut t, P1, ManaType::U, 2);
    mana(&mut t, P1, ManaType::C, 1);
    t.cast(P1, cancel).target(spell).go();
    t.resolve_all();
    assert_eq!(t.zone(bolt), Zone::Graveyard(P0));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn island_of_wak_wak_sets_base_power_and_isnt_an_island() {
    cr!("613.4b", "613.4c", "205.3i");
    ruling!(
        "Island of Wak-Wak",
        "Sets power to 0 instead of giving the creature -X/-0."
    );
    ruling!("Island of Wak-Wak", "This is not an Island.");
    let mut t = TestGame::new(2);
    let wak = t.battlefield(P0, "Island of Wak-Wak");
    assert!(!t.obj_now(wak).chars.has_subtype("Island"));
    let angel = t.battlefield(P1, "Serra Angel");
    t.g.add_counters(obj(angel), counters::PLUS1, 1, None);
    t.activate(P0, wak, 0, &[obj(angel)]).unwrap();
    t.resolve_all();
    // Base power 0, then the counter: 1/5 (not 0/5 or 5-4).
    assert_eq!(t.pt(angel), (1, 5));
    // A later pump still applies on top.
    let giant = t.hand(P1, "Giant Growth");
    mana(&mut t, P1, ManaType::G, 1);
    t.cast(P1, giant).target(angel).go();
    t.resolve_all();
    assert_eq!(t.pt(angel), (4, 8));
}

#[test]
fn rix_maadi_discards_are_chosen_in_apnap_order_then_happen_together() {
    cr!("101.4", "701.9a");
    ruling!(
        "Rix Maadi, Dungeon Palace",
        "When the second ability resolves, first the active player chooses a card to discard, then the nonactive player chooses, then all chosen cards are discarded at the same time. No one sees what the other players are discarding before deciding which cards to discard."
    );
    let mut t = TestGame::new(2);
    let rix = t.battlefield(P0, "Rix Maadi, Dungeon Palace");
    t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Hill Giant");
    t.hand(P1, "Shock");
    t.hand(P1, "Island");
    // When each player chooses: the number of cards in graveyards then.
    let seen0 = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseEntities { .. }),
        |g| g.players.iter().map(|p| p.graveyard.len()).sum::<usize>(),
    );
    let seen1 = watch(
        &mut t,
        P1,
        |d| matches!(d, Decision::ChooseEntities { .. }),
        |g| g.players.iter().map(|p| p.graveyard.len()).sum::<usize>(),
    );
    let from = t.asked().len();
    mana(&mut t, P0, ManaType::B, 1);
    mana(&mut t, P0, ManaType::R, 1);
    mana(&mut t, P0, ManaType::C, 1);
    activate_containing(&mut t, P0, rix, "discards").unwrap();
    t.resolve_all();
    let order: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(order, vec![P0, P1]);
    // Nothing was discarded before P1 chose.
    assert_eq!(*seen0.lock().unwrap(), vec![0]);
    assert_eq!(*seen1.lock().unwrap(), vec![0]);
    assert_eq!((t.hand_size(P0), t.hand_size(P1)), (1, 1));
    assert_eq!((t.graveyard_size(P0), t.graveyard_size(P1)), (1, 1));
}

#[test]
fn svogthos_power_follows_creature_cards_in_the_graveyard() {
    cr!("604.3", "613.4a");
    ruling!(
        "Svogthos, the Restless Tomb",
        "While animated, Svogthos’s power and toughness changes each time a creature card enters or leaves its controller’s graveyard."
    );
    let mut t = TestGame::new(2);
    let svog = t.battlefield(P0, "Svogthos, the Restless Tomb");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P1, "Hill Giant");
    t.graveyard(P0, "Forest");
    mana(&mut t, P0, ManaType::B, 1);
    mana(&mut t, P0, ManaType::G, 1);
    mana(&mut t, P0, ManaType::C, 3);
    activate_containing(&mut t, P0, svog, "becomes").unwrap();
    t.resolve_all();
    assert!(is_creature(&t, svog));
    assert_eq!(t.pt(svog), (1, 1));
    let giant = t.graveyard(P0, "Hill Giant");
    assert_eq!(t.pt(svog), (2, 2));
    move_to(&mut t, giant, Zone::Exile);
    assert_eq!(t.pt(svog), (1, 1));
}

// ---------------------------------------------------------------------------------------
// Diamond City
// ---------------------------------------------------------------------------------------

/// P0's Diamond City (entered with its shield counter) after two creatures entered under
/// P0's control this turn, having moved its shield counter onto `target`.
fn diamond_shield(t: &mut TestGame, target: ObjectId) -> ObjectId {
    let city = t.enter(P0, "Diamond City");
    t.settle();
    assert_eq!(t.counters(city, "shield"), 1);
    t.enter(P0, "Memnite");
    t.enter(P0, "Ornithopter");
    t.settle();
    t.activate(P0, city, 1, &[obj(target)]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(target, "shield"), 1);
    assert_eq!(t.counters(city, "shield"), 0);
    city
}

#[test]
fn diamond_city_shield_counter_isnt_regeneration() {
    cr!("122.1c", "701.19a");
    ruling!(
        "Diamond City",
        "Removing a shield counter when a permanent would be dealt damage or destroyed isn’t the same as regenerating a permanent."
    );
    supported("Nekrataal");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    diamond_shield(&mut t, bears);
    // Attacking, so a regeneration would remove it from combat and tap it.
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.g.objects[bears.0 as usize].tapped = false;
    // "That creature can't be regenerated" doesn't stop the shield counter.
    t.answer_targets(P1, &[obj(bears)]);
    t.enter(P1, "Nekrataal");
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.counters(bears, "shield"), 0);
    assert!(!t.obj_now(bears).tapped);
    assert!(t
        .g
        .combat
        .as_ref()
        .is_some_and(|c| c.attackers.iter().any(|a| a.id == bears)));
}

#[test]
fn diamond_city_shield_counter_doesnt_stop_sacrifice() {
    cr!("122.1c", "701.21a");
    ruling!(
        "Diamond City",
        "Shield counters don’t prevent players from sacrificing permanents."
    );
    supported("Diabolic Edict");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    diamond_shield(&mut t, bears);
    // Only the Bears: the Memnite and Ornithopter leave first.
    for o in ["Memnite", "Ornithopter"] {
        let id = t.named_on_battlefield(o)[0];
        move_to(&mut t, id, Zone::Graveyard(P0));
    }
    t.set_step(P1, Step::PrecombatMain);
    let edict = t.hand(P1, "Diabolic Edict");
    mana(&mut t, P1, ManaType::B, 1);
    mana(&mut t, P1, ManaType::C, 1);
    t.cast(P1, edict).target(Entity::Player(P0)).go();
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Graveyard(P0));
}

#[test]
fn diamond_city_shield_counter_works_without_abilities() {
    cr!("122.1c", "613.1f");
    ruling!(
        "Diamond City",
        "“Shield” is not an ability that permanents have and shield counters are not keyword counters. If a permanent with a shield counter loses its abilities, the shield counter will still protect it as normal."
    );
    supported("Turn to Frog");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    diamond_shield(&mut t, bears);
    t.set_step(P1, Step::PrecombatMain);
    let frog = t.hand(P1, "Turn to Frog");
    mana(&mut t, P1, ManaType::U, 1);
    mana(&mut t, P1, ManaType::C, 1);
    t.cast(P1, frog).target(bears).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (1, 1));
    crate::r_p194_common::shock(&mut t, P1, obj(bears));
    assert!(t.on_battlefield(bears));
    assert_eq!(t.counters(bears, "shield"), 0);
    assert_eq!(t.obj_now(bears).damage, 0);
}
