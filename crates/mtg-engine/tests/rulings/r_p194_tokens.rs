//! Rulings batch P194 — abilities that create tokens: tokens put onto the battlefield
//! attacking (CR 508.4), X read on resolution, state-based actions after the spell
//! finishes resolving (CR 608.2n, 704.3), Frontline Heroism's copies (CR 707.10, 608.3f),
//! and the tokens' types and text.

use crate::r_p189_common::{bounce, triggers_of};
use crate::r_p194_common::*;
use crate::r_s01_common::{attack_with, supported, with_subtype};
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Ghalta and Mavren attacks with `others`; its trigger's `mode` is chosen. Resolves
/// everything.
fn ghalta_attacks(t: &mut TestGame, others: &[ObjectId], mode: usize) -> ObjectId {
    let gm = t.battlefield(P0, "Ghalta and Mavren");
    let mut decl = vec![(gm, pl(P1))];
    decl.extend(others.iter().map(|o| (*o, pl(P1))));
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![mode]));
    attack_with(t, &decl);
    gm
}

#[test]
fn ghalta_and_mavrens_dinosaur_wasnt_declared_as_an_attacker() {
    cr!("508.4", "508.3a");
    ruling!(
        "Ghalta and Mavren",
        "Although the Dinosaur token enters the battlefield attacking, it was never declared as an attacking creature (for the purposes of abilities that trigger whenever a creature attacks, for example)."
    );
    supported("Ghalta and Mavren");
    supported("Fervent Charge");
    let mut t = TestGame::new(2);
    let charge = t.battlefield(P0, "Fervent Charge");
    let bears = t.battlefield(P0, "Grizzly Bears");
    ghalta_attacks(&mut t, &[bears], 0);
    t.resolve_all();
    let dinos = with_subtype(&t, P0, "Dinosaur");
    // Ghalta and Mavren itself, and the token.
    assert_eq!(dinos.len(), 2);
    assert!(dinos.iter().all(|d| t.g.is_attacking(*d)));
    // Fervent Charge triggered for the two declared attackers only.
    assert_eq!(triggers_of(&t, charge), 2);
}

#[test]
fn ghalta_and_mavrens_x_is_read_on_resolution() {
    cr!("608.2h", "704.5f", "107.3f");
    ruling!(
        "Ghalta and Mavren",
        "For both modes, the value of X is determined as the ability resolves."
    );
    ruling!(
        "Ghalta and Mavren",
        "If you choose the first mode and there are no other attacking creatures, you’ll create a 0/0 Dinosaur token. Unless something else is immediately raising its toughness, the token will die. You won’t have time to respond with a spell or ability to try and save it. A similar situation occurs if the greatest power among other attacking creatures is 0 or less."
    );
    ruling!(
        "Ghalta and Mavren",
        "If you choose the second mode and there are no other attacking creatures, you won’t create any Vampire tokens."
    );
    // A Giant Growth in response raises X.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    ghalta_attacks(&mut t, &[bears], 0);
    assert_eq!(t.stack_len(), 1);
    let gg = t.hand(P0, "Giant Growth");
    mana(&mut t, P0, ManaType::G, 1);
    t.cast(P0, gg).target(bears).go();
    t.resolve_all();
    let token: Vec<ObjectId> = with_subtype(&t, P0, "Dinosaur")
        .into_iter()
        .filter(|d| t.obj_now(*d).is_token())
        .collect();
    assert_eq!(token.len(), 1);
    assert_eq!(t.pt(token[0]), (5, 5));
    // Alone: a 0/0 Dinosaur that dies at once.
    let mut t = TestGame::new(2);
    ghalta_attacks(&mut t, &[], 0);
    let before = t.g.objects.len();
    t.resolve_all();
    assert!(t.g.objects.len() > before, "a token was created");
    assert_eq!(with_subtype(&t, P0, "Dinosaur").len(), 1);
    // Alone, second mode: no Vampires. With two others: two.
    let mut t = TestGame::new(2);
    ghalta_attacks(&mut t, &[], 1);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Vampire").len(), 1);
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    ghalta_attacks(&mut t, &[a, b], 1);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Vampire").len(), 1 + 2);
}

