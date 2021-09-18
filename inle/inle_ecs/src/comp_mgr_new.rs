use crate::ecs_world::Entity;
use anymap::any::UncheckedAnyExt;
use anymap::Map;
use std::any::{type_name, TypeId};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

pub struct Component_Manager {
    storages: Map<dyn Component_Storage_Interface>,
}

#[cfg(debug_assertions)]
fn assert_gen_consistency<T>(storage: &Component_Storage<T>, entity: Entity) {
    debug_assert_eq!(
        storage.entity_comp_index.len(),
        storage.entity_comp_generation.len()
    );
    debug_assert_eq!(
                entity.gen,
                storage.entity_comp_generation[entity.index as usize],
                "Entity {:?} is not the same as the one contained in Component_Storage<{:?}> ({:?})! Probably it was destroyed and recreated but its components were not updated properly!", entity, type_name::<T>(), storage.entity_comp_generation[entity.index as usize]);
}

impl Component_Manager {
    pub fn new() -> Self {
        Self {
            storages: Map::new(),
        }
    }

    #[inline]
    pub fn has_component<T: 'static>(&self, entity: Entity) -> bool {
        if let Some(storage) = self.get_component_storage::<T>() {
            storage.has_component(entity)
        } else {
            false
        }
    }

    //#[inline]
    //pub fn get_component<T: 'static>(&self, entity: Entity) -> Option<&T> {
    //if let Some(storage) = self.get_component_storage::<T>() {
    //storage.get_component(entity)
    //} else {
    //None
    //}
    //}

    //#[inline]
    //pub fn get_component_mut<T: 'static>(&mut self, entity: Entity) -> Option<&mut T> {
    //if let Some(storage) = self.get_component_storage_mut::<T>() {
    //storage.get_component_mut(entity)
    //} else {
    //None
    //}
    //}

    #[inline]
    pub fn add_component<T: 'static>(&mut self, entity: Entity, data: T) {
        let storage = self
            .storages
            .entry::<Component_Storage<T>>()
            .or_insert_with(Component_Storage::<T>::default);

        let mut components = storage.components.write().unwrap();
        let cur_components_len = components.len();

        // Ensure the entity doesn't have this component already
        {
            let slot = storage.entity_comp_index.get(entity.index as usize);
            if slot.copied().flatten().is_some() {
                fatal!(
                    "Component {:?} added twice to entity {:?}!",
                    type_name::<T>(),
                    entity
                );
            }

            storage
                .entity_comp_index
                .resize(entity.index as usize + 1, None);
            storage
                .entity_comp_index
                .insert(entity.index as usize, Some(cur_components_len));
        }

        #[cfg(debug_assertions)]
        {
            storage
                .entity_comp_generation
                .resize(entity.index as usize + 1, 0);
            storage
                .entity_comp_generation
                .insert(entity.index as usize, entity.gen);
        }

        components.push(data);
    }

    #[inline]
    pub fn remove_component<T: 'static>(&mut self, entity: Entity) {
        if let Some(storage) = self.get_component_storage_mut::<T>() {
            storage.remove_component(entity);
        } else {
            lerr!(
                "Tried to remove inexisting component {:?} from entity {:?}",
                type_name::<T>(),
                entity
            );
        }
    }

    #[inline]
    pub fn remove_all_components(&mut self, entity: Entity) {
        for raw_storage in self.storages.as_mut().iter_mut() {
            raw_storage.remove_component(entity);
        }
    }

    //#[inline]
    //pub fn get_components<T: 'static>(&self) -> impl Iterator<Item = &T> {
    //if let Some(storage) = self.get_component_storage::<T>() {
    //storage.components.iter()
    //} else {
    //[].iter()
    //}
    //}

    //#[inline]
    //pub fn get_components_mut<T: 'static>(&self) -> impl Iterator<Item = &mut T> {
    //if let Some(storage) = self.get_component_storage_mut::<T>() {
    //storage.components.iter_mut()
    //} else {
    //[].iter_mut()
    //}
    //}

    #[inline]
    pub fn get_component_storage<T: 'static>(&self) -> Option<&Component_Storage<T>> {
        self.storages.get::<Component_Storage<T>>()
    }

    #[inline]
    pub fn get_component_storage_mut<T: 'static>(&mut self) -> Option<&mut Component_Storage<T>> {
        self.storages.get_mut::<Component_Storage<T>>()
    }
}

#[cfg(debug_assertions)]
impl Component_Manager {
    pub fn get_comp_name_list_for_entity(&self, entity: Entity) -> Vec<&'static str> {
        self.storages
            .as_ref()
            .iter()
            .filter_map(|storage| {
                if storage.has_component(entity) {
                    Some(storage.comp_name())
                } else {
                    None
                }
            })
            .collect()
    }
}

