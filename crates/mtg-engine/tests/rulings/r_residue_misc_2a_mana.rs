//! Item `rulings-residue-misc-2`, pass A — mana with spending restrictions and unspent
//! mana (CR 106.6, 106.4, 500.5, 605): the open rulings of batches P141, P142 (Omnath,
//! Locus of All) and P148 (restricted mana).

use crate::r_residue_misc_2a_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn pool(t: &TestGame, p: PlayerId) -> Vec<ManaType> {
    t.g.player(p).mana_pool.mana.iter().map(|m| m.ty).collect()
}

fn pool_total(t: &TestGame, p: PlayerId) -> usize {
    t.g.player(p).mana_pool.total()
}

/// Queues `p`'s choice of a color of mana ("W", "U", "B", "R" or "G").
fn color(t: &mut TestGame, p: PlayerId, c: &str) {
    let i = ["W", "U", "B", "R", "G"]
        .iter()
        .position(|x| *x == c)
        .unwrap();
    t.answer(p, DecisionKind::Option, Answer::Index(i));
}

/// The index of a creature type in the "Choose a creature type" options.
fn creature_type(name: &str) -> usize {
    mtg_engine::types::subtype_lists()
        .creature
        .iter()
        .position(|t| t == name)
        .expect("a creature type")
}

// --- Omnath, Locus of All --------------------------------------------------------------------

#[test]
fn omnath_keeps_unspent_mana_as_black_until_it_leaves() {
    cr!("106.4", "500.5", "614.1a");
    ruling!(
        "Omnath, Locus of All",
        "As long as Omnath, Locus of All remains under your control, you'll retain unspent mana as steps and phases end, although that mana will become black. This means you can add mana and spend it during a future step, phase, or turn. Once Omnath leaves your control, you'll have until the end of the current step or phase to spend the mana before it is lost."
    );
    supported("Omnath, Locus of All");
    let mut t = TestGame::new(2);
    let omnath = t.battlefield(P0, "Omnath, Locus of All");
    t.set_step(P0, Step::Upkeep);
    let m = t.battlefield(P0, "Mountain");
    t.activate(P0, m, 0, &[]).unwrap();
    t.advance_to(P0, Step::Draw);
    assert_eq!(pool(&t, P0), vec![ManaType::B]);
    // Into the opponent's turn.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(pool(&t, P0), vec![ManaType::B]);
    // Omnath leaves: the mana is still there for the rest of this step, then lost.
    destroy(&mut t, omnath);
    assert_eq!(pool(&t, P0), vec![ManaType::B]);
    t.advance_to(P1, Step::Draw);
    assert!(pool(&t, P0).is_empty());
}

