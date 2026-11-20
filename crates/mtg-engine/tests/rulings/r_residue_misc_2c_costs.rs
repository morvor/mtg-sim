//! Item `rulings-residue-misc-2`, pass C — casting, costs and timing: Jadzi's alternative
//! cost, prowl with an added creature type and cost increases, Heart of Kiran's crew
//! alternative, Raul's permission, Brazen Cannonade's exiled card, Celestial Reunion's
//! behold, cloak's face-up special action, Path of Ancestry, Autumn Willow on the stack,
//! Storm of Memories' copies, Mission Briefing, Shivan Sand-Mage's modes, extra phases
//! (World at War, Moraug), clash (Whirlpool Whelm), connive (Change of Plans), and
//! Elemental Expressionist's granted abilities.

use crate::r_residue_misc_2c_common::*;
use crate::r_s02_common::can_play_land;
use mtg_engine::decision::{Action, Answer, Decision, SpecialAction};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Whether P0 could cast the card now (any way).
fn castable(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    crate::r_p079_common::can_cast_now(t, p, card)
}

// --- Jadzi, Oracle of Arcavios ----------------------------------------------------------------

const JADZI: &str = "Jadzi, Oracle of Arcavios // Journey to the Oracle";

/// P0 controls Jadzi and casts Opt with `top` on top of the library (under the Opt's draw:
/// the magecraft trigger resolves first and reveals it); P0 casts it for {1}.
fn jadzi_reveals(t: &mut TestGame, top: &str) -> ObjectId {
    t.battlefield(P0, JADZI);
    t.lands(P0, "Island", 2);
    let card = t.library_top(P0, top);
    let opt = t.hand(P0, "Opt");
    t.answer_yes(P0, true);
    t.cast(P0, opt).go();
    card
}

#[test]
fn jadzi_casts_for_one_with_additional_costs_but_no_other_alternative_cost() {
    cr!("118.9a", "118.8", "601.2b", "601.2f");
    ruling!(
        "Jadzi, Oracle of Arcavios // Journey to the Oracle",
        "Because the magecraft ability supplies an alternative cost of {1} for the spell, you cannot pay any other alternative costs that may apply. However, you may pay additional costs and must pay any additional costs that are required."
    );
    // A required additional cost: Village Rites' sacrifice is paid.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(bears)]);
    jadzi_reveals(&mut t, "Village Rites");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Village Rites"), "{}", t.dump_log());
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    // An optional one: Burst Lightning kicked ({1} and {4}) deals 4.
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 4);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_targets(P0, &[pl(P1)]);
    jadzi_reveals(&mut t, "Burst Lightning");
    t.resolve_all();
    assert_eq!(t.life(P1), 16, "{}", t.dump_log());
}

#[test]
fn jadzi_casts_an_x_spell_with_x_zero() {
    cr!("107.3b", "118.9a");
    ruling!(
        "Jadzi, Oracle of Arcavios // Journey to the Oracle",
        "If you cast a spell with {X} in its mana cost using Jadzi’s magecraft ability, you must choose 0 as the value of X."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 5);
    t.answer(P0, DecisionKind::X, Answer::Number(4));
    jadzi_reveals(&mut t, "Endless One");
    t.resolve_all();
    // A 0/0 Endless One: it died.
    assert!(t.in_graveyard(P0, "Endless One"), "{}", t.dump_log());
    assert!(t.named_on_battlefield("Endless One").is_empty());
}

// --- Enigma Thief (prowl) ---------------------------------------------------------------------

const PROWL: CastMethod = CastMethod::Keyword(KeywordKind::Prowl);

/// P0's `attacker` dealt combat damage to P1 this turn; P0 has Enigma Thief in hand and
/// lands for {3}{U} plus `extra`.
fn after_damage(t: &mut TestGame, attacker: &str, extra: usize) -> ObjectId {
    let a = t.battlefield(P0, attacker);
    attack_with(t, &[(a, pl(P1))]);
    t.advance_to(P0, Step::PostcombatMain);
    t.resolve_all();
    assert!(t.life(P1) < 20);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 3 + extra);
    t.hand(P0, "Enigma Thief")
}

