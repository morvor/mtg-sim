//! Rulings batch P100 — "mana value matters": what a mana value is (CR 202.3): the total
//! of the mana cost's symbols, X as chosen on the stack and 0 elsewhere (202.3e), 0 for
//! tokens and objects with no mana cost (202.3a), that of the copied object for a copy
//! (707.2), unchanged by alternative or additional costs and cost reductions (118.8d,
//! 118.9c), a half's for a split spell (202.3d), 0 for a face-down spell (708.2).

use crate::r_p100_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, FaceState, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const FACE_DOWN: CastMethod = CastMethod::FaceDown(KeywordKind::Morph);
const AWAKEN: CastMethod = CastMethod::Keyword(KeywordKind::Awaken);

/// Steel Hellkite under P0 deals combat damage to P1 (whose permanents `setup` set up,
/// returning what the test needs); `after_combat` then changes the battlefield, and P0
/// activates the Hellkite's last ability with X = `x`; everything resolves.
fn hellkite_hits_then_x<S>(
    x: i64,
    setup: impl FnOnce(&mut TestGame) -> S,
    after_combat: impl FnOnce(&mut TestGame, &S),
) -> (TestGame, S) {
    let mut t = TestGame::new(2);
    let hellkite = t.battlefield(P0, "Steel Hellkite");
    let s = setup(&mut t);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(hellkite, pl(P1))], &[]);
    assert_eq!(t.life(P1), 15);
    after_combat(&mut t, &s);
    mana(&mut t, P0, &format!("{x}"));
    answer_x(&mut t, P0, x);
    act(&mut t, P0, hellkite, "Destroy each nonland permanent", &[]).expect("Hellkite");
    t.resolve_all();
    (t, s)
}

#[test]
fn steel_hellkite_uses_printed_mana_values_tokens_as_0_and_copies_as_the_copied() {
    cr!("202.3", "202.3a", "202.3e", "707.2", "118.8d");
    ruling!(
        "Steel Hellkite",
        "If Steel Hellkite's third ability is activated with X equal to 0, it will destroy each nonland permanent with mana value 0 the appropriate players control."
    );
    ruling!(
        "Steel Hellkite",
        "If a nonland permanent has no mana symbols in its upper right corner (because it's a token that's not copying something else, for example), its mana value is 0."
    );
    ruling!(
        "Steel Hellkite",
        "If a permanent is copying something else, its mana value is the mana value of whatever it's copying."
    );
    ruling!(
        "Steel Hellkite",
        "The mana value of a permanent is determined solely by the mana symbols printed in its upper right corner, unless it's copying something else (see below). The mana value is the total amount of mana in that cost, regardless of color. For example, a card with mana cost {3}{U}{U} has mana value 5."
    );
    ruling!(
        "Steel Hellkite",
        "In all cases, ignore any alternative costs or additional costs (such as kicker) paid when the permanent was cast."
    );
    ruling!(
        "Steel Hellkite",
        "It doesn't matter who controlled those permanents at the time Steel Hellkite dealt combat damage, or if those permanents were even on the battlefield at that time."
    );
    supported("Steel Hellkite");
    // X = 0: a Thopter token, a Chalice of the Void with two charge counters ({X}{X}, 0)
    // and a Memnite that entered after combat are destroyed; Grizzly Bears (2) isn't;
    // P0's own Memnite isn't (P0 wasn't dealt damage).
    let (t, (token, bears, own)) = hellkite_hits_then_x(
        0,
        |t| {
            let token = create_token(t, P1, "Thopter");
            let chalice = with_counters(t, P1, "Chalice of the Void", 0);
            t.g.objects[chalice.0 as usize].counters.insert("charge".into(), 2);
            let bears = t.battlefield(P1, "Grizzly Bears");
            let own = t.battlefield(P0, "Memnite");
            (token, bears, own)
        },
        |t, _| {
            t.battlefield(P1, "Memnite");
        },
    );
    assert!(!t.on_battlefield(token));
    assert!(t.in_graveyard(P1, "Chalice of the Void"));
    assert!(t.in_graveyard(P1, "Memnite"));
    assert!(t.on_battlefield(bears));
    assert!(t.on_battlefield(own));
    // X = 2: Grizzly Bears, a Clone copying Grizzly Bears (2), a Gnarlid Colony cast
    // kicked for {1}{G} + {2}{G} (still 2) and P0's Grizzly Bears P1 gained control of
    // after combat are destroyed; Hill Giant (4) isn't.
    let (t, (clone, colony, giant, stolen)) = hellkite_hits_then_x(
        2,
        |t| {
            let bears = t.battlefield(P1, "Grizzly Bears");
            t.answer_yes(P1, true);
            t.answer_choose(P1, &[obj(bears)]);
            let clone = t.enter(P1, "Clone");
            t.settle();
            assert_eq!(t.obj_now(clone).chars.name, "Grizzly Bears");
            let giant = t.battlefield(P1, "Hill Giant");
            let stolen = t.battlefield(P0, "Grizzly Bears");
            // P1 casts Gnarlid Colony kicked in their own turn first.
            t.set_step(P1, Step::PrecombatMain);
            t.lands(P1, "Forest", 5);
            let c = t.hand(P1, "Gnarlid Colony");
            let colony = t.cast(P1, c).kicked(true).go();
            t.resolve_all();
            assert_eq!(t.pt(colony), (4, 4), "kicked");
            assert_eq!(mana_value(t, colony), 2);
            (clone, colony, giant, stolen)
        },
        |t, (_, _, _, stolen)| {
            give_control(t, *stolen, P1);
        },
    );
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 0, "{:?}", battlefield_names(&t));
    assert!(!t.on_battlefield(clone));
    assert!(!t.on_battlefield(colony));
    assert!(!t.on_battlefield(stolen));
    assert!(t.on_battlefield(giant));
    // X = 5: Air Elemental ({3}{U}{U}) is destroyed, Craw Wurm (6) isn't.
    let (t, (elemental, wurm)) = hellkite_hits_then_x(
        5,
        |t| (t.battlefield(P1, "Air Elemental"), t.battlefield(P1, "Craw Wurm")),
        |_, _| {},
    );
    assert!(!t.on_battlefield(elemental));
    assert!(t.on_battlefield(wurm));
}