#[test]
fn cynette_leaving_can_make_marked_damage_lethal() {
    cr!("120.6", "704.5g", "613.4c");
    ruling!(
        "Cynette, Jelly Drover",
        "Because damage remains marked on a creature until it's removed as the turn ends, nonlethal damage dealt to creatures you control with flying may become lethal if Cynette leaves the battlefield during that turn."
    );
    supported("Cynette, Jelly Drover");
    let mut t = TestGame::new(2);
    let cyn = t.enter(P0, "Cynette, Jelly Drover");
    t.resolve_all();
    let jelly = with_subtype(&t, P0, "Jellyfish");
    assert_eq!(jelly.len(), 1);
    let jelly = jelly[0];
    assert_eq!(t.pt(jelly), (2, 2));
    damage(&mut t, cyn, obj(jelly), 1);
    assert!(t.on_battlefield(jelly));
    bounce(&mut t, cyn);
    t.settle();
    assert!(!t.on_battlefield(jelly));
}

#[test]
fn frontline_heroism_copy_resolves_first_and_isnt_cast() {
    cr!("707.10", "603.3", "701.6a");
    ruling!(
        "Frontline Heroism",
        "Frontline Heroism's last ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack without resolving."
    );
    ruling!(
        "Frontline Heroism",
        "The copy created by Frontline Heroism's last ability is created on the stack, so it's not \"cast\". Abilities that trigger when a player casts a spell (like Frontline Heroism's own ability) won't trigger."
    );
    supported("Frontline Heroism");
    // The trigger and the copy resolve before Giant Growth.
    let mut t = TestGame::new(2);
    let fh = t.battlefield(P0, "Frontline Heroism");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let gg = t.hand(P0, "Giant Growth");
    mana(&mut t, P0, ManaType::G, 1);
    let spell = t.cast(P0, gg).target(bears).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    let soldiers = with_subtype(&t, P0, "Soldier");
    assert_eq!(soldiers.len(), 1);
    // The copy is on the stack above the original.
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(t.pt(soldiers[0]), (4, 4));
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(t.g.obj(spell).zone, Zone::Stack);
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5));
    assert_eq!(triggers_of(&t, fh), 1);
    // Countered in response: the trigger still makes its token.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Frontline Heroism");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let gg = t.hand(P0, "Giant Growth");
    mana(&mut t, P0, ManaType::G, 1);
    let spell = t.cast(P0, gg).target(bears).go();
    t.settle();
    let cs = t.hand(P1, "Counterspell");
    mana(&mut t, P1, ManaType::U, 2);
    t.cast(P1, cs).target(spell).go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Giant Growth"));
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Soldier").len(), 1);
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn frontline_heroism_copied_aura_becomes_a_token_that_isnt_created() {
    cr!("608.3f", "707.10f", "111.1");
    ruling!(
        "Frontline Heroism",
        "If a permanent spell is copied, the copy is put onto the battlefield as a token as the spell resolves rather than putting the copy of the spell onto the battlefield. The rules that apply to a permanent spell becoming a permanent apply to a copy of a spell becoming a token."
    );
    ruling!(
        "Frontline Heroism",
        "The token that a resolving copy of a permanent spell becomes isn't \"created\" and won't interact with abilities that care about tokens being created, such as the first ability of Doubling Season."
    );
    supported("Doubling Season");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Frontline Heroism");
    t.battlefield(P0, "Doubling Season");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let hs = t.hand(P0, "Holy Strength");
    mana(&mut t, P0, ManaType::W, 1);
    t.cast(P0, hs).target(bears).go();
    t.resolve_all();
    // Doubling Season doubles the Soldier the trigger creates, not the Aura token.
    let soldiers = with_subtype(&t, P0, "Soldier");
    assert_eq!(soldiers.len(), 2);
    let auras: Vec<ObjectId> = t
        .g
        .permanents()
        .filter(|o| o.chars.name == "Holy Strength")
        .map(|o| o.id)
        .collect();
    assert_eq!(auras.len(), 2);
    let token_auras: Vec<ObjectId> = auras
        .iter()
        .copied()
        .filter(|a| t.obj_now(*a).is_token())
        .collect();
    assert_eq!(token_auras.len(), 1);
    let on = t.obj_now(token_auras[0]).attached_to.and_then(|e| e.object());
    assert!(on.is_some_and(|o| soldiers.contains(&o)));
    assert_eq!(t.pt(bears), (3, 4));
}

