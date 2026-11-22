//! Rulings batch P156 — cards with scry (grouped by the scry function): blocks that stay
//! once declared, loyalty abilities activated twice, characteristic-defining bonuses
//! outside the battlefield, effects that lock in the affected set as they resolve, cost
//! changes and X, untap restrictions that follow the creature, and target legality.

use crate::r_p062_common::{is_tapped, misses_one_untap, tap, tapped_creature};
use crate::r_p117_common::modify;
use crate::r_s01_common::*;
use crate::r_s03_common::to_blockers;
use crate::r_s05_common::enter;
use crate::r_s06_common::activate_containing;
use crate::r_s25_common::{cast_new, lands_for_cost, x_of};
use crate::r_s29_common::{choose_modes, put_counters};
use mtg_engine::ability::Modification;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

fn pump_power(t: &mut TestGame, id: ObjectId, n: i32) {
    use mtg_engine::ability::Value;
    modify(t, id, vec![Modification::ModifyPT(Value::c(n), Value::c(0))]);
}

#[test]
fn blocks_stay_after_evasion_or_power_changes() {
    cr!("509.1b", "506.4", "509.1h");
    ruling!(
        "Cavern Stomper",
        "Activating Cavern Stomper's last ability after it has been blocked by one or more creatures with power 2 or less won't change or undo those blocks."
    );
    ruling!(
        "Cavern Stomper",
        "Reducing a creature's power after it has blocked Cavern Stomper will not remove that blocking creature from combat or make Cavern Stomper unblocked."
    );
    ruling!(
        "April O'Neil, Kunoichi Trainee",
        "Once April has been blocked, changing the blocking creature's power to 3 or greater won't cause April to become unblocked."
    );
    supported("Cavern Stomper");
    supported("April O'Neil, Kunoichi Trainee");
    // Blocked by Grizzly Bears, then the ability: still blocked.
    let mut t = TestGame::new(2);
    let stomper = t.battlefield(P0, "Cavern Stomper");
    t.lands(P0, "Forest", 4);
    let bears = t.battlefield(P1, "Grizzly Bears");
    to_blockers(&mut t, &[(stomper, Entity::Player(P1))], &[(bears, stomper)]);
    activate_containing(&mut t, P0, stomper, "can't be blocked").expect("activate");
    t.resolve_all();
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.zone(bears), Zone::Graveyard(P1));
    // The ability first: Hill Giant (power 3) blocks, then gets -2/-0: still blocked.
    let mut t = TestGame::new(2);
    let stomper = t.battlefield(P0, "Cavern Stomper");
    t.lands(P0, "Forest", 4);
    let giant = t.battlefield(P1, "Hill Giant");
    activate_containing(&mut t, P0, stomper, "can't be blocked").expect("activate");
    t.resolve_all();
    to_blockers(&mut t, &[(stomper, Entity::Player(P1))], &[(giant, stomper)]);
    pump_power(&mut t, giant, -2);
    assert_eq!(t.pt(giant).0, 1);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.zone(giant), Zone::Graveyard(P1));
    // Control: with the ability, Grizzly Bears can't block it.
    let mut t = TestGame::new(2);
    let stomper = t.battlefield(P0, "Cavern Stomper");
    t.lands(P0, "Forest", 4);
    let bears = t.battlefield(P1, "Grizzly Bears");
    activate_containing(&mut t, P0, stomper, "can't be blocked").expect("activate");
    t.resolve_all();
    t.attack(&[(stomper, Entity::Player(P1))], &[(bears, stomper)]);
    assert_eq!(t.life(P1), 13);
    // April O'Neil, blocked by Grizzly Bears which then gets +2/+0: still blocked.
    let mut t = TestGame::new(2);
    let april = t.battlefield(P0, "April O'Neil, Kunoichi Trainee");
    let bears = t.battlefield(P1, "Grizzly Bears");
    to_blockers(&mut t, &[(april, Entity::Player(P1))], &[(bears, april)]);
    pump_power(&mut t, bears, 2);
    assert_eq!(t.pt(bears).0, 4);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.zone(april), Zone::Graveyard(P0));
}