#[test]
fn culling_scales_sees_tokens_as_0_copies_as_the_copied_and_can_destroy_itself() {
    cr!("202.3a", "707.2", "115.1");
    ruling!(
        "Culling Scales",
        "Most tokens have a mana value of 0. A token that's a copy of another permanent or card has a mana value equal to that permanent or card's mana value."
    );
    ruling!("Culling Scales", "If the lowest mana value is 3, Culling Scales can destroy itself.");
    supported("Culling Scales");
    // A Thopter token (0) is the only legal target beside Grizzly Bears (2) and the Scales.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Culling Scales");
    t.battlefield(P0, "Grizzly Bears");
    let token = create_token(&mut t, P1, "Thopter");
    let from = t.asked().len();
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(target_candidate_names(&t, P0, from), ["Thopter"]);
    assert!(!t.on_battlefield(token));
    // A token copy of Hill Giant (4) with the Scales (3): the Scales has the lowest mana
    // value and is the only target; it destroys itself.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Culling Scales");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(false));
    cast_resolve(&mut t, P0, "Rite of Replication", "2UU", &[obj(giant)]);
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 2);
    destroy(&mut t, giant);
    let copy = tokens(&t, P0)[0];
    assert_eq!(mana_value(&t, copy), 4);
    let from = t.asked().len();
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(target_candidate_names(&t, P0, from), ["Culling Scales"]);
    assert!(t.in_graveyard(P0, "Culling Scales"));
    assert!(t.on_battlefield(copy));
}