#[test]
fn omnath_keeps_the_restrictions_of_mana_that_becomes_black() {
    cr!("106.6", "500.5");
    ruling!(
        "Omnath, Locus of All",
        "If unspent mana you have has any restrictions or riders associated with it (for example, if it was produced by Omen Hawker), those restrictions or riders remain associated with that mana when it becomes black."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Omnath, Locus of All");
    t.set_step(P0, Step::Upkeep);
    let shop = t.battlefield(P0, "Mishra's Workshop");
    t.activate(P0, shop, 0, &[]).unwrap();
    t.advance_to(P0, Step::PrecombatMain);
    // Omnath's own trigger may add mana; look at the Workshop's three.
    let kept: Vec<_> =
        t.g.player(P0)
            .mana_pool
            .mana
            .iter()
            .filter(|m| m.restriction.is_some())
            .map(|m| m.ty)
            .collect();
    assert_eq!(kept, vec![ManaType::B; 3]);
    // Still only for artifact spells: not a black creature spell ({B}).
    t.resolve_all();
    let pawn = t.hand(P0, "Vampire Lacerator");
    assert!(t.cast(P0, pawn).try_go().is_err());
    let walker = t.hand(P0, "Bonesplitter");
    t.cast(P0, walker).go();
}

#[test]
fn omnath_s_last_ability_uses_the_stack() {
    cr!("605.1b", "605.5a");
    ruling!(
        "Omnath, Locus of All",
        "Omnath's last ability isn't a mana ability even though it can cause you to add mana. It uses the stack and can be responded to."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Omnath, Locus of All");
    t.library_top(P0, "Cryptic Command");
    t.set_step(P0, Step::Draw);
    t.advance_to(P0, Step::PrecombatMain);
    t.settle();
    assert_eq!(t.stack_len(), 1, "on the stack, not resolved immediately");
    assert!(pool(&t, P0).is_empty());
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.resolve_all();
    assert!(t.in_hand(P0, "Cryptic Command"));
}

// --- Rosheen, Roaring Prophet -------------------------------------------------------------------

/// P0's Rosheen, Roaring Prophet reveals two {X} cards: four colorless mana for costs that
/// contain {X}.
fn rosheen() -> TestGame {
    supported("Rosheen, Roaring Prophet");
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Rosheen, Roaring Prophet");
    let a = t.hand(P0, "Fireball");
    let b = t.hand(P0, "Blaze");
    t.answer_choose(P0, &[obj(a), obj(b)]);
    t.activate(P0, r, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(pool_total(&t, P0), 4);
    t
}

#[test]
fn rosheen_roaring_prophet_mana_pays_any_part_of_x_costs_and_may_be_split() {
    cr!("106.6", "107.3", "601.2f", "602.2b");
    ruling!(
        "Rosheen, Roaring Prophet",
        "You can spend mana generated with Rosheen's last ability on any part of a cost that contains {X}. You're not limited to spending it only on the {X} part."
    );
    ruling!(
        "Rosheen, Roaring Prophet",
        "You don't have to spend all of the mana generated with Rosheen's last ability on the same cost."
    );
    ruling!(
        "Rosheen, Roaring Prophet",
        "You can use the mana generated with Rosheen's last ability in any cost you pay that includes {X}, including the mana costs of spells, activation costs"
    );
    ruling!(
        "Rosheen, Roaring Prophet",
        "A \"cost that contains {X}\" may be a spell's total cost, an activated ability's cost"
    );
    let mut t = rosheen();
    // Not a cost without {X}.
    let splitter = t.hand(P0, "Bonesplitter");
    assert!(t.cast(P0, splitter).try_go().is_err());
    // Stroke of Genius ({X}{2}{U}) with X = 0: the {2} from Rosheen's mana.
    t.lands(P0, "Island", 1);
    let stroke = t.hand(P0, "Stroke of Genius");
    t.cast(P0, stroke).x(0).target(pl(P0)).go();
    t.resolve_all();
    assert_eq!(pool_total(&t, P0), 2);
    // The rest pays an activated ability's {X}: Mirror Entity, X = 2.
    let entity = t.battlefield(P0, "Mirror Entity");
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.activate(P0, entity, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(pool_total(&t, P0), 0);
    assert_eq!(t.pt(entity), (2, 2));
}

// --- Chandra's Embercat ---------------------------------------------------------------------------

#[test]
fn chandra_s_embercat_mana_is_for_chandra_planeswalkers_not_chandra_named_cards() {
    cr!("106.6", "205.3j");
    ruling!(
        "Chandra's Embercat",
        "A Chandra planeswalker spell is a planeswalker spell with the subtype Chandra. Cards that use Chandra’s name in their title or her likeness in their illustration (such as Chandra’s Regulator or Chandra’s Outrage) aren’t Chandra planeswalker spells."
    );
    supported("Chandra's Embercat");
    // {2}{R}{R} each: one {R} must come from the Embercat.
    for (name, ok) in [
        ("Chandra, Torch of Defiance", true),
        ("Chandra's Outrage", false),
    ] {
        let mut t = TestGame::new(2);
        let cat = t.battlefield(P0, "Chandra's Embercat");
        t.activate(P0, cat, 0, &[]).unwrap();
        t.lands(P0, "Mountain", 1);
        t.lands(P0, "Wastes", 2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let card = t.hand(P0, name);
        t.answer_targets(P0, &[obj(bears)]);
        assert_eq!(t.cast(P0, card).try_go().is_ok(), ok, "{name}");
    }
}

// --- Interplanar Beacon ------------------------------------------------------------------------------

#[test]
fn interplanar_beacon_mana_may_go_to_one_or_two_planeswalker_spells() {
    cr!("106.6");
    ruling!(
        "Interplanar Beacon",
        "Both mana produced by Interplanar Beacon’s last ability may be spent on the same planeswalker spell, or each may be spent on different planeswalker spells."
    );
    supported("Interplanar Beacon");
    supported("Ajani Vengeant");
    // Both on one: Ajani Vengeant ({2}{R}{W}) with R and W from the Beacon.
    let mut t = TestGame::new(2);
    let beacon = t.battlefield(P0, "Interplanar Beacon");
    t.lands(P0, "Wastes", 1);
    color(&mut t, P0, "R");
    color(&mut t, P0, "W");
    t.activate(P0, beacon, 1, &[]).unwrap();
    assert_eq!(pool_total(&t, P0), 2);
    t.lands(P0, "Wastes", 2);
    let ajani = t.hand(P0, "Ajani Vengeant");
    t.cast(P0, ajani).go();
    assert!(pool(&t, P0).is_empty());
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Ajani Vengeant").len(), 1);
    // Each on a different planeswalker spell.
    let mut t = TestGame::new(2);
    let beacon = t.battlefield(P0, "Interplanar Beacon");
    t.lands(P0, "Wastes", 1);
    color(&mut t, P0, "U");
    color(&mut t, P0, "R");
    t.activate(P0, beacon, 1, &[]).unwrap();
    t.lands(P0, "Island", 1);
    t.lands(P0, "Mountain", 4);
    let jace = t.hand(P0, "Jace Beleren");
    t.cast(P0, jace).go();
    t.resolve_all();
    assert!(pool_total(&t, P0) <= 1, "{}", t.dump_log());
    let chandra = t.hand(P0, "Chandra, Torch of Defiance");
    t.cast(P0, chandra).go();
    t.resolve_all();
    assert!(pool(&t, P0).is_empty());
    // Not on a nonplaneswalker spell.
    let mut t = TestGame::new(2);
    let beacon = t.battlefield(P0, "Interplanar Beacon");
    t.lands(P0, "Wastes", 1);
    color(&mut t, P0, "G");
    color(&mut t, P0, "U");
    t.activate(P0, beacon, 1, &[]).unwrap();
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(t.cast(P0, bears).try_go().is_err());
}

#[test]
fn interplanar_beacon_s_life_trigger_resolves_first_even_if_the_spell_is_countered() {
    cr!("603.3", "601.2i", "701.6a");
    ruling!(
        "Interplanar Beacon",
        "Interplanar Beacon’s first ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Interplanar Beacon");
    t.lands(P0, "Island", 3);
    let jace = t.hand(P0, "Jace Beleren");
    t.cast(P0, jace).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    // P1 counters the planeswalker spell above the trigger... no: the trigger is on top.
    t.lands(P1, "Island", 2);
    let cs = t.hand(P1, "Counterspell");
    t.cast(P1, cs).target(obj(jace)).go();
    t.resolve(); // Counterspell
    assert!(t.in_graveyard(P0, "Jace Beleren"));
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    // Without a counter, the life is gained while the spell is still on the stack.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Interplanar Beacon");
    t.lands(P0, "Island", 3);
    let jace = t.hand(P0, "Jace Beleren");
    t.cast(P0, jace).go();
    t.resolve();
    assert_eq!(t.life(P0), 21);
    assert_eq!(t.stack_len(), 1);
}

// --- Secluded Courtyard ----------------------------------------------------------------------------------

#[test]
fn secluded_courtyard_without_a_chosen_type_mana_pays_nothing() {
    cr!("106.6", "607.2d");
    ruling!(
        "Secluded Courtyard",
        "If Secluded Courtyard is somehow on the battlefield without a chosen type, mana from its second mana ability can't be spent on anything."
    );
    supported("Secluded Courtyard");
    // With Bear chosen (entered for real), its G pays for Grizzly Bears; without a choice
    // (put onto the battlefield directly), it doesn't.
    for chosen in [true, false] {
        let mut t = TestGame::new(2);
        let yard = if chosen {
            t.answer(
                P0,
                DecisionKind::Option,
                Answer::Index(creature_type("Bear")),
            );
            let y = t.enter(P0, "Secluded Courtyard");
            t.g.current(y)
        } else {
            t.battlefield(P0, "Secluded Courtyard")
        };
        t.clear_answers();
        color(&mut t, P0, "G");
        t.activate(P0, yard, 1, &[]).unwrap();
        assert_eq!(pool(&t, P0), vec![ManaType::G]);
        t.lands(P0, "Wastes", 1);
        let bears = t.hand(P0, "Grizzly Bears");
        assert_eq!(
            t.cast(P0, bears).try_go().is_ok(),
            chosen,
            "chosen {chosen}"
        );
    }
}

// --- Séance Board -----------------------------------------------------------------------------------------

#[test]
fn seance_board_doesnt_trigger_for_a_creature_dying_during_the_end_step() {
    cr!("603.4", "513.1");
    ruling!(
        "Séance Board",
        "If no creatures have died in a turn by the time that turn's end step begins, Séance Board's first ability won't trigger at all. Causing a creature to die during the end step won't cause the ability to trigger."
    );
    // The data also attaches it to Secret Arcade // Dusty Parlor, which it doesn't
    // describe (docs/RULING_DISCREPANCIES.md); it takes part here as a Room on the
    // battlefield that makes Séance Board an enchantment, changing nothing.
    ruling!(
        "Secret Arcade // Dusty Parlor",
        "If no creatures have died in a turn by the time that turn's end step begins, Séance Board's first ability won't trigger at all. Causing a creature to die during the end step won't cause the ability to trigger."
    );
    supported("Séance Board");
    supported("Secret Arcade // Dusty Parlor");
    let mut t = TestGame::new(2);
    let arcade = t.battlefield(P0, "Secret Arcade // Dusty Parlor");
    assert!(mtg_engine::rooms::unlock(&mut t.g, arcade, 0, P0));
    let board = t.battlefield(P0, "Séance Board");
    t.g.recompute();
    assert!(t
        .obj_now(board)
        .is(mtg_engine::types::CardType::Enchantment));
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    destroy(&mut t, bears);
    t.resolve_all();
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.counters(board, "soul"), 0);
    // Baseline: a creature that died before the end step began.
    let bears = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, bears);
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert_eq!(t.counters(board, "soul"), 1);
}

// --- Soldevi Machinist ------------------------------------------------------------------------------------

#[test]
fn soldevi_machinist_mana_is_only_for_abilities_of_artifacts() {
    cr!("106.6", "602.1");
    ruling!(
        "Soldevi Machinist",
        "Mana from the ability can only be spent on activation costs for abilities from artifacts. It can't be spent on artifact spells, or on costs resulting from triggered abilities."
    );
    supported("Soldevi Machinist");
    supported("Mana Vault");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Soldevi Machinist");
    t.activate(P0, m, 0, &[]).unwrap();
    assert_eq!(pool_total(&t, P0), 2);
    // Not an artifact spell.
    let splitter = t.hand(P0, "Bonesplitter");
    assert!(t.cast(P0, splitter).try_go().is_err());
    // An artifact's activated ability: Mind Stone's {1}.
    let stone = t.battlefield(P0, "Mind Stone");
    t.activate(P0, stone, 1, &[])
        .expect("Mind Stone's draw ability");
    t.resolve_all();
    assert_eq!(pool_total(&t, P0), 1);
    // Not a cost a triggered ability asks for: Mana Vault's upkeep {4} (with the other
    // three from lands, the Machinist's mana can't make up the fourth).
    let mut t = TestGame::new(2);
    let vault = t.battlefield(P0, "Mana Vault");
    t.g.tap(vault);
    let m = t.battlefield(P0, "Soldevi Machinist");
    t.lands(P0, "Wastes", 3);
    t.set_step(P0, Step::Untap);
    t.advance_to(P0, Step::Upkeep);
    t.activate(P0, m, 0, &[]).unwrap();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.obj_now(vault).tapped, "{}", t.dump_log());
}

// --- Creeping Peeper --------------------------------------------------------------------------------------

#[test]
fn creeping_peeper_mana_turns_a_morph_face_up() {
    cr!("106.6", "702.37e", "116.2b");
    ruling!(
        "Creeping Peeper",
        "Mana produced by Creeping Peeper's activated ability can be used to pay morph or disguise costs when turning a face-down permanent face up"
    );
    supported("Creeping Peeper");
    supported("Echo Tracer");
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 3);
    let tracer = t.hand(P0, "Echo Tracer");
    t.cast(P0, tracer)
        .method(mtg_engine::object::CastMethod::FaceDown(
            mtg_engine::keywords::KeywordKind::Morph,
        ))
        .go();
    t.resolve_all();
    let tracer = t.g.current(tracer);
    assert!(t.g.obj(tracer).face_down);
    // Echo Tracer's morph cost {2}{U}: the {U} from the Peeper.
    let peeper = t.battlefield(P0, "Creeping Peeper");
    t.activate(P0, peeper, 0, &[]).unwrap();
    t.lands(P0, "Wastes", 2);
    let up =
        mtg_engine::decision::Action::Special(mtg_engine::decision::SpecialAction::TurnFaceUp {
            obj: tracer,
        });
    t.g.turn.priority = Some(P0);
    t.g.perform_action(P0, up).expect("turned face up");
    assert!(!t.g.obj(t.g.current(tracer)).face_down);
}

// --- Cultivator Drone ---------------------------------------------------------------------------------------

#[test]
fn cultivator_drones_pay_a_cost_a_triggered_ability_asks_for() {
    cr!("106.6", "603.5");
    ruling!(
        "Cultivator Drone",
        "Some triggered abilities include a cost as part of their resolution. For example, Bearer of Silence’s ability says, in part, “When you cast Bearer of Silence, you may pay {1}{C}.” You may use mana produced by Cultivator Drone to help pay that cost. In fact, you could tap two Cultivator Drones to pay the entire cost."
    );
    supported("Cultivator Drone");
    supported("Bearer of Silence");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 2);
    let bearer = t.hand(P0, "Bearer of Silence");
    t.cast(P0, bearer).go();
    t.settle();
    for _ in 0..2 {
        let d = t.battlefield(P0, "Cultivator Drone");
        t.activate(P0, d, 0, &[]).unwrap();
    }
    assert_eq!(pool_total(&t, P0), 2);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[pl(P1)]);
    t.resolve_all();
    assert_eq!(pool_total(&t, P0), 0, "{}", t.dump_log());
    assert!(!t.on_battlefield(bears));
}

