
pub fn type_name<T>()->&'static str{
    let s = std::any::type_name::<T>();
    s.rsplit_once("::").map(|(a,b)|b).unwrap_or(s)
}