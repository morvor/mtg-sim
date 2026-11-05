//! Rulings batch P169 — copies of spells (CR 707.10, 707.10f permanent spell copies
//! become tokens) and tokens that are copies of permanents (CR 707.2, 707.4, 707.9b,
//! 111.4), token replacement effects (CR 614.1a), and investigate (CR 701.16a).

use crate::r_p169_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0 controls Feather and a Grizzly Bears; P0 will choose the Bears for a copy and pay
/// {2}. Returns (Feather, Bears).
fn feather(t: &mut TestGame) -> (ObjectId, ObjectId) {
    supported("Feather, Radiant Arbiter");
    let f = t.battlefield(P0, "Feather, Radiant Arbiter");
    let b = t.battlefield(P0, "Grizzly Bears");
    // "You may" choose, then pay.
    t.answer_choose(P0, &[obj(b)]);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    mana(t, P0, ManaType::C, 2);
    (f, b)
}

#[test]
fn feather_triggers_only_for_spells_targeting_only_feather() {
    cr!("603.2", "115.1");
    ruling!(
        "Feather, Radiant Arbiter",
        "Feather, Radiant Arbiter's ability triggers whenever you cast a noncreature spell that targets only Feather and no other object or player."
    );
    // Giant Growth on Feather: a copy for the Bears.
    let mut t = TestGame::new(2);
    let (f, b) = feather(&mut t);
    cast_new(&mut t, P0, "Giant Growth", &[obj(f)]);
    t.resolve_all();
    assert_eq!(triggers_of(&t, f), 1);
    assert_eq!((t.pt(f), t.pt(b)), ((7, 6), (5, 5)));
    // Giant Growth on the Bears: no trigger.
    let mut t = TestGame::new(2);
    let (f, b) = feather(&mut t);
    cast_new(&mut t, P0, "Giant Growth", &[obj(b)]);
    t.resolve_all();
    assert_eq!(triggers_of(&t, f), 0);
    // Electrolyze targeting Feather and a player: no trigger.
    let mut t = TestGame::new(2);
    let (f, _) = feather(&mut t);
    t.answer_targets(P0, &[obj(f), Entity::Player(P1)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 1]));
    let spell = cast_new(&mut t, P0, "Electrolyze", &[]);
    assert_eq!(targets_of(&t, spell).len(), 2);
    t.settle();
    assert_eq!(triggers_of(&t, f), 0);
}

#[test]
fn feather_copies_of_an_aura_become_tokens() {
    cr!("707.10f", "608.3f");
    ruling!(
        "Feather, Radiant Arbiter",
        "If a permanent spell is copied, the copies are put onto the battlefield as a token as the spell resolves rather than putting the copy of the spell onto the battlefield. The rules that apply to a permanent spell becoming a permanent apply to a copy of a spell becoming a token."
    );
    let mut t = TestGame::new(2);
    let (f, b) = feather(&mut t);
    cast_new(&mut t, P0, "Holy Strength", &[obj(f)]);
    t.settle();
    t.resolve();
    // The copy is on the stack above the Aura spell, as a copy of a spell.
    let copies = spell_copies(&t);
    assert_eq!(copies.len(), 1);
    assert_eq!(targets_of(&t, copies[0]), vec![obj(b)]);
    t.resolve_all();
    assert_eq!(t.pt(b), (3, 4));
    assert_eq!(t.pt(f), (5, 5));
    let auras: Vec<ObjectId> =
        t.g.permanents()
            .filter(|o| o.chars.name == "Holy Strength")
            .map(|o| o.id)
            .collect();
    assert_eq!(auras.len(), 2);
    let token = auras
        .iter()
        .copied()
        .find(|a| t.obj_now(*a).is_token())
        .expect("a token Aura");
    assert_eq!(t.obj_now(token).attached_to, Some(obj(t.g.current(b))));
}