// --- Eclipsed Realms --------------------------------------------------------------------------------------------

#[test]
fn eclipsed_realms_mana_activates_abilities_of_cards_of_the_type_in_hand() {
    cr!("106.6", "702.29a");
    ruling!(
        "Eclipsed Realms",
        "Sources of the chosen creature type include any objects with the chosen subtype. For example, if you chose Kithkin, you could spend the mana to activate the ability of a Kithkin permanent you control or a Kithkin card in your hand or graveyard."
    );
    supported("Eclipsed Realms");
    supported("Sandbar Merfolk");
    // Merfolk chosen (the seventh option); Sandbar Merfolk's cycling {2} from hand.
    for (card, ok) in [("Sandbar Merfolk", true), ("Lava Serpent", false)] {
        let mut t = TestGame::new(2);
        t.answer(P0, DecisionKind::Option, Answer::Index(6));
        let realms = t.enter(P0, "Eclipsed Realms");
        let realms = t.g.current(realms);
        t.clear_answers();
        color(&mut t, P0, "U");
        t.activate(P0, realms, 1, &[]).unwrap();
        t.lands(P0, "Wastes", 1);
        let c = t.hand(P0, card);
        assert_eq!(
            activate_containing(&mut t, P0, c, "Cycling").is_ok(),
            ok,
            "{card}"
        );
    }
}

