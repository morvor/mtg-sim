//! CR 612: text-changing effects (and layer 3, CR 613.1c).

use crate::r609_common::*;
use mtg_engine::ability::*;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::text_change::TextWords;
use mtg_engine::types::*;
use mtg_engine::*;

fn change_text(words: TextWords, target: TargetSpec) -> Body {
    Body::simple(
        vec![target],
        Effect::ChangeText {
            what: Sel::Target(0),
            words,
            exclude: vec![],
            duration: Duration::Permanent,
        },
    )
}

fn target_permanent() -> TargetSpec {
    TargetSpec::object(Filter::Permanent, "target permanent")
}

fn target_spell() -> TargetSpec {
    TargetSpec {
        what: TargetKind::Spell(Filter::Any),
        ..TargetSpec::object(Filter::Any, "target spell")
    }
}

/// Answers the two word choices by index into the offered lists.
fn choose_words(t: &mut TestGame, p: PlayerId, from: usize, to: usize) {
    t.answer(p, DecisionKind::Option, Answer::Index(from));
    t.answer(p, DecisionKind::Option, Answer::Index(to));
}

/// Index of a color in the color word list, and in the list without `from`.
fn color_idx(c: Color, without: Option<Color>) -> usize {
    Color::ALL
        .iter()
        .filter(|x| Some(**x) != without)
        .position(|x| *x == c)
        .unwrap()
}

fn protection_filter(t: &TestGame, id: ObjectId) -> Vec<String> {
    t.obj_now(id)
        .chars
        .keywords()
        .filter(|k| k.kind == KeywordKind::Protection)
        .map(|k| format!("{:?}", k.filter))
        .collect()
}

#[test]
fn color_word_change_changes_protection_not_the_name() {
    // CR 612.1, 612.2: Sleight of Mind-style change from "black" to "red" on White
    // Knight (protection from black). The color word in its name isn't changed.
    cr!("612.1", "612.2", "613.1c");
    ruling!("Sleight of Mind", "You can’t Sleight proper nouns");
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P1, "White Knight");
    let red = t.battlefield(P1, "Raging Goblin");
    let black = t.custom(
        P1,
        creature("Black Bear", 2, 2, &[Color::Black]),
        Zone::Battlefield,
    );
    assert!(t.g.protected_from(knight, black));
    assert!(!t.g.protected_from(knight, red));
    choose_words(
        &mut t,
        P0,
        color_idx(Color::Black, None),
        color_idx(Color::Red, Some(Color::Black)),
    );
    cast_resolve(
        &mut t,
        P0,
        change_text(TextWords::Color, target_permanent()),
        &[Entity::Object(knight)],
    );
    assert_eq!(t.obj_now(knight).chars.name, "White Knight");
    assert_eq!(
        protection_filter(&t, knight),
        vec!["Some(Color(Red))".to_string()]
    );
    assert!(t.g.protected_from(knight, red));
    assert!(!t.g.protected_from(knight, black));
}

#[test]
fn land_type_change_changes_the_type_line_and_landwalk() {
    // CR 612.1: a text-changing effect can change the type line; a Forest whose "Forest"
    // becomes "Island" is an Island (and taps for {U}), but is still named Forest
    // (CR 612.2). Forestwalk becomes islandwalk.
    cr!("612.1", "612.2", "613.1c");
    ruling!(
        "Magical Hack",
        "Forest can be changed to Island so it produces blue mana"
    );
    let lands = ["Plains", "Island", "Swamp", "Mountain", "Forest"];
    let idx = |w: &str, without: Option<&str>| {
        lands
            .iter()
            .filter(|x| Some(**x) != without)
            .position(|x| *x == w)
            .unwrap()
    };
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    choose_words(
        &mut t,
        P0,
        idx("Forest", None),
        idx("Island", Some("Forest")),
    );
    cast_resolve(
        &mut t,
        P0,
        change_text(TextWords::BasicLandType, target_permanent()),
        &[Entity::Object(forest)],
    );
    let ch = &t.obj_now(forest).chars;
    assert_eq!(ch.name, "Forest");
    assert!(ch.has_subtype("Island"));
    assert!(!ch.has_subtype("Forest"));
    assert!(ch.abilities.iter().any(|a| a.text == "{T}: Add {U}."));
    assert!(!ch.abilities.iter().any(|a| a.text == "{T}: Add {G}."));

    let dryads = t.battlefield(P0, "Shanodin Dryads");
    let walk = |t: &TestGame| {
        t.obj_now(dryads)
            .chars
            .keywords()
            .filter(|k| k.kind == KeywordKind::Landwalk)
            .map(|k| format!("{:?}", k.filter))
            .collect::<Vec<_>>()
    };
    assert_eq!(walk(&t), vec!["Some(Subtype(\"Forest\"))".to_string()]);
    choose_words(
        &mut t,
        P0,
        idx("Forest", None),
        idx("Island", Some("Forest")),
    );
    cast_resolve(
        &mut t,
        P0,
        change_text(TextWords::BasicLandType, target_permanent()),
        &[Entity::Object(dryads)],
    );
    assert_eq!(walk(&t), vec!["Some(Subtype(\"Island\"))".to_string()]);
}

