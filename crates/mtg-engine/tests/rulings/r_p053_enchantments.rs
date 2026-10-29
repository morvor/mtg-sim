//! Rulings batch P053 — enchantment engines and enchantmentfall: "whenever you cast an
//! [enchantment / artifact / instant or sorcery] spell" triggers (CR 603.2, 603.3), Tuvasa,
//! constellation (enchantments entering, CR 603.6a), Ondu Spiritdancer's and Yenna's token
//! copies of enchantments (CR 707.2, 707.4, 303.4f), multiple instances of double strike
//! (CR 702.4d) and Goldenhide Ox's block requirement (CR 509.1c).

use crate::r_p053_common::*;
use mtg_engine::ability::{Effect, PlayerRef, TokenSpec, Value};
use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::{counters, CardType, ColorSet};
use mtg_engine::*;

#[test]
fn spell_cast_triggers_resolve_first_even_if_the_spell_is_countered() {
    cr!("603.3", "601.2i");
    ruling!(
        "Argothian Enchantress",
        "Argothian Enchantress’s last ability will resolve before the spell that caused it to trigger."
    );
    ruling!(
        "Argothian Enchantress",
        "If the enchantment spell is countered, Argothian Enchantress’s last ability still resolves and causes you to draw a card."
    );
    ruling!(
        "Satyr Enchanter",
        "Satyr Enchanter’s triggered ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    ruling!(
        "Enchantress's Presence",
        "The ability is independent from the spell that caused it to trigger. Notably, if the enchantment spell is countered or doesn't resolve for any other reason, you'll still draw a card"
    );
    ruling!(
        "Enchantress's Presence",
        "The triggered ability will resolve before the spell that caused it to trigger."
    );
    ruling!(
        "Tuvasa the Sunlit",
        "Tuvasa's triggered ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    ruling!(
        "Dr. Madison Li",
        "Dr. Madison Li’s first ability resolves before the spell that caused it to trigger."
    );
    ruling!(
        "Dynavolt Tower",
        "Dynavolt Tower's triggered ability resolves before the spell that caused it to trigger."
    );
    // (engine, spell cast, what the trigger does: cards drawn, energy gained)
    for (engine, spell, drawn, e) in [
        ("Argothian Enchantress", "Pacifism", 1, 0),
        ("Satyr Enchanter", "Pacifism", 1, 0),
        ("Enchantress's Presence", "Pacifism", 1, 0),
        ("Tuvasa the Sunlit", "Pacifism", 1, 0),
        ("Dr. Madison Li", "Ornithopter", 0, 1),
        ("Dynavolt Tower", "Divination", 0, 2),
    ] {
        supported(engine);
        let mut t = TestGame::new(2);
        t.battlefield(P0, engine);
        let bears = t.battlefield(P1, "Grizzly Bears");
        crate::r_s25_common::lands_for_cost(&mut t, P0, spell);
        let c = t.hand(P0, spell);
        let s = if spell == "Pacifism" {
            t.cast(P0, c).target(obj(bears)).go()
        } else {
            t.cast(P0, c).go()
        };
        t.settle();
        assert_eq!(t.stack_len(), 2, "{engine}");
        assert!(is_trigger(&t, top(&t)), "{engine}");
        let hand = t.hand_size(P0);
        t.resolve();
        // The trigger resolved; the spell is still on the stack.
        assert_eq!(t.hand_size(P0), hand + drawn, "{engine}");
        assert_eq!(energy(&t, P0), e, "{engine}");
        assert_eq!(t.g.stack, vec![s], "{engine}");
        // And it's independent of the spell: countering a second one changes nothing.
        let mut t2 = TestGame::new(2);
        t2.battlefield(P0, engine);
        let bears = t2.battlefield(P1, "Grizzly Bears");
        crate::r_s25_common::lands_for_cost(&mut t2, P0, spell);
        let c = t2.hand(P0, spell);
        let s = if spell == "Pacifism" {
            t2.cast(P0, c).target(obj(bears)).go()
        } else {
            t2.cast(P0, c).go()
        };
        t2.settle();
        t2.lands(P1, "Island", 2);
        let cs = t2.hand(P1, "Counterspell");
        t2.cast(P1, cs).target(obj(s)).go();
        let hand = t2.hand_size(P0);
        t2.resolve_all();
        assert_eq!(t2.zone(s), Zone::Graveyard(P0), "{engine}");
        assert_eq!(t2.hand_size(P0), hand + drawn, "{engine}");
        assert_eq!(energy(&t2, P0), e, "{engine}");
    }
}

