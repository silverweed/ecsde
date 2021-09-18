#[macro_export]
macro_rules! foreach_entity {
    ($world: expr, $(+$req: ty,)* $(~$exc: ty,)* $fn: expr) => {
        let mut entity_stream = $crate::entity_stream::new_entity_stream($world);
        $(entity_stream = entity_stream.require::<$req>();)*
        $(entity_stream = entity_stream.exclude::<$exc>();)*

        let mut entity_stream = entity_stream.build();

        loop {
            let entity = entity_stream.next($world);
            if entity.is_none() {
                break;
            }
            let entity = entity.unwrap();

            $fn(entity);
        }
    };
}

#[macro_export]
macro_rules! foreach_entity_enumerate {
    ($world: expr, $(+$req: ty,)* $(~$exc: ty,)* $fn: expr) => {
        let mut entity_stream = $crate::entity_stream::new_entity_stream($world);
        $(entity_stream = entity_stream.require::<$req>();)*
        $(entity_stream = entity_stream.exclude::<$exc>();)*

        let mut entity_stream = entity_stream.build();

        let mut i = 0;
        loop {
            let entity = entity_stream.next($world);
            if entity.is_none() {
                break;
            }
            let entity = entity.unwrap();

            $fn(entity, i);

            i += 1;
        }
    };
}

//
// tpl_map macro wizardry courtesy of
// https://stackoverflow.com/questions/66396814/generating-tuple-indices-based-on-macro-rules-repetition-expansion/66420824#66420824
//

#[macro_export]
macro_rules! tpl_map_apply {
    ($f:ident, $e:expr) => {
        $f!($e)
    };
}

#[macro_export]
macro_rules! tpl_map {
    (@, [], [$(($idx:tt))*], $tpl:ident, $fn:ident, ($($result:tt)*)) => {($($result)*)};
    (@, [$queue0:expr, $($queue:expr,)*], [($idx0:tt) $(($idx:tt))*], $tpl:ident, $fn:ident, ($($result:tt)*)) => {
        tpl_map!(@,
            [$($queue,)*],
            [$(($idx))*],
            $tpl,
            $fn,
            ($($result)* tpl_map_apply!($fn, ($tpl . $idx0)), )
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

#[macro_export]
macro_rules! foreach_entity_new {
    ($ecs_world: expr, read: $($read: ty),*; write: $($writ: ty),*; $fn: expr) => {
        let mut query = $crate::ecs_query::Ecs_Query::new($ecs_world);
        $(query = query.read::<$read>();)*
        $(query = query.write::<$writ>();)*

        let storages = query.storages();
        let comp_reads = ($(storages.begin_read::<$read>(),)*);
        let mut comp_writs = ($(storages.begin_write::<$writ>(),)*);
        for &entity in query.entities() {
            macro_rules! tpl_map_get     { ($elem:expr) => { $elem.must_get(entity) } }
            macro_rules! tpl_map_get_mut { ($elem:expr) => { $elem.must_get_mut(entity) } }

            let reads = tpl_map!([$(std::mem::size_of::<$read>(),)*], comp_reads, tpl_map_get);
            let writs = tpl_map!([$(std::mem::size_of::<$writ>(),)*], comp_writs, tpl_map_get_mut);
            $fn(entity, reads, writs);
        }
    };
}