/// Puts Urza Assembles the Titans onto the battlefield and resolves its chapters I-III.
fn urza_chapter_three(t: &mut TestGame) {
    let saga = t.battlefield(P0, "Urza Assembles the Titans");
    put_counters(t, saga, "lore", 2);
    t.resolve_all();
    put_counters(t, saga, "lore", 1);
    t.resolve_all();
}

#[test]
fn urza_assembles_the_titans_allows_exactly_two_loyalty_activations() {
    cr!("606.3", "714.2b");
    ruling!(
        "Urza Assembles the Titans",
        "After this Saga’s third chapter ability resolves, you may activate the same ability of a planeswalker twice that turn, or you may activate two different abilities of that planeswalker that turn."
    );
    ruling!(
        "Urza Assembles the Titans",
        "If more than one effect allows you to activate loyalty abilities of planeswalkers your control twice rather than only once, those effects don’t continue to increase the number of activations. You won’t be able to activate abilities of planeswalkers you control more than twice in a turn."
    );
    supported("Urza Assembles the Titans");
    supported("Jace Beleren");
    for (sagas, second) in [(1, "Each player"), (1, "Target player"), (2, "Each player")] {
        let mut t = TestGame::new(2);
        for _ in 0..sagas {
            urza_chapter_three(&mut t);
        }
        let jace = t.battlefield(P0, "Jace Beleren");
        activate_containing(&mut t, P0, jace, "+2").expect("first");
        t.resolve_all();
        t.answer_targets(P0, &[Entity::Player(P0)]);
        activate_containing(&mut t, P0, jace, second).expect("second");
        t.resolve_all();
        assert!(
            activate_containing(&mut t, P0, jace, "+2").is_err(),
            "{sagas} sagas: a third activation"
        );
        t.clear_answers();
    }
    // Control: without the Saga, only once.
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P0, "Jace Beleren");
    activate_containing(&mut t, P0, jace, "+2").expect("first");
    t.resolve_all();
    assert!(activate_containing(&mut t, P0, jace, "+2").is_err());
}

#[test]
fn alrund_is_1_1_outside_the_battlefield() {
    cr!("611.3a", "604.2");
    ruling!(
        "Alrund, God of the Cosmos // Hakka, Whispering Raven",
        "Alrund's first ability applies only while Alrund is on the battlefield. In every other zone, Alrund is 1/1."
    );
    supported("Alrund, God of the Cosmos // Hakka, Whispering Raven");
    let mut t = TestGame::new(2);
    t.hand(P0, "Island");
    t.hand(P0, "Island");
    let in_hand = t.hand(P0, "Alrund, God of the Cosmos // Hakka, Whispering Raven");
    let in_gy = t.graveyard(P0, "Alrund, God of the Cosmos // Hakka, Whispering Raven");
    t.g.recompute();
    assert_eq!(t.pt(in_hand), (1, 1));
    assert_eq!(t.pt(in_gy), (1, 1));
    let on_bf = t.battlefield(P0, "Alrund, God of the Cosmos // Hakka, Whispering Raven");
    t.g.recompute();
    // Three cards in hand (two Islands and the other Alrund).
    assert_eq!(t.pt(on_bf), (4, 4));
}

