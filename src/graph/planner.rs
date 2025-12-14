use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

use crate::ast::hex_path::HexPath;
use crate::ast::hexmake_file::{HexRule, HexmakeFile, RuleName};
use crate::graph::task::Task;

/// Make a plan for building the given targets.
/// The targets can be either the names of outputs or
/// the names of rules.
pub fn plan_build(hex_file: &HexmakeFile, targets: &Vec<Arc<String>>) -> Result<BuildPlan, String> {
    Planner::new(hex_file).plan(targets)
}

pub struct BuildPlan {
    #[allow(unused)]
    pub target_rules: BTreeSet<RuleName>,
    pub tasks: BTreeMap<RuleName, Arc<Mutex<Task>>>,
}

struct Planner {
    target_rules: BTreeSet<RuleName>,
    rule_map: BTreeMap<RuleName, Arc<HexRule>>,
    rule_by_output: BTreeMap<HexPath, RuleName>,
    task_for_rule: BTreeMap<RuleName, Arc<Mutex<Task>>>,
}

impl Planner {
    fn new(hex_file: &HexmakeFile) -> Self {
        let target_rules: BTreeSet<RuleName> = BTreeSet::new();
        let mut rule_map = BTreeMap::new();
        let mut rule_by_output = BTreeMap::new();

        for rule in &hex_file.rules {
            rule_map.insert(rule.name.clone(), rule.clone());
            for output in &rule.outputs {
                rule_by_output.insert(output.clone(), rule.name.clone());
            }
        }

        let task_for_rule = BTreeMap::new();
        Self {
            target_rules,
            rule_map,
            rule_by_output,
            task_for_rule,
        }
    }

    fn plan(mut self, targets: &Vec<Arc<String>>) -> Result<BuildPlan, String> {
        for target in targets {
            let target_rule_name = self.plan_one_target(target, &BTreeSet::new())?;
            self.target_rules.insert(target_rule_name);
        }

        Ok(BuildPlan {
            target_rules: self.target_rules,
            tasks: self.task_for_rule,
        })
    }

