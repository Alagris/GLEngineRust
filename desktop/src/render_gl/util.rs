use std::mem::MaybeUninit;

pub fn type_name<T>() -> &'static str {
    let s = std::any::type_name::<T>();
    s.rsplit_once("::").map(|(_a, b)| b).unwrap_or(s)
}

pub fn init_array<T, F: FnMut() -> T, const S: usize>(mut f: F) -> [T; S] {
    let mut arr: [MaybeUninit<T>; S] = unsafe { MaybeUninit::uninit().assume_init() };
    for elem in &mut arr {
        *elem = MaybeUninit::new(f());
    }
    unsafe{MaybeUninit::array_assume_init(arr)}
}