pub struct Component_Storage<T> {
    /// Indexed by entity_comp_index
    components: RwLock<Vec<T>>,

    /// Indexed by entity index.
    entity_comp_index: Vec<Option<usize>>,

    #[cfg(debug_assertions)]
    /// Keeps track of the generation of the entity when the component was last added to ensure consistency.
    /// Indexed by entity index
    entity_comp_generation: Vec<inle_alloc::gen_alloc::Gen_Type>,
}

impl<T> Default for Component_Storage<T> {
    fn default() -> Self {
        Self {
            components: RwLock::new(vec![]),
            entity_comp_index: vec![],
            #[cfg(debug_assertions)]
            entity_comp_generation: vec![],
        }
    }
}

pub struct Component_Storage_Read<'a, T> {
    components: RwLockReadGuard<'a, Vec<T>>,
    entity_comp_index: &'a [Option<usize>],
}

pub struct Component_Storage_Write<'a, T> {
    components: RwLockWriteGuard<'a, Vec<T>>,
    entity_comp_index: &'a [Option<usize>],
}

impl<T> Component_Storage<T> {
    fn lock_for_read(&self) -> Component_Storage_Read<'_, T> {
        Component_Storage_Read {
            components: self.components.read().unwrap(),
            entity_comp_index: &self.entity_comp_index,
        }
    }

    fn lock_for_write(&self) -> Component_Storage_Write<'_, T> {
        Component_Storage_Write {
            components: self.components.write().unwrap(),
            entity_comp_index: &self.entity_comp_index,
        }
    }
}

impl<T> Component_Storage_Read<'_, T> {
    pub fn get(&self, entity: Entity) -> &T {
        //#[cfg(debug_assertions)]
        //{
        //assert_gen_consistency(self, entity);
        //}
        ldebug!("{:?}", self.components.len());
        ldebug!("{:?}", self.entity_comp_index);
        let idx = self.entity_comp_index[entity.index as usize].unwrap();
        &self.components[idx]
    }
}

impl<T> Component_Storage_Write<'_, T> {
    pub fn get_mut(&mut self, entity: Entity) -> &mut T {
        //#[cfg(debug_assertions)]
        //{
        //assert_gen_consistency(self, entity);
        //}
        let idx = self.entity_comp_index[entity.index as usize].unwrap();
        &mut self.components[idx]
    }
}

struct Query<'mgr, 'str> {
    comp_mgr: &'mgr Component_Manager,
    storages: Storages<'str>,
    entities: Vec<Entity>,
}

#[derive(Default)]
struct Storages<'a> {
    reads: Vec<&'a dyn Component_Storage_Interface>,
    writes: Vec<&'a dyn Component_Storage_Interface>,

    read_indices: std::collections::HashMap<TypeId, usize>,
    write_indices: std::collections::HashMap<TypeId, usize>,
}

impl Storages<'_> {
    pub fn begin_read<T: 'static>(&self) -> Component_Storage_Read<T> {
        let idx = self.read_indices.get(&TypeId::of::<T>()).unwrap();
        let storage = unsafe { self.reads[*idx].downcast_ref_unchecked::<Component_Storage<T>>() };
        storage.lock_for_read()
    }

    pub fn begin_write<T: 'static>(&self) -> Component_Storage_Write<T> {
        let idx = self.write_indices.get(&TypeId::of::<T>()).unwrap();
        let storage = unsafe { self.writes[*idx].downcast_ref_unchecked::<Component_Storage<T>>() };
        storage.lock_for_write()
    }

    //pub fn get_mut<T: 'static>(&self, entity: Entity) -> &mut T {
    //let idx = self.write_indices.get(&TypeId::of::<T>()).unwrap();
    //let storage = unsafe { self.writes[*idx].downcast_ref_unchecked::<Component_Storage<T>>() };
    //storage.get_component_mut(entity).unwrap()
    //}
}