#[test]
fn enchantment_cast_triggers_need_a_cast_spell() {
    cr!("603.2", "601.2i");
    ruling!(
        "Argothian Enchantress",
        "Enchantments put onto the battlefield without being cast won’t cause Argothian Enchantress’s last ability to trigger."
    );
    ruling!(
        "Sythis, Harvest's Hand",
        "Sythis, Harvest Hand's ability doesn't trigger when it is cast."
    );
    supported("Sythis, Harvest's Hand");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Argothian Enchantress");
    t.enter(P0, "Glorious Anthem");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    let mut t = TestGame::new(2);
    crate::r_s25_common::lands_for_cost(&mut t, P0, "Sythis, Harvest's Hand");
    let c = t.hand(P0, "Sythis, Harvest's Hand");
    t.cast(P0, c).go();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    // A second enchantment spell does trigger it.
    t.lands(P0, "Plains", 3);
    let anthem = t.hand(P0, "Glorious Anthem");
    t.cast(P0, anthem).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
}

#[test]
fn tuvasa_damage_becomes_lethal_when_enchantments_leave() {
    cr!("704.5g", "613.4c");
    ruling!(
        "Tuvasa the Sunlit",
        "Because damage remains marked on a creature until it's removed as the turn ends, nonlethal damage dealt to Tuvasa may become lethal if some of your enchantments leave the battlefield during that turn."
    );
    supported("Tuvasa the Sunlit");
    let mut t = TestGame::new(2);
    let tuvasa = t.battlefield(P0, "Tuvasa the Sunlit");
    let anthem = t.battlefield(P0, "Glorious Anthem");
    t.battlefield(P0, "Enchantress's Presence");
    // 1/1 +2/+2 (two enchantments) +1/+1 (the Anthem): 4/4.
    assert_eq!(t.pt(tuvasa), (4, 4));
    damage(&mut t, tuvasa, 2, tuvasa);
    assert!(t.on_battlefield(tuvasa));
    destroy(&mut t, anthem);
    assert!(!t.on_battlefield(tuvasa));
}

#[test]
fn setessan_champion_draws_even_if_it_left() {
    cr!("608.2h", "603.10");
    ruling!(
        "Setessan Champion",
        "If Setessan Champion leaves the battlefield after its ability has triggered, you still draw a card even though you won't put a +1/+1 counter on Setessan Champion."
    );
    supported("Setessan Champion");
    let mut t = TestGame::new(2);
    let champ = t.battlefield(P0, "Setessan Champion");
    t.enter(P0, "Glorious Anthem");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, champ);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.counters(champ, counters::PLUS1), 0);
}

#[test]
fn constellation_can_target_the_enchantment_creature_that_entered() {
    cr!("603.2", "115.1");
    ruling!(
        "Pious Wayfarer",
        "If an enchantment creature enters the battlefield under your control, Pious Wayfarer’s ability can target it."
    );
    ruling!(
        "Oath of the Ancient Wood",
        "If the enchantment that enters the battlefield is somehow also a creature, you may choose it as the target of Oath of the Ancient Wood’s ability."
    );
    supported("Pious Wayfarer");
    supported("Oath of the Ancient Wood");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Pious Wayfarer");
    t.battlefield(P0, "Oath of the Ancient Wood");
    t.answer_yes(P0, true);
    let dryad = t.enter(P0, "Leafcrown Dryad");
    // Both triggers target the Dryad (the targets are asked as they go on the stack).
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(dryad)]);
    t.answer_targets(P0, &[obj(dryad)]);
    t.settle();
    let offered = crate::r_s02_common::target_candidates(&t, P0, from);
    assert_eq!(offered.len(), 2);
    assert!(offered.iter().all(|c| c.contains(&obj(dryad))));
    t.resolve_all();
    assert_eq!(t.counters(dryad, counters::PLUS1), 1);
    assert_eq!(t.pt(dryad), (4, 4));
}

