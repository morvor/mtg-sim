//! Rulings batch P188 — costs, mana and other single-card rulings: Naga Vitalist, Dune
//! Diviner, Gorilla Shaman, Giant Caterpillar, Alien Symbiosis, Tarrian's Journal and
//! Coward // Killer.

use crate::r_s01_common::supported;
use crate::r_s28_common::cast_card;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::mana_abilities::could_produce;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn o(id: ObjectId) -> Entity {
    Entity::Object(id)
}

// ---------------------------------------------------------------------------------------
// Naga Vitalist
// ---------------------------------------------------------------------------------------

#[test]
fn naga_vitalist_ignores_its_lands_costs_and_activation_restrictions() {
    cr!("106.7");
    ruling!(
        "Naga Vitalist",
        "Naga Vitalist checks the effects of all mana-producing abilities of lands you control, but it doesn't check their costs or legality."
    );
    supported("Naga Vitalist");
    supported("Spire of Industry");
    // Spire of Industry: "{T}: Add {C}. {T}, Pay 1 life: Add one mana of any color.
    // Activate only if you control an artifact." P0 controls no artifact, and the Spire is
    // tapped.
    let mut t = TestGame::new(2);
    let naga = t.battlefield(P0, "Naga Vitalist");
    let spire = t.battlefield(P0, "Spire of Industry");
    t.g.objects[spire.0 as usize].tapped = true;
    let types = could_produce(&t.g, naga);
    for ty in [
        ManaType::W,
        ManaType::U,
        ManaType::B,
        ManaType::R,
        ManaType::G,
        ManaType::C,
    ] {
        assert!(types.contains(&ty), "{ty:?} missing from {types:?}");
    }
    // Naga Vitalist taps for red to cast Lightning Bolt.
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(t.obj(naga).tapped);
}

#[test]
fn naga_vitalist_ignores_spending_restrictions_on_its_lands_mana() {
    cr!("106.6", "106.7");
    ruling!(
        "Naga Vitalist",
        "Naga Vitalist doesn't care about any restrictions or riders your lands put on the mana they produce"
    );
    supported("Ancient Ziggurat");
    // Ancient Ziggurat: "{T}: Add one mana of any color. Spend this mana only to cast a
    // creature spell." Naga Vitalist's red mana pays for Lightning Bolt.
    let mut t = TestGame::new(2);
    let naga = t.battlefield(P0, "Naga Vitalist");
    let zig = t.battlefield(P0, "Ancient Ziggurat");
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(t.obj(naga).tapped);
    assert!(!t.obj(zig).tapped);
}

// ---------------------------------------------------------------------------------------
// Dune Diviner, Gorilla Shaman
// ---------------------------------------------------------------------------------------

#[test]
fn dune_diviner_cant_tap_one_desert_for_both_costs() {
    cr!("118.3", "602.2b", "601.2h");
    ruling!(
        "Dune Diviner",
        "You can’t tap a single untapped Desert both to pay {1} and also to pay “Tap an untapped Desert you control.”"
    );
    supported("Dune Diviner");
    supported("Desert of the Fervent");
    // "{1}, Tap an untapped Desert you control: You gain 1 life."
    let mut t = TestGame::new(2);
    let dd = t.battlefield(P0, "Dune Diviner");
    let desert = t.battlefield(P0, "Desert of the Fervent");
    assert!(t.activate(P0, dd, 0, &[]).is_err());
    assert!(!t.obj(desert).tapped);
    assert_eq!(t.life(P0), 20);
    // With another land for the {1}, it works.
    let mountain = t.battlefield(P0, "Mountain");
    t.activate(P0, dd, 0, &[]).expect("diviner");
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    assert!(t.obj(desert).tapped && t.obj(mountain).tapped);
    assert!(t.g.player(P0).mana_pool.is_empty());

    // A failed first try at paying (the Desert tapped for the {1}) is undone completely:
    // Manabarbs triggers once, for the Mountain, not also for the Desert.
    supported("Manabarbs");
    let mut t = TestGame::new(2);
    let dd = t.battlefield(P0, "Dune Diviner");
    let desert = t.battlefield(P0, "Desert of the Fervent");
    let mountain = t.battlefield(P0, "Mountain");
    t.battlefield(P1, "Manabarbs");
    t.activate(P0, dd, 0, &[]).expect("diviner");
    t.resolve_all();
    assert!(t.obj(desert).tapped && t.obj(mountain).tapped);
    assert!(t.g.player(P0).mana_pool.is_empty());
    assert_eq!(t.life(P0), 20, "1 life gained, 1 damage from Manabarbs");
}

#[test]
fn gorilla_shaman_costs_two_x_plus_one() {
    cr!("107.3a", "601.2f");
    ruling!(
        "Gorilla Shaman",
        "The cost to destroy a noncreature artifact with mana value 0 is {0} + {0} + {1} = {1}. The cost to destroy a noncreature artifact with mana value 1 is {1} + {1} + {1} = {3}."
    );
    supported("Gorilla Shaman");
    // "{X}{X}{1}: Destroy target noncreature artifact with mana value X."
    let mut t = TestGame::new(2);
    let gs = t.battlefield(P0, "Gorilla Shaman");
    let crypt = t.battlefield(P1, "Tormod's Crypt");
    let ring = t.battlefield(P1, "Sol Ring");
    t.lands(P0, "Mountain", 1);
    t.answer(P0, DecisionKind::X, Answer::Number(0));
    t.activate(P0, gs, 0, &[o(crypt)]).expect("X = 0");
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Tormod's Crypt"));
    // X = 1 for Sol Ring: {3}. Two lands aren't enough; three are.
    t.lands(P0, "Mountain", 2);
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    assert!(t.activate(P0, gs, 0, &[o(ring)]).is_err());
    t.clear_answers();
    t.lands(P0, "Mountain", 1);
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    t.activate(P0, gs, 0, &[o(ring)]).expect("X = 1");
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Sol Ring"));
}