#[test]
fn feather_copies_keep_x_and_modes() {
    cr!("707.10", "707.10e");
    ruling!(
        "Feather, Radiant Arbiter",
        "If the spell that's copied has an X whose value was determined as it was cast, the copies have the same value of X."
    );
    ruling!(
        "Feather, Radiant Arbiter",
        "If the spell that's copied is modal (that is, it says \"Choose one —\" or the like), the copies will have the same mode. You can't choose different modes."
    );
    // Blaze with X = 2 on Feather: the copy deals 2 to the Hill Giant.
    let mut t = TestGame::new(2);
    supported("Feather, Radiant Arbiter");
    let f = t.battlefield(P0, "Feather, Radiant Arbiter");
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_choose(P0, &[obj(giant)]);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    mana(&mut t, P0, ManaType::C, 2);
    mana(&mut t, P0, ManaType::R, 3);
    let blaze = t.hand(P0, "Blaze");
    t.cast(P0, blaze).target(f).x(2).go();
    t.settle();
    t.resolve();
    let copy = spell_copies(&t)[0];
    assert_eq!(crate::r_s25_common::x_of(&t, copy), Some(2));
    t.resolve_all();
    assert_eq!(marked(&t, giant), 2);
    assert_eq!(marked(&t, f), 2);
    // Selesnya Charm's first mode on Feather: the copy has the same mode.
    // (Were P0 asked for the copy's mode, P0 would choose the Knight.)
    let mut t = TestGame::new(2);
    let (f, b) = feather(&mut t);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![2]));
    let from = t.asked().len();
    cast_new(&mut t, P0, "Selesnya Charm", &[obj(f)]);
    t.resolve_all();
    assert_eq!(triggers_of(&t, f), 1);
    let modes_asked = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, mtg_engine::decision::Decision::ChooseModes { .. }))
        .count();
    assert_eq!(modes_asked, 1);
    assert_eq!(t.pt(b), (4, 4));
    assert!(has_kw(&mut t, b, KeywordKind::Trample));
    assert_eq!(creature_tokens(&t, P0), 0);
}

#[test]
fn feather_triggers_for_several_targets_that_are_all_feather() {
    cr!("115.3", "707.10e");
    ruling!(
        "Feather, Radiant Arbiter",
        "If you cast a noncreature spell that has multiple targets, but it's targeting Feather, Radiant Arbiter with all of them, Feather's last ability will trigger. In that case, each of those copies will also only target one of the chosen creatures."
    );
    supported("Seeds of Strength");
    let mut t = TestGame::new(2);
    let (f, b) = feather(&mut t);
    let giant = t.battlefield(P0, "Hill Giant");
    // Copies for both the Bears and the Giant ({4}).
    t.clear_answers();
    t.answer_choose(P0, &[obj(b), obj(giant)]);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    mana(&mut t, P0, ManaType::C, 4);
    cast_new(&mut t, P0, "Seeds of Strength", &[obj(f), obj(f), obj(f)]);
    t.settle();
    assert_eq!(triggers_of(&t, f), 1);
    t.resolve();
    let copies = spell_copies(&t);
    assert_eq!(copies.len(), 2);
    for c in &copies {
        let ts = targets_of(&t, *c);
        assert_eq!(ts.len(), 3);
        assert!(ts.iter().all(|x| *x == ts[0]), "{ts:?}");
    }
    t.resolve_all();
    assert_eq!(t.pt(f), (7, 6));
    assert_eq!(t.pt(b), (5, 5));
    assert_eq!(t.pt(giant), (6, 6));
}

/// Advances to P0's beginning of combat with Mishra, Eminent One targeting `target`, and
/// resolves the trigger. Returns the Warform token.
fn mishra_copy(t: &mut TestGame, target: ObjectId) -> ObjectId {
    supported("Mishra, Eminent One");
    t.battlefield(P0, "Mishra, Eminent One");
    t.answer_targets(P0, &[obj(target)]);
    go_to(t, Step::BeginningOfCombat);
    t.resolve_all();
    t.g.permanents()
        .find(|o| o.chars.name == "Mishra's Warform")
        .map(|o| o.id)
        .expect("a Warform")
}