#[test]
fn changing_a_spells_text_can_make_its_target_illegal() {
    // CR 612.1: text-changing effects can apply to spells. Changing Doom Blade's
    // "nonblack" to "nonwhite" doesn't change its target, which is now illegal.
    cr!("612.1");
    ruling!(
        "Sleight of Mind",
        "Changing the text of a spell will not allow you to change the targets"
    );
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P1, "Serra Angel");
    t.lands(P0, "Swamp", 2);
    let blade = t.hand(P0, "Doom Blade");
    let spell = t.cast(P0, blade).target(angel).go();
    choose_words(
        &mut t,
        P1,
        color_idx(Color::Black, None),
        color_idx(Color::White, Some(Color::Black)),
    );
    let card = t.custom(
        P1,
        instant("Sleight", change_text(TextWords::Color, target_spell())),
        Zone::Hand(P1),
    );
    t.cast_with(P1, card, &[Entity::Object(spell)]).unwrap();
    t.resolve();
    t.resolve();
    assert!(t.on_battlefield(angel));
    assert!(t.in_graveyard(P0, "Doom Blade"));
}

#[test]
fn text_change_on_a_permanent_spell_stays_with_the_permanent() {
    cr!("612.1");
    ruling!(
        "Sleight of Mind",
        "If you change the text of a spell which is to become a permanent"
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    let knight = t.hand(P0, "White Knight");
    let spell = t.cast(P0, knight).go();
    choose_words(
        &mut t,
        P0,
        color_idx(Color::Black, None),
        color_idx(Color::Green, Some(Color::Black)),
    );
    let card = t.custom(
        P0,
        instant("Sleight", change_text(TextWords::Color, target_spell())),
        Zone::Hand(P0),
    );
    t.cast_with(P0, card, &[Entity::Object(spell)]).unwrap();
    t.resolve();
    t.resolve();
    let k = t.named_on_battlefield("White Knight")[0];
    assert_eq!(
        protection_filter(&t, k),
        vec!["Some(Color(Green))".to_string()]
    );
}

#[test]
fn token_names_that_are_creature_types_can_change() {
    // CR 612.2a: a spell that creates a Squirrel token uses "Squirrel" as both its name
    // and creature type; changing the creature type in the spell's text changes both.
    cr!("612.2a", "612.2");
    let creature_types = &subtype_lists().creature;
    let from = creature_types.iter().position(|x| x == "Squirrel").unwrap();
    let to = creature_types
        .iter()
        .filter(|x| *x != "Squirrel")
        .position(|x| x == "Goblin")
        .unwrap();
    let mut t = TestGame::new(2);
    let spec = TokenSpec {
        name: "Squirrel".into(),
        colors: colors(&[Color::Green]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: vec!["Squirrel".into()],
        power: Some(1),
        toughness: Some(1),
        abilities: vec![],
        scryfall_name: None,
    };
    let make = t.custom(
        P0,
        instant(
            "Squirrel Call",
            Body::effect(Effect::CreateToken {
                spec,
                count: Value::c(1),
                controller: PlayerRef::You,
                tapped: false,
                attacking: false,
            }),
        ),
        Zone::Hand(P0),
    );
    let spell = t.cast_with(P0, make, &[]).unwrap();
    choose_words(&mut t, P0, from, to);
    let card = t.custom(
        P0,
        instant(
            "Evolution",
            change_text(TextWords::CreatureType, target_spell()),
        ),
        Zone::Hand(P0),
    );
    t.cast_with(P0, card, &[Entity::Object(spell)]).unwrap();
    t.resolve_all();
    let goblins = t.named_on_battlefield("Goblin");
    assert_eq!(goblins.len(), 1);
    assert!(t.obj_now(goblins[0]).chars.has_subtype("Goblin"));
    assert!(!t.obj_now(goblins[0]).chars.has_subtype("Squirrel"));
    assert!(t.named_on_battlefield("Squirrel").is_empty());
}

fn aura_granting(name: &str, mods: Vec<Modification>) -> CardDef {
    let mut enchant = Keyword::new(KeywordKind::Enchant);
    enchant.filter = Some(Filter::creature());
    let mut d = permanent(
        name,
        &[CardType::Enchantment],
        vec![
            AbilityDef::new(AbilityKind::Keyword(enchant), "Enchant creature"),
            continuous(Filter::AttachedToSource, mods),
        ],
    );
    d.faces[0].chars.subtypes.push("Aura".into());
    d
}

#[test]
fn granted_abilities_are_not_changed() {
    // CR 612.3: abilities granted by other effects aren't part of the object's text, so
    // changing the creature's text doesn't change them; changing the Aura's text does.
    cr!("612.3");
    let mut t = TestGame::new(2);
    let c = t.custom(
        P0,
        creature("Bear", 2, 2, &[Color::Green]),
        Zone::Battlefield,
    );
    let mut prot = Keyword::new(KeywordKind::Protection);
    prot.filter = Some(Filter::Color(Color::Black));
    let aura = t.custom(
        P0,
        aura_granting("Ward of Night", vec![Modification::AddKeyword(prot)]),
        Zone::Battlefield,
    );
    assert!(t.g.attach(aura, Entity::Object(c)));
    t.recompute();
    assert_eq!(
        protection_filter(&t, c),
        vec!["Some(Color(Black))".to_string()]
    );
    choose_words(
        &mut t,
        P0,
        color_idx(Color::Black, None),
        color_idx(Color::Red, Some(Color::Black)),
    );
    cast_resolve(
        &mut t,
        P0,
        change_text(TextWords::Color, target_permanent()),
        &[Entity::Object(c)],
    );
    assert_eq!(
        protection_filter(&t, c),
        vec!["Some(Color(Black))".to_string()]
    );
    choose_words(
        &mut t,
        P0,
        color_idx(Color::Black, None),
        color_idx(Color::Red, Some(Color::Black)),
    );
    cast_resolve(
        &mut t,
        P0,
        change_text(TextWords::Color, target_permanent()),
        &[Entity::Object(aura)],
    );
    assert_eq!(
        protection_filter(&t, c),
        vec!["Some(Color(Red))".to_string()]
    );
}

#[test]
fn token_subtypes_and_rules_text_can_change() {
    // CR 612.4: a token's subtypes and rules text are defined by the effect that created
    // it; text-changing effects that affect the token can change them (but not its name,
    // CR 612.2).
    cr!("612.4");
    let mut t = TestGame::new(2);
    let mut walk = Keyword::new(KeywordKind::Landwalk);
    walk.filter = Some(Filter::Subtype("Mountain".into()));
    walk.text = Some("Mountainwalk".into());
    let spec = TokenSpec {
        name: "Goblin".into(),
        colors: colors(&[Color::Red]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: vec!["Goblin".into()],
        power: Some(1),
        toughness: Some(1),
        abilities: vec![AbilityDef::new(AbilityKind::Keyword(walk), "Mountainwalk")],
        scryfall_name: None,
    };
    resolve_effect(
        &mut t,
        P0,
        vec![],
        &[],
        Effect::CreateToken {
            spec,
            count: Value::c(1),
            controller: PlayerRef::You,
            tapped: false,
            attacking: false,
        },
    );
    let tok = t.named_on_battlefield("Goblin")[0];
    let lands = ["Plains", "Island", "Swamp", "Mountain", "Forest"];
    choose_words(&mut t, P0, 3, 3);
    let _ = lands;
    cast_resolve(
        &mut t,
        P0,
        change_text(TextWords::BasicLandType, target_permanent()),
        &[Entity::Object(tok)],
    );
    let walks: Vec<String> = t
        .obj_now(tok)
        .chars
        .keywords()
        .filter(|k| k.kind == KeywordKind::Landwalk)
        .map(|k| format!("{:?}", k.filter))
        .collect();
    assert_eq!(walks, vec!["Some(Subtype(\"Forest\"))".to_string()]);
    // The creature types on the object are offered first.
    let creature_types = &subtype_lists().creature;
    let to = creature_types
        .iter()
        .filter(|x| *x != "Goblin")
        .position(|x| x == "Elf")
        .unwrap();
    choose_words(&mut t, P0, 0, to);
    cast_resolve(
        &mut t,
        P0,
        change_text(TextWords::CreatureType, target_permanent()),
        &[Entity::Object(tok)],
    );
    let ch = &t.obj_now(tok).chars;
    assert!(ch.has_subtype("Elf"));
    assert!(!ch.has_subtype("Goblin"));
    assert_eq!(ch.name, "Goblin");
}

#[test]
fn exchanging_text_boxes() {
    // CR 612.5: exchanging the text boxes of two creatures replaces each one's rules text
    // with the other's. Granted abilities aren't exchanged (Exchange of Words ruling), and
    // once exchanged, one creature leaving doesn't affect the other.
    cr!("612.5");
    ruling!(
        "Exchange of Words",
        "either of the two creatures leaving the battlefield has no effect"
    );
    let mut t = TestGame::new(2);
    let flier = t.custom(
        P0,
        creature_with("Flier", 1, 1, &[], vec![keyword(KeywordKind::Flying)]),
        Zone::Battlefield,
    );
    let lifer = t.custom(
        P1,
        creature_with("Lifer", 3, 3, &[], vec![keyword(KeywordKind::Lifelink)]),
        Zone::Battlefield,
    );
    modify_target(
        &mut t,
        P0,
        lifer,
        vec![Modification::AddKeyword(Keyword::new(KeywordKind::Haste))],
    );
    let words = t.custom(
        P0,
        permanent("Exchange of Words", &[CardType::Enchantment], vec![]),
        Zone::Battlefield,
    );
    let mut ctx = mtg_engine::eval::Ctx::new(Some(words), P0);
    ctx.targets = vec![vec![Entity::Object(flier)], vec![Entity::Object(lifer)]];
    t.g.exec(
        &Effect::Modify {
            what: Sel::AllTargets,
            mods: vec![Modification::ExchangeText],
            duration: Duration::WhileSourceOnBattlefield,
        },
        &mut ctx,
    );
    t.recompute();
    assert!(has_kw(&t, flier, KeywordKind::Lifelink));
    assert!(!has_kw(&t, flier, KeywordKind::Flying));
    assert!(has_kw(&t, lifer, KeywordKind::Flying));
    assert!(!has_kw(&t, lifer, KeywordKind::Lifelink));
    // The granted haste stays on Lifer; P/T aren't text.
    assert!(has_kw(&t, lifer, KeywordKind::Haste));
    assert!(!has_kw(&t, flier, KeywordKind::Haste));
    assert_eq!(t.pt(flier), (1, 1));
    // One creature leaving doesn't give the other its text back.
    t.g.move_object(
        lifer,
        Zone::Graveyard(P1),
        mtg_engine::events::MoveCause::Effect,
        None,
    );
    t.recompute();
    assert!(has_kw(&t, flier, KeywordKind::Lifelink));
    // The exchange ends when its source leaves the battlefield.
    t.g.move_object(
        words,
        Zone::Graveyard(P0),
        mtg_engine::events::MoveCause::Effect,
        None,
    );
    t.recompute();
    assert!(has_kw(&t, flier, KeywordKind::Flying));
}

#[test]
fn full_text_of_another_card() {
    // CR 612.6: "has the full text of" changes the name, mana cost, color indicator, type
    // line, rules text, power, and toughness (Volrath's Shapeshifter). Its color follows.
    cr!("612.6");
    ruling!(
        "Volrath's Shapeshifter",
        "including color and any other types"
    );
    let mut t = TestGame::new(2);
    let top = Sel::TopOfGraveyard(PlayerRef::You);
    let discard = activated(
        Cost::mana(mtg_engine::mana::ManaCost::parse("{2}").unwrap()),
        Body::effect(Effect::Discard {
            who: PlayerRef::You,
            n: Value::c(1),
            random: false,
            filter: Filter::Any,
        }),
    );
    let shifter = t.custom(
        P0,
        creature_with(
            "Shapeshifter",
            0,
            1,
            &[Color::Blue],
            vec![continuous_if(
                Condition::SelMatches(top.clone(), Filter::creature()),
                Filter::Source,
                vec![
                    Modification::FullTextOf(Box::new(top)),
                    Modification::AddAbility(discard),
                ],
            )],
        ),
        Zone::Battlefield,
    );
    assert_eq!(t.pt(shifter), (0, 1));
    t.graveyard(P0, "Serra Angel");
    t.recompute();
    let o = t.obj_now(shifter);
    assert_eq!(o.chars.name, "Serra Angel");
    assert_eq!((o.power(), o.toughness()), (4, 4));
    assert_eq!(o.chars.colors, colors(&[Color::White]));
    assert!(o.has_keyword(KeywordKind::Flying));
    assert!(o.has_keyword(KeywordKind::Vigilance));
    assert_eq!(t.g.mana_value_of(shifter), 5);
    // It keeps the discard ability.
    assert!(o
        .chars
        .abilities
        .iter()
        .any(|a| matches!(a.kind, AbilityKind::Activated(_))));
    // A noncreature card on top: it's itself again.
    t.graveyard(P0, "Lightning Bolt");
    t.recompute();
    assert_eq!(t.obj_now(shifter).chars.name, "Shapeshifter");
    assert_eq!(t.pt(shifter), (0, 1));
}

#[test]
fn setting_a_name() {
    // CR 612.8: an effect that sets an object's name: it has only that name.
    cr!("612.8");
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    modify_target(&mut t, P0, bear, vec![Modification::SetName("Bob".into())]);
    let ctx = mtg_engine::eval::Ctx::new(None, P0);
    assert!(t.g.matches(bear, &Filter::Named("Bob".into()), &ctx));
    assert!(!t
        .g
        .matches(bear, &Filter::Named("Grizzly Bears".into()), &ctx));
}