// --- Mishra's Workshop --------------------------------------------------------------------------------------------

#[test]
fn mishra_s_workshop_mana_pays_artifact_spells_not_echo() {
    cr!("106.6", "702.30a");
    ruling!(
        "Mishra's Workshop",
        "The mana can't be used to pay Echo costs."
    );
    ruling!(
        "Mishra's Workshop",
        "You can spend the mana on costs on the spell's text."
    );
    supported("Mishra's Workshop");
    supported("Ticking Gnomes");
    supported("Skyclave Relic");
    // A cost on the spell's text: Skyclave Relic's kicker {3}, with three Workshop mana
    // and three Wastes for the six.
    let mut t = TestGame::new(2);
    let shop = t.battlefield(P0, "Mishra's Workshop");
    t.lands(P0, "Wastes", 3);
    t.activate(P0, shop, 0, &[]).unwrap();
    let relic = t.hand(P0, "Skyclave Relic");
    t.cast(P0, relic).kicked(true).go();
    t.resolve_all();
    assert_eq!(
        t.named_on_battlefield("Skyclave Relic").len(),
        3,
        "{}",
        t.dump_log()
    );
    // Echo: Ticking Gnomes's echo {3} at P0's next upkeep, with only Workshop mana.
    for workshop in [true, false] {
        let mut t = TestGame::new(2);
        let gnomes = t.enter(P0, "Ticking Gnomes");
        let shop = t.battlefield(P0, "Mishra's Workshop");
        if !workshop {
            t.lands(P0, "Wastes", 3);
        }
        t.advance_to(P0, Step::Upkeep);
        t.settle();
        if workshop {
            t.activate(P0, shop, 0, &[]).unwrap();
            assert_eq!(pool_total(&t, P0), 3);
        }
        t.answer_yes(P0, true);
        t.resolve_all();
        assert_eq!(t.on_battlefield(gnomes), !workshop, "{}", t.dump_log());
    }
}