impl<'mgr, 'str> Query<'mgr, 'str>
where
    'mgr: 'str,
{
    // @Refactor: construct using Ecs_World rather than separate comp_mgr/entities
    fn new(comp_mgr: &'mgr Component_Manager, entities: &[Entity]) -> Self {
        Self {
            comp_mgr,
            storages: Storages::default(),
            entities: entities.to_vec(),
        }
    }

    fn read<T: 'static>(mut self) -> Self {
        let storage = self
            .comp_mgr
            .get_component_storage::<T>()
            .expect("TODO: handle missing comp");
        self.storages.reads.push(storage);
        self.storages
            .read_indices
            .insert(TypeId::of::<T>(), self.storages.reads.len() - 1);

        // @Temporary I guess
        let comp_mgr = self.comp_mgr;
        self.entities.retain(|&e| comp_mgr.has_component::<T>(e));

        self
    }

    fn write<T: 'static>(mut self) -> Self {
        let storage = self
            .comp_mgr
            .get_component_storage::<T>()
            .expect("TODO: handle missing comp");
        self.storages.writes.push(storage);
        self.storages
            .write_indices
            .insert(TypeId::of::<T>(), self.storages.writes.len() - 1);

        // @Temporary I guess
        let comp_mgr = self.comp_mgr;
        self.entities.retain(|&e| comp_mgr.has_component::<T>(e));

        self
    }

    //fn next(&mut self) -> (Entity, Storages<'str>) {}
}

/*
    let stream = entities(world).read::<C_1>().read::<C_2>().write::<C_3>();

    for (entity, storages) in stream {
        let c1s = storages.begin_read::<C_1>();
        let c1 = c1s.get(entity);
        let c3 = storages.get_mut::<C_3>(entity);
    }

*/
macro_rules! apply {
    ($f:ident, $e:expr) => {
        $f!($e)
    };
}
macro_rules! tpl_map {
    (@, [], [$(($idx:tt))*], $tpl:ident, $fn:ident, ($($result:tt)*)) => {($($result)*)};
    (@, [$queue0:expr, $($queue:expr,)*], [($idx0:tt) $(($idx:tt))*], $tpl:ident, $fn:ident, ($($result:tt)*)) => {
        tpl_map!(@,
            [$($queue,)*],
            [$(($idx))*],
            $tpl,
            $fn,
            ($($result)* apply!($fn, ($tpl . $idx0)), )
        )
    };
    ([$($queue:expr,)*], $tpl:ident, $fn:ident) => {
        tpl_map!(@,
            [$($queue,)*],
            [(0) (1) (2) (3) (4) (5) (6) (7) (8) (9)], // Hard limit to 10 read/write components per query!
            $tpl,
            $fn,
            ()
        )
    }
}

macro_rules! do_query {
    ($cm: expr, $ent: expr, read: $($read: ty),*; write: $($writ: ty),*; $fn: expr) => {
        let mut query = Query::new(&$cm,  &$ent);
        $(query = query.read::<$read>();)*
        $(query = query.write::<$writ>();)*

        let storages = &query.storages;
        let comp_reads = ($(storages.begin_read::<$read>(),)*);
        let mut comp_writs = ($(storages.begin_write::<$writ>(),)*);
        for &entity in &query.entities {
            macro_rules! tpl_map_get     { ($elem:expr) => { $elem.get(entity) } }
            macro_rules! tpl_map_get_mut { ($elem:expr) => { $elem.get_mut(entity) } }

            let reads = tpl_map!([$(std::mem::size_of::<$read>(),)*], comp_reads, tpl_map_get);
            let writs = tpl_map!([$(std::mem::size_of::<$writ>(),)*], comp_writs, tpl_map_get_mut);

            $fn(entity, reads, writs);
        }
    };
}

#[test]
fn test_flow() {
    struct A {
        x: u32,
    }
    struct B {
        y: f32,
    }
    struct C {
        s: String,
    }
    struct D {
        v: Vec<String>,
    }

    let mut comp_mgr = Component_Manager::new();
    let entities = [
        Entity { index: 1, gen: 1 },
        Entity { index: 2, gen: 1 },
        Entity { index: 3, gen: 1 },
    ];

    for e in &entities {
        comp_mgr.add_component(*e, A { x: 42 });
        comp_mgr.add_component(*e, B { y: 0.3 });
        comp_mgr.add_component(
            *e,
            C {
                s: "Hello sailor".to_string(),
            },
        );
        comp_mgr.add_component(
            *e,
            D {
                v: vec!["asd".to_string(), "bar".to_string()],
            },
        );
    }

    //let mut query = Query::new(&comp_mgr, &entities)
    //.read::<A>()
    //.read::<B>()
    //.write::<C>();

    //let storages = &query.storages;
    //for &entity in &query.entities {
    //let As = storages.begin_read::<A>();
    //let Bs = storages.begin_read::<B>();
    //let mut Cs = storages.begin_write::<C>();

    //let a = As.get(entity);
    //let b = Bs.get(entity);
    //let c = Cs.get_mut(entity);

    //c.s = format!("{}, {}", a.x, b.y);
    //ldebug!("{}", c.s);
    //}

    do_query!(comp_mgr, entities,
        read: A, B;
        write: C, D;
    |entity, (a, b): (&A, &B), (c, d): (&mut C, &mut D)| {
        c.s = format!("{}, {}", a.x, b.y);
        d.v.push(c.s.clone());
        ldebug!("{:?}", d.v);
    });

    //for (entity, storages) in stream {
    //let c1s = storages.begin_read::<C_1>();
    //let c1 = c1s.get(entity);
    //let c3 = storages.get_mut::<C_3>(entity);
    //}
}