#[test]
fn temporary_lockdown_exiles_tokens_for_good() {
    cr!("202.3a", "704.5d", "610.3");
    ruling!(
        "Temporary Lockdown",
        "If a token is exiled, it ceases to exist. It won’t be returned to the battlefield. The mana value of a token that isn’t a copy of another permanent is always 0."
    );
    supported("Temporary Lockdown");
    let mut t = TestGame::new(2);
    let token = create_token(&mut t, P1, "Thopter");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let lockdown = t.enter(P0, "Temporary Lockdown");
    t.resolve_all();
    assert!(!t.g.is_live(token) || t.zone(token) != Zone::Battlefield);
    assert!(t.g.find_in_zone(Zone::Exile, "Thopter").is_empty(), "ceased to exist");
    assert_eq!(t.zone(bears), Zone::Exile);
    assert!(t.on_battlefield(giant));
    destroy(&mut t, lockdown);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert!(tokens(&t, P1).is_empty());
}

#[test]
fn azors_gateway_counts_a_land_card_as_mana_value_0_and_sanctum_has_a_mana_ability() {
    cr!("202.3a", "605.1a", "605.3b", "406.3");
    ruling!(
        "Azor's Gateway // Sanctum of the Sun",
        "If an object has no mana cost, its mana value is 0. A mana value of 0 can still help unlock Azor's Gateway."
    );
    ruling!(
        "Azor's Gateway // Sanctum of the Sun",
        "The ability of Sanctum of the Sun is a mana ability. It doesn't use the stack and can't be responded to."
    );
    ruling!(
        "Azor's Gateway // Sanctum of the Sun",
        "The cards you exile from your hand are exiled face up."
    );
    supported("Azor's Gateway // Sanctum of the Sun");
    let mut t = TestGame::new(2);
    let gate = t.battlefield(P0, "Azor's Gateway // Sanctum of the Sun");
    // Forest (0), Shock (1), Grizzly Bears (2), Hill Giant (4), Air Elemental (5).
    let cards: Vec<ObjectId> = ["Forest", "Shock", "Grizzly Bears", "Hill Giant", "Air Elemental"]
        .iter()
        .map(|c| t.hand(P0, c))
        .collect();
    for (i, c) in cards.iter().enumerate() {
        t.g.objects[gate.0 as usize].tapped = false;
        mana(&mut t, P0, "1");
        t.answer_choose(P0, &[obj(*c)]);
        t.activate(P0, gate, 0, &[]).expect("Gateway");
        t.resolve_all();
        assert_eq!(t.zone(*c), Zone::Exile, "card {i}");
        assert!(!t.obj_now(*c).face_down, "card {i}: exiled face up");
        if i < 4 {
            assert_eq!(t.obj_now(gate).face, FaceState::Front, "after {} cards", i + 1);
        }
    }
    assert_eq!(t.life(P0), 25, "five different mana values, 0 among them");
    assert_eq!(t.obj_now(gate).face, FaceState::Back);
    assert_eq!(t.obj_now(gate).chars.name, "Sanctum of the Sun");
    assert!(!t.obj_now(gate).tapped);
    // Sanctum's ability adds 25 mana without using the stack.
    let from = t.asked().len();
    act(&mut t, P0, gate, "Add X mana", &[]).expect("Sanctum");
    assert_eq!(t.stack_len(), 0);
    assert_eq!(pool(&t, P0), 25, "{:?}", asked_of(&t, P0, from));
}

#[test]
fn soul_search_exiling_a_card_with_no_mana_cost_makes_a_spirit() {
    cr!("202.3a");
    ruling!("Soul Search", "If the exiled card doesn't have a mana cost, its mana value is 0.");
    supported("Soul Search");
    let mut t = TestGame::new(2);
    let vision = t.hand(P1, "Ancestral Vision");
    t.hand(P1, "Forest");
    t.answer_choose(P0, &[obj(vision)]);
    cast_resolve(&mut t, P0, "Soul Search", "WB", &[pl(P1)]);
    assert_eq!(t.zone(vision), Zone::Exile);
    assert_eq!(tokens(&t, P0).len(), 1);
}

