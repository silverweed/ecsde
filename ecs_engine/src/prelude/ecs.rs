#[macro_export]
macro_rules! foreach_entity {
    ($world: expr, $(+$req: ty,)* $(~$exc: ty,)* $fn: expr) => {
        let mut entity_stream = $crate::ecs::entity_stream::new_entity_stream($world);
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
        let mut entity_stream = $crate::ecs::entity_stream::new_entity_stream($world);
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