#[test]
fn enigma_thief_prowl_compares_the_spells_creature_types_as_they_are() {
    cr!("702.76a", "613.1d");
    ruling!(
        "Enigma Thief",
        "Prowl compares the creature types of the spell with the creature types of the creatures that dealt combat damage to players this turn. Normally this means that you can cast Enigma Thief for its prowl cost only if a Sphinx or Rogue dealt combat damage to a player, but if an effect causes the spell to have other creature types, the prowl ability is also satisfied by a creature with those additional types having dealt combat damage to a player."
    );
    supported("Arcane Adaptation");
    // Llanowar Elves dealt the damage; without an effect, no prowl.
    let mut t = TestGame::new(2);
    let thief = after_damage(&mut t, "Llanowar Elves", 0);
    assert!(!can_cast(&mut t, P0, thief, PROWL));
    // Arcane Adaptation naming Elf: the Thief spell is an Elf too.
    let mut t = TestGame::new(2);
    choose_creature_type(&mut t, P0, "Elf");
    t.enter(P0, "Arcane Adaptation");
    t.settle();
    let thief = after_damage(&mut t, "Llanowar Elves", 0);
    assert!(can_cast(&mut t, P0, thief, PROWL), "{}", t.dump_log());
    t.cast(P0, thief).method(PROWL).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Enigma Thief").len(), 1);
}

#[test]
fn enigma_thief_prowl_cost_is_increased_and_its_mana_value_stays_seven() {
    cr!("601.2f", "202.3a", "118.9c");
    ruling!(
        "Enigma Thief",
        "To determine the total cost of a spell, start with the mana cost or alternative cost (such as a prowl cost) you're paying, add any cost increases, then apply any cost reductions. The mana value of the spell is determined only by its mana cost, no matter what the total cost to cast the spell was."
    );
    supported("Sphere of Resistance");
    // Sphere of Resistance: spells cost {1} more. {3}{U} + {1}: not with four lands.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Sphere of Resistance");
    // Changeling Outcast (a Shapeshifter with changeling: a Rogue) deals the damage.
    let thief = after_damage(&mut t, "Changeling Outcast", 0);
    assert!(!can_cast(&mut t, P0, thief, PROWL));
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Sphere of Resistance");
    let thief = after_damage(&mut t, "Changeling Outcast", 1);
    assert!(can_cast(&mut t, P0, thief, PROWL), "{}", t.dump_log());
    let spell = t.cast(P0, thief).method(PROWL).go();
    assert_eq!(t.g.mana_value_of(spell), 7);
}

// --- Heart of Kiran ---------------------------------------------------------------------------

#[test]
fn heart_of_kiran_loyalty_crew_isnt_a_loyalty_ability() {
    cr!("606.3", "702.122a", "118.9");
    ruling!(
        "Heart of Kiran",
        "Paying Heart of Kiran’s alternative crew cost isn’t a loyalty ability. It can be done even if you’ve already activated a loyalty ability of the planeswalker this turn, and it can be done any time you could activate Heart of Kiran’s crew ability."
    );
    let mut t = TestGame::new(2);
    let heart = t.battlefield(P0, "Heart of Kiran");
    let jace = t.battlefield(P0, "Jace Beleren");
    t.activate(P0, jace, 0, &[]).expect("Jace +2");
    t.resolve_all();
    let loyalty = t.counters(jace, counters::LOYALTY);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.activate(P0, heart, 0, &[])
        .expect("crew with a loyalty counter");
    t.resolve_all();
    assert_eq!(t.counters(jace, counters::LOYALTY), loyalty - 1);
    assert!(is_creature(&mut t, heart));
    // On P1's turn too (crew is an instant-speed ability).
    t.advance_to(P1, Step::DeclareAttackers);
    assert!(!is_creature(&mut t, heart));
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.activate(P0, heart, 0, &[]).expect("crew on P1's turn");
    t.resolve_all();
    assert!(is_creature(&mut t, heart));
    assert_eq!(t.counters(jace, counters::LOYALTY), loyalty - 2);
}

// --- Raul, Trouble Shooter --------------------------------------------------------------------

/// P0's Raul mills `n` cards (untapping in between); returns Raul.
fn raul_mills(t: &mut TestGame, n: usize) -> ObjectId {
    let raul = t.battlefield(P0, "Raul, Trouble Shooter");
    for _ in 0..n {
        let now = t.g.current(raul);
        t.g.objects[now.0 as usize].tapped = false;
        t.activate(P0, raul, 0, &[]).expect("Raul mills");
        t.resolve_all();
    }
    raul
}