#[test]
fn eidolon_sees_x_as_chosen_and_a_face_down_spell_as_0() {
    cr!("202.3e", "708.2", "702.37a");
    ruling!(
        "Eidolon of the Great Revel",
        "The mana value of a spell with {X} in its mana cost includes the value chosen for X."
    );
    ruling!(
        "Eidolon of the Great Revel",
        "The amount of mana spent to cast a spell doesn't change its mana value. For example, a card cast with its morph ability has a mana value of 0 even though {3} was paid."
    );
    supported("Eidolon of the Great Revel");
    for (x, life) in [(2, 18), (3, 20)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Eidolon of the Great Revel");
        t.set_step(P1, Step::PrecombatMain);
        let blaze = cast_x(&mut t, P1, "Blaze", x, &[pl(P0)]);
        assert_eq!(mana_value(&t, blaze), x as u32 + 1);
        t.resolve_all();
        assert_eq!(t.life(P1), life, "X = {x}");
    }
    // Scornful Egotist cast face down for {3}: mana value 0, Eidolon deals 2 damage.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Eidolon of the Great Revel");
    t.set_step(P1, Step::PrecombatMain);
    let egotist = t.hand(P1, "Scornful Egotist");
    mana(&mut t, P1, "3");
    let spell = t.cast(P1, egotist).method(FACE_DOWN).go();
    assert_eq!(mana_value(&t, spell), 0);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert!(t.obj_now(spell).face_down && t.on_battlefield(spell));
}

#[test]
fn chalice_with_no_counters_counters_face_down_spells() {
    cr!("202.3a", "708.2", "702.37a");
    ruling!(
        "Chalice of the Void",
        "If there are zero charge counters on Chalice of the Void, it counters each spell with a mana value of 0. This includes face-down creature spells cast with morph's alternative cost."
    );
    supported("Chalice of the Void");
    let mut t = TestGame::new(2);
    let chalice = t.battlefield(P1, "Chalice of the Void");
    assert_eq!(t.counters(chalice, "charge"), 0);
    let egotist = t.hand(P0, "Scornful Egotist");
    mana(&mut t, P0, "3");
    let spell = t.cast(P0, egotist).method(FACE_DOWN).go();
    t.resolve_all();
    assert!(!t.on_battlefield(spell));
    assert!(t.in_graveyard(P0, "Scornful Egotist"), "countered: {}", t.dump_log());
    // A Memnite (0) is countered too; Grizzly Bears (2) resolves.
    let memnite = t.hand(P0, "Memnite");
    t.cast(P0, memnite).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Memnite"));
    let bears = t.hand(P0, "Grizzly Bears");
    mana(&mut t, P0, "1G");
    t.cast(P0, bears).go();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
}

#[test]
fn a_spells_mana_value_ignores_an_awaken_cost() {
    cr!("118.9c", "202.3");
    ruling!(
        "Kozilek, the Great Distortion",
        "The mana value of a spell doesn't change, even if it's been cast using an alternative cost (such as an awaken cost). For example, the mana value of a Sheer Drop (a spell with mana cost {2}{W}) that's been cast for its awaken cost of {5}{W} is 3."
    );
    supported("Kozilek, the Great Distortion");
    supported("Sheer Drop");
    let mut t = TestGame::new(2);
    let kozilek = t.battlefield(P0, "Kozilek, the Great Distortion");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.objects[bears.0 as usize].tapped = true;
    let ogre = t.hand(P0, "Gray Ogre");
    let giant = t.hand(P0, "Hill Giant");
    t.set_step(P1, Step::PrecombatMain);
    let land = t.battlefield(P1, "Plains");
    let drop = t.hand(P1, "Sheer Drop");
    mana(&mut t, P1, "5W");
    let spell = t.cast(P1, drop).method(AWAKEN).targets(&[obj(bears), obj(land)]).go();
    assert_eq!(pool(&t, P1), 0, "{{5}}{{W}} paid");
    assert_eq!(mana_value(&t, spell), 3);
    // X = 3: Gray Ogre ({2}{R}) may be discarded to counter it, not Hill Giant (4).
    let from = t.asked().len();
    answer_x(&mut t, P0, 3);
    t.answer_choose(P0, &[obj(ogre)]);
    act(&mut t, P0, kozilek, "Discard a card", &[obj(spell)]).expect("Kozilek");
    assert!(was_offered(&t, P0, from, ogre));
    assert!(!was_offered(&t, P0, from, giant));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Sheer Drop"), "countered");
    assert!(t.on_battlefield(bears));
}

