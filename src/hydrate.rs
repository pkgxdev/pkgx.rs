use crate::types::PackageReq;
use libsemverator::range::Range as VersionReq;
use std::collections::{HashMap, HashSet};
use std::error::Error;

#[derive(Clone)]
struct Node {
    parent: Option<Box<Node>>,
    pkg: PackageReq,
    children: HashSet<String>,
}

impl Node {
    fn new(pkg: PackageReq, parent: Option<Box<Node>>) -> Self {
        Self {
            parent,
            pkg,
            children: HashSet::new(),
        }
    }

    fn count(&self) -> usize {
        let mut count = 0;
        let mut node = self.parent.as_ref();
        while let Some(parent_node) = node {
            count += 1;
            node = parent_node.parent.as_ref();
        }
        count
    }
}

/// Hydrates dependencies and returns a topologically sorted list of packages.
pub async fn hydrate<F>(
    input: &Vec<PackageReq>,
    get_deps: F,
) -> Result<Vec<PackageReq>, Box<dyn Error>>
where
    F: Fn(String) -> Result<Vec<PackageReq>, Box<dyn Error>>,
{
    let dry = condense(input);
    let mut graph: HashMap<String, Box<Node>> = HashMap::new();
    let mut stack: Vec<Box<Node>> = vec![];

    for pkg in dry.iter() {
        let node = graph
            .entry(pkg.project.clone())
            .or_insert_with(|| Box::new(Node::new(pkg.clone(), None)));
        node.pkg.constraint = intersect_constraints(&node.pkg.constraint, &pkg.constraint)?;
        stack.push(node.clone());
    }

    while let Some(mut current) = stack.pop() {
        for child_pkg in get_deps(current.pkg.project.clone())? {
            let child_node = graph
                .entry(child_pkg.project.clone())
                .or_insert_with(|| Box::new(Node::new(child_pkg.clone(), Some(current.clone()))));
            child_node.pkg.constraint =
                intersect_constraints(&child_node.pkg.constraint, &child_pkg.constraint)?;
            current.children.insert(child_node.pkg.project.clone());
            stack.push(child_node.clone());
        }
    }

    let mut nodes: Vec<&Box<Node>> = graph.values().collect();
    nodes.sort_by_key(|node| node.count());
    let nodes = nodes.into_iter().map(|node| node.pkg.clone()).collect();
    Ok(nodes)
}

/// Condenses a list of `PackageRequirement` by intersecting constraints for duplicates.
fn condense(pkgs: &Vec<PackageReq>) -> Vec<PackageReq> {
    let mut out: Vec<PackageReq> = vec![];
    for pkg in pkgs {
        if let Some(existing) = out.iter_mut().find(|p| p.project == pkg.project) {
            existing.constraint =
                intersect_constraints(&existing.constraint, &pkg.constraint).unwrap();
        } else {
            out.push(pkg.clone());
        }
    }
    out
}

/// Intersects two version constraints.
fn intersect_constraints(a: &VersionReq, b: &VersionReq) -> Result<VersionReq, Box<dyn Error>> {
    let rv = a.intersect(b)?;
    Ok(rv)
}