#[test]
fn raul_permission_isnt_used_by_another_permission_and_a_new_raul_gives_another() {
    cr!("601.2", "400.7", "702.34a");
    ruling!(
        "Raul, Trouble Shooter",
        "If you cast a spell from among cards in your graveyard that were milled this turn using another permission, Raul’s effect doesn't apply. You can cast another spell among cards in your graveyard that were milled this turn using the permission granted by Raul’s first ability."
    );
    ruling!(
        "Raul, Trouble Shooter",
        "If you cast a spell from among cards in your graveyard that were milled this turn using the permission granted by Raul’s first ability and then have a new Raul, Trouble Shooter come under your control in the same turn, you may cast another spell from among cards in your graveyard that were milled this turn."
    );
    let mut t = TestGame::new(2);
    stack_library(
        &mut t,
        P0,
        &["Think Twice", "Grizzly Bears", "Savannah Lions"],
    );
    let raul = raul_mills(&mut t, 3);
    let gy = |t: &TestGame, n: &str| t.g.find_in_zone(Zone::Graveyard(P0), n)[0];
    // Think Twice with its own flashback permission.
    t.lands(P0, "Island", 3);
    let tt = gy(&t, "Think Twice");
    t.cast(P0, tt)
        .method(CastMethod::Keyword(KeywordKind::Flashback))
        .go();
    t.resolve_all();
    // Raul's permission is still there: the Bears.
    t.lands(P0, "Forest", 2);
    let bears = gy(&t, "Grizzly Bears");
    assert!(castable(&mut t, P0, bears));
    t.cast(P0, bears).go();
    t.resolve_all();
    // Used: the Lions can't be cast now.
    t.lands(P0, "Plains", 1);
    let lions = gy(&t, "Savannah Lions");
    assert!(!castable(&mut t, P0, lions));
    // A new Raul (the old one leaves, another comes under P0's control): the Lions.
    kill(&mut t, raul);
    t.battlefield(P0, "Raul, Trouble Shooter");
    assert!(castable(&mut t, P0, lions), "{}", t.dump_log());
}

// --- Brazen Cannonade -------------------------------------------------------------------------

/// P0 attacks with Grizzly Bears; Brazen Cannonade's raid trigger exiles `top` in P0's
/// postcombat main phase. Returns the exiled card.
fn cannonade_exiles(t: &mut TestGame, top: &str) -> ObjectId {
    t.battlefield(P0, "Brazen Cannonade");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let card = t.library_top(P0, top);
    attack_with(t, &[(bears, pl(P1))]);
    t.advance_to(P0, Step::PostcombatMain);
    t.resolve_all();
    let exiled = t.g.current(card);
    assert_eq!(t.zone(exiled), Zone::Exile);
    exiled
}

#[test]
fn brazen_cannonade_exiled_land_needs_an_available_land_play() {
    cr!("305.2", "305.3");
    ruling!(
        "Brazen Cannonade",
        "If you exile a land card with the raid ability, you may play that land only if you have any available land plays. Normally, this means you can play the land only if you haven't played a land yet that turn."
    );
    supported("Brazen Cannonade");
    let mut t = TestGame::new(2);
    let forest = cannonade_exiles(&mut t, "Forest");
    assert!(can_play_land(&mut t, P0, forest));
    let mut t = TestGame::new(2);
    let played = t.hand(P0, "Plains");
    t.play_land(P0, played).expect("land");
    let forest = cannonade_exiles(&mut t, "Forest");
    assert!(!can_play_land(&mut t, P0, forest));
}

#[test]
fn brazen_cannonade_exiled_card_is_face_up_and_follows_normal_timing() {
    cr!("307.1", "302.1", "117.1a");
    ruling!(
        "Brazen Cannonade",
        "The card exiled by the raid ability is exiled face up. Playing a card exiled with this ability follows the normal rules for playing the card. You must pay its costs, and you must follow all applicable timing rules. For example, if it's a creature card, you can cast it only during your main phase while the stack is empty."
    );
    let mut t = TestGame::new(2);
    let giant = cannonade_exiles(&mut t, "Hill Giant");
    assert!(!t.obj_now(giant).face_down);
    // No mana: no.
    assert!(!castable(&mut t, P0, giant));
    t.lands(P0, "Mountain", 4);
    assert!(castable(&mut t, P0, giant));
    // Not with something on the stack, nor on P1's turn.
    let bolt = crate::r_p224_common::bolt(&mut t, P0, pl(P1));
    assert!(!castable(&mut t, P0, giant));
    let _ = bolt;
    t.resolve_all();
    t.advance_to(P1, Step::PrecombatMain);
    assert!(!castable(&mut t, P0, giant));
}