// --- Meeting of the Five ------------------------------------------------------------------------------------------

#[test]
fn meeting_of_the_five_mana_pays_generic_parts_of_three_color_spells() {
    cr!("106.6", "105.2");
    ruling!(
        "Meeting of the Five",
        "The mana created by Meeting of the Five can be spent on any portion of the cost for a spell with exactly three colors, including any generic cost component or additional costs it has."
    );
    supported("Meeting of the Five");
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Meeting of the Five", &[]);
    t.resolve_all();
    assert_eq!(pool_total(&t, P0), 10);
    // Not a two-color spell, nor a colorless one.
    let charm = t.hand(P0, "Bonesplitter");
    assert!(t.cast(P0, charm).try_go().is_err());
    // Siege Rhino ({1}{W}{B}{G}): the {1} too.
    let rhino = t.hand(P0, "Siege Rhino");
    t.cast(P0, rhino).go();
    t.resolve_all();
    assert_eq!(pool_total(&t, P0), 6);
    assert_eq!(t.named_on_battlefield("Siege Rhino").len(), 1);
}

// --- Tin Street Gossip --------------------------------------------------------------------------------------------

#[test]
fn tin_street_gossip_mana_cant_cast_a_spell_that_manifests() {
    cr!("106.6", "701.40a");
    ruling!(
        "Tin Street Gossip",
        "The mana produced by Tin Street Gossip can't be used to cast a spell that instructs you to cloak or manifest cards."
    );
    supported("Tin Street Gossip");
    supported("Soul Summons");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Tin Street Gossip");
    t.activate(P0, g, 0, &[]).unwrap();
    t.lands(P0, "Plains", 1);
    let summons = t.hand(P0, "Soul Summons");
    assert!(t.cast(P0, summons).try_go().is_err());
    // A face-down spell can: Echo Tracer cast face down ({3}).
    t.lands(P0, "Wastes", 1);
    let tracer = t.hand(P0, "Echo Tracer");
    t.cast(P0, tracer)
        .method(mtg_engine::object::CastMethod::FaceDown(
            mtg_engine::keywords::KeywordKind::Morph,
        ))
        .go();
}