#[test]
fn alrund_offers_only_card_types() {
    cr!("205.2a", "608.2d");
    ruling!(
        "Alrund, God of the Cosmos // Hakka, Whispering Raven",
        "You may choose any card type, but the ones that may appear in your library are artifact, battle, creature, enchantment, instant, kindred, land, planeswalker, and sorcery. Supertypes such as snow and basic may not be chosen. Subtypes such as God, Aura, and Forest also may not be chosen."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Alrund, God of the Cosmos // Hakka, Whispering Raven");
    let from = t.asked().len();
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let options: Vec<String> = asked_since(&t, from)
        .into_iter()
        .find_map(|(p, d)| match d {
            Decision::ChooseOption { options, .. } if p == P0 => Some(options),
            _ => None,
        })
        .expect("a card type choice");
    let has = |s: &str| options.iter().any(|o| o.eq_ignore_ascii_case(s));
    for ty in [
        "Artifact",
        "Battle",
        "Creature",
        "Enchantment",
        "Instant",
        "Kindred",
        "Land",
        "Planeswalker",
        "Sorcery",
    ] {
        assert!(has(ty), "{ty} missing from {options:?}");
    }
    for not in ["Snow", "Basic", "Legendary", "God", "Aura", "Forest"] {
        assert!(!has(not), "{not} offered: {options:?}");
    }
}

#[test]
fn effects_that_lock_in_the_affected_creatures_as_they_resolve() {
    cr!("611.2c");
    ruling!(
        "Behold the Unspeakable // Vision of the Unspeakable",
        "Behold the Unspeakable's first chapter ability affects creatures you don't control at the time it resolves. It does not apply to creatures that come under your opponents' control after it resolves. They will have to wait to behold it another time."
    );
    ruling!(
        "Drown in Sorrow",
        "Drown in Sorrow affects only creatures on the battlefield at the time it resolves. Creatures that enter the battlefield or become creatures later in the turn won't get -2/-2."
    );
    supported("Behold the Unspeakable // Vision of the Unspeakable");
    supported("Drown in Sorrow");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let saga = t.battlefield(P0, "Behold the Unspeakable // Vision of the Unspeakable");
    put_counters(&mut t, saga, "lore", 1);
    t.resolve_all();
    assert_eq!(t.pt(bears), (0, 2));
    assert_eq!(t.pt(giant), (3, 3));
    // The Giant comes under P1's control afterwards: unaffected.
    crate::r_p147_common::take_control(&mut t, P1, giant);
    t.g.recompute();
    assert_eq!(t.obj_now(giant).controller, P1);
    assert_eq!(t.pt(giant), (3, 3));
    // Drown in Sorrow: a creature entering afterwards isn't affected.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    cast_new(&mut t, P0, "Drown in Sorrow", &[]);
    t.resolve_all();
    assert_eq!(t.pt(wurm), (4, 2));
    let later = enter(&mut t, P1, "Craw Wurm");
    t.g.recompute();
    assert_eq!(t.pt(later), (6, 4));
}

#[test]
fn charming_prince_can_flicker_a_creature_you_own_that_another_player_controls() {
    cr!("115.1", "603.7", "110.2");
    ruling!(
        "Charming Prince",
        "Charming Prince's third mode can target any creature you own, including those another player controls."
    );
    supported("Charming Prince");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_p147_common::take_control(&mut t, P1, bears);
    assert_eq!(t.obj_now(bears).controller, P1);
    choose_modes(&mut t, P0, &[2]);
    t.answer_targets(P0, &[obj(bears)]);
    enter(&mut t, P0, "Charming Prince");
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1);
    assert_eq!(t.obj(back[0]).controller, P0);
}

#[test]
fn delightful_discovery_cant_cost_less_than_u() {
    cr!("601.2f", "118.7a");
    ruling!(
        "Delightful Discovery",
        "Delightful Discovery's first ability can't reduce its cost below {U}."
    );
    supported("Delightful Discovery");
    let setup = |opp_spells: usize, land: &str| {
        let mut t = TestGame::new(2);
        for _ in 0..opp_spells {
            cast_new(&mut t, P1, "Dark Ritual", &[]);
            t.resolve_all();
        }
        t.set_step(P0, Step::PrecombatMain);
        t.lands(P0, land, 1);
        let card = t.hand(P0, "Delightful Discovery");
        let ok = t.cast_with(P0, card, &[]).is_ok();
        (t, ok)
    };
    // Five opponent spells: {U} remains; one Island pays it.
    let (mut t, ok) = setup(5, "Island");
    assert!(ok);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    // ...but a colorless land can't: the {U} isn't reduced.
    assert!(!setup(6, "Wastes").1);
    // Control: without opponent spells, one Island isn't enough.
    assert!(!setup(0, "Island").1);
}