#[test]
fn gaddock_teeg_lets_a_cheap_half_of_a_split_card_be_cast() {
    cr!("202.3d", "709.3", "601.3", "601.3a");
    ruling!(
        "Gaddock Teeg",
        "If one half of a split card has a mana value of 3 or less and doesn't have an {X} in its mana cost, Gaddock Teeg lets you cast that half."
    );
    supported("Gaddock Teeg");
    supported("Supply // Demand");
    supported("Hit // Run");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Gaddock Teeg");
    mana(&mut t, P0, "3WWUUGGRR");
    // Supply ({X}{G}{W}) has X in its cost: not castable; Demand ({1}{W}{U}, 3) is.
    let sd = t.hand(P0, "Supply // Demand");
    assert!(!can_cast(&mut t, P0, sd, CastMethod::Half(0)));
    assert!(can_cast(&mut t, P0, sd, CastMethod::Half(1)));
    // Hit ({1}{B}{R}, 3) is castable, Run ({3}{R}{G}, 5) isn't, nor the fused spell (8).
    mana(&mut t, P0, "3BBRRGG");
    let hr = t.hand(P0, "Hit // Run");
    assert!(can_cast(&mut t, P0, hr, CastMethod::Half(0)));
    assert!(!can_cast(&mut t, P0, hr, CastMethod::Half(1)));
    assert!(!can_cast(&mut t, P0, hr, CastMethod::Keyword(KeywordKind::Fuse)));
}

#[test]
fn counterbalance_compares_a_half_with_the_whole_split_card() {
    cr!("202.3d", "709.4b");
    ruling!(
        "Counterbalance",
        "If an opponent casts half of a split card (for example, Hit), it won't be countered if you reveal that same split card. The mana value of a split card is determined by the combined mana cost of its two halves, but the mana value of Hit is determined using only that half's mana cost."
    );
    supported("Counterbalance");
    for (top, countered) in [("Hit // Run", false), ("Gray Ogre", true)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Counterbalance");
        stack_library(&mut t, P0, &[top, "Forest"]);
        t.set_step(P1, Step::PrecombatMain);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let hr = t.hand(P1, "Hit // Run");
        mana(&mut t, P1, "1BR");
        t.answer_yes(P0, true);
        // Hit: "Target player sacrifices an artifact or creature. You gain life equal to
        // its mana value."
        let spell = t.cast(P1, hr).method(CastMethod::Half(0)).target(pl(P0)).go();
        assert_eq!(mana_value(&t, spell), 3);
        t.resolve_all();
        assert_eq!(t.on_battlefield(bears), countered, "{top}");
        assert!(t.in_graveyard(P1, "Hit // Run"), "{top}");
    }
}

