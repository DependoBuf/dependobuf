use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::ast::parsed as p;
use crate::ast::parsed::definition::Definition;
use crate::error::elaborating::Error;
use crate::error::elaborating::Error::Cycle;

type ModuleRef<'a, Loc, Str> = Vec<&'a Definition<Loc, Str, p::TypeDeclaration<Loc, Str>>>;

/// Topologically sorts declarations in a parsed module
/// # Errors
///  Returns `Err` containing one cycle if the graph has a cycle
pub fn topological_sort<'a, Loc, Str>(
    module: &'a p::Module<Loc, Str>,
) -> Result<ModuleRef<'a, Loc, Str>, Error>
where
    Str: Ord + Clone + ToString,
{
    let declared: BTreeSet<String> = module.iter().map(|def| def.name.to_string()).collect();

    let by_name: BTreeMap<String, &'a Definition<Loc, Str, p::TypeDeclaration<Loc, Str>>> = module
        .iter()
        .map(|def| (def.name.to_string(), def))
        .collect();

    let deps: BTreeMap<String, BTreeSet<String>> = module
        .iter()
        .map(|def| {
            let def_name = def.name.to_string();
            let mut refs = BTreeSet::new();
            for dep in &def.data.dependencies {
                add_ref(&dep.data, &mut refs);
            }
            match &def.data.body {
                p::TypeDefinition::Message(fields) => {
                    for field in fields {
                        add_ref(&field.data, &mut refs);
                    }
                }
                p::TypeDefinition::Enum(branches) => {
                    for branch in branches {
                        for ctor in &branch.constructors {
                            for field in &ctor.data {
                                add_ref(&field.data, &mut refs);
                            }
                        }
                    }
                }
            }
            let node_deps = refs
                .into_iter()
                .filter(|r| declared.contains(r) && *r != def_name)
                .collect();
            (def_name, node_deps)
        })
        .collect();

    let mut in_degree: BTreeMap<String, usize> =
        deps.iter().map(|(k, v)| (k.clone(), v.len())).collect();

    let mut rdeps: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (node, node_deps) in &deps {
        for dep in node_deps {
            rdeps.entry(dep.clone()).or_default().push(node.clone());
        }
    }

    let mut queue: VecDeque<String> = in_degree
        .iter()
        .filter_map(|(k, &v)| if v == 0 { Some(k.clone()) } else { None })
        .collect();

    let mut sorted: Vec<&'a Definition<Loc, Str, p::TypeDeclaration<Loc, Str>>> =
        Vec::with_capacity(module.len());

    while let Some(name) = queue.pop_front() {
        sorted.push(by_name[&name]);

        let Some(dependents) = rdeps.get(&name) else {
            continue;
        };
        let newly_free: Vec<_> = dependents
            .iter()
            .filter_map(|dep| {
                let deg = in_degree.get_mut(dep)?;
                *deg -= 1;
                if *deg == 0 { Some(dep.clone()) } else { None }
            })
            .collect();
        queue.extend(newly_free);
    }

    if sorted.len() == module.len() {
        Ok(sorted)
    } else {
        Err(find_cycle(&deps))
    }
}

/// Returns the names of types that have no initial constructor
#[must_use]
pub fn check_initial_constructors<Loc, Str>(module: &p::Module<Loc, Str>) -> Vec<String>
where
    Str: ToString,
{
    module
        .iter()
        .filter(|def| match &def.data.body {
            p::TypeDefinition::Message(fields) => !fields
                .iter()
                .all(|field| is_self_recursive(&def.name, field)),
            p::TypeDefinition::Enum(branches) => !branches.iter().any(|branch| {
                branch.constructors.iter().any(|ctor| {
                    ctor.data
                        .iter()
                        .all(|field| is_self_recursive(&def.name, field))
                })
            }),
        })
        .map(|def| def.name.to_string())
        .collect()
}

fn is_self_recursive<Loc, Str: ToString>(
    self_name: &Str,
    field: &Definition<Loc, Str, p::TypeExpression<Loc, Str>>,
) -> bool {
    if let p::ExpressionNode::FunCall { fun, .. } = &field.data.node {
        fun.to_string() != self_name.to_string()
    } else {
        true
    }
}

fn add_ref<Loc, Str: ToString>(expr: &p::Expression<Loc, Str>, refs: &mut BTreeSet<String>) {
    if let p::ExpressionNode::FunCall { fun, .. } = &expr.node {
        refs.insert(fun.to_string());
    }
}