#[test]
fn multiple_choice_x_cost_changes_free_casting_and_no_targets() {
    cr!("601.2b", "601.2f", "107.3b", "115.1");
    ruling!(
        "Multiple Choice",
        "Effects that increase or decrease costs are applied after you choose the value of X, and will not impact the results of the spell."
    );
    ruling!(
        "Multiple Choice",
        "If an effect allows you to cast Multiple Choice without paying its mana cost, you must choose 0 as the value of X. Cheating on tests is frowned upon."
    );
    ruling!("Multiple Choice", "Multiple Choice doesn’t have any targets.");
    ruling!(
        "Multiple Choice",
        "You can choose 0 as the value of X as you cast Multiple Choice, but it will have no effect when it resolves."
    );
    supported("Multiple Choice");
    supported("Thalia, Guardian of Thraben");
    supported("Leyline of Sanctity");
    // Thalia: {1} more. X = 1, paying {1}{U} + {1}: X is still 1.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 2);
    let card = t.hand(P0, "Multiple Choice");
    let hand = t.hand_size(P0);
    let spell = t.cast(P0, card).x(1).go();
    assert_eq!(x_of(&t, spell), Some(1));
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand); // cast one, drew one
    assert_eq!(tokens(&t, P0).len(), 0);
    // Without paying its mana cost: X is 0 even if 3 is asked for; nothing happens.
    // Omniscience: "You may cast spells from your hand without paying their mana costs."
    supported("Omniscience");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Omniscience");
    let card = t.hand(P0, "Multiple Choice");
    let hand = t.hand_size(P0);
    let free = crate::r_s08_common::legal_cast_methods(&mut t, P0, card)
        .into_iter()
        .find(|m| *m != CastMethod::Normal)
        .expect("a free cast");
    let spell = t.cast(P0, card).x(3).method(free).go();
    assert_eq!(x_of(&t, spell).unwrap_or(0), 0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand - 1);
    assert_eq!(tokens(&t, P0).len(), 0);
    // X = 0 cast normally: no effect.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 1);
    let card = t.hand(P0, "Multiple Choice");
    let hand = t.hand_size(P0);
    t.cast(P0, card).x(0).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand - 1);
    assert_eq!(tokens(&t, P0).len(), 0);
    // X = 2: the chosen player isn't targeted, so a player with hexproof can be chosen.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Leyline of Sanctity");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 3);
    let card = t.hand(P0, "Multiple Choice");
    t.answer_choose(P0, &[Entity::Player(P1)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Player(P1)]);
    t.answer_choose(P1, &[obj(bears)]);
    t.cast(P0, card).x(2).go();
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Hand(P1));
}