// --- Cavern of Souls -----------------------------------------------------------------------------------------------

#[test]
fn cavern_of_souls_mana_on_a_kicker_cost_makes_the_spell_uncounterable() {
    cr!("106.6", "601.2f", "101.2");
    ruling!(
        "Cavern of Souls",
        "The spell can't be countered if the mana produced by Cavern of Souls is spent to cover any cost of the spell, even an additional cost such as a kicker cost."
    );
    supported("Cavern of Souls");
    supported("Kavu Titan");
    let mut t = TestGame::new(2);
    t.answer(
        P0,
        DecisionKind::Option,
        Answer::Index(creature_type("Kavu")),
    );
    let cavern = t.enter(P0, "Cavern of Souls");
    let cavern = t.g.current(cavern);
    t.clear_answers();
    color(&mut t, P0, "G");
    t.activate(P0, cavern, 1, &[]).unwrap();
    // Kavu Titan ({1}{G}, kicker {2}{G}): Forest and three Wastes for the rest.
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 3);
    let kavu = t.hand(P0, "Kavu Titan");
    t.cast(P0, kavu).kicked(true).go();
    t.lands(P1, "Island", 2);
    let cs = t.hand(P1, "Counterspell");
    t.cast(P1, cs).target(obj(kavu)).go();
    t.resolve_all();
    assert_eq!(
        t.named_on_battlefield("Kavu Titan").len(),
        1,
        "{}",
        t.dump_log()
    );
    assert_eq!(
        t.counters(t.named_on_battlefield("Kavu Titan")[0], "+1/+1"),
        3
    );
}

#[test]
fn cavern_of_souls_offers_creature_types_only() {
    cr!("205.3m");
    ruling!(
        "Cavern of Souls",
        "You must choose an existing creature type, such as Human or Warrior. Card types such as artifact can't be chosen."
    );
    let mut t = TestGame::new(2);
    let from = t.asked().len();
    t.enter(P0, "Cavern of Souls");
    let options: Vec<Vec<String>> = asked_since(&t, from)
        .into_iter()
        .filter_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseOption { options, .. } => Some(options),
            _ => None,
        })
        .collect();
    assert_eq!(options.len(), 1);
    let o = &options[0];
    assert!(o.iter().any(|s| s == "Human") && o.iter().any(|s| s == "Warrior"));
    assert!(o
        .iter()
        .all(|s| s != "Artifact" && s != "Creature" && s != "Forest"));
}