// --- Celestial Reunion ------------------------------------------------------------------------

#[test]
fn celestial_reunion_beholds_creatures_only_not_kindred_cards() {
    cr!("701.4a", "601.2b", "308.1");
    ruling!(
        "Celestial Reunion",
        "Notably, Celestial Reunion's behold instruction is limited to \"creatures.\" It doesn't allow you to behold kindred cards or kindred permanents, which are permitted for most other behold instructions."
    );
    supported("Celestial Reunion");
    // One Elf on the battlefield and an Elf card in hand; Elvish Mystic in the library.
    let run = |in_hand: &str| {
        let mut t = TestGame::new(2);
        t.lands(P0, "Forest", 2);
        t.battlefield(P0, "Llanowar Elves");
        t.hand(P0, in_hand);
        let found = t.library_top(P0, "Elvish Mystic");
        let spell = t.hand(P0, "Celestial Reunion");
        t.answer_yes(P0, true);
        t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
        t.answer_choose(P0, &[obj(found)]);
        let _ = t.cast(P0, spell).x(1).try_go();
        t.resolve_all();
        let now = t.g.current(found);
        t.zone(now)
    };
    // A creature card: the cost is paid, the Mystic enters.
    assert_eq!(run("Elvish Visionary"), Zone::Battlefield);
    // A kindred Elf card (Elvish Promenade, a Kindred Sorcery — Elf) can't be beheld.
    assert_eq!(run("Elvish Promenade"), Zone::Hand(P0));
}

// --- Become Anonymous (cloak) -----------------------------------------------------------------

#[test]
fn a_cloaked_creature_card_turns_face_up_for_its_mana_cost_as_a_special_action() {
    cr!("701.58c", "116.2b", "702.37e");
    ruling!(
        "Become Anonymous",
        "Any time you have priority, you can turn a cloaked permanent you control face up by revealing that it’s a creature card (ignoring any copy effects or type-changing effects that might be applying to it) and paying its mana cost. This is a special action. It doesn’t use the stack and can’t be responded to."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 4);
    let bears = t.battlefield(P0, "Grizzly Bears");
    stack_library(&mut t, P0, &["Forest", "Island"]);
    cast_new(&mut t, P0, "Become Anonymous", &[obj(bears)]);
    t.resolve_all();
    let cloaked: Vec<ObjectId> =
        t.g.permanents()
            .filter(|o| o.face_down)
            .map(|o| o.id)
            .collect();
    assert_eq!(cloaked.len(), 3);
    let card_named = |t: &TestGame, n: &str| {
        *cloaked
            .iter()
            .find(|id| t.g.obj(**id).card.as_ref().unwrap().front().chars.name == n)
            .unwrap()
    };
    let bear_cloak = card_named(&t, "Grizzly Bears");
    let forest_cloak = card_named(&t, "Forest");
    // During P1's turn, with priority: the Bears can be turned face up, the Forest can't
    // (it's a face-down 2/2 creature, but not a creature card).
    t.set_step(P1, Step::DeclareAttackers);
    t.lands(P0, "Forest", 2);
    t.g.turn.priority = Some(P0);
    t.g.recompute();
    let actions = t.g.legal_actions(P0);
    let up = |o: ObjectId| Action::Special(SpecialAction::TurnFaceUp { obj: o });
    assert!(actions.contains(&up(bear_cloak)), "{actions:?}");
    assert!(!actions.contains(&up(forest_cloak)));
    let stack = t.stack_len();
    t.g.perform_action(P0, up(bear_cloak))
        .expect("turned face up");
    assert_eq!(t.stack_len(), stack);
    assert!(!t.obj_now(bear_cloak).face_down);
    assert_eq!(t.obj_now(bear_cloak).chars.name, "Grizzly Bears");
}

// --- Path of Ancestry -------------------------------------------------------------------------

