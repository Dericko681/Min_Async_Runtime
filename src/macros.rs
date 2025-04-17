#[macro_export]
macro_rules! mini_rt {
    (async fn $name:ident() $body:block) => {
        fn $name() {
            let mut rt = MiniRuntime::new();
            rt.block_on(async $body);
        }
    };
}

#[macro_export]
macro_rules! join_all {
    ($($handle:expr),*) => {
        async {
            let mut handles = vec![$($handle),*];
            for handle in handles.iter_mut() {
                handle.await;
            }
        }
    };
}