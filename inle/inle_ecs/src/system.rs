use super::ecs_world::Ecs_World;
use std::any::TypeId;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

trait System: std::fmt::Debug + Send {
    fn get_read_comps(&self) -> HashSet<TypeId>;
    fn get_write_comps(&self) -> HashSet<TypeId>;

    fn update(&mut self, world: Arc<Mutex<Ecs_World>>) {
        println!(
            "updating {:?}{:?}",
            self.get_read_comps(),
            self.get_write_comps()
        );
    }
}

struct Comp_A;
struct Comp_B;
struct Comp_C;
struct Comp_D;

#[derive(Debug)]
struct Sys_A;
#[derive(Debug)]
struct Sys_B;
#[derive(Debug)]
struct Sys_C;
#[derive(Debug)]
struct Sys_D;

macro_rules! hashset {
    ($($x: expr),*$(,)?) => {{
        let mut set = HashSet::new();
        $(
            set.insert($x);
        )*
        set
    }};
    () => {{ HashSet::new() }};
}

impl System for Sys_A {
    fn get_read_comps(&self) -> HashSet<TypeId> {
        return hashset![TypeId::of::<Comp_A>()];
    }

    fn get_write_comps(&self) -> HashSet<TypeId> {
        return hashset![TypeId::of::<Comp_B>()];
    }
}

impl System for Sys_B {
    fn get_read_comps(&self) -> HashSet<TypeId> {
        return hashset![TypeId::of::<Comp_A>(), TypeId::of::<Comp_B>()];
    }

    fn get_write_comps(&self) -> HashSet<TypeId> {
        return hashset![TypeId::of::<Comp_C>()];
    }
}

impl System for Sys_C {
    fn get_read_comps(&self) -> HashSet<TypeId> {
        return hashset![TypeId::of::<Comp_C>(), TypeId::of::<Comp_D>()];
    }

    fn get_write_comps(&self) -> HashSet<TypeId> {
        return hashset![];
    }
}

impl System for Sys_D {
    fn get_read_comps(&self) -> HashSet<TypeId> {
        return hashset![];
    }

    fn get_write_comps(&self) -> HashSet<TypeId> {
        return hashset![TypeId::of::<Comp_D>()];
    }
}

type System_Dep_Graph = Vec<Vec<Box<dyn System + 'static>>>;

fn build_sys_dep_graph(systems: Vec<Box<dyn System>>) -> System_Dep_Graph {
    let mut graph: System_Dep_Graph = Vec::with_capacity(systems.len());

    for system in systems {
        let mut inserted = false;
        let list = graph
            .iter_mut()
            .find(|list| list.iter().all(|sys| systems_compatible(&*system, &**sys)));
        if let Some(list) = list {
            list.push(system);
        } else {
            graph.push(vec![system]);
        }
    }

    graph
}

fn update_systems(dep_graph: &mut System_Dep_Graph, world: Arc<Mutex<Ecs_World>>) {
    use rayon::prelude::*;

    for list in dep_graph {
        list.par_iter_mut().for_each(|sys| {
            sys.update(world.clone());
        })
    }
}

fn systems_compatible(a: &dyn System, b: &dyn System) -> bool {
    let read_a = a.get_read_comps();
    let write_a = a.get_write_comps();
    let read_b = b.get_read_comps();
    let write_b = b.get_write_comps();

    read_a.intersection(&write_b).next().is_none()
        && write_a.intersection(&read_b).next().is_none()
        && write_a.intersection(&write_b).next().is_none()
}

pub fn dothething() {
    let systems: Vec<Box<dyn System>> = vec![
        Box::new(Sys_A {}),
        Box::new(Sys_B {}),
        Box::new(Sys_C {}),
        Box::new(Sys_D {}),
    ];
    let mut graph = build_sys_dep_graph(systems);
    let world = Arc::new(Mutex::new(Ecs_World::new()));
    loop {
        update_systems(&mut graph, world.clone());
        println!("-----");
    }
    println!("{:?}", graph);
}