#[test]
fn sudden_storm_untap_restriction_follows_the_creature_and_skips_illegal_targets() {
    cr!("608.2b", "502.3", "611.2c");
    ruling!(
        "Sudden Storm",
        "If a creature affected by Sudden Storm changes controllers before its old controller’s next untap step, Sudden Storm will prevent it from becoming untapped during its new controller’s next untap step."
    );
    ruling!(
        "Sudden Storm",
        "This spell can target tapped creatures. If a targeted creature is already tapped when the spell resolves, that creature remains tapped and doesn’t untap during its controller’s next untap step."
    );
    ruling!(
        "Sudden Storm",
        "If you chose two targets and one is an illegal target when Sudden Storm resolves, that creature won’t become tapped and it won’t be stopped from untapping during its controller’s next untap step. It won’t be affected by Sudden Storm in any way."
    );
    supported("Sudden Storm");
    // P1's Bears, then P0 gains control of it: it misses P0's next untap step.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Sudden Storm", &[obj(bears)]);
    t.resolve_all();
    assert!(is_tapped(&t, bears));
    crate::r_p147_common::take_control(&mut t, P0, bears);
    misses_one_untap(&mut t, bears, P0);
    // Already tapped: stays tapped and misses its controller's next untap step.
    let mut t = TestGame::new(2);
    let bears = tapped_creature(&mut t, P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Sudden Storm", &[obj(bears)]);
    t.resolve_all();
    misses_one_untap(&mut t, bears, P1);
    // Two targets; the Giant gets shroud in response: it isn't affected at all.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(bears), obj(giant)]);
    lands_for_cost(&mut t, P0, "Sudden Storm");
    let card = t.hand(P0, "Sudden Storm");
    t.cast_with(P0, card, &[]).expect("cast");
    modify(
        &mut t,
        giant,
        vec![Modification::AddKeyword(mtg_engine::keywords::Keyword::new(
            KeywordKind::Shroud,
        ))],
    );
    t.resolve_all();
    assert!(is_tapped(&t, bears));
    assert!(!is_tapped(&t, giant));
    tap(&mut t, giant);
    let mut t2 = t;
    crate::r_s04_common::next_upkeep(&mut t2, P1);
    assert!(is_tapped(&t2, bears));
    assert!(!is_tapped(&t2, giant));
}

#[test]
fn stun_spells_on_an_already_tapped_creature_still_add_three_stun_counters() {
    cr!("122.1d", "701.26a");
    ruling!(
        "Freeze in Place",
        "You may target a creature that is already tapped with Freeze in Place. If the target creature is already tapped as Freeze in Place resolves, you will still put three stun counters on it."
    );
    ruling!(
        "Impede Momentum",
        "You may target a creature that is already tapped with Impede Momentum. If the target creature is tapped as it resolves, you will still put three stun counters on it."
    );
    for name in ["Freeze in Place", "Impede Momentum"] {
        supported(name);
        let mut t = TestGame::new(2);
        let bears = tapped_creature(&mut t, P1, "Grizzly Bears");
        cast_new(&mut t, P0, name, &[obj(bears)]);
        t.resolve_all();
        assert!(is_tapped(&t, bears), "{name}");
        assert_eq!(t.counters(bears, "stun"), 3, "{name}");
    }
}

#[test]
fn portent_of_betrayal_can_target_your_own_or_an_untapped_creature() {
    cr!("115.1", "701.26b");
    ruling!(
        "Portent of Betrayal",
        "Portent of Betrayal can target any creature, even one that's untapped or one you already control."
    );
    supported("Portent of Betrayal");
    let mut t = TestGame::new(2);
    let mine = tapped_creature(&mut t, P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Portent of Betrayal", &[obj(mine)]);
    t.resolve_all();
    assert!(!is_tapped(&t, mine));
    assert!(t.obj_now(mine).has_keyword(KeywordKind::Haste));
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Portent of Betrayal", &[obj(theirs)]);
    t.resolve_all();
    assert_eq!(t.obj_now(theirs).controller, P0);
    assert!(!is_tapped(&t, theirs));
}

#[test]
fn condescend_needs_another_spell_to_target() {
    cr!("601.2c", "115.5");
    ruling!(
        "Condescend",
        "You must be able to target another spell to cast Condescend. Condescend can’t target itself."
    );
    supported("Condescend");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 1);
    let card = t.hand(P0, "Condescend");
    assert!(t.cast(P0, card).x(0).try_go().is_err());
    assert_eq!(t.zone(card), Zone::Hand(P0));
    // With another spell on the stack, it can be cast targeting that spell.
    let mut t = TestGame::new(2);
    let ritual = cast_new(&mut t, P0, "Dark Ritual", &[]);
    t.lands(P0, "Island", 1);
    let card = t.hand(P0, "Condescend");
    let spell = t.cast(P0, card).x(0).target(ritual).go();
    assert_eq!(crate::r_s25_common::targets_of(&t, spell), vec![obj(ritual)]);
}

fn choose_creature_type(_g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
    match d {
        Decision::ChooseOption { options, .. } => options
            .iter()
            .position(|o| o.eq_ignore_ascii_case("creature"))
            .map(Answer::Index),
        _ => None,
    }
}

#[test]
fn alrund_takes_only_cards_revealed_this_way() {
    cr!("701.20a", "608.2c");
    ruling!(
        "Alrund, God of the Cosmos // Hakka, Whispering Raven",
        "Alrund recieved day 1 eratta to clarify that you select from the cards \"revealed this way\"."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Alrund, God of the Cosmos // Hakka, Whispering Raven");
    // Library top: Grizzly Bears, Island, then another Grizzly Bears (not revealed).
    let lib = stack_library(&mut t, P0, &["Grizzly Bears", "Island", "Grizzly Bears"]);
    crate::r_s03_common::respond(&mut t, P0, choose_creature_type);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.zone(lib[0]), Zone::Hand(P0));
    assert_eq!(t.zone(lib[1]), Zone::Library(P0));
    assert_eq!(t.zone(lib[2]), Zone::Library(P0));
    // The unrevealed Bears is now on top; the Island went to the bottom.
    assert_eq!(t.g.player(P0).library.last(), Some(&t.g.current(lib[2])));
    assert_eq!(t.g.player(P0).library.first(), Some(&t.g.current(lib[1])));
}