#[test]
fn mizzix_reduces_only_generic_mana_without_changing_the_mana_value() {
    cr!("601.2f", "601.2b", "107.3a");
    ruling!(
        "Mizzix of the Izmagnus",
        "Mizzix's last ability doesn't change the mana cost or mana value of any spell. It changes only the total cost you pay."
    );
    ruling!(
        "Mizzix of the Izmagnus",
        "Mizzix's last ability can't reduce the amount of colored mana you pay for a spell. It reduces only the generic component of that cost."
    );
    ruling!(
        "Mizzix of the Izmagnus",
        "If an instant or sorcery spell you cast has {X} in its mana cost, you choose the value of X before calculating the spell's total cost. For example, if that spell's mana cost is {X}{R} and you have one experience counter, you could choose 5 as the value of X and pay {4}{R} to cast the spell."
    );
    supported("Mizzix of the Izmagnus");
    // Two experience counters: Lightning Strike ({1}{R}) costs {R} and has mana value 2;
    // Shock ({R}) still costs {R}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mizzix of the Izmagnus");
    crate::r_p034_common::put(&mut t, pl(P0), "experience", 2);
    let strike = t.hand(P0, "Lightning Strike");
    mana(&mut t, P0, "R");
    let spell = t.cast(P0, strike).target(pl(P1)).go();
    assert_eq!(pool(&t, P0), 0);
    assert_eq!(mana_value(&t, spell), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    let shock = t.hand(P0, "Shock");
    assert!(!can_cast(&mut t, P0, shock, CastMethod::Normal), "no mana: Shock costs {{R}}");
    mana(&mut t, P0, "R");
    assert!(can_cast(&mut t, P0, shock, CastMethod::Normal));
    // One experience counter: Blaze with X = 5 costs {4}{R} and has mana value 6.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mizzix of the Izmagnus");
    crate::r_p034_common::put(&mut t, pl(P0), "experience", 1);
    let blaze = t.hand(P0, "Blaze");
    mana(&mut t, P0, "4R");
    let spell = t.cast(P0, blaze).x(5).target(pl(P1)).go();
    assert_eq!(pool(&t, P0), 0);
    assert_eq!(mana_value(&t, spell), 6);
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
}

#[test]
fn cost_reductions_dont_change_a_spells_mana_value() {
    cr!("601.2f", "202.3");
    ruling!(
        "Hollow Marauder",
        "Hollow Marauder’s first ability doesn’t change its mana value, which is always 7."
    );
    ruling!(
        "The Capitoline Triad",
        "The Capitoline Triad’s mana value doesn’t change no matter how many historic cards are in your graveyard."
    );
    supported("Hollow Marauder");
    supported("The Capitoline Triad");
    // Three creature cards in the graveyard: Hollow Marauder costs {3}{B}; mana value 7 on
    // the stack and on the battlefield.
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        t.graveyard(P0, "Grizzly Bears");
    }
    let marauder = t.hand(P0, "Hollow Marauder");
    mana(&mut t, P0, "3B");
    t.answer_targets(P0, &[pl(P1)]);
    let spell = t.cast(P0, marauder).go();
    assert_eq!(pool(&t, P0), 0);
    assert_eq!(mana_value(&t, spell), 7);
    t.resolve_all();
    assert!(t.on_battlefield(spell));
    assert_eq!(mana_value(&t, spell), 7);
    // Five Sol Ring cards in the graveyard: The Capitoline Triad costs {5}; mana value 10.
    let mut t = TestGame::new(2);
    for _ in 0..5 {
        t.graveyard(P0, "Sol Ring");
    }
    let triad = t.hand(P0, "The Capitoline Triad");
    mana(&mut t, P0, "5");
    let spell = t.cast(P0, triad).go();
    assert_eq!(pool(&t, P0), 0);
    assert_eq!(mana_value(&t, spell), 10);
    t.resolve_all();
    assert_eq!(mana_value(&t, spell), 10);
}

#[test]
fn oswald_fiddlebender_uses_the_copied_mana_value_of_a_copy() {
    cr!("707.2", "202.3");
    ruling!(
        "Oswald Fiddlebender",
        "If an artifact is copying something else, its mana value is the mana value of whatever it's copying."
    );
    supported("Oswald Fiddlebender");
    // A Clone ({3}{U}) copying Memnite is an artifact with mana value 0: sacrificing it
    // finds an artifact card with mana value 1 (Sol Ring), not 5.
    let mut t = TestGame::new(2);
    let oswald = t.battlefield(P0, "Oswald Fiddlebender");
    let memnite = t.battlefield(P0, "Memnite");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(memnite)]);
    let clone = t.enter(P0, "Clone");
    t.settle();
    assert_eq!(mana_value(&t, clone), 0);
    let ring = t.library_top(P0, "Sol Ring");
    let stone = t.library_top(P0, "Mind Stone");
    mana(&mut t, P0, "W");
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(clone)]);
    t.answer_choose(P0, &[obj(ring)]);
    act(&mut t, P0, oswald, "Sacrifice an artifact", &[]).expect("Oswald");
    t.resolve_all();
    assert!(!t.on_battlefield(clone));
    assert!(was_offered(&t, P0, from, ring), "{:?}", asked_of(&t, P0, from));
    assert!(!was_offered(&t, P0, from, stone));
    assert!(t.on_battlefield(ring));
}