/// A Commander game where P0's commanders are `cmdrs` (in the command zone), with Path of
/// Ancestry; P0 taps it for mana of type `ty` (choosing it among the offered colors).
fn path_mana(cmdrs: &[&str], ty: ManaType) -> TestGame {
    let mut t = commander_game(2);
    t.set_step(P0, Step::PrecombatMain);
    for c in cmdrs {
        let id = t.command(P0, c);
        make_commander(&mut t, id);
    }
    let path = t.battlefield(P0, "Path of Ancestry");
    let colors = [
        ManaType::W,
        ManaType::U,
        ManaType::B,
        ManaType::R,
        ManaType::G,
    ];
    // The offered colors are the identity's, in WUBRG order.
    let ident: Vec<ManaType> = colors
        .into_iter()
        .filter(|c| {
            cmdrs.iter().any(|n| {
                let card = mtg_engine::card::card(n);
                format!(
                    "{}",
                    card.front().chars.mana_cost.clone().unwrap_or_default()
                )
                .contains(&format!("{c:?}"))
            })
        })
        .collect();
    if ident.len() > 1 {
        let i = ident.iter().position(|c| *c == ty).unwrap();
        t.answer(P0, DecisionKind::Option, Answer::Index(i));
    }
    t.g.turn.priority = Some(P0);
    let uid = crate::r_s27_common::ability_containing(&mut t, path, "commander's color identity");
    t.g.activate_ability(P0, path, uid).expect("Path");
    t.g.flush_events();
    assert_eq!(t.g.player(P0).mana_pool.count(ty), 1, "{}", t.dump_log());
    t
}

fn scries(t: &TestGame) -> usize {
    t.asked()
        .iter()
        .filter(|(_, d)| matches!(d, Decision::Scry { .. }))
        .count()
}

#[test]
fn path_of_ancestry_scries_for_the_commander_itself_and_for_either_partner() {
    cr!("903.3", "903.4", "106.6");
    ruling!(
        "Path of Ancestry",
        "If you cast your commander with mana from Path of Ancestry, and your commander hasn't somehow lost all of its creature types while on the stack, you'll scry 1."
    );
    ruling!(
        "Path of Ancestry",
        "If you have two commanders, the last ability adds one mana of any color in their combined color identities. When you spend that mana on a creature spell that shares a creature type with either of your commanders, you'll scry 1."
    );
    // The commander (Grizzly Bears) cast from the command zone with Path's {G}.
    let mut t = path_mana(&["Grizzly Bears"], ManaType::G);
    t.lands(P0, "Wastes", 1);
    let cmdr = t.g.find_in_zone(Zone::Command, "Grizzly Bears")[0];
    t.cast(P0, cmdr).go();
    t.resolve_all();
    assert_eq!(scries(&t), 1, "{}", t.dump_log());
    // Two commanders (Grizzly Bears and Storm Crow): {G} spent on Birds of Paradise, a
    // Bird like Storm Crow.
    let mut t = path_mana(&["Grizzly Bears", "Storm Crow"], ManaType::G);
    let birds = t.hand(P0, "Birds of Paradise");
    t.cast(P0, birds).go();
    t.resolve_all();
    assert_eq!(scries(&t), 1, "{}", t.dump_log());
    // ... but not on a creature sharing a type with neither (Llanowar Elves).
    let mut t = path_mana(&["Grizzly Bears", "Storm Crow"], ManaType::G);
    let elves = t.hand(P0, "Llanowar Elves");
    t.cast(P0, elves).go();
    t.resolve_all();
    assert_eq!(scries(&t), 0);
}

// --- Autumn Willow ----------------------------------------------------------------------------

#[test]
fn autumn_willow_can_be_countered_shroud_works_only_once_it_enters() {
    cr!("702.18a", "112.1", "701.6a");
    ruling!(
        "Autumn Willow",
        "This spell can be countered. Shroud does not work until it enters."
    );
    supported("Autumn Willow");
    let mut t = TestGame::new(2);
    let willow = cast_new(&mut t, P0, "Autumn Willow", &[]);
    crate::r_p224_common::counterspell(&mut t, P1, willow);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Autumn Willow"), "{}", t.dump_log());
}

// --- Storm of Memories ------------------------------------------------------------------------