// ---------------------------------------------------------------------------------------
// Giant Caterpillar
// ---------------------------------------------------------------------------------------

#[test]
fn giant_caterpillar_makes_an_insect_token_named_butterfly() {
    cr!("111.4", "603.7a");
    ruling!(
        "Giant Caterpillar",
        "It used to make Butterfly tokens. Now it makes Insect tokens named Butterfly."
    );
    supported("Giant Caterpillar");
    let mut t = TestGame::new(2);
    let gc = t.battlefield(P0, "Giant Caterpillar");
    t.lands(P0, "Forest", 1);
    t.activate(P0, gc, 0, &[]).expect("caterpillar");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Giant Caterpillar"));
    assert!(t.named_on_battlefield("Butterfly").is_empty());
    t.advance_to(P0, Step::End);
    t.settle();
    t.resolve_all();
    let b = t.named_on_battlefield("Butterfly");
    assert_eq!(b.len(), 1);
    let tok = t.obj(b[0]);
    assert!(tok.is_token());
    assert!(tok.chars.has_subtype("Insect") && !tok.chars.has_subtype("Butterfly"));
    assert_eq!(tok.chars.colors, ColorSet::single(Color::Green));
    assert!(tok.has_keyword(KeywordKind::Flying));
    assert_eq!(t.pt(b[0]), (1, 1));
}

// ---------------------------------------------------------------------------------------
// Alien Symbiosis, Tarrian's Journal
// ---------------------------------------------------------------------------------------

#[test]
fn alien_symbiosis_from_the_graveyard_follows_normal_timing() {
    cr!("601.3", "303.1");
    ruling!(
        "Alien Symbiosis",
        "You must follow all normal timing rules when casting Alien Symbiosis using its last ability."
    );
    supported("Alien Symbiosis");
    for main in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let aura = t.graveyard(P0, "Alien Symbiosis");
        t.hand(P0, "Island");
        t.lands(P0, "Swamp", 2);
        if !main {
            t.set_step(P0, Step::BeginningOfCombat);
        }
        // Cast with its own "You may cast this card from your graveyard by discarding a
        // card in addition to paying its other costs."
        let uid = t
            .obj(aura)
            .chars
            .abilities
            .iter()
            .find(|a| a.text.contains("from your graveyard by discarding"))
            .unwrap_or_else(|| panic!("{:?}", t.obj(aura).chars.abilities))
            .uid;
        let r = t
            .cast(P0, aura)
            .method(CastMethod::Alternative(uid))
            .target(bears)
            .try_go();
        assert_eq!(r.is_ok(), main, "main phase: {main}");
        if main {
            t.resolve_all();
            assert_eq!(t.pt(bears), (3, 3));
            assert!(t.obj_now(bears).chars.has_subtype("Symbiote"));
            // The card in hand was discarded as the additional cost.
            assert!(t.in_graveyard(P0, "Island"));
        }
    }
}

#[test]
fn tarrians_journal_can_discard_an_empty_hand() {
    cr!("602.2", "701.9a");
    ruling!(
        "Tarrian's Journal // The Tomb of Aclazotz",
        "You can discard your hand even if you hand has zero cards in it."
    );
    // "{2}, {T}, Discard your hand: Transform Tarrian's Journal."
    let mut t = TestGame::new(2);
    let journal = t.battlefield(P0, "Tarrian's Journal // The Tomb of Aclazotz");
    t.lands(P0, "Wastes", 2);
    assert_eq!(t.hand_size(P0), 0);
    t.activate(P0, journal, 1, &[]).expect("transform");
    t.resolve_all();
    let j = t.obj_now(journal);
    assert_eq!(j.chars.name, "The Tomb of Aclazotz");
    assert!(j.is(CardType::Land));
}

// ---------------------------------------------------------------------------------------
// Coward // Killer
// ---------------------------------------------------------------------------------------

#[test]
fn killer_with_an_illegal_target_deals_no_damage() {
    cr!("608.2b");
    ruling!(
        "Coward // Killer",
        "If the target of Killer is illegal as the spell tries to resolve, it won't deal damage to any creatures."
    );
    // Killer: "Killer deals 3 damage to target creature and each other creature that
    // shares a creature type with it."
    for illegal in [false, true] {
        let mut t = TestGame::new(2);
        let b1 = t.battlefield(P1, "Grizzly Bears");
        let b2 = t.battlefield(P1, "Grizzly Bears");
        let elves = t.battlefield(P1, "Llanowar Elves");
        t.lands(P0, "Mountain", 4);
        let card = t.hand(P0, "Coward // Killer");
        t.cast(P0, card).method(CastMethod::Half(1)).target(b1).go();
        if illegal {
            // The target leaves the battlefield in response.
            let b1 = t.g.current(b1);
            t.g.move_object(
                b1,
                Zone::Hand(P1),
                mtg_engine::events::MoveCause::Effect,
                Some(P0),
            );
            t.g.flush_events();
        }
        t.resolve_all();
        assert_eq!(t.on_battlefield(b2), illegal, "illegal: {illegal}");
        assert!(t.on_battlefield(elves));
        if illegal {
            assert_eq!(t.obj_now(b2).damage, 0);
            assert!(t.in_hand(P1, "Grizzly Bears"));
        } else {
            assert!(!t.on_battlefield(b1));
        }
    }
}