#[test]
fn multiple_instances_of_double_strike_are_redundant() {
    cr!("702.4d");
    ruling!(
        "Favored of Iroas",
        "Multiple instances of double strike on the same creature are redundant."
    );
    ruling!(
        "Terror of Mount Velus",
        "Multiple instances of double strike on the same creature are redundant."
    );
    ruling!(
        "Valiant Knight",
        "Multiple instances of double strike on the same creature are redundant."
    );
    // Favored of Iroas: two enchantments entered.
    supported("Favored of Iroas");
    let mut t = TestGame::new(2);
    let favored = t.battlefield(P0, "Favored of Iroas");
    t.enter(P0, "Glorious Anthem");
    t.enter(P0, "Enchantress's Presence");
    t.resolve_all();
    attack_with(&mut t, &[(favored, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 20 - 2 * 3);
    // Terror of Mount Velus: double strike, and its own trigger gives it double strike.
    supported("Terror of Mount Velus");
    let mut t = TestGame::new(2);
    let terror = t.enter(P0, "Terror of Mount Velus");
    t.resolve_all();
    t.g.objects[terror.0 as usize].summoning_sick = false;
    attack_with(&mut t, &[(terror, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 20 - 2 * 5);
    // Valiant Knight: its ability activated twice.
    supported("Valiant Knight");
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "Valiant Knight");
    t.lands(P0, "Plains", 10);
    for _ in 0..2 {
        activate_containing(&mut t, P0, knight, "double strike").unwrap();
        t.resolve_all();
    }
    attack_with(&mut t, &[(knight, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 20 - 2 * 3);
}

#[test]
fn goldenhide_ox_target_must_be_blocked_if_able() {
    cr!("509.1c");
    ruling!(
        "Goldenhide Ox",
        "The defending player must assign at least one blocker to the target creature during the declare blockers step if that player controls any creatures that could block it."
    );
    supported("Goldenhide Ox");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Gray Ogre");
    let x = t.battlefield(P1, "Hill Giant");
    let y = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(a)]);
    t.enter(P0, "Goldenhide Ox");
    t.resolve_all();
    attack_with(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    assert!(!legal_blocks(&mut t, P1, &[]));
    assert!(!legal_blocks(&mut t, P1, &[(x, b)]));
    assert!(legal_blocks(&mut t, P1, &[(x, a)]));
    assert!(legal_blocks(&mut t, P1, &[(x, a), (y, a)]));
    assert!(legal_blocks(&mut t, P1, &[(x, b), (y, a)]));
}

/// `p` creates a 1/1 white Spirit enchantment creature token as a resolving effect would
/// (with flush and settle, so its enters triggers trigger).
fn spirit_enchantment_token(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let spec = TokenSpec {
        name: "Spirit".into(),
        colors: ColorSet::single(mtg_engine::types::Color::White),
        supertypes: vec![],
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: vec!["Spirit".into()],
        power: Some(1),
        toughness: Some(1),
        abilities: vec![],
        scryfall_name: None,
        pt_values: None,
    };
    let before = t.g.battlefield.clone();
    let mut ctx = mtg_engine::eval::Ctx::new(None, p);
    t.g.exec(
        &Effect::CreateToken {
            spec,
            count: Value::c(1),
            controller: PlayerRef::You,
            tapped: false,
            attacking: false,
        },
        &mut ctx,
    );
    t.g.recompute();
    let token = t
        .g
        .battlefield
        .iter()
        .copied()
        .find(|id| !before.contains(id))
        .expect("no token");
    t.g.flush_events();
    t.settle();
    token
}

#[test]
fn ondu_spiritdancer_copies_the_copiable_values() {
    cr!("707.2", "707.4", "111.1");
    ruling!(
        "Ondu Spiritdancer",
        "Each token copies exactly what was printed on the original enchantment and nothing else"
    );
    ruling!(
        "Ondu Spiritdancer",
        "If the copied enchantment is a token, the token that's created copies the original characteristics of that token as stated by the effect that created the token."
    );
    ruling!(
        "Ondu Spiritdancer",
        "If the copied enchantment is copying something else, then the token enters the battlefield as whatever that enchantment copied."
    );
    supported("Ondu Spiritdancer");
    // A dressed-up Leafcrown Dryad: the copy is a fresh 2/2.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ondu Spiritdancer");
    t.answer_yes(P0, true);
    let dryad = t.enter(P0, "Leafcrown Dryad");
    t.settle();
    crate::r_s26_common::dress_up(&mut t, dryad);
    t.resolve_all();
    let toks = crate::r_s01_common::tokens(&t, P0);
    assert_eq!(toks.len(), 1);
    assert_eq!(t.obj_now(toks[0]).chars.name, "Leafcrown Dryad");
    assert_eq!(t.pt(toks[0]), (2, 2));
    assert!(!t.obj_now(toks[0]).tapped);
    assert_eq!(
        t.obj_now(toks[0]).chars.colors,
        ColorSet::single(mtg_engine::types::Color::Green)
    );
    // A token: the copy has the token's original characteristics (+1/+1 counters and a
    // pump on the original aren't copied).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ondu Spiritdancer");
    t.answer_yes(P0, true);
    let spirit = spirit_enchantment_token(&mut t, P0);
    put_counters(&mut t, spirit, counters::PLUS1, 2);
    t.resolve_all();
    let toks: Vec<_> = crate::r_s01_common::tokens(&t, P0)
        .into_iter()
        .filter(|id| *id != t.g.current(spirit))
        .collect();
    assert_eq!(toks.len(), 1);
    assert_eq!(t.obj_now(toks[0]).chars.name, "Spirit");
    assert!(t.obj_now(toks[0]).is(CardType::Enchantment));
    assert_eq!(t.pt(toks[0]), (1, 1));
    // Copy Enchantment copying Glorious Anthem: the token is a Glorious Anthem.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ondu Spiritdancer");
    let anthem = t.battlefield(P0, "Glorious Anthem");
    t.answer_choose(P0, &[obj(anthem)]);
    t.answer_yes(P0, true);
    t.enter(P0, "Copy Enchantment");
    t.resolve_all();
    let toks = crate::r_s01_common::tokens(&t, P0);
    assert_eq!(toks.len(), 1);
    assert_eq!(t.obj_now(toks[0]).chars.name, "Glorious Anthem");
    assert_eq!(t.named_on_battlefield("Glorious Anthem").len(), 3);
}

#[test]
fn ondu_spiritdancer_only_once_each_turn() {
    cr!("603.2");
    ruling!(
        "Ondu Spiritdancer",
        "If multiple instances of the ability are on the stack (perhaps because multiple enchantments entered the battlefield under your control at the same time), you will be able to create a token copy of an enchantment for only one of those instances."
    );
    ruling!(
        "Ondu Spiritdancer",
        "Once you choose to create a token copy of an enchantment during the resolution of Ondu Spiritdancer's ability, that ability won't trigger again that turn"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ondu Spiritdancer");
    for _ in 0..4 {
        t.answer_yes(P0, true);
    }
    enter_together(&mut t, &[(P0, "Glorious Anthem"), (P0, "Honor of the Pure")]);
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(crate::r_s01_common::tokens(&t, P0).len(), 1);
    t.enter(P0, "Enchantress's Presence");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    t.resolve_all();
    assert_eq!(crate::r_s01_common::tokens(&t, P0).len(), 1);
    // Declining doesn't use it up.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ondu Spiritdancer");
    t.answer_yes(P0, false);
    t.enter(P0, "Glorious Anthem");
    t.resolve_all();
    t.answer_yes(P0, true);
    t.enter(P0, "Honor of the Pure");
    t.resolve_all();
    assert_eq!(crate::r_s01_common::tokens(&t, P0).len(), 1);
}

#[test]
fn copied_enchantments_enters_abilities_work_for_the_token() {
    cr!("707.4", "603.6a", "614.12");
    ruling!(
        "Ondu Spiritdancer",
        "Any enters-the-battlefield abilities of the copied enchantment will trigger when the token enters the battlefield. Any \"as [this enchantment] enters the battlefield\" or \"[this enchantment] enters the battlefield with\" abilities of the enchantment will also work."
    );
    ruling!(
        "Yenna, Redtooth Regent",
        "Any enters-the-battlefield abilities of the copied enchantment will trigger when the token enters the battlefield."
    );
    supported("Yenna, Redtooth Regent");
    // Ondu: a token Banishing Light exiles a second creature.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ondu Spiritdancer");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Gray Ogre");
    t.answer_targets(P0, &[obj(a)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(b)]);
    t.enter(P0, "Banishing Light");
    t.resolve_all();
    assert_eq!(t.zone(a), Zone::Exile);
    assert_eq!(t.zone(b), Zone::Exile);
    // Yenna: the same with her ability.
    let mut t = TestGame::new(2);
    let yenna = t.battlefield(P0, "Yenna, Redtooth Regent");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Gray Ogre");
    t.answer_targets(P0, &[obj(a)]);
    let light = t.enter(P0, "Banishing Light");
    t.resolve_all();
    assert_eq!(t.zone(a), Zone::Exile);
    t.lands(P0, "Plains", 2);
    t.answer_targets(P0, &[obj(b)]);
    activate_containing(&mut t, P0, yenna, "Create a token").unwrap();
    let _ = light;
    t.resolve_all();
    assert_eq!(t.zone(b), Zone::Exile);
    // "As ... enters" works for the token: Story Circle's color choice.
    supported("Story Circle");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ondu Spiritdancer");
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.answer_yes(P0, true);
    let from = t.asked().len();
    t.enter(P0, "Story Circle");
    t.resolve_all();
    let toks = crate::r_s01_common::tokens(&t, P0);
    assert_eq!(toks.len(), 1);
    let colors_asked = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, mtg_engine::decision::Decision::ChooseOption { .. }))
        .count();
    assert_eq!(colors_asked, 2, "{:?}", asked_debug(&t, from));
}

#[test]
fn aura_token_copies_attach_without_targeting() {
    cr!("303.4f", "707.4");
    ruling!(
        "Ondu Spiritdancer",
        "If an Aura is put onto the battlefield without being cast, such as by being created as a token, the Aura's controller-to-be chooses what it will enchant as it enters the battlefield. An Aura put onto the battlefield this way doesn't target anything (so it could be attached to an opponent's permanent with hexproof, for example), but the Aura's enchant ability restricts what it can be attached to. If an Aura token can't legally be attached to anything, it isn't created."
    );
    ruling!(
        "Mythos of Illuna",
        "If an Aura is put onto the battlefield without being cast, such as by being created as a token, the Aura's controller-to-be chooses what it will enchant as it enters the battlefield."
    );
    supported("Mythos of Illuna");
    // Ondu copies Pacifism onto P1's hexproof Gladecover Scout.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ondu Spiritdancer");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let scout = t.battlefield(P1, "Gladecover Scout");
    t.lands(P0, "Plains", 2);
    let pacifism = t.hand(P0, "Pacifism");
    t.cast(P0, pacifism).target(obj(bears)).go();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(scout)]);
    t.resolve_all();
    let toks = crate::r_s01_common::tokens(&t, P0);
    assert_eq!(toks.len(), 1);
    assert_eq!(attached_to(&t, toks[0]), Some(obj(scout)));
    // Nothing left to enchant as the trigger resolves: no token.
    // (With the Spiritdancer left, the token can only enchant it.)
    for ondu_too in [false, true] {
        let mut t = TestGame::new(2);
        let ondu = t.battlefield(P0, "Ondu Spiritdancer");
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.lands(P0, "Plains", 2);
        let pacifism = t.hand(P0, "Pacifism");
        t.cast(P0, pacifism).target(obj(bears)).go();
        t.answer_yes(P0, true);
        t.resolve();
        assert_eq!(t.stack_len(), 1);
        destroy(&mut t, bears);
        if ondu_too {
            destroy(&mut t, ondu);
        }
        t.resolve_all();
        let toks = crate::r_s01_common::tokens(&t, P0);
        if ondu_too {
            assert!(toks.is_empty());
        } else {
            assert_eq!(toks.len(), 1);
            assert_eq!(attached_to(&t, toks[0]), Some(obj(ondu)));
        }
    }
    // Mythos of Illuna copying Pacifism: the copy's controller picks the creature.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let ogre = t.battlefield(P1, "Gray Ogre");
    let aura = attach_new(&mut t, P1, "Pacifism", obj(bears));
    t.lands(P0, "Island", 4);
    let mythos = t.hand(P0, "Mythos of Illuna");
    t.answer_choose(P0, &[obj(ogre)]);
    t.cast(P0, mythos).target(obj(aura)).go();
    t.resolve_all();
    let toks = crate::r_s01_common::tokens(&t, P0);
    assert_eq!(toks.len(), 1);
    assert_eq!(attached_to(&t, toks[0]), Some(obj(ogre)));
}