#[test]
fn storm_of_memories_each_cast_spell_resolves_before_the_next_copy() {
    cr!("702.40a", "608.2g", "707.10");
    ruling!(
        "Storm of Memories",
        "The instants or sorceries cast using copies of Storm of Memories go on the stack as each copy of Storm of Memories resolves, and those spells will each resolve before more copies of Storm of Memories resolve."
    );
    supported("Storm of Memories");
    let mut t = TestGame::new(2);
    // Two spells cast before it: a copy for each.
    for _ in 0..2 {
        cast_new(&mut t, P0, "Ornithopter", &[]);
        t.resolve_all();
    }
    for _ in 0..3 {
        t.graveyard(P0, "Lightning Bolt");
    }
    for _ in 0..3 {
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[pl(P1)]);
    }
    t.lands(P0, "Mountain", 5);
    let spell = t.hand(P0, "Storm of Memories");
    t.cast(P0, spell).go();
    t.settle();
    // The storm trigger, then two copies above the spell.
    t.resolve();
    assert_eq!(t.stack_len(), 3, "{}", t.dump_log());
    t.resolve();
    // The first copy resolved: the Bolt it cast is on top, above the other copy.
    assert_eq!(t.stack_len(), 3, "{}", t.dump_log());
    let top = *t.g.stack.last().unwrap();
    assert_eq!(t.g.obj(top).chars.name, "Lightning Bolt");
    t.resolve();
    assert_eq!(t.life(P1), 17);
    t.resolve_all();
    assert_eq!(t.life(P1), 11);
}

// --- Mission Briefing -------------------------------------------------------------------------

#[test]
fn mission_briefing_may_choose_a_card_it_just_surveilled_into_the_graveyard() {
    cr!("701.25a", "608.2c");
    ruling!(
        "Mission Briefing",
        "The instant or sorcery card you choose may be one that you just surveilled into your graveyard."
    );
    let mut t = TestGame::new(2);
    let bolt = t.library_top(P0, "Lightning Bolt");
    let lib = &t.g.players[0].library;
    let under = lib[lib.len() - 2];
    t.answer(
        P0,
        DecisionKind::Surveil,
        Answer::Split(vec![under], vec![bolt]),
    );
    t.answer_choose_moved(P0, &[obj(bolt)]);
    cast_new(&mut t, P0, "Mission Briefing", &[]);
    t.resolve_all();
    let in_gy = t.g.find_in_zone(Zone::Graveyard(P0), "Lightning Bolt");
    assert_eq!(in_gy.len(), 1, "{}", t.dump_log());
    t.lands(P0, "Mountain", 1);
    t.cast(P0, in_gy[0]).target(pl(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(t.in_exile("Lightning Bolt"));
}

// --- Shivan Sand-Mage -------------------------------------------------------------------------

#[test]
fn shivan_sand_mage_with_nothing_suspended_must_choose_the_first_mode() {
    cr!("700.2a", "603.3c", "115.1");
    ruling!(
        "Shivan Sand-Mage",
        "When Shivan Sand-Mage enters, if there are no suspended cards and no permanents with time counters on them, you can't choose the second mode. You'll have to choose the first mode, and will have to choose a permanent as a target (though, in this case, the ability won't do anything when it resolves)."
    );
    supported("Shivan Sand-Mage");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.answer_targets(P0, &[obj(bears)]);
    let from = t.asked().len();
    t.enter(P0, "Shivan Sand-Mage");
    t.settle();
    t.resolve_all();
    let modes: Vec<Vec<usize>> = asked_since(&t, from)
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseModes { available, .. } => Some(available),
            _ => None,
        })
        .collect();
    assert!(modes.iter().all(|a| !a.contains(&1)), "{modes:?}");
    let targeted = asked_since(&t, from)
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseTargets { candidates, .. } if candidates.contains(&obj(bears))));
    assert!(targeted, "{}", t.dump_log());
    assert!(t.on_battlefield(bears));
}

// --- World at War, Moraug ---------------------------------------------------------------------