impl<T: 'static> Component_Storage<T> {
    #[inline]
    pub fn has_component(&self, entity: Entity) -> bool {
        if let Some(slot) = self.entity_comp_index.get(entity.index as usize) {
            #[cfg(debug_assertions)]
            {
                assert_gen_consistency(self, entity);
            }

            slot.is_some()
        } else {
            false
        }
    }

    //#[inline]
    //pub fn get_component(&self, entity: Entity) -> Option<&T> {
    //if let Some(slot) = self.entity_comp_index.get(entity.index as usize) {
    //#[cfg(debug_assertions)]
    //{
    //assert_gen_consistency(self, entity);
    //}

    //slot.map(|idx| &self.components.read().unwrap()[idx])
    //} else {
    //None
    //}
    //}

    //#[inline]
    //pub fn get_component_mut(&mut self, entity: Entity) -> Option<&mut T> {
    //if let Some(slot) = self.entity_comp_index.get(entity.index as usize) {
    //#[cfg(debug_assertions)]
    //{
    //assert_gen_consistency(self, entity);
    //}

    //slot.map(move |idx| &mut self.components.write().unwrap()[idx])
    //} else {
    //None
    //}
    //}
}

//impl<T> AsRef<[T]> for Component_Storage<T> {
//fn as_ref(&self) -> &[T] {
//self.components.read().unwrap().as_slice()
//}
//}

//impl<T> AsMut<[T]> for Component_Storage<T> {
//fn as_mut(&mut self) -> &mut [T] {
//self.components.write().unwrap().as_mut_slice()
//}
//}

// This trait is required to define common methods for all storages
// so we can iterate on them in an untyped fashion and still be
// able to do stuff on them (e.g. remove_all_components)
trait Component_Storage_Interface: anymap::any::Any {
    fn remove_component(&mut self, entity: Entity);

    fn has_component(&self, entity: Entity) -> bool;

    #[cfg(debug_assertions)]
    fn comp_name(&self) -> &'static str;
}

impl<T: 'static> Component_Storage_Interface for Component_Storage<T> {
    fn remove_component(&mut self, entity: Entity) {
        if let Some(slot) = self.entity_comp_index.get_mut(entity.index as usize) {
            *slot = None;
        } else {
            lerr!(
                "Tried to remove inexisting component {:?} from entity {:?}",
                type_name::<T>(),
                entity
            );
        }
    }

    // @Cleanup: I wonder if this is the best way to do this.
    // Should we just expose the trait and have all methods be
    // trait methods? But would that mean that we'd have to go through
    // a vtable all the time? I guess not if we have the concrete type?
    // Investigate on this.
    fn has_component(&self, entity: Entity) -> bool {
        Component_Storage::<T>::has_component(self, entity)
    }

    #[cfg(debug_assertions)]
    fn comp_name(&self) -> &'static str {
        // Note: we assume that type_name returns a full path like foo::bar::Type, although that's not guaranteed.
        // This won't break if that's not the case anyway.
        let full_name = type_name::<T>();
        let base_name = full_name.rsplit(':').next().unwrap_or(full_name);
        base_name
    }
}

//
// The following traits are to make anymap::Map work with our trait
//

struct Comp_Map_Entry(RwLock<dyn Component_Storage_Interface>);

impl UncheckedAnyExt for dyn Component_Storage_Interface {
    #[inline]
    unsafe fn downcast_ref_unchecked<T: 'static>(&self) -> &T {
        &*(self as *const Self as *const T)
    }

    #[inline]
    unsafe fn downcast_mut_unchecked<T: 'static>(&mut self) -> &mut T {
        &mut *(self as *mut Self as *mut T)
    }

    #[inline]
    unsafe fn downcast_unchecked<T: 'static>(self: Box<Self>) -> Box<T> {
        Box::from_raw(Box::into_raw(self) as *mut T)
    }
}

impl<T: 'static> anymap::any::IntoBox<dyn Component_Storage_Interface> for Component_Storage<T> {
    #[inline]
    fn into_box(self) -> Box<dyn Component_Storage_Interface> {
        Box::new(self)
    }
}