    /// Plan the build for one target, updating the fields of the
    /// planner as it goes. Return the rule name for building the
    /// one requested target.
    fn plan_one_target(
        &mut self,
        target: &Arc<String>,
        targets_in_progress: &BTreeSet<RuleName>,
    ) -> Result<RuleName, String> {
        let target_as_path = HexPath::try_from(target.as_str()).unwrap();
        let rule_name = if target_as_path.is_output() {
            // It's an output. Find the rule that goes with it.
            match self.rule_by_output.get(&target_as_path) {
                Some(rule_name) => rule_name.clone(),
                None => return Err(format!("No rule exists to build `{target}`")),
            }
        } else {
            // If it's not an output, it must be a rule name
            RuleName::from(target)
        };

        if targets_in_progress.contains(&rule_name) {
            return Err(format!("Rule cycle involving rule `{rule_name}`"));
        }
        let mut targets_in_progress = targets_in_progress.clone();
        targets_in_progress.insert(rule_name.clone());

        if self.task_for_rule.contains_key(&rule_name) {
            // There's already a task for this rule. There's nothing
            // more to do.
            return Ok(rule_name);
        }

        // Make a new task
        let rule = match self.rule_map.get(&rule_name) {
            Some(rule) => rule.clone(),
            None => return Err(format!("No rule exists named `{rule_name}`")),
        };
        let task = Arc::new(Mutex::new(Task::new(rule.clone())));

        // Add subtasks for inputs
        for input in &rule.inputs {
            if input.is_output() {
                let input_rule_name = self.plan_one_target(&input.path, &targets_in_progress)?;
                let sub_task = &self.task_for_rule[&input_rule_name];
                Task::add_dependency(&task, sub_task);
            }
        }

        self.task_for_rule.insert(rule_name.clone(), task);

        Ok(rule_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::hexmake_file::{HexRule, HexmakeFile};
    use indoc::indoc;
    use itertools::join;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_basics() {
        let hexmake_file = foo_bar_hexmake_file();

        let build_plan = plan_build(
            &hexmake_file,
            &vec!["foo".to_string().into(), "bar".to_string().into()],
        );

        assert_eq!(
            build_plan_summary(&build_plan),
            indoc! {r"
              Task: bar
                Depends on tasks: bar.o
              Task: bar.o
                Used by tasks: bar
              Task: foo
                Depends on tasks: foo.o
              Task: foo.o
                Used by tasks: foo
            "}
        );

        check_build_plan(&build_plan)
    }

    #[test]
    fn test_reuse_tasks() {
        let hexmake_file = foo_bar_hexmake_file();

        // foo.o and then foo
        let build_plan = plan_build(
            &hexmake_file,
            &vec!["foo.o".to_string().into(), "foo".to_string().into()],
        );

        assert_eq!(
            build_plan_summary(&build_plan),
            indoc! {r"
              Task: foo
                Depends on tasks: foo.o
              Task: foo.o
                Used by tasks: foo
            "}
        );

        check_build_plan(&build_plan)
    }

    #[test]
    fn test_skip_top_level_tasks_already_made() {
        let hexmake_file = foo_bar_hexmake_file();

        // foo and then foo.o
        let build_plan = plan_build(
            &hexmake_file,
            &vec!["foo".to_string().into(), "foo.o".to_string().into()],
        );
        assert_eq!(
            build_plan_summary(&build_plan),
            indoc! {r"
              Task: foo
                Depends on tasks: foo.o
              Task: foo.o
                Used by tasks: foo
            "}
        );

        check_build_plan(&build_plan)
    }

    #[test]
    fn test_rule_with_multiple_outputs() {
        let hexmake_file = HexmakeFile {
            environ: vec![],
            rules: vec![
                HexRule {
                    name: "foo".into(),
                    outputs: vec![HexPath::try_from("out/foo").unwrap()],
                    inputs: vec![
                        HexPath::try_from("out/foo.c").unwrap(),
                        HexPath::try_from("out/bar.c").unwrap(),
                    ],
                    commands: vec!["gcc -o out/foo out/foo.c out/bar.c".into()],
                }
                .into(),
                HexRule {
                    name: "gensources".into(),
                    outputs: vec![
                        HexPath::try_from("out/foo.c").unwrap(),
                        HexPath::try_from("out/bar.c").unwrap(),
                    ],
                    inputs: vec![],
                    commands: vec!["scripts/gensources".into()],
                }
                .into(),
            ],
        };

        let build_plan = plan_build(&hexmake_file, &vec!["foo".to_string().into()]);

        assert_eq!(
            build_plan_summary(&build_plan),
            indoc! {r"
              Task: foo
                Depends on tasks: gensources
              Task: gensources
                Used by tasks: foo
            "}
        );

        check_build_plan(&build_plan);
    }

    #[test]
    fn test_no_such_output() {
        let hexmake_file = foo_bar_hexmake_file();

        let build_plan = plan_build(&hexmake_file, &vec!["out/bogus".to_string().into()]);

        assert_eq!(
            build_plan_summary(&build_plan),
            "No rule exists to build `out/bogus`"
        );

        check_build_plan(&build_plan)
    }

    #[test]
    fn test_no_such_rule() {
        let hexmake_file = foo_bar_hexmake_file();

        let build_plan = plan_build(&hexmake_file, &vec!["bogus".to_string().into()]);

        assert_eq!(
            build_plan_summary(&build_plan),
            "No rule exists named `bogus`"
        );

        check_build_plan(&build_plan)
    }

    #[test]
    fn test_cycle() {
        let hexmake_file = HexmakeFile {
            environ: vec![],
            rules: vec![
                HexRule {
                    name: "foo".into(),
                    outputs: vec![HexPath::try_from("out/foo").unwrap()],
                    inputs: vec![HexPath::try_from("out/bar").unwrap()],
                    commands: vec!["echo foo".into()],
                }
                .into(),
                HexRule {
                    name: "bar".into(),
                    outputs: vec![HexPath::try_from("out/bar").unwrap()],
                    inputs: vec![HexPath::try_from("out/foo").unwrap()],
                    commands: vec!["echo bar".into()],
                }
                .into(),
            ],
        };

        let build_plan = plan_build(&hexmake_file, &vec!["foo".to_string().into()]);

        assert_eq!(
            build_plan_summary(&build_plan),
            "Rule cycle involving rule `foo`"
        );
    }

    /// A Hexmake file that compiles two C files into two binaries
    fn foo_bar_hexmake_file() -> HexmakeFile {
        HexmakeFile {
            environ: vec![],
            rules: vec![
                HexRule {
                    name: "foo".into(),
                    outputs: vec![HexPath::try_from("out/foo").unwrap()],
                    inputs: vec![HexPath::try_from("out/foo.o").unwrap()],
                    commands: vec!["gcc -o out/foo out/foo.o".into()],
                }
                .into(),
                HexRule {
                    name: "foo.o".into(),
                    outputs: vec![HexPath::try_from("out/foo.o").unwrap()],
                    inputs: vec![HexPath::try_from("foo.c").unwrap()],
                    commands: vec!["gcc -o out/foo.o out/foo.c".into()],
                }
                .into(),
                HexRule {
                    name: "bar".into(),
                    outputs: vec![HexPath::try_from("out/bar").unwrap()],
                    inputs: vec![HexPath::try_from("out/bar.o").unwrap()],
                    commands: vec!["gcc -o out/bar out/bar.o".into()],
                }
                .into(),
                HexRule {
                    name: "bar.o".into(),
                    outputs: vec![HexPath::try_from("out/bar.o").unwrap()],
                    inputs: vec![HexPath::try_from("bar.c").unwrap()],
                    commands: vec!["gcc -o out/bar.o out/bar.c".into()],
                }
                .into(),
            ],
        }
    }

    /// Generate a string summary of a build plan for testing
    fn build_plan_summary(build_plan: &Result<BuildPlan, String>) -> String {
        let build_plan = match build_plan {
            Ok(build_plan) => build_plan,
            Err(error) => return error.clone(),
        };

        let mut result = String::new();
        for task in build_plan.tasks.values() {
            let task = task.lock().unwrap();
            result.push_str(&format!("Task: {}\n", task.rule_name()));
            if !task.depends_on.is_empty() {
                result.push_str(&format!(
                    "  Depends on tasks: {}\n",
                    task_list_summary(&task.depends_on),
                ));
            }
            if !task.used_by.is_empty() {
                result.push_str(&format!(
                    "  Used by tasks: {}\n",
                    task_list_summary(&task.used_by)
                ));
            }
        }
        result
    }

    /// Summarize a list of tasks by combining their rule names between commas
    fn task_list_summary(tasks: &[Arc<Mutex<Task>>]) -> String {
        join(tasks.iter().map(|t| t.lock().unwrap().rule_name()), ", ")
    }

    /// Internal consistency checks for a build plan
    #[track_caller]
    fn check_build_plan(build_plan: &Result<BuildPlan, String>) {
        let build_plan = match build_plan {
            Ok(build_plan) => build_plan,
            Err(_error) => return,
        };

        for task in build_plan.tasks.values() {
            let rule_name = { task.lock().unwrap().rule_name() };

            // Check that deps and used_by are inverses
            for dep in &{ task.lock().unwrap().depends_on.clone() } {
                assert!(
                    dep.lock()
                        .unwrap()
                        .used_by
                        .iter()
                        .any(|t| t.lock().unwrap().rule_name() == rule_name)
                );
            }

            for used_by in &{ task.lock().unwrap().used_by.clone() } {
                assert!(
                    used_by.lock().unwrap().depends_on.iter().any(|t| t
                        .lock()
                        .unwrap()
                        .rule_name()
                        == rule_name)
                );
            }
        }
    }
}