#[test]
fn world_at_war_cast_after_the_first_postcombat_main_phase_adds_nothing() {
    cr!("500.8", "505.1a", "702.88a");
    ruling!(
        "World at War",
        "As long as World at War resolves before the first postcombat main phase of a turn ends, it will have its full effect. That will happen in most cases, since it’s a sorcery (meaning it’s probably cast during the first or second main phase of a turn), and it has rebound (meaning it’ll be cast during your upkeep). If, however, it’s cast later than that, it won’t create any new phases. For example, if one World at War creates a second combat phase and a third main phase in a turn, then a second World at War is cast during that third main phase, no additional phases are created. The rebound effect still works, though."
    );
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PostcombatMain);
    cast_new(&mut t, P0, "World at War", &[]);
    t.resolve_all();
    // The additional combat, then the additional (third) main phase.
    t.advance_to(P0, Step::BeginningOfCombat);
    t.advance_to(P0, Step::PostcombatMain);
    assert_eq!(t.g.turn.combat_phases, 2);
    cast_new(&mut t, P0, "World at War", &[]);
    t.resolve_all();
    assert_eq!(
        t.g.find_in_zone(Zone::Exile, "World at War").len(),
        2,
        "rebound: {}",
        t.dump_log()
    );
    t.advance_to(P0, Step::End);
    assert_eq!(t.g.turn.combat_phases, 2, "{}", t.dump_log());
}

#[test]
fn moraug_twice_in_the_postcombat_main_phase_two_combats_then_the_ending_phase() {
    cr!("500.8", "505.1a");
    ruling!(
        "Moraug, Fury of Akoum",
        "There won't be an additional main phase after the additional combat phase. For example, if the landfall ability resolves twice during your postcombat main phase, you'll get two consecutive additional combat phases after your main phase (untapping your creatures at the beginning of each), followed by your ending phase."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Moraug, Fury of Akoum");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P0, Step::PostcombatMain);
    let f = t.hand(P0, "Forest");
    t.play_land(P0, f).expect("land");
    t.enter(P0, "Plains");
    t.resolve_all();
    assert_eq!(t.g.turn.main_phases, 2);
    // Two additional combats in a row, each untapping the Bears, with no main phase.
    for n in [2, 3] {
        t.g.objects[bears.0 as usize].tapped = true;
        t.advance_to(P0, Step::BeginningOfCombat);
        t.resolve_all();
        assert_eq!(t.g.turn.combat_phases, n, "{}", t.dump_log());
        assert_eq!(t.g.turn.main_phases, 2);
        assert!(
            !t.obj_now(bears).tapped,
            "untapped at the beginning of combat {n}"
        );
        t.advance_to(P0, Step::EndOfCombat);
    }
    // Then the ending phase.
    t.advance_to_step(Step::End);
    assert_eq!(t.g.turn.active, P0);
    assert_eq!((t.g.turn.combat_phases, t.g.turn.main_phases), (3, 2));
}

// --- Whirlpool Whelm (clash) ------------------------------------------------------------------

/// P0 casts Whirlpool Whelm at P1's Grizzly Bears; P0 wins the clash if `win`.
fn whelm(win: bool) -> (TestGame, ObjectId, usize) {
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    if win {
        t.library_top(P0, "Colossal Dreadmaw");
        t.library_top(P1, "Forest");
    } else {
        t.library_top(P1, "Colossal Dreadmaw");
        t.library_top(P0, "Forest");
    }
    let bear = t.battlefield(P1, "Grizzly Bears");
    let w = t.hand(P0, "Whirlpool Whelm");
    let from = t.asked().len();
    t.cast(P0, w).target(bear).go();
    (t, bear, from)
}

#[test]
fn whirlpool_whelm_targets_on_cast_and_the_owner_places_the_clash_card_first() {
    cr!("701.30a", "701.30c", "601.2c", "608.2c");
    ruling!(
        "Whirlpool Whelm",
        "You choose the target when you play Whirlpool Whelm, not when you clash."
    );
    ruling!(
        "Whirlpool Whelm",
        "If you clash with the owner of the targeted creature and win, the owner of the creature decides where to put to card revealed during the clash before you decide whether to put the creature on top of that library. You know the player's decision before you make your decision."
    );
    ruling!(
        "Whirlpool Whelm",
        "If you don't win the clash, the targeted creature always goes to its owner's hand. If you win the clash, you choose whether the creature goes to its owner's hand or the top of its owner's library."
    );
    supported("Whirlpool Whelm");
    // Won: P1 puts the Forest on the bottom; then P0 puts the Bears on top.
    let (mut t, bear, from) = whelm(true);
    // The target is chosen as it's cast: it's on the spell already.
    let spell = *t.g.stack.last().unwrap();
    assert_eq!(crate::r_s25_common::targets_of(&t, spell), vec![obj(bear)]);
    assert!(asked_since(&t, from)
        .iter()
        .all(|(_, d)| !matches!(d, Decision::YesNo { .. })));
    t.answer_yes(P0, false); // P0's clash card stays on top
    t.answer_yes(P1, true); // P1's goes to the bottom
    t.answer_yes(P0, true); // the Bears on top
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::YesNo { prompt, .. } if !prompt.contains("Clash")),
        |g| {
            let lib = &g.player(PlayerId(1)).library;
            g.obj(*lib.first().unwrap()).chars.name.to_string()
        },
    );
    t.resolve_all();
    // When P0 decided, P1's Forest was already on the bottom of P1's library.
    assert_eq!(
        *seen.lock().unwrap(),
        vec!["Forest".to_string()],
        "{}",
        t.dump_log()
    );
    let now = t.g.current(bear);
    assert_eq!(t.g.player(P1).library.last(), Some(&now));
    // Not won: to the hand, with no choice.
    let (mut t, _, _) = whelm(false);
    let from = t.asked().len();
    t.resolve_all();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert!(asked_since(&t, from).iter().all(|(p, d)| !(*p == P0
        && matches!(d, Decision::YesNo { prompt, .. } if !prompt.contains("Clash")))));
}