#[test]
fn mishras_token_gets_the_copied_permanents_enter_abilities() {
    cr!("707.2", "614.1c", "603.6a");
    ruling!(
        "Mishra, Eminent One",
        "Any enters-the-battlefield abilities of the copied permanent will trigger when the token enters the battlefield. Any “as [this permanent] enters the battlefield” or “[this permanent] enters the battlefield with” abilities of the target permanent will also work."
    );
    let mut t = TestGame::new(2);
    let closet = t.enter(P0, "Costume Closet");
    assert_eq!(t.counters(closet, counters::PLUS1), 2);
    let w = mishra_copy(&mut t, closet);
    assert_eq!(t.counters(w, counters::PLUS1), 2);
    assert_eq!(t.pt(w), (6, 6));
    let mut t = TestGame::new(2);
    let well = t.battlefield(P0, "Ichor Wellspring");
    let hand = t.hand_size(P0);
    mishra_copy(&mut t, well);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn mishras_token_copies_only_the_printed_values() {
    cr!("707.2", "707.9b");
    ruling!(
        "Mishra, Eminent One",
        "Except for name, power, toughness, and types, the token copies exactly what was printed on the original permanent and nothing else (unless that permanent is copying something else or it is a token; see below). It doesn’t copy whether that permanent is tapped or untapped, whether it has any counters on it or Auras attached to it, and so on."
    );
    let mut t = TestGame::new(2);
    let well = t.battlefield(P0, "Ichor Wellspring");
    t.g.tap(well);
    put_counters(&mut t, well, counters::CHARGE, 2);
    // An Aura attached to it.
    let aura = t.battlefield(P0, "Viridian Harvest");
    assert!(t.g.attach(aura, obj(well)));
    let w = mishra_copy(&mut t, well);
    let o = t.obj_now(w);
    assert!(!o.tapped);
    assert!(o.counters.values().all(|n| *n == 0));
    assert_eq!(t.g.attachments_of(obj(w)), vec![]);
    assert_eq!(o.chars.name, "Mishra's Warform");
    assert!(o.chars.card_types.contains(CardType::Artifact));
    assert!(o.chars.card_types.contains(CardType::Creature));
    assert!(o.chars.has_subtype("Construct"));
    assert_eq!(t.pt(w), (4, 4));
    assert!(has_kw(&mut t, w, KeywordKind::Haste));
    // Ichor Wellspring's printed ability is copied: dying draws a card.
    let hand = t.hand_size(P0);
    destroy(&mut t, w);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn mishras_copy_of_a_token_uses_the_creating_effects_characteristics() {
    cr!("707.2", "111.3");
    ruling!(
        "Mishra, Eminent One",
        "If the copied permanent is a token, the new token that’s created copies the original characteristics of that token as stated by the effect that created that token (with the listed exceptions)."
    );
    let mut t = TestGame::new(2);
    let treasure = crate::r_s02_common::create_token(&mut t, P0, "Treasure");
    let w = mishra_copy(&mut t, treasure);
    assert!(t.obj_now(w).chars.has_subtype("Treasure"));
    assert!(t.obj_now(w).chars.has_subtype("Construct"));
    // Its Treasure mana ability works.
    let pool = t.g.player(P0).mana_pool.total();
    t.activate(P0, w, 0, &[]).unwrap();
    assert_eq!(t.g.player(P0).mana_pool.total(), pool + 1);
    assert!(!t.on_battlefield(w));
}

#[test]
fn caretakers_talent_token_copy_gets_its_enters_with_counters() {
    cr!("707.2", "614.1c");
    ruling!(
        "Caretaker's Talent",
        "Any \"as [this permanent] enters\" or \"[this permanent] enters with\" abilities of the new token will work."
    );
    supported("Caretaker's Talent");
    let mut t = TestGame::new(2);
    let talent = t.battlefield(P0, "Caretaker's Talent");
    let monument = t.battlefield(P0, "Rampaging Monument");
    put_counters(&mut t, monument, counters::PLUS1, 3);
    // A token copy of Rampaging Monument enters with three counters.
    cast_new(&mut t, P0, "Cackling Counterpart", &[obj(monument)]);
    t.resolve_all();
    let first = tokens(&t, P0);
    assert_eq!(first.len(), 1);
    assert_eq!(t.counters(first[0], counters::PLUS1), 3);
    // Level 2 copies that token: its copy also enters with three.
    mana(&mut t, P0, ManaType::W, 1);
    t.answer_targets(P0, &[obj(first[0])]);
    t.activate(P0, talent, 0, &[]).unwrap();
    t.resolve_all();
    let all = tokens(&t, P0);
    assert_eq!(all.len(), 2);
    for tk in all {
        assert_eq!(t.counters(tk, counters::PLUS1), 3);
        assert_eq!(t.pt(tk), (3, 3));
    }
}

#[test]
fn cori_steel_cutter_attaches_to_one_of_several_monks() {
    cr!("614.1a", "701.3a");
    ruling!(
        "Cori-Steel Cutter",
        "If the triggered ability causes two or more Monks to be created (due to an effect such as that of Doubling Season), you may choose to attach Cori-Steel Cutter to one of them."
    );
    supported("Cori-Steel Cutter");
    let mut t = TestGame::new(2);
    let cutter = t.battlefield(P0, "Cori-Steel Cutter");
    t.battlefield(P0, "Doubling Season");
    cast_opt(&mut t, P0);
    t.resolve_all();
    t.answer_yes(P0, true);
    let from = t.asked().len();
    cast_opt(&mut t, P0);
    t.resolve_all();
    let monks = crate::r_s01_common::with_subtype(&t, P0, "Monk");
    assert_eq!(monks.len(), 2);
    // P0 chose which of the two to attach it to.
    let mut both: Vec<Entity> = monks.iter().map(|m| obj(*m)).collect();
    both.sort();
    assert!(t.asked()[from..].iter().any(|(p, d)| *p == P0
        && matches!(d, mtg_engine::decision::Decision::ChooseEntities { candidates, max: 1, .. }
            if { let mut c = candidates.clone(); c.sort(); c == both })));
    let on: Vec<ObjectId> = monks
        .iter()
        .copied()
        .filter(|m| t.obj_now(cutter).attached_to == Some(obj(*m)))
        .collect();
    assert_eq!(on.len(), 1);
    assert!(has_kw(&mut t, on[0], KeywordKind::Trample));
}

#[test]
fn fae_offerings_clue_isnt_investigating() {
    cr!("701.16a", "111.10f");
    ruling!(
        "Fae Offering",
        "Creating a Clue token this way isn't the same thing as investigating. Notably, an ability that triggers whenever you investigate won't trigger because Fae Offering's ability created a Clue token."
    );
    supported("Fae Offering");
    supported("Erdwal Illuminator");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fae Offering");
    let erdwal = t.battlefield(P0, "Erdwal Illuminator");
    cast_new(&mut t, P0, "Grizzly Bears", &[]);
    t.resolve_all();
    cast_opt(&mut t, P0);
    t.resolve_all();
    end_step(&mut t, P0);
    assert_eq!(crate::r_s01_common::with_subtype(&t, P0, "Clue").len(), 1);
    assert_eq!(triggers_of(&t, erdwal), 0);
    // Positive control: investigating triggers it.
    let mut t = TestGame::new(2);
    let erdwal = t.battlefield(P0, "Erdwal Illuminator");
    t.enter(P0, "Thraben Inspector");
    t.resolve_all();
    assert_eq!(triggers_of(&t, erdwal), 1);
    assert_eq!(crate::r_s01_common::with_subtype(&t, P0, "Clue").len(), 2);
}

#[test]
fn fae_offering_with_academy_manufactor_makes_three_of_each() {
    cr!("614.1a", "614.5");
    ruling!(
        "Fae Offering",
        "If an effect replaces the number or type of tokens you would create, it applies to each of these tokens individually. For example, if you control Fae Offering and Academy Manufactor, you will create three of each type when the triggered ability resolves."
    );
    supported("Academy Manufactor");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fae Offering");
    t.battlefield(P0, "Academy Manufactor");
    cast_new(&mut t, P0, "Grizzly Bears", &[]);
    t.resolve_all();
    cast_opt(&mut t, P0);
    t.resolve_all();
    end_step(&mut t, P0);
    for kind in ["Clue", "Food", "Treasure"] {
        assert_eq!(
            crate::r_s01_common::with_subtype(&t, P0, kind).len(),
            3,
            "{kind}"
        );
    }
}

#[test]
fn fae_offering_counts_spells_whatever_happened_to_them() {
    cr!("603.4", "601.2i");
    ruling!(
        "Fae Offering",
        "It doesn't matter what happened to the spells after you cast them. They each may have resolved, been countered, failed to resolve for some other reason, or another player may have gained control of them. As long as you cast a creature spell and a noncreature spell that turn, the ability will trigger."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fae Offering");
    // The creature spell is countered.
    let bears = cast_new(&mut t, P0, "Grizzly Bears", &[]);
    counterspell(&mut t, bears);
    // The noncreature spell fails to resolve (its target is gone).
    let target = t.battlefield(P1, "Hill Giant");
    mana(&mut t, P0, ManaType::R, 1);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(target).go();
    crate::r_s02_common::destroy(&mut t, target);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Shock"));
    end_step(&mut t, P0);
    assert_eq!(tokens(&t, P0).len(), 3);
    // The noncreature spell is now controlled by P1 (Sudden Substitution) when it
    // resolves; the creature spell resolved.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fae Offering");
    cast_new(&mut t, P0, "Grizzly Bears", &[]);
    t.resolve_all();
    let opt = cast_opt(&mut t, P0);
    let giant = t.battlefield(P1, "Hill Giant");
    mana(&mut t, P1, ManaType::U, 4);
    let ss = t.hand(P1, "Sudden Substitution");
    t.answer_targets(P1, &[obj(opt)]);
    t.answer_targets(P1, &[obj(giant)]);
    t.cast(P1, ss).go();
    t.resolve();
    assert_eq!(t.obj_now(opt).controller, P1);
    t.resolve_all();
    end_step(&mut t, P0);
    assert_eq!(tokens(&t, P0).len(), 3);
    // Positive control: only a creature spell: nothing.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fae Offering");
    cast_new(&mut t, P0, "Grizzly Bears", &[]);
    t.resolve_all();
    end_step(&mut t, P0);
    assert_eq!(tokens(&t, P0).len(), 0);
}