fn find_cycle(deps: &BTreeMap<String, BTreeSet<String>>) -> Error {
    let mut visited = BTreeSet::new();
    let mut in_progress = BTreeSet::new();
    let mut path = Vec::new();

    for start in deps.keys() {
        if let Some(cycle) = dfs(
            start.as_str(),
            deps,
            &mut visited,
            &mut in_progress,
            &mut path,
        ) {
            return Cycle(cycle);
        }
    }
    Cycle(vec![])
}

fn dfs<'a>(
    node: &'a str,
    deps: &'a BTreeMap<String, BTreeSet<String>>,
    visited: &mut BTreeSet<&'a str>,
    in_progress: &mut BTreeSet<&'a str>,
    path: &mut Vec<&'a str>,
) -> Option<Vec<String>> {
    if visited.contains(node) {
        return None;
    }
    if in_progress.contains(node) {
        let start = path.iter().position(|&n| n == node).unwrap_or(0);
        return Some(path[start..].iter().map(ToString::to_string).collect());
    }
    in_progress.insert(node);
    path.push(node);
    if let Some(node_deps) = deps.get(node) {
        for dep in node_deps {
            if let Some(cycle) = dfs(dep.as_str(), deps, visited, in_progress, path) {
                return Some(cycle);
            }
        }
    }
    path.pop();
    in_progress.remove(node);
    visited.insert(node);
    None
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::ast::parsed::{ExpressionNode, TypeDeclaration, TypeDefinition};
    use crate::error::elaborating::Error::Cycle;

    type Loc = ();
    type Str = String;

    fn fun_call(name: &str) -> p::TypeExpression<Loc, Str> {
        p::Expression {
            loc: (),
            node: ExpressionNode::FunCall {
                fun: name.to_string(),
                args: Arc::from(vec![].into_boxed_slice()),
            },
        }
    }

    fn make_decl(
        name: &str,
        dep_types: &[&str],
        field_types: &[&str],
    ) -> Definition<Loc, Str, TypeDeclaration<Loc, Str>> {
        let dependencies = dep_types
            .iter()
            .enumerate()
            .map(|(i, ty)| Definition {
                loc: (),
                name: format!("d{i}"),
                data: fun_call(ty),
            })
            .collect();
        let fields = field_types
            .iter()
            .enumerate()
            .map(|(i, ty)| Definition {
                loc: (),
                name: format!("f{i}"),
                data: fun_call(ty),
            })
            .collect();
        Definition {
            loc: (),
            name: name.to_string(),
            data: TypeDeclaration {
                dependencies,
                body: TypeDefinition::Message(fields),
            },
        }
    }

    fn names<'a>(sorted: &[&'a Definition<Loc, Str, TypeDeclaration<Loc, Str>>]) -> Vec<&'a str> {
        sorted.iter().map(|d| d.name.as_str()).collect()
    }

    fn pos(sorted: &[&Definition<Loc, Str, TypeDeclaration<Loc, Str>>], name: &str) -> usize {
        sorted.iter().position(|d| d.name == name).unwrap()
    }

    #[test]
    fn no_deps() {
        let module = vec![make_decl("A", &[], &[]), make_decl("B", &[], &[])];
        let sorted = topological_sort(&module).unwrap();
        let ns = names(&sorted);
        assert!(ns.contains(&"A") && ns.contains(&"B"));
    }

    #[test]
    fn linear_field_deps() {
        let module = vec![
            make_decl("C", &[], &["B"]),
            make_decl("B", &[], &["A"]),
            make_decl("A", &[], &[]),
        ];
        let sorted = topological_sort(&module).unwrap();
        assert!(pos(&sorted, "A") < pos(&sorted, "B"));
        assert!(pos(&sorted, "B") < pos(&sorted, "C"));
    }

    #[test]
    fn linear_param_deps() {
        let module = vec![
            make_decl("C", &["B"], &[]),
            make_decl("B", &["A"], &[]),
            make_decl("A", &[], &[]),
        ];
        let sorted = topological_sort(&module).unwrap();
        assert!(pos(&sorted, "A") < pos(&sorted, "B"));
        assert!(pos(&sorted, "B") < pos(&sorted, "C"));
    }

    #[test]
    fn diamond() {
        let module = vec![
            make_decl("D", &[], &["B", "C"]),
            make_decl("B", &[], &["A"]),
            make_decl("C", &[], &["A"]),
            make_decl("A", &[], &[]),
        ];
        let sorted = topological_sort(&module).unwrap();
        assert!(pos(&sorted, "A") < pos(&sorted, "B"));
        assert!(pos(&sorted, "A") < pos(&sorted, "C"));
        assert!(pos(&sorted, "B") < pos(&sorted, "D"));
        assert!(pos(&sorted, "C") < pos(&sorted, "D"));
    }

    #[test]
    fn cycle_detected() {
        let module = vec![make_decl("A", &[], &["B"]), make_decl("B", &[], &["A"])];
        let Cycle(cycle) = topological_sort(&module).unwrap_err() else {
            panic!("expected Cycle");
        };
        assert!(cycle.contains(&"A".to_string()));
        assert!(cycle.contains(&"B".to_string()));
    }

    #[test]
    fn self_reference_not_a_cycle() {
        let module = vec![make_decl("List", &[], &["List"])];
        assert!(topological_sort(&module).is_ok());
    }

    #[test]
    fn external_type_refs_ignored() {
        let module = vec![
            make_decl("A", &[], &["UInt"]),
            make_decl("B", &[], &["UInt"]),
        ];
        let sorted = topological_sort(&module).unwrap();
        assert_eq!(sorted.len(), 2);
    }

    fn make_enum_decl(
        name: &str,
        constructors: &[&[&str]],
    ) -> Definition<Loc, Str, TypeDeclaration<Loc, Str>> {
        use crate::ast::parsed::{EnumBranch, Pattern};
        let branch = EnumBranch {
            patterns: Vec::<Pattern<Loc, Str>>::new(),
            constructors: constructors
                .iter()
                .enumerate()
                .map(|(ci, field_types)| {
                    let fields = field_types
                        .iter()
                        .enumerate()
                        .map(|(fi, ty)| Definition {
                            loc: (),
                            name: format!("f{fi}"),
                            data: fun_call(ty),
                        })
                        .collect();
                    Definition {
                        loc: (),
                        name: format!("C{ci}"),
                        data: fields,
                    }
                })
                .collect(),
        };
        Definition {
            loc: (),
            name: name.to_string(),
            data: TypeDeclaration {
                dependencies: vec![],
                body: TypeDefinition::Enum(vec![branch]),
            },
        }
    }

    #[test]
    fn message_no_fields_has_initial() {
        let module = vec![make_decl("Unit", &[], &[])];
        assert!(check_initial_constructors(&module).is_empty());
    }

    #[test]
    fn message_nonrecursive_fields_has_initial() {
        let module = vec![make_decl("Wrap", &[], &["Int"])];
        assert!(check_initial_constructors(&module).is_empty());
    }

    #[test]
    fn message_fully_recursive_no_initial() {
        let module = vec![make_decl("Inf", &[], &["Inf"])];
        let errs = check_initial_constructors(&module);
        assert_eq!(errs.len(), 1);
        assert_eq!(errs[0], "Inf");
    }

    #[test]
    fn message_mixed_fields_no_initial() {
        let module = vec![make_decl("A", &[], &["Int", "A"])];
        let errs = check_initial_constructors(&module);
        assert_eq!(errs.len(), 1);
    }

    #[test]
    fn enum_with_base_case_has_initial() {
        let module = vec![make_enum_decl("Nat", &[&[], &["Nat"]])];
        assert!(check_initial_constructors(&module).is_empty());
    }

    #[test]
    fn enum_all_recursive_no_initial() {
        let module = vec![make_enum_decl("Bad", &[&["Bad"]])];
        let errs = check_initial_constructors(&module);
        assert_eq!(errs.len(), 1);
        assert_eq!(errs[0], "Bad");
    }

    #[test]
    fn multiple_types_reports_all_bad() {
        let module = vec![
            make_decl("Good", &[], &["Int"]),
            make_decl("Bad1", &[], &["Bad1"]),
            make_enum_decl("Bad2", &[&["Bad2"]]),
        ];
        let errs = check_initial_constructors(&module);
        assert!(errs.contains(&"Bad1".to_string()));
        assert!(errs.contains(&"Bad2".to_string()));
        assert!(!errs.contains(&"Good".to_string()));
    }
}