// --- Change of Plans (connive) ----------------------------------------------------------------

#[test]
fn change_of_plans_creatures_connive_one_at_a_time_then_phasing_is_chosen() {
    cr!("701.50c", "701.50a", "702.26a");
    ruling!(
        "Change of Plans",
        "If multiple creatures are instructed to connive simultaneously, they connive one at a time in the order of their controller's choice."
    );
    ruling!(
        "Change of Plans",
        "You choose which creatures to phase out after all of those creatures have connived."
    );
    supported("Change of Plans");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 4);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Savannah Lions");
    let cards = stack_library(&mut t, P0, &["Hill Giant", "Forest"]);
    // The Lions connive first (drawing and discarding the Hill Giant: a counter), then
    // the Bears (the Forest: none).
    t.answer_choose(P0, &[obj(b)]);
    t.answer_choose_moved(P0, &[obj(cards[0])]);
    t.answer_choose_moved(P0, &[obj(cards[1])]);
    let cop = t.hand(P0, "Change of Plans");
    let from = t.asked().len();
    t.cast(P0, cop).x(2).targets(&[obj(a), obj(b)]).go();
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(
        t.counters(b, counters::PLUS1),
        1,
        "{:#?}",
        prompts_since(&t, from)
    );
    assert_eq!(t.counters(a, counters::PLUS1), 0);
    let asked = asked_since(&t, from);
    let last_discard = asked
        .iter()
        .rposition(|(_, d)| matches!(d, Decision::ChooseEntities { prompt, .. } if prompt.contains("discard")))
        .expect("discards");
    let first_phase = asked
        .iter()
        .position(|(_, d)| matches!(d, Decision::YesNo { .. }))
        .expect("phase-out choices");
    assert!(last_discard < first_phase, "{:#?}", prompts_since(&t, from));
}

// --- Elemental Expressionist ------------------------------------------------------------------

#[test]
fn elemental_expressionist_twice_on_one_creature_makes_two_tokens() {
    cr!("113.2c", "603.2", "614.1a");
    ruling!(
        "Elemental Expressionist",
        "If the magecraft ability triggers and resolves more than once with the same creature as the target, that creature will have multiple instances of each of the two triggered abilities the magecraft ability grants it. Each instance of the second ability will trigger separately, and you'll create an Elemental token for each of them. (Multiple instances of the first granted ability are redundant.)"
    );
    supported("Elemental Expressionist");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Elemental Expressionist");
    let bears = t.battlefield(P0, "Grizzly Bears");
    for _ in 0..2 {
        t.answer_targets(P0, &[obj(bears)]);
        cast_new(&mut t, P0, "Opt", &[]);
        t.resolve_all();
    }
    kill(&mut t, bears);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"), "{}", t.dump_log());
    assert_eq!(
        tokens_with(&t, P0, "Elemental").len(),
        2,
        "{}",
        t.dump_log()
    );
}