#[test]
fn frontline_heroism_copy_of_a_multi_target_spell_targets_the_token_each_time() {
    cr!("707.10e");
    ruling!(
        "Frontline Heroism",
        "If you cast a spell that has multiple targets but all of them are the same creature you control, Frontline Heroism's last ability will trigger. In that case, the copy's targets will all be the token created by the ability."
    );
    supported("Seeds of Strength");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Frontline Heroism");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let s = t.hand(P0, "Seeds of Strength");
    mana(&mut t, P0, ManaType::G, 1);
    mana(&mut t, P0, ManaType::W, 1);
    t.cast(P0, s)
        .target(bears)
        .target(bears)
        .target(bears)
        .go();
    t.resolve_all();
    let soldiers = with_subtype(&t, P0, "Soldier");
    assert_eq!(soldiers.len(), 1);
    assert_eq!(t.pt(soldiers[0]), (4, 4));
    assert_eq!(t.pt(bears), (5, 5));
}

#[test]
fn sound_the_call_wolf_survives_until_the_card_is_in_the_graveyard() {
    cr!("608.2n", "704.3", "704.5f");
    ruling!(
        "Sound the Call",
        "If Sound the Call resolves while all graveyards are empty, first a 1/1 token is created, then the Sound the Call card is put into your graveyard, which makes the token 2/2."
    );
    supported("Sound the Call");
    supported("Night of Souls' Betrayal");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Night of Souls' Betrayal");
    let s = t.hand(P0, "Sound the Call");
    mana(&mut t, P0, ManaType::G, 3);
    t.cast(P0, s).go();
    t.resolve_all();
    let wolves = with_subtype(&t, P0, "Wolf");
    assert_eq!(wolves.len(), 1);
    assert_eq!(t.pt(wolves[0]), (1, 1));
}

#[test]
fn master_of_the_hunt_can_activate_repeatedly() {
    cr!("602.1");
    ruling!(
        "Master of the Hunt",
        "Since tapping is not part of the cost, you can use this more than once a turn."
    );
    supported("Master of the Hunt");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Master of the Hunt");
    for _ in 0..2 {
        mana(&mut t, P0, ManaType::G, 4);
        t.activate(P0, m, 0, &[]).unwrap();
        t.resolve_all();
    }
    assert_eq!(t.named_on_battlefield("Wolves of the Hunt").len(), 2);
    assert!(!t.obj_now(m).tapped);
}

#[test]
fn goblin_scouts_tokens_are_goblins_with_mountainwalk_text() {
    cr!("111.4", "612.1", "702.14a");
    ruling!(
        "Goblin Scouts",
        "The tokens are of creature type Goblin and Scout. They are affected by things that affect Goblins."
    );
    ruling!(
        "Goblin Scouts",
        "The tokens have ‘Mountainwalk’ in their text, and will be affected by text-changing effects accordingly."
    );
    supported("Goblin Scouts");
    supported("Goblin King");
    supported("Magical Hack");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Goblin King");
    let gs = t.hand(P0, "Goblin Scouts");
    mana(&mut t, P0, ManaType::R, 5);
    t.cast(P0, gs).go();
    t.resolve_all();
    let scouts = with_subtype(&t, P0, "Scout");
    assert_eq!(scouts.len(), 3);
    assert!(scouts.iter().all(|s| t.obj_now(*s).chars.has_subtype("Goblin")));
    assert!(scouts.iter().all(|s| t.pt(*s) == (2, 2)));
    // Magical Hack: Mountain → Island on one token (indices: the five basic land types,
    // then the four others).
    let walk = |t: &TestGame, id: ObjectId| -> Vec<String> {
        t.obj_now(id)
            .chars
            .keywords()
            .filter(|k| k.kind == KeywordKind::Landwalk)
            .map(|k| format!("{:?}", k.filter))
            .collect()
    };
    // Without Goblin King (which grants mountainwalk too).
    let mut t = TestGame::new(2);
    let gs = t.hand(P0, "Goblin Scouts");
    mana(&mut t, P0, ManaType::R, 5);
    t.cast(P0, gs).go();
    t.resolve_all();
    let scouts = with_subtype(&t, P0, "Scout");
    let target = scouts[0];
    let hack = t.hand(P0, "Magical Hack");
    mana(&mut t, P0, ManaType::U, 1);
    t.answer(P0, DecisionKind::Option, Answer::Index(3));
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.cast(P0, hack).target(target).go();
    t.resolve_all();
    assert_eq!(walk(&t, target), vec!["Some(Subtype(\"Island\"))".to_string()]);
    assert_eq!(
        walk(&t, scouts[1]),
        vec!["Some(Subtype(\"Mountain\"))".to_string()]
    );
}
