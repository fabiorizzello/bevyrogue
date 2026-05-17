use bevyrogue::{
    combat::{
        api::{ExtRegistries, register_kernel_builtins},
        blueprints::agumon::register_agumon_ext,
    },
    data::{
        skill_timeline::compile_skill_book_timelines,
        skills_ron::{SkillBook, validate_skill_book},
    },
};

#[test]
fn inspect() {
    let book: SkillBook = ron::from_str(include_str!("../assets/data/skills.ron")).unwrap();
    validate_skill_book(&book).unwrap();
    let mut regs = ExtRegistries::default();
    register_kernel_builtins(&mut regs);
    register_agumon_ext(&mut regs);
    let compiled = compile_skill_book_timelines(&book, &regs).unwrap();
    println!("len={}", compiled.len());
    for t in compiled {
        println!("{}", t.id);
    }
}